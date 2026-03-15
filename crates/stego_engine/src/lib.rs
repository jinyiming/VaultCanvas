use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use aes::Aes256;
use cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use common::{atomic_write_with, crypto_error, io_error, OperationResult, VaultError, VaultResult};

type Aes256CbcEnc = cbc::Encryptor<Aes256>;
type Aes256CbcDec = cbc::Decryptor<Aes256>;

const APPEND_MAGIC: &[u8; 4] = b"\x89STE";
const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const CHECKSUM_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;
const HEADER_LEN: usize = APPEND_MAGIC.len() + 4 + IV_LEN + SALT_LEN;
const SCAN_CHUNK: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StegoMode {
    Append,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedRequest {
    pub carrier_path: String,
    pub payload_path: String,
    pub output_path: String,
    pub password: String,
    pub mode: StegoMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractRequest {
    pub carrier_path: String,
    pub output_path: String,
    pub password: String,
    pub mode: StegoMode,
}

pub fn embed_file(request: EmbedRequest) -> VaultResult<OperationResult> {
    let EmbedRequest {
        carrier_path,
        payload_path,
        output_path,
        password,
        mode,
    } = request;
    let password = Zeroizing::new(password);
    match mode {
        StegoMode::Append => embed_append(
            &carrier_path,
            &payload_path,
            &output_path,
            password.as_bytes(),
        ),
    }
}

pub fn extract_file(request: ExtractRequest) -> VaultResult<OperationResult> {
    let ExtractRequest {
        carrier_path,
        output_path,
        password,
        mode,
    } = request;
    let password = Zeroizing::new(password);
    match mode {
        StegoMode::Append => extract_append(&carrier_path, &output_path, password.as_bytes()),
    }
}

fn embed_append(
    carrier_path: &str,
    payload_path: &str,
    output_path: &str,
    password: &[u8],
) -> VaultResult<OperationResult> {
    if !Path::new(carrier_path).is_file() {
        return Err(VaultError::InvalidInput(
            "carrier file does not exist".into(),
        ));
    }
    if !Path::new(payload_path).is_file() {
        return Err(VaultError::InvalidInput(
            "payload file does not exist".into(),
        ));
    }

    let payload_size = fs::metadata(payload_path).map_err(io_error)?.len();
    let compressed = compress_file(payload_path)?;
    let salt = random_bytes::<SALT_LEN>();
    let iv = random_bytes::<IV_LEN>();
    let key = derive_key(password, &salt);
    let encrypted = encrypt_payload(compressed, &key, &iv)?;
    let checksum = Sha256::digest(&encrypted);

    atomic_write_with(output_path, |file| {
        let mut carrier = fs::File::open(carrier_path).map_err(io_error)?;
        std::io::copy(&mut carrier, file).map_err(io_error)?;
        file.write_all(APPEND_MAGIC).map_err(io_error)?;
        file.write_all(&(encrypted.len() as u32).to_be_bytes())
            .map_err(io_error)?;
        file.write_all(&iv).map_err(io_error)?;
        file.write_all(&salt).map_err(io_error)?;
        file.write_all(&encrypted).map_err(io_error)?;
        file.write_all(checksum.as_slice()).map_err(io_error)?;
        Ok(())
    })?;
    Ok(OperationResult {
        output_path: output_path.to_string(),
        bytes_processed: payload_size,
    })
}

fn extract_append(
    carrier_path: &str,
    output_path: &str,
    password: &[u8],
) -> VaultResult<OperationResult> {
    if !Path::new(carrier_path).is_file() {
        return Err(VaultError::InvalidInput(
            "carrier file does not exist".into(),
        ));
    }

    let mut carrier = fs::File::open(carrier_path).map_err(io_error)?;
    let carrier_len = carrier.metadata().map_err(io_error)?.len() as usize;
    if carrier_len < HEADER_LEN + CHECKSUM_LEN {
        return Err(VaultError::InvalidInput("carrier file is too short".into()));
    }

    let start = find_last_marker(&mut carrier, APPEND_MAGIC)?
        .ok_or_else(|| VaultError::InvalidInput("hidden payload not found".into()))?;

    let header_start = start + APPEND_MAGIC.len() as u64;
    let minimum_end = header_start as usize + 4 + IV_LEN + SALT_LEN + CHECKSUM_LEN;
    if carrier_len < minimum_end {
        return Err(VaultError::InvalidInput(
            "hidden payload is incomplete".into(),
        ));
    }

    carrier
        .seek(SeekFrom::Start(header_start))
        .map_err(io_error)?;

    let mut encrypted_len_bytes = [0_u8; 4];
    let mut iv = [0_u8; IV_LEN];
    let mut salt = [0_u8; SALT_LEN];
    carrier
        .read_exact(&mut encrypted_len_bytes)
        .map_err(io_error)?;
    carrier.read_exact(&mut iv).map_err(io_error)?;
    carrier.read_exact(&mut salt).map_err(io_error)?;

    let encrypted_len = u32::from_be_bytes(encrypted_len_bytes) as usize;
    let encrypted_start = header_start as usize + 4 + IV_LEN + SALT_LEN;
    let encrypted_end = encrypted_start
        .checked_add(encrypted_len)
        .ok_or_else(|| VaultError::InvalidInput("hidden payload is malformed".into()))?;
    let checksum_end = encrypted_end
        .checked_add(CHECKSUM_LEN)
        .ok_or_else(|| VaultError::InvalidInput("hidden payload is malformed".into()))?;

    if carrier_len < checksum_end {
        return Err(VaultError::InvalidInput(
            "hidden payload is truncated".into(),
        ));
    }

    let mut encrypted = vec![0_u8; encrypted_len];
    carrier.read_exact(&mut encrypted).map_err(io_error)?;
    let mut checksum = [0_u8; CHECKSUM_LEN];
    carrier.read_exact(&mut checksum).map_err(io_error)?;

    let actual_checksum = Sha256::digest(&encrypted);
    if checksum != actual_checksum.as_slice() {
        return Err(VaultError::InvalidInput(
            "hidden payload checksum failed".into(),
        ));
    }

    let key = derive_key(password, &salt);
    let decrypted = decrypt_payload(encrypted, &key, &iv)?;
    let restored_size = decompress_to_output(output_path, &decrypted)?;
    Ok(OperationResult {
        output_path: output_path.to_string(),
        bytes_processed: restored_size,
    })
}

fn derive_key(password: &[u8], salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0_u8; KEY_LEN];
    pbkdf2_hmac::<Sha1>(password, salt, PBKDF2_ITERATIONS, &mut key);
    key
}

fn encrypt_payload(
    mut bytes: Vec<u8>,
    key: &[u8; KEY_LEN],
    iv: &[u8; IV_LEN],
) -> VaultResult<Vec<u8>> {
    let cipher = Aes256CbcEnc::new(key.into(), iv.into());
    let source_len = bytes.len();
    bytes.resize(source_len + 16, 0);
    let encrypted_len = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut bytes, source_len)
        .map_err(|_| crypto_error("payload encryption failed"))?
        .len();
    bytes.truncate(encrypted_len);
    Ok(bytes)
}

