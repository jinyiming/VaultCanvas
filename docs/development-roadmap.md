# Development Roadmap

## Immediate Build Tasks

### 1. Bootstrap Flutter Desktop

- run `flutter create apps/desktop`
- preserve `lib/` structure from this scaffold
- enable desktop targets required by the product

### 2. Wire Rust Bridge

- add `flutter_rust_bridge` codegen
- expose functions from `crates/security_core`
- define DTO mapping between Dart and Rust

### 3. Implement Password Engine

- preserve current `PasswordPolicy` model
- add memorable passphrase mode
- add unit tests for entropy scoring

### 4. Implement Crypto Engine

- define versioned file header
- implement Argon2id key derivation
- implement authenticated encryption
- add stream-based file processing
- add integration tests with fixture files

### 5. Implement Stego Engine

- append mode first
- PNG LSB mode second
- add capacity estimator and extraction validation

## Suggested Test Matrix

- small text file encryption/decryption
- large binary file encryption/decryption
- wrong-password rejection
- corrupted-header rejection
- append-mode embed/extract
- PNG LSB embed/extract
- empty-file and oversized-payload cases

## Packaging Goals

- Windows installer
- macOS signed app bundle
- Linux AppImage or deb package
