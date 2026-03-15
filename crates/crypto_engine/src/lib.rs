use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use aes_gcm::aead::{AeadInPlace, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce, Tag};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use zeroize::Zeroizing;

use common::{
    atomic_write, atomic_write_with, crypto_error, io_error, OperationResult, VaultError,
    VaultResult,
};

const FILE_HEADER: &[u8] = b"SECURE_ENC_V5";
const FILE_VERSION: u8 = 5;
const ARGON_TIME_COST: u32 = 2;
const ARGON_MEMORY_COST: u32 = 64 * 1024;
const ARGON_PARALLELISM: u32 = 2;
const KEY_LEN: usize = 32;
const SALT_LEN: usize = 24;
const IV_LEN: usize = 12;
const MAC_LEN: usize = 16;
const METADATA_NONCE_LEN: usize = 16;
const SHA3_LEN: usize = 32;
const FIXED_PREFIX_LEN: usize =
    FILE_HEADER.len() + 1 + SALT_LEN + IV_LEN + METADATA_NONCE_LEN + MAC_LEN + SHA3_LEN + SHA3_LEN;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptRequest {
    pub input_path: String,
    pub output_path: String,
    pub password: String,
    pub id_password: String,
    pub algorithm: EncryptionAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptRequest {
    pub input_path: String,
    pub output_path: String,
    pub password: String,
    pub id_password: String,
}

pub fn encrypt_file(request: EncryptRequest) -> VaultResult<OperationResult> {
    let EncryptRequest {
        input_path,
        output_path,
        password,
        id_password,
        algorithm: _,
    } = request;

    if !Path::new(&input_path).is_file() {
        return Err(VaultError::InvalidInput("input file does not exist".into()));
    }

    let password = Zeroizing::new(password);
    let id_password = Zeroizing::new(id_password);

    let mut plaintext = fs::read(&input_path).map_err(io_error)?;
    let bytes_processed = plaintext.len() as u64;
    let salt = random_bytes::<SALT_LEN>();
    let iv = random_bytes::<IV_LEN>();
    let metadata_nonce = random_bytes::<METADATA_NONCE_LEN>();
    let key = derive_key(password.as_bytes(), &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| crypto_error("cipher init failed"))?;
    let tag = cipher
        .encrypt_in_place_detached(Nonce::from_slice(&iv), &metadata_nonce, &mut plaintext)
        .map_err(|_| crypto_error("encryption failed"))?;

    let metadata = [&salt[..], &iv[..], &metadata_nonce[..], tag.as_slice()].concat();
    let id_hash = Sha3_256::digest(id_password.as_bytes());
    let metadata_sign = metadata_signature(&metadata, id_hash.as_slice());

    atomic_write_with(&output_path, |file| {
        file.write_all(FILE_HEADER).map_err(io_error)?;
        file.write_all(&[FILE_VERSION]).map_err(io_error)?;
        file.write_all(&salt).map_err(io_error)?;
        file.write_all(&iv).map_err(io_error)?;
        file.write_all(&metadata_nonce).map_err(io_error)?;
        file.write_all(tag.as_slice()).map_err(io_error)?;
        file.write_all(metadata_sign.as_slice()).map_err(io_error)?;
        file.write_all(id_hash.as_slice()).map_err(io_error)?;
        file.write_all(&plaintext).map_err(io_error)?;
        Ok(())
    })?;
    Ok(OperationResult {
        output_path,
        bytes_processed,
    })
}

pub fn decrypt_file(request: DecryptRequest) -> VaultResult<OperationResult> {
    let DecryptRequest {
        input_path,
        output_path,
        password,
        id_password,
    } = request;

    if !Path::new(&input_path).is_file() {
        return Err(VaultError::InvalidInput("input file does not exist".into()));
    }

    let password = Zeroizing::new(password);
    let id_password = Zeroizing::new(id_password);
    let mut input = fs::File::open(&input_path).map_err(io_error)?;
    let input_len = input.metadata().map_err(io_error)?.len() as usize;
    if input_len < FIXED_PREFIX_LEN {
        return Err(VaultError::InvalidInput(
            "encrypted file is too short".into(),
        ));
    }

    let mut header = [0_u8; FILE_HEADER.len()];
    input.read_exact(&mut header).map_err(io_error)?;
    if header != FILE_HEADER {
        return Err(VaultError::InvalidInput(
            "invalid encrypted file header".into(),
        ));
    }
    let mut version = [0_u8; 1];
    input.read_exact(&mut version).map_err(io_error)?;
    if version[0] != FILE_VERSION {
        return Err(VaultError::InvalidInput("unsupported file version".into()));
    }

    let mut salt = [0_u8; SALT_LEN];
    let mut iv = [0_u8; IV_LEN];
    let mut metadata_nonce = [0_u8; METADATA_NONCE_LEN];
    let mut tag = [0_u8; MAC_LEN];
    let mut metadata_sign = [0_u8; SHA3_LEN];
    let mut id_hash = [0_u8; SHA3_LEN];
    input.read_exact(&mut salt).map_err(io_error)?;
    input.read_exact(&mut iv).map_err(io_error)?;
    input.read_exact(&mut metadata_nonce).map_err(io_error)?;
    input.read_exact(&mut tag).map_err(io_error)?;
    input.read_exact(&mut metadata_sign).map_err(io_error)?;
    input.read_exact(&mut id_hash).map_err(io_error)?;

    let ciphertext_len = input_len - FIXED_PREFIX_LEN;
    let mut plaintext = vec![0_u8; ciphertext_len];
    input.read_exact(&mut plaintext).map_err(io_error)?;

    let expected_id_hash = Sha3_256::digest(id_password.as_bytes());
    if id_hash != expected_id_hash.as_slice() {
        return Err(VaultError::InvalidInput(
            "ID password verification failed".into(),
        ));
    }

    let metadata = [&salt[..], &iv[..], &metadata_nonce[..], &tag[..]].concat();
    let computed_sign = metadata_signature(&metadata, &id_hash);
    if metadata_sign != computed_sign.as_slice() {
        return Err(VaultError::InvalidInput(
            "file integrity verification failed".into(),
        ));
    }

    let key = derive_key(password.as_bytes(), &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| crypto_error("cipher init failed"))?;
    let tag = Tag::from_slice(&tag);
    cipher
        .decrypt_in_place_detached(Nonce::from_slice(&iv), &metadata_nonce, &mut plaintext, tag)
        .map_err(|_| crypto_error("decryption failed or password incorrect"))?;

    atomic_write(&output_path, &plaintext)?;
    Ok(OperationResult {
        output_path,
        bytes_processed: plaintext.len() as u64,
    })
}

fn derive_key(password: &[u8], salt: &[u8]) -> VaultResult<[u8; KEY_LEN]> {
    let params = Params::new(
        ARGON_MEMORY_COST,
        ARGON_TIME_COST,
        ARGON_PARALLELISM,
        Some(KEY_LEN),
    )
    .map_err(|e| crypto_error(format!("invalid argon2 params: {e}")))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0_u8; KEY_LEN];
    argon2
        .hash_password_into(password, salt, &mut key)
        .map_err(|_| crypto_error("key derivation failed"))?;
    Ok(key)
}

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut value = [0_u8; N];
    rand::thread_rng().fill_bytes(&mut value);
    value
}

