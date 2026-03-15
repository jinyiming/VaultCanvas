param(
  [int[]]$SizesMb = @(4, 16, 64),
  [string]$MainPassword = "bench-main-password",
  [string]$IdPassword = "bench-id-password",
  [string]$StegoPassword = "bench-stego-password",
  [int]$ProbeIntervalMs = 40
)

$ErrorActionPreference = "Stop"
$MaxAllowedSizeMb = 2048

function Resolve-CargoPath {
  $candidate = Get-Command cargo -ErrorAction SilentlyContinue
  if ($candidate) {
    return $candidate.Source
  }
  $fallback = "C:\Users\Admin\.cargo\bin\cargo.exe"
  if (Test-Path $fallback) {
    return $fallback
  }
  throw "cargo executable was not found"
}

function New-RandomFile {
  param(
    [Parameter(Mandatory = $true)][string]$Path,
    [Parameter(Mandatory = $true)][long]$SizeBytes
  )

  $buffer = New-Object byte[] (1024 * 1024)
  $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
  $stream = [System.IO.File]::Open(
    $Path,
    [System.IO.FileMode]::Create,
    [System.IO.FileAccess]::Write,
    [System.IO.FileShare]::None
  )
  try {
    $remaining = $SizeBytes
    while ($remaining -gt 0) {
      $chunk = [Math]::Min($remaining, $buffer.Length)
      $rng.GetBytes($buffer)
      $stream.Write($buffer, 0, [int]$chunk)
      $remaining -= $chunk
    }
  }
  finally {
    $stream.Dispose()
    $rng.Dispose()
  }
}

function Invoke-BenchCommand {
  param(
    [Parameter(Mandatory = $true)][string]$Executable,
    [Parameter(Mandatory = $true)][string[]]$Arguments,
    [int]$ProbeMs = 40
  )

  $quoted = New-Object System.Collections.Generic.List[string]
  foreach ($arg in $Arguments) {
    $argText = if ($null -eq $arg) { "" } else { [string]$arg }
    if (($argText -notmatch "[\s`"]") -and ($argText.Length -gt 0)) {
      $quoted.Add($argText)
    }
    else {
      $escaped = $argText -replace '(\\*)"', '$1$1\"'
      $escaped = $escaped -replace '(\\+)$', '$1$1'
      $quoted.Add(('"{0}"' -f $escaped))
    }
  }

  $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
  $startInfo.FileName = $Executable
  $startInfo.Arguments = [string]::Join(" ", $quoted)
  $startInfo.UseShellExecute = $false
  $startInfo.RedirectStandardOutput = $true
  $startInfo.RedirectStandardError = $true
  $startInfo.CreateNoWindow = $true

  $process = [System.Diagnostics.Process]::new()
  $process.StartInfo = $startInfo
  $timer = [System.Diagnostics.Stopwatch]::StartNew()
  if (-not $process.Start()) {
    throw "failed to start process: $Executable"
  }

  $peakBytes = 0L
  while (-not $process.HasExited) {
    $process.Refresh()
    if ($process.WorkingSet64 -gt $peakBytes) {
      $peakBytes = $process.WorkingSet64
    }
    Start-Sleep -Milliseconds $ProbeMs
  }

  $process.WaitForExit()
  $timer.Stop()
  if ($process.PeakWorkingSet64 -gt $peakBytes) {
    $peakBytes = $process.PeakWorkingSet64
  }

  $stdOutReader = $process.StandardOutput
  $stdErrReader = $process.StandardError
  $stdOut = if ($stdOutReader) { $stdOutReader.ReadToEnd().Trim() } else { "" }
  $stdErr = if ($stdErrReader) { $stdErrReader.ReadToEnd().Trim() } else { "" }

  [PSCustomObject]@{
    ExitCode    = $process.ExitCode
    ElapsedMs   = [Math]::Round($timer.Elapsed.TotalMilliseconds, 2)
    PeakBytes   = $peakBytes
    StdOut      = $stdOut
    StdErr      = $stdErr
  }
}

