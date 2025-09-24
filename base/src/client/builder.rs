pub fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(crate::version::user_agent())
        .build()
        .expect("Failed to build HTTP client")
}
