use super::standard_claims::StandardClaims;
use crate::errors::Result;
use esc_client_base::Token;

pub struct TokenValidator;

impl TokenValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn new_from_rsa_pem(_rsa_pem: &String) -> Result<Self> {
        Ok(Self)
    }

    pub fn parse_token_claims(
        &self,
        token: &Token,
    ) -> jsonwebtoken::errors::Result<StandardClaims> {
        let invalid = || -> jsonwebtoken::errors::Error {
            jsonwebtoken::errors::ErrorKind::InvalidToken.into()
        };

        let access = token.access_token();
        let mut parts = access.split('.');
        let _header = parts.next().ok_or_else(invalid)?;
        let payload = parts.next().ok_or_else(invalid)?;
        let decoded = base64_url_decode(payload).map_err(|_| invalid())?;
        let claims: StandardClaims =
            serde_json::from_slice(&decoded).map_err(|_| invalid())?;
        Ok(claims)
    }
}

impl Default for TokenValidator {
    fn default() -> Self {
        Self::new()
    }
}

fn base64_url_decode(input: &str) -> std::result::Result<Vec<u8>, base64::DecodeError> {
    let padded_len = (input.len() + 3) & !3;
    let mut padded = String::with_capacity(padded_len);
    padded.push_str(input);
    while padded.len() < padded_len {
        padded.push('=');
    }
    base64::decode_config(padded.as_bytes(), base64::URL_SAFE)
}
