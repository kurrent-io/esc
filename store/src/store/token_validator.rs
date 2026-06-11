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
        let decoded =
            base64::decode_config(payload, base64::URL_SAFE_NO_PAD).map_err(|_| invalid())?;
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