function Append-Result {
  param(
    [Parameter(Mandatory = $true)]$Rows,
    [Parameter(Mandatory = $true)][string]$Scenario,
    [Parameter(Mandatory = $true)][string]$Operation,
    [Parameter(Mandatory = $true)][int]$InputMb,
    [Parameter(Mandatory = $true)][double]$ElapsedMs,
    [Parameter(Mandatory = $true)][long]$PeakBytes,
    [Parameter(Mandatory = $true)][long]$OutputBytes,
    [Parameter(Mandatory = $true)][string]$Integrity
  )

  $Rows.Add([PSCustomObject]@{
      Scenario      = $Scenario
      Operation     = $Operation
      InputMb       = $InputMb
      ElapsedMs     = [Math]::Round($ElapsedMs, 2)
      PeakMemoryMb  = [Math]::Round(($PeakBytes / 1MB), 2)
      OutputMb      = [Math]::Round(($OutputBytes / 1MB), 2)
      Integrity     = $Integrity
    })
}

$root = Split-Path -Parent $PSScriptRoot
$cargo = Resolve-CargoPath

Push-Location $root
try {
  & $cargo build -p bench_runner --release
  if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed"
  }
}
finally {
  Pop-Location
}

$benchExe = Join-Path $root "target\release\bench_runner.exe"
if (!(Test-Path $benchExe)) {
  throw "bench runner executable not found: $benchExe"
}

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$reportDir = Join-Path $root "dist\benchmarks\$timestamp"
$inputDir = Join-Path $reportDir "inputs"
$outputDir = Join-Path $reportDir "outputs"
New-Item -ItemType Directory -Force -Path $inputDir | Out-Null
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

$rows = New-Object System.Collections.Generic.List[object]

