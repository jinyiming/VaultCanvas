use std::env;
use std::process;

use security_service::{
    decrypt_file_command, embed_file_command, encrypt_file_command, extract_file_command,
    DecryptRequest, EmbedRequest, EncryptRequest, EncryptionAlgorithm, ExtractRequest, StegoMode,
    VaultResult,
};

fn main() {
    if let Err(message) = run() {
        eprintln!("{message}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args();
    let bin = args.next().unwrap_or_else(|| "bench_runner".to_string());
    let command = args.next().ok_or_else(|| usage(&bin))?;

    match command.as_str() {
        "crypto-encrypt" => run_crypto_encrypt(args.collect()),
        "crypto-decrypt" => run_crypto_decrypt(args.collect()),
        "stego-embed" => run_stego_embed(args.collect()),
        "stego-extract" => run_stego_extract(args.collect()),
        _ => Err(usage(&bin)),
    }
}

fn run_crypto_encrypt(args: Vec<String>) -> Result<(), String> {
    if args.len() != 4 {
        return Err("usage: bench_runner crypto-encrypt <input> <output> <main> <id>".into());
    }
    map_vault_result(encrypt_file_command(EncryptRequest {
        input_path: args[0].clone(),
        output_path: args[1].clone(),
        password: args[2].clone(),
        id_password: args[3].clone(),
        algorithm: EncryptionAlgorithm::Aes256Gcm,
    }))
}

fn run_crypto_decrypt(args: Vec<String>) -> Result<(), String> {
    if args.len() != 4 {
        return Err("usage: bench_runner crypto-decrypt <input> <output> <main> <id>".into());
    }
    map_vault_result(decrypt_file_command(DecryptRequest {
        input_path: args[0].clone(),
        output_path: args[1].clone(),
        password: args[2].clone(),
        id_password: args[3].clone(),
    }))
}

fn run_stego_embed(args: Vec<String>) -> Result<(), String> {
    if args.len() != 4 {
        return Err("usage: bench_runner stego-embed <carrier> <payload> <output> <password>".into());
    }
    map_vault_result(embed_file_command(EmbedRequest {
        carrier_path: args[0].clone(),
        payload_path: args[1].clone(),
        output_path: args[2].clone(),
        password: args[3].clone(),
        mode: StegoMode::Append,
    }))
}

fn run_stego_extract(args: Vec<String>) -> Result<(), String> {
    if args.len() != 3 {
        return Err("usage: bench_runner stego-extract <carrier> <output> <password>".into());
    }
    map_vault_result(extract_file_command(ExtractRequest {
        carrier_path: args[0].clone(),
        output_path: args[1].clone(),
        password: args[2].clone(),
        mode: StegoMode::Append,
    }))
}

fn map_vault_result(result: VaultResult<security_service::OperationResult>) -> Result<(), String> {
    result.map(|_| ()).map_err(|err| err.to_string())
}

fn usage(bin: &str) -> String {
    format!(
        "usage:\n  {bin} crypto-encrypt <input> <output> <main> <id>\n  {bin} crypto-decrypt <input> <output> <main> <id>\n  {bin} stego-embed <carrier> <payload> <output> <password>\n  {bin} stego-extract <carrier> <output> <password>"
    )
}
