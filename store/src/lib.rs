#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

pub mod config;
pub mod errors;
pub mod store;
pub mod typical;

// pub use typical::settings();

pub use config::Settings;
pub use errors::StoreError;
pub use store::prompt_for_otp;
pub use store::TokenStore;
pub use store::TokenValidator;

pub use typical::load_settings;
pub use typical::token_store;
pub use typical::token_store_kind;
pub use typical::TokenKind;
