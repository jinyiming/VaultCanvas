pub use common::{OperationResult, ProgressUpdate, VaultError, VaultResult};
pub use crypto_engine::{
    decrypt_file, encrypt_file, DecryptRequest, EncryptRequest, EncryptionAlgorithm,
};
pub use password_engine::{generate_password_from_secrets, PasswordGeneratorRequest};
pub use stego_engine::{embed_file, extract_file, EmbedRequest, ExtractRequest, StegoMode};

pub fn generate_derived_password_command(request: PasswordGeneratorRequest) -> VaultResult<String> {
    generate_password_from_secrets(request)
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