foreach ($sizeMb in $SizesMb) {
  if ($sizeMb -le 0) {
    throw "invalid size value: $sizeMb"
  }
  if ($sizeMb -gt $MaxAllowedSizeMb) {
    throw "size value $sizeMb MB exceeds safety limit ($MaxAllowedSizeMb MB). If you used '-File ... -SizesMb 4,16,64', run with '& .\\scripts\\benchmark_engines.ps1 -SizesMb @(4,16,64)' instead."
  }

  $sizeBytes = [long]$sizeMb * 1MB
  $payload = Join-Path $inputDir ("payload_{0}mb.bin" -f $sizeMb)
  $carrier = Join-Path $inputDir ("carrier_{0}mb.bin" -f $sizeMb)
  $encrypted = Join-Path $outputDir ("crypto_{0}mb.enc" -f $sizeMb)
  $decrypted = Join-Path $outputDir ("crypto_{0}mb.dec" -f $sizeMb)
  $embedded = Join-Path $outputDir ("stego_{0}mb.ste" -f $sizeMb)
  $extracted = Join-Path $outputDir ("stego_{0}mb.out" -f $sizeMb)

  Write-Host "Preparing ${sizeMb}MB input files..."
  New-RandomFile -Path $payload -SizeBytes $sizeBytes
  New-RandomFile -Path $carrier -SizeBytes $sizeBytes

  foreach ($path in @($encrypted, $decrypted, $embedded, $extracted)) {
    if (Test-Path $path) {
      Remove-Item -Force $path
    }
  }

  $sourceHash = (Get-FileHash -Path $payload -Algorithm SHA256).Hash

  Write-Host "Running crypto-encrypt (${sizeMb}MB)..."
  $cryptoEncrypt = Invoke-BenchCommand -Executable $benchExe -Arguments @(
    "crypto-encrypt", $payload, $encrypted, $MainPassword, $IdPassword
  ) -ProbeMs $ProbeIntervalMs
  if ($cryptoEncrypt.ExitCode -ne 0) {
    throw "crypto-encrypt failed at ${sizeMb}MB: $($cryptoEncrypt.StdErr)"
  }
  Append-Result -Rows $rows -Scenario "crypto" -Operation "encrypt" -InputMb $sizeMb `
    -ElapsedMs $cryptoEncrypt.ElapsedMs -PeakBytes $cryptoEncrypt.PeakBytes `
    -OutputBytes (Get-Item $encrypted).Length -Integrity "n/a"

  Write-Host "Running crypto-decrypt (${sizeMb}MB)..."
  $cryptoDecrypt = Invoke-BenchCommand -Executable $benchExe -Arguments @(
    "crypto-decrypt", $encrypted, $decrypted, $MainPassword, $IdPassword
  ) -ProbeMs $ProbeIntervalMs
  if ($cryptoDecrypt.ExitCode -ne 0) {
    throw "crypto-decrypt failed at ${sizeMb}MB: $($cryptoDecrypt.StdErr)"
  }
  $decryptedHash = (Get-FileHash -Path $decrypted -Algorithm SHA256).Hash
  if ($decryptedHash -ne $sourceHash) {
    throw "crypto integrity mismatch at ${sizeMb}MB"
  }
  Append-Result -Rows $rows -Scenario "crypto" -Operation "decrypt" -InputMb $sizeMb `
    -ElapsedMs $cryptoDecrypt.ElapsedMs -PeakBytes $cryptoDecrypt.PeakBytes `
    -OutputBytes (Get-Item $decrypted).Length -Integrity "ok"

  Write-Host "Running stego-embed (${sizeMb}MB)..."
  $stegoEmbed = Invoke-BenchCommand -Executable $benchExe -Arguments @(
    "stego-embed", $carrier, $payload, $embedded, $StegoPassword
  ) -ProbeMs $ProbeIntervalMs
  if ($stegoEmbed.ExitCode -ne 0) {
    throw "stego-embed failed at ${sizeMb}MB: $($stegoEmbed.StdErr)"
  }
  Append-Result -Rows $rows -Scenario "stego" -Operation "embed" -InputMb $sizeMb `
    -ElapsedMs $stegoEmbed.ElapsedMs -PeakBytes $stegoEmbed.PeakBytes `
    -OutputBytes (Get-Item $embedded).Length -Integrity "n/a"

  Write-Host "Running stego-extract (${sizeMb}MB)..."
  $stegoExtract = Invoke-BenchCommand -Executable $benchExe -Arguments @(
    "stego-extract", $embedded, $extracted, $StegoPassword
  ) -ProbeMs $ProbeIntervalMs
  if ($stegoExtract.ExitCode -ne 0) {
    throw "stego-extract failed at ${sizeMb}MB: $($stegoExtract.StdErr)"
  }
  $extractedHash = (Get-FileHash -Path $extracted -Algorithm SHA256).Hash
  if ($extractedHash -ne $sourceHash) {
    throw "stego integrity mismatch at ${sizeMb}MB"
  }
  Append-Result -Rows $rows -Scenario "stego" -Operation "extract" -InputMb $sizeMb `
    -ElapsedMs $stegoExtract.ElapsedMs -PeakBytes $stegoExtract.PeakBytes `
    -OutputBytes (Get-Item $extracted).Length -Integrity "ok"
}

$jsonPath = Join-Path $reportDir "benchmark.json"
$markdownPath = Join-Path $reportDir "benchmark.md"
[System.IO.File]::WriteAllText(
  $jsonPath,
  ($rows | ConvertTo-Json -Depth 5),
  [System.Text.UTF8Encoding]::new($false)
)

$lines = New-Object System.Collections.Generic.List[string]
$lines.Add("# VaultCanvas Engine Benchmark")
$lines.Add("")
$lines.Add(("- Generated: {0}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss")))
$lines.Add(("- Sizes (MB): {0}" -f ($SizesMb -join ", ")))
$lines.Add("")
$lines.Add("| Scenario | Operation | Input (MB) | Time (ms) | Peak Memory (MB) | Output (MB) | Integrity |")
$lines.Add("| --- | --- | ---: | ---: | ---: | ---: | --- |")
foreach ($row in $rows) {
  $lines.Add(
    ("| {0} | {1} | {2} | {3} | {4} | {5} | {6} |" -f `
      $row.Scenario, $row.Operation, $row.InputMb, $row.ElapsedMs, $row.PeakMemoryMb, $row.OutputMb, $row.Integrity)
  )
}
[System.IO.File]::WriteAllLines($markdownPath, $lines, [System.Text.UTF8Encoding]::new($false))

Write-Host ""
Write-Host "Benchmark completed."
Write-Host "JSON report: $jsonPath"
Write-Host "Markdown report: $markdownPath"
