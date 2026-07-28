#[derive(Clone)]
pub struct TokenConfig {
    // Identifies the recipients the JWT is intended for.
    // See https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.3
    pub audience: String,
    // Identifies the client, but not the user. Legacy (Auth0) client used for
    // password / refresh-token / MFA flows.
    pub client_id: String,
    // WorkOS client id used for the interactive login flow (authorize + code
    // exchange against idp_um_url). Override per environment via the
    // token-config profile.
    pub idp_client_id: String,
    // Base URL of the legacy (Auth0) identity API. Used for password / refresh-token flows.
    pub identity_url: String,
    // Base URL of the WorkOS AuthKit token endpoint. Used for the Service Account
    // client-credentials grant ({idp_kit_url}/oauth2/token). This is the
    // tenant-specific AuthKit domain, NOT api.workos.com (which 404s on
    // /oauth2/token). Override per environment via the token-config profile.
    pub idp_kit_url: String,
    // Base URL of the WorkOS User Management API. Used by the interactive login
    // flow: {idp_um_url}/user_management/authorize (browser) and
    // {idp_um_url}/user_management/authenticate (code exchange). Override per
    // environment via the token-config profile.
    pub idp_um_url: String,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            audience: "https://api.eventstore.cloud".to_owned(),
            client_id: "OraYp3cFES9O8aWuQtnqi1A7m534iTwt".to_owned(),
            idp_client_id: "client_01KNKQ1Q05JFK2HMRSV3H2QFF0".to_owned(),
            identity_url: "https://identity.eventstore.com".to_owned(),
            idp_kit_url: "https://auth.kurrent.io".to_owned(),
            idp_um_url: "https://api.auth.kurrent.io".to_owned(),
        }
    }
}
