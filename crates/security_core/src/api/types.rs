use serde::{Deserialize, Serialize};

use crate::{
    DecryptRequest, EmbedRequest, EncryptRequest, EncryptionAlgorithm, ExtractRequest,
    OperationResult, PassphraseRequest, PasswordPolicy, StegoMode,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeEncryptionAlgorithm {
    Aes256Gcm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeStegoMode {
    Append,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgePasswordPolicy {
    pub length: usize,
    pub use_lowercase: bool,
    pub use_uppercase: bool,
    pub use_digits: bool,
    pub use_symbols: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgePassphraseRequest {
    pub word_count: usize,
    pub separator: String,
    pub capitalize_words: bool,
    pub append_number: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeEncryptRequest {
    pub input_path: String,
    pub output_path: String,
    pub password: String,
    pub id_password: String,
    pub algorithm: BridgeEncryptionAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeDecryptRequest {
    pub input_path: String,
    pub output_path: String,
    pub password: String,
    pub id_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeEmbedRequest {
    pub carrier_path: String,
    pub payload_path: String,
    pub output_path: String,
    pub password: String,
    pub mode: BridgeStegoMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeExtractRequest {
    pub carrier_path: String,
    pub output_path: String,
    pub password: String,
    pub mode: BridgeStegoMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeOperationResult {
    pub output_path: String,
    pub bytes_processed: u64,
}

impl From<BridgeEncryptionAlgorithm> for EncryptionAlgorithm {
    fn from(value: BridgeEncryptionAlgorithm) -> Self {
        match value {
            BridgeEncryptionAlgorithm::Aes256Gcm => Self::Aes256Gcm,
        }
    }
}

impl From<BridgeStegoMode> for StegoMode {
    fn from(value: BridgeStegoMode) -> Self {
        match value {
            BridgeStegoMode::Append => Self::Append,
        }
    }
}

impl From<BridgePasswordPolicy> for PasswordPolicy {
    fn from(value: BridgePasswordPolicy) -> Self {
        Self {
            length: value.length,
            use_lowercase: value.use_lowercase,
            use_uppercase: value.use_uppercase,
            use_digits: value.use_digits,
            use_symbols: value.use_symbols,
        }
    }
}

impl From<BridgePassphraseRequest> for PassphraseRequest {
    fn from(value: BridgePassphraseRequest) -> Self {
        Self {
            word_count: value.word_count,
            separator: value.separator,
            capitalize_words: value.capitalize_words,
            append_number: value.append_number,
        }
    }
}

impl From<BridgeEncryptRequest> for EncryptRequest {
    fn from(value: BridgeEncryptRequest) -> Self {
        Self {
            input_path: value.input_path,
            output_path: value.output_path,
            password: value.password,
            id_password: value.id_password,
            algorithm: value.algorithm.into(),
        }
    }
}

impl From<BridgeDecryptRequest> for DecryptRequest {
    fn from(value: BridgeDecryptRequest) -> Self {
        Self {
            input_path: value.input_path,
            output_path: value.output_path,
            password: value.password,
            id_password: value.id_password,
        }
    }
}

impl From<BridgeEmbedRequest> for EmbedRequest {
    fn from(value: BridgeEmbedRequest) -> Self {
        Self {
            carrier_path: value.carrier_path,
            payload_path: value.payload_path,
            output_path: value.output_path,
            password: value.password,
            mode: value.mode.into(),
        }
    }
}

impl From<BridgeExtractRequest> for ExtractRequest {
    fn from(value: BridgeExtractRequest) -> Self {
        Self {
            carrier_path: value.carrier_path,
            output_path: value.output_path,
            password: value.password,
            mode: value.mode.into(),
        }
    }
}

impl From<OperationResult> for BridgeOperationResult {
    fn from(value: OperationResult) -> Self {
        Self {
            output_path: value.output_path,
            bytes_processed: value.bytes_processed,
        }
    }
}