fn metadata_signature(metadata: &[u8], id_hash: &[u8]) -> [u8; SHA3_LEN] {
    let mut hasher = Sha3_256::new();
    hasher.update(metadata);
    hasher.update(id_hash);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn encrypts_and_decrypts_round_trip() {
        let base = std::env::temp_dir().join(unique_name("vaultcanvas_crypto"));
        fs::create_dir_all(&base).expect("temp dir should be created");

        let input = base.join("sample.txt");
        let encrypted = base.join("sample.enc");
        let output = base.join("sample.dec");
        fs::write(&input, b"vaultcanvas test payload").expect("input should be written");

        encrypt_file(EncryptRequest {
            input_path: input.to_string_lossy().into_owned(),
            output_path: encrypted.to_string_lossy().into_owned(),
            password: "correct-horse-main-pass".into(),
            id_password: "id-secret".into(),
            algorithm: EncryptionAlgorithm::Aes256Gcm,
        })
        .expect("encryption should succeed");

        decrypt_file(DecryptRequest {
            input_path: encrypted.to_string_lossy().into_owned(),
            output_path: output.to_string_lossy().into_owned(),
            password: "correct-horse-main-pass".into(),
            id_password: "id-secret".into(),
        })
        .expect("decryption should succeed");

        let restored = fs::read(&output).expect("output should be readable");
        assert_eq!(restored, b"vaultcanvas test payload");

        let _ = fs::remove_dir_all(base);
    }

    fn unique_name(prefix: &str) -> String {
        let ticks = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();
        format!("{prefix}_{ticks}")
    }
}
