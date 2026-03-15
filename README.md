# VaultCanvas

Local-first file security tool with shared Rust core and native desktop shells (Windows/macOS).

## Workspace Layout

- `apps/native_gui`: shared native GUI logic (`eframe`/`egui`)
- `apps/windows_native`: Windows desktop entrypoint
- `apps/macos_native`: macOS desktop entrypoint
- `crates/common`: shared errors and result types
- `crates/password_engine`: password algorithm implementation
- `crates/crypto_engine`: file encryption/decryption implementation
- `crates/stego_engine`: file steganography/extraction implementation
- `crates/security_service`: service facade used by the GUI
- `crates/bench_runner`: benchmark CLI for engine performance/integrity runs
- `demo`: reference Python scripts for algorithm parity
- `docs/product-architecture.md`: current architecture spec

## Implemented Features

- Password generator: `main password + id password` deterministic output, fixed 16 chars with mixed charset
- File encryption/decryption: Argon2id + AES-256-GCM with `SECURE_ENC_V5` file header
- File stego/reveal: append mode pipeline (`compress -> encrypt -> append -> checksum verify`)
- Fully local execution with no cloud dependency

## Local Verification

- `cargo check -p vaultcanvas_windows_native -p vaultcanvas_macos_native`
- `cargo test -p password_engine -p crypto_engine -p stego_engine`
- `& .\scripts\benchmark_engines.ps1 -SizesMb @(4,16,64)`

## Outputs and Packaging

- Main EXE: `dist/windows-native/VaultCanvas.exe`
- Single EXE package script: `scripts/package_windows_single.ps1`
- Portable folder package script: `scripts/package_windows_portable.ps1`
- macOS app package script: `scripts/package_macos_app.sh`
- Benchmark reports: `dist/benchmarks/<timestamp>/benchmark.{json,md}`
