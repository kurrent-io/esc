/// The version of the ESC client, set at build time
pub const CLIENT_VERSION: &str = match option_env!("ESC_CLIENT_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"), // fallback to base crate version
};

/// Creates the user agent string for HTTP requests
pub fn user_agent() -> String {
    format!("esc-client/{}", CLIENT_VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_agent_format() {
        let ua = user_agent();
        assert!(ua.starts_with("esc-client/"));
        // Should contain either the environment variable version or fallback version
        assert!(ua.len() > "esc-client/".len());
    }

    #[test]
    fn test_client_version_exists() {
        // CLIENT_VERSION should always have a value
        assert!(!CLIENT_VERSION.is_empty());
    }
}
