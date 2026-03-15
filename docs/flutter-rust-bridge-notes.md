# Flutter Rust Bridge Notes

## Current Status

Bridge code has been generated into:

- `C:\Users\Admin\Desktop\111222\apps\desktop\lib\services\generated\api.dart`
- `C:\Users\Admin\Desktop\111222\apps\desktop\lib\services\generated\frb_generated.dart`
- `C:\Users\Admin\Desktop\111222\apps\desktop\lib\services\generated\frb_generated.io.dart`

Rust bridge entry module:

- `C:\Users\Admin\Desktop\111222\crates\security_core\src\api\mod.rs`

## Recommended Integration Shape

Use `flutter_rust_bridge` with `security_core` as the exported facade crate.

Suggested first exported functions:

- `generate_password_command(policy)`
- `generate_passphrase_command(request)`
- `encrypt_file_command(request)`
- `decrypt_file_command(request)`
- `embed_file_command(request)`
- `extract_file_command(request)`

## DTO Mapping

Rust types already prepared in:

- `C:\Users\Admin\Desktop\111222\crates\password_engine\src\lib.rs`
- `C:\Users\Admin\Desktop\111222\crates\crypto_engine\src\lib.rs`
- `C:\Users\Admin\Desktop\111222\crates\stego_engine\src\lib.rs`
- `C:\Users\Admin\Desktop\111222\crates\security_core\src\lib.rs`

Flutter placeholder DTOs exist in:

- `C:\Users\Admin\Desktop\111222\apps\desktop\lib\services\security_bridge.dart`

## Current Limitation

The bridge now uses FRB-friendly value DTOs and the Flutter smoke test can already call Rust password generation successfully.

Remaining limitation:

- the desktop build currently depends on `security_core.dll` being present beside the executable on Windows
- larger packaging work is still needed to automate Rust artifact placement across all desktop targets

## Next Codegen Step

After Flutter and Rust toolchains are available:

1. add `flutter_rust_bridge` dependencies
2. expose Rust APIs in the bridge module
3. generate Dart bindings
4. replace `UnimplementedError` methods in `security_bridge.dart`
5. route page actions to real bridge calls

## UI Readiness

The following pages now include input forms, validation, and busy states:

- `C:\Users\Admin\Desktop\111222\apps\desktop\lib\features\passwords\passwords_page.dart`
- `C:\Users\Admin\Desktop\111222\apps\desktop\lib\features\crypto\crypto_page.dart`
- `C:\Users\Admin\Desktop\111222\apps\desktop\lib\features\stego\stego_page.dart`

## Immediate UI Targets

- add file picker integration
- add validation and disabled submit states
- add job progress and success receipts
- add clipboard copy action for generated secrets
