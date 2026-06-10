use axum::http::HeaderMap;

use crate::common::{AppError, AppResult};

pub fn user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.chars().take(512).collect())
}

pub fn device_name_from(headers: &HeaderMap, provided: Option<String>) -> Option<String> {
    provided
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.chars().take(128).collect())
        .or_else(|| user_agent(headers).map(|value| value.chars().take(128).collect()))
}

pub fn clean_key(value: &str, field: &str) -> AppResult<String> {
    let value = value.trim().to_lowercase();
    if value.is_empty() {
        return Err(AppError::BadRequest(format!("{field} is required")));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn uses_explicit_device_name() {
        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_static("browser"));

        assert_eq!(
            device_name_from(&headers, Some("phone".into())).unwrap(),
            "phone"
        );
    }

    #[test]
    fn falls_back_to_user_agent_for_device_name() {
        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_static("browser"));

        assert_eq!(device_name_from(&headers, None).unwrap(), "browser");
    }
}

