use once_cell::sync::Lazy;
use regex::Regex;

use crate::common::error::{AppError, AppResult};

static EMAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").expect("valid email regex"));

pub fn normalize_email(email: &str) -> AppResult<String> {
    let email = email.trim().to_lowercase();
    if !EMAIL_RE.is_match(&email) {
        return Err(AppError::BadRequest("invalid email".into()));
    }
    Ok(email)
}

pub fn validate_password(password: &str) -> AppResult<()> {
    if password.len() < 8 {
        return Err(AppError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_password_length() {
        assert!(validate_password("long-enough").is_ok());
        assert!(validate_password("short").is_err());
    }
}
