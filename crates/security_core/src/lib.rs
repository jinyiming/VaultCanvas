pub mod api;
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */

pub use common::{OperationResult, ProgressUpdate, VaultError, VaultResult};
pub use crypto_engine::{
    decrypt_file, encrypt_file, DecryptRequest, EncryptRequest, EncryptionAlgorithm,
};
pub use password_engine::{
    generate_passphrase, generate_password, score_password, PassphraseRequest, PasswordPolicy,
    PasswordStrength,
};
pub use stego_engine::{embed_file, extract_file, EmbedRequest, ExtractRequest, StegoMode};

pub fn generate_password_command(policy: PasswordPolicy) -> VaultResult<String> {
    generate_password(policy)
}

pub fn generate_passphrase_command(request: PassphraseRequest) -> VaultResult<String> {
    generate_passphrase(request)
}

pub fn encrypt_file_command(request: EncryptRequest) -> VaultResult<OperationResult> {
    encrypt_file(request)
}

pub fn decrypt_file_command(request: DecryptRequest) -> VaultResult<OperationResult> {
    decrypt_file(request)
}

pub fn embed_file_command(request: EmbedRequest) -> VaultResult<OperationResult> {
    embed_file(request)
}

pub fn extract_file_command(request: ExtractRequest) -> VaultResult<OperationResult> {
    extract_file(request)
}
