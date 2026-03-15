# VaultCanvas Product Architecture (Current)

## 1. Product Scope

VaultCanvas is a local security utility with three core functions:

- password generation
- file encryption/decryption
- file steganography/reveal

## 2. Stack

- GUI: Rust `eframe/egui`
- Core: Rust workspace with modular crates
- File dialogs: `rfd`

## 3. Module Layers

```text
apps/windows_native
    -> crates/security_service
        -> crates/password_engine
        -> crates/crypto_engine
        -> crates/stego_engine
        -> crates/common

crates/bench_runner
    -> crates/security_service
```

## 4. Algorithm Parity (with demo scripts)

### 4.1 Crypto

- KDF: Argon2id (`time_cost=2`, `memory_cost=64MB`, `parallelism=2`)
- Cipher: AES-256-GCM
- Envelope: `SECURE_ENC_V5 + version + salt + iv + metadata_nonce + tag + metadata_sign + id_hash + ciphertext`
- Integrity check: `SHA3-256(metadata + id_hash)`

### 4.2 Stego (Append mode)

- Payload pipeline: source file -> zlib compress -> PBKDF2-SHA1 key derive -> AES-256-CBC encrypt
- Payload format: `APPEND_MAGIC(0x89STE) + length + iv + salt + encrypted + SHA256(encrypted)`
- Reveal pipeline: locate magic -> verify checksum -> decrypt -> decompress -> restore

### 4.3 Password Generator

- Input: main password + id password
- Output: fixed 16-char password with uppercase, lowercase, digits, and symbols
- Behavior: deterministic output for same input pair

## 5. UX Rules

- Three tabs only: password, crypto, stego
- Save location is selected when user starts a job
- Output file names auto-avoid collisions

## 6. Non-Functional Constraints

- Local-only execution
- Password fields are masked
- User-facing error messages are normalized by the GUI

## 7. Release Form

- Primary artifact: `dist/windows-native/VaultCanvas.exe`
- Single EXE package: `scripts/package_windows_single.ps1`
- Portable package folder: `scripts/package_windows_portable.ps1`
- Benchmark entrypoint: `scripts/benchmark_engines.ps1`
