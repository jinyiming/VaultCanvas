use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use common::{VaultError, VaultResult};

const LOWER: &[u8] = b"abcdefghijkmnopqrstuvwxyz";
const UPPER: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
const DIGITS: &[u8] = b"23456789";
const SYMBOLS: &[u8] = b"!@#$%^&*-_=+?";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub length: usize,
    pub use_lowercase: bool,
    pub use_uppercase: bool,
    pub use_digits: bool,
    pub use_symbols: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordGeneratorRequest {
    pub main_password: String,
    pub id_password: String,
    pub length: usize,
    pub use_lowercase: bool,
    pub use_uppercase: bool,
    pub use_digits: bool,
    pub use_symbols: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordStrength {
    pub score: u8,
    pub label: String,
    pub entropy_bits: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassphraseRequest {
    pub word_count: usize,
    pub separator: String,
    pub capitalize_words: bool,
    pub append_number: bool,
}

const WORDS: &[&str] = &[
    "anchor", "atlas", "aurora", "bamboo", "binary", "canyon", "cobalt", "comet", "delta", "ember",
    "falcon", "forest", "galaxy", "harbor", "horizon", "ivory", "jungle", "lantern", "matrix",
    "meadow", "nebula", "onyx", "orchid", "prairie", "quartz", "ripple", "summit", "tundra",
    "umbra", "velvet", "voyage", "willow", "zephyr",
];

pub fn generate_password(policy: PasswordPolicy) -> VaultResult<String> {
    if policy.length == 0 {
        return Err(VaultError::InvalidInput(
            "password length must be greater than 0".into(),
        ));
    }

    let mut alphabet = Vec::new();
    if policy.use_lowercase {
        alphabet.extend_from_slice(LOWER);
    }
    if policy.use_uppercase {
        alphabet.extend_from_slice(UPPER);
    }
    if policy.use_digits {
        alphabet.extend_from_slice(DIGITS);
    }
    if policy.use_symbols {
        alphabet.extend_from_slice(SYMBOLS);
    }

    if alphabet.is_empty() {
        return Err(VaultError::InvalidInput(
            "at least one character set must be enabled".into(),
        ));
    }

    let mut rng = thread_rng();
    let mut output = String::with_capacity(policy.length);
    for _ in 0..policy.length {
        let ch = alphabet
            .choose(&mut rng)
            .ok_or_else(|| VaultError::InvalidInput("failed to choose character".into()))?;
        output.push(*ch as char);
    }
    Ok(output)
}

pub fn generate_password_from_secrets(request: PasswordGeneratorRequest) -> VaultResult<String> {
    let PasswordGeneratorRequest {
        main_password,
        id_password,
        length,
        use_lowercase,
        use_uppercase,
        use_digits,
        use_symbols,
    } = request;

    if length == 0 {
        return Err(VaultError::InvalidInput(
            "password length must be greater than 0".into(),
        ));
    }
    if main_password.trim().is_empty() {
        return Err(VaultError::InvalidInput("main password is required".into()));
    }
    if id_password.trim().is_empty() {
        return Err(VaultError::InvalidInput("id password is required".into()));
    }

    let main_password = Zeroizing::new(main_password);
    let id_password = Zeroizing::new(id_password);

    let mut groups: Vec<&[u8]> = Vec::new();
    let mut alphabet = Vec::new();
    if use_lowercase {
        groups.push(LOWER);
        alphabet.extend_from_slice(LOWER);
    }
    if use_uppercase {
        groups.push(UPPER);
        alphabet.extend_from_slice(UPPER);
    }
    if use_digits {
        groups.push(DIGITS);
        alphabet.extend_from_slice(DIGITS);
    }
    if use_symbols {
        groups.push(SYMBOLS);
        alphabet.extend_from_slice(SYMBOLS);
    }

    if groups.is_empty() {
        return Err(VaultError::InvalidInput(
            "at least one character set must be enabled".into(),
        ));
    }
    if length < groups.len() {
        return Err(VaultError::InvalidInput(
            "password length is too short for selected character sets".into(),
        ));
    }

    let mut output = String::with_capacity(length);
    let mut derived = Zeroizing::new(derived_bytes(&main_password, &id_password, length * 4));
    let mut cursor = 0usize;

    for group in &groups {
        let byte = derived[cursor];
        cursor += 1;
        let index = (byte as usize) % group.len();
        output.push(group[index] as char);
    }

    while output.len() < length {
        let byte = derived[cursor];
        cursor += 1;
        if cursor >= derived.len() {
            derived.extend_from_slice(&derived_bytes(
                &main_password,
                &id_password,
                length * 2 + cursor,
            ));
        }
        let index = (byte as usize) % alphabet.len();
        output.push(alphabet[index] as char);
    }

    let mut chars = output.into_bytes();
    for i in 0..chars.len() {
        let byte = derived[cursor % derived.len()];
        cursor += 1;
        let j = (byte as usize) % chars.len();
        chars.swap(i, j);
    }

    String::from_utf8(chars)
        .map_err(|_| VaultError::InvalidInput("generated password contains invalid utf-8".into()))
}

fn derived_bytes(main_password: &str, id_password: &str, min_len: usize) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(min_len.max(32));
    let mut counter = 0_u32;
    while buffer.len() < min_len {
        let mut hasher = blake3::Hasher::new();
        hasher.update(main_password.as_bytes());
        hasher.update(&[0x1f]);
        hasher.update(id_password.as_bytes());
        hasher.update(&[0x1e]);
        hasher.update(&counter.to_le_bytes());
        buffer.extend_from_slice(hasher.finalize().as_bytes());
        counter = counter.saturating_add(1);
    }
    buffer
}

pub fn score_password(password: &str) -> PasswordStrength {
    let mut pool = 0_u32;
    if password.chars().any(|c| c.is_ascii_lowercase()) {
        pool += LOWER.len() as u32;
    }
    if password.chars().any(|c| c.is_ascii_uppercase()) {
        pool += UPPER.len() as u32;
    }
    if password.chars().any(|c| c.is_ascii_digit()) {
        pool += DIGITS.len() as u32;
    }
    if password.chars().any(|c| SYMBOLS.contains(&(c as u8))) {
        pool += SYMBOLS.len() as u32;
    }

    let entropy = if pool > 0 {
        (password.len() as f32) * (pool as f32).log2()
    } else {
        0.0
    };

    let (score, label) = if entropy >= 96.0 {
        (4, "excellent")
    } else if entropy >= 72.0 {
        (3, "strong")
    } else if entropy >= 56.0 {
        (2, "moderate")
    } else if entropy >= 40.0 {
        (1, "weak")
    } else {
        (0, "very weak")
    };

    PasswordStrength {
        score,
        label: label.into(),
        entropy_bits: entropy,
    }
}

pub fn generate_passphrase(request: PassphraseRequest) -> VaultResult<String> {
    if request.word_count < 3 {
        return Err(VaultError::InvalidInput(
            "passphrase must contain at least 3 words".into(),
        ));
    }

    let mut rng = thread_rng();
    let mut words = Vec::with_capacity(request.word_count);
    for _ in 0..request.word_count {
        let mut word = WORDS
            .choose(&mut rng)
            .ok_or_else(|| VaultError::InvalidInput("word list is empty".into()))?
            .to_string();
        if request.capitalize_words {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                word = first.to_uppercase().collect::<String>() + chars.as_str();
            }
        }
        words.push(word);
    }

    let mut passphrase = words.join(&request.separator);
    if request.append_number {
        let digits = (0..3)
            .map(|_| char::from(b'0' + rand::random::<u8>() % 10))
            .collect::<String>();
        passphrase.push_str(&request.separator);
        passphrase.push_str(&digits);
    }
    Ok(passphrase)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_password_with_requested_length() {
        let password = generate_password(PasswordPolicy {
            length: 20,
            use_lowercase: true,
            use_uppercase: true,
            use_digits: true,
            use_symbols: true,
        })
        .expect("password should be generated");

        assert_eq!(password.len(), 20);
    }

    #[test]
    fn allows_short_password_lengths() {
        let password = generate_password(PasswordPolicy {
            length: 3,
            use_lowercase: true,
            use_uppercase: false,
            use_digits: false,
            use_symbols: false,
        })
        .expect("short password should be generated");

        assert_eq!(password.len(), 3);
    }

    #[test]
    fn scores_stronger_password_higher() {
        let weak = score_password("abc123");
        let strong = score_password("Aq9!Ze7@Lm3#Rt8$");
        assert!(strong.score > weak.score);
    }

    #[test]
    fn generates_deterministic_password_from_secrets() {
        let first = generate_password_from_secrets(PasswordGeneratorRequest {
            main_password: "master".into(),
            id_password: "site-a".into(),
            length: 16,
            use_lowercase: true,
            use_uppercase: true,
            use_digits: true,
            use_symbols: false,
        })
        .expect("derived password should be generated");

        let second = generate_password_from_secrets(PasswordGeneratorRequest {
            main_password: "master".into(),
            id_password: "site-a".into(),
            length: 16,
            use_lowercase: true,
            use_uppercase: true,
            use_digits: true,
            use_symbols: false,
        })
        .expect("derived password should be generated");

        assert_eq!(first, second);
    }

    #[test]
    fn generated_password_contains_all_enabled_sets() {
        let password = generate_password_from_secrets(PasswordGeneratorRequest {
            main_password: "master".into(),
            id_password: "site-a".into(),
            length: 16,
            use_lowercase: true,
            use_uppercase: true,
            use_digits: true,
            use_symbols: true,
        })
        .expect("derived password should be generated");

        assert!(password.chars().any(|c| c.is_ascii_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
        assert!(password.chars().any(|c| SYMBOLS.contains(&(c as u8))));
    }

    #[test]
    fn generates_passphrase_with_separator() {
        let phrase = generate_passphrase(PassphraseRequest {
            word_count: 4,
            separator: "-".into(),
            capitalize_words: true,
            append_number: false,
        })
        .expect("passphrase should be generated");

        assert_eq!(phrase.split('-').count(), 4);
    }
}
