use crate::config::Settings;
use crate::errors::{Result, StoreError};
use crate::store::TokenStore;
use crate::store::TokenValidator;
use esc_client_base::identity::TokenConfig;
use std::path::{Path, PathBuf};

// TokenKind selects which on-disk token namespace a store reads and writes.
pub enum TokenKind {
    // Pkce tokens come from the browser-based `esc login` flow.
    Pkce,
    // Legacy tokens come from the password / --refresh-token mechanics.
    Legacy,
}

fn get_esc_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        StoreError::from_message("unsupported platform - no home directory".to_string())
    })?;
    Ok(home_dir.join(".esc"))
}

fn token_dir_for(esc_dir: &Path, kind: TokenKind) -> PathBuf {
    match kind {
        TokenKind::Legacy => esc_dir.join("tokens"),
        TokenKind::Pkce => esc_dir.join("pkce-tokens"),
    }
}

pub async fn load_settings() -> Result<Settings> {
    let esc_dir = get_esc_dir()?;
    let settings_file = esc_dir.join("settings.toml");
    if !settings_file.exists() {
        info!("Creating initial ESC settings file...");
        let settings = Settings::default();
        settings.persist(&settings_file).await?;
        Ok(settings)
    } else {
        Settings::load_settings(settings_file).await
    }
}

// token_store builds a TokenStore for the legacy token namespace. Retained as a
// thin alias so existing call sites don't churn; new code should prefer
// token_store_kind.
pub async fn token_store(token_config: TokenConfig) -> Result<TokenStore> {
    token_store_kind(TokenKind::Legacy, token_config).await
}

// token_store_kind builds a TokenStore rooted at the directory for the given kind.
pub async fn token_store_kind(kind: TokenKind, token_config: TokenConfig) -> Result<TokenStore> {
    let esc_dir = get_esc_dir()?;
    let token_dir = token_dir_for(&esc_dir, kind);
    let validator = TokenValidator::new_from_rsa_pem(&token_config.public_key)?;
    let ts = TokenStore::new(&token_dir, token_config, validator).map_err(|err| {
        StoreError::new("error creating token store").source(Box::new(err))
    })?;
    Ok(ts)
}