fn decrypt_payload(
    mut bytes: Vec<u8>,
    key: &[u8; KEY_LEN],
    iv: &[u8; IV_LEN],
) -> VaultResult<Vec<u8>> {
    let cipher = Aes256CbcDec::new(key.into(), iv.into());
    let decrypted_len = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut bytes)
        .map_err(|_| crypto_error("payload decryption failed or password incorrect"))?
        .len();
    bytes.truncate(decrypted_len);
    Ok(bytes)
}

fn compress_file(path: &str) -> VaultResult<Vec<u8>> {
    let mut input = fs::File::open(path).map_err(io_error)?;
    let input_len = input.metadata().map_err(io_error)?.len();
    let initial_capacity = estimated_compressed_capacity(input_len);
    let mut encoder = ZlibEncoder::new(Vec::with_capacity(initial_capacity), Compression::best());
    std::io::copy(&mut input, &mut encoder).map_err(io_error)?;
    encoder.finish().map_err(io_error)
}

fn estimated_compressed_capacity(input_len: u64) -> usize {
    let base = input_len
        .min(usize::MAX as u64)
        .try_into()
        .unwrap_or(usize::MAX);
    base.saturating_add(64 * 1024)
}

fn decompress_to_output(output_path: &str, bytes: &[u8]) -> VaultResult<u64> {
    atomic_write_with(output_path, |file| {
        let mut decoder = ZlibDecoder::new(bytes);
        std::io::copy(&mut decoder, file)
            .map_err(io_error)
            .map(|_| ())
    })?;

    fs::metadata(output_path)
        .map_err(io_error)
        .map(|meta| meta.len())
}

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut value = [0_u8; N];
    rand::thread_rng().fill_bytes(&mut value);
    value
}

fn find_last_marker(file: &mut fs::File, marker: &[u8]) -> VaultResult<Option<u64>> {
    let file_len = file.metadata().map_err(io_error)?.len() as usize;
    if file_len < marker.len() {
        return Ok(None);
    }

    let overlap = marker.len().saturating_sub(1);
    let mut end = file_len;
    let mut suffix = Vec::with_capacity(overlap);
    let mut chunk = vec![0_u8; SCAN_CHUNK];
    let mut window = Vec::with_capacity(SCAN_CHUNK + overlap);

    while end > 0 {
        let start = end.saturating_sub(SCAN_CHUNK);
        let chunk_len = end - start;

        file.seek(SeekFrom::Start(start as u64)).map_err(io_error)?;
        file.read_exact(&mut chunk[..chunk_len]).map_err(io_error)?;

        window.clear();
        window.extend_from_slice(&chunk[..chunk_len]);
        window.extend_from_slice(&suffix);
        if let Some(index) = window
            .windows(marker.len())
            .rposition(|slice| slice == marker)
        {
            if index < chunk_len {
                return Ok(Some((start + index) as u64));
            }
        }

        suffix.clear();
        let keep = overlap.min(chunk_len);
        suffix.extend_from_slice(&chunk[..keep]);
        end = start;
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn embeds_and_extracts_round_trip() {
        let base = std::env::temp_dir().join(unique_name("vaultcanvas_stego"));
        fs::create_dir_all(&base).expect("temp dir should be created");

        let carrier = base.join("carrier.bin");
        let payload = base.join("secret.txt");
        let embedded = base.join("carrier_with_payload.bin");
        let extracted = base.join("restored.txt");

        fs::write(&carrier, b"carrier-bytes").expect("carrier should be written");
        fs::write(&payload, b"hidden payload").expect("payload should be written");

        embed_file(EmbedRequest {
            carrier_path: carrier.to_string_lossy().into_owned(),
            payload_path: payload.to_string_lossy().into_owned(),
            output_path: embedded.to_string_lossy().into_owned(),
            password: "strong-password".into(),
            mode: StegoMode::Append,
        })
        .expect("embed should succeed");

        extract_file(ExtractRequest {
            carrier_path: embedded.to_string_lossy().into_owned(),
            output_path: extracted.to_string_lossy().into_owned(),
            password: "strong-password".into(),
            mode: StegoMode::Append,
        })
        .expect("extract should succeed");

        let restored = fs::read(&extracted).expect("extracted should be readable");
        assert_eq!(restored, b"hidden payload");

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
