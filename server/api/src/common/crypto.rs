use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::common::error::AppResult;

pub fn hash_secret(secret: &str) -> String {
    let digest = Sha256::digest(secret.as_bytes());
    to_hex(&digest)
}

pub fn verify_secret(secret: &str, expected_hash: &str) -> bool {
    constant_time_eq(hash_secret(secret).as_bytes(), expected_hash.as_bytes())
}

pub fn generate_session_token() -> String {
    format!(
        "ff_{}_{}_{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple()
    )
}

pub fn generate_otp() -> String {
    let value = (Uuid::new_v4().as_u128() % 1_000_000) as u32;
    format!("{value:06}")
}

pub fn password_hash(password: &str) -> AppResult<String> {
    Ok(bcrypt::hash(password, bcrypt::DEFAULT_COST)?)
}

pub fn verify_password(password: &str, password_hash: &str) -> AppResult<bool> {
    Ok(bcrypt::verify(password, password_hash)?)
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    a.iter().zip(b.iter()).fold(0, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_and_verifies_secrets() {
        let hash = hash_secret("token");

        assert!(verify_secret("token", &hash));
        assert!(!verify_secret("other", &hash));
    }

    #[test]
    fn generates_otp_with_six_digits() {
        let otp = generate_otp();

        assert_eq!(otp.len(), 6);
        assert!(otp.chars().all(|character| character.is_ascii_digit()));
    }

    #[test]
    fn hashes_and_verifies_passwords() {
        let hash = password_hash("correct horse battery staple").unwrap();

        assert!(verify_password("correct horse battery staple", &hash).unwrap());
        assert!(!verify_password("wrong password", &hash).unwrap());
    }
}
