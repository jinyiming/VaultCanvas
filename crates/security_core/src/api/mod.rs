pub mod types;

pub use types::{
    BridgeDecryptRequest, BridgeEmbedRequest, BridgeEncryptRequest, BridgeEncryptionAlgorithm,
    BridgeExtractRequest, BridgeOperationResult, BridgePassphraseRequest, BridgePasswordPolicy,
    BridgeStegoMode,
};

use crate::{
    decrypt_file_command, embed_file_command, encrypt_file_command, extract_file_command,
    generate_passphrase_command, generate_password_command,
};

pub fn generate_password(policy: BridgePasswordPolicy) -> Result<String, String> {
    generate_password_command(policy.into()).map_err(|err| err.to_string())
}

pub fn generate_passphrase(request: BridgePassphraseRequest) -> Result<String, String> {
    generate_passphrase_command(request.into()).map_err(|err| err.to_string())
}

pub fn encrypt_file(request: BridgeEncryptRequest) -> Result<BridgeOperationResult, String> {
    encrypt_file_command(request.into())
        .map(Into::into)
        .map_err(|err| err.to_string())
}

pub fn decrypt_file(request: BridgeDecryptRequest) -> Result<BridgeOperationResult, String> {
    decrypt_file_command(request.into())
        .map(Into::into)
        .map_err(|err| err.to_string())
}

pub fn embed_file(request: BridgeEmbedRequest) -> Result<BridgeOperationResult, String> {
    embed_file_command(request.into())
        .map(Into::into)
        .map_err(|err| err.to_string())
}

pub fn extract_file(request: BridgeExtractRequest) -> Result<BridgeOperationResult, String> {
    extract_file_command(request.into())
        .map(Into::into)
        .map_err(|err| err.to_string())
}
