use super::authorization::StaticTokenAuthorizer;
use super::client::Client;
use crate::identity::Token;
use crate::requests::RequestSender;

pub fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(&crate::version::user_agent())
        .build()
        .expect("Failed to build HTTP client")
}

fn static_token_client(token: Token) -> Client {
    let authorizer = StaticTokenAuthorizer { token };
    Client {
        authorization: std::sync::Arc::new(authorizer),
        base_url: String::new(),
        sender: RequestSender::new(build_http_client(), None),
    }
}
