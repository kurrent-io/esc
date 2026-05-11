#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Token {
    pub(crate) access_token: String,
    pub(crate) refresh_token: Option<String>,
    #[serde(default)]
    pub(crate) scope: String,
    #[serde(default)]
    pub(crate) expires_in: i64,
    #[serde(default = "default_token_type")]
    pub(crate) token_type: String,
}

fn default_token_type() -> String {
    "Bearer".to_string()
}

impl Token {
    pub fn refresh_token(&self) -> Option<&String> {
        self.refresh_token.as_ref()
    }

    pub fn access_token(&self) -> &str {
        self.access_token.as_str()
    }

    pub fn authorization_header(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }

    pub fn update_access_token(self, access_token: &str) -> Self {
        Token {
            access_token: access_token.to_string(),
            ..self
        }
    }
}
