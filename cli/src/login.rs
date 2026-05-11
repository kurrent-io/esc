use esc_client_base::build_http_client;
use esc_client_base::identity::operations;
use esc_client_base::identity::TokenConfig;
use esc_client_base::Token;
use rand::RngCore;
use sha2::Digest;
use sha2::Sha256;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

const RESPONSE_BODY: &str =
    "<html><body><h2>esc login complete.</h2><p>You may close this window.</p></body></html>";

pub struct LoginOptions {
    pub timeout: Duration,
}

pub async fn run_login(
    token_config: &TokenConfig,
    opts: LoginOptions,
) -> Result<Token, Box<dyn std::error::Error>> {
    let verifier = generate_verifier();
    let challenge = code_challenge_s256(&verifier);
    let state = random_url_safe(16);

    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let bound = listener.local_addr()?;
    listener.set_nonblocking(false)?;

    let redirect_uri = format!("http://localhost:{}/callback", bound.port());

    let auth_url = build_authorize_url(token_config, &redirect_uri, &state, &challenge);

    println!("Open the following URL in your browser to log in:");
    println!("  {}", auth_url);

    let join = tokio::task::spawn_blocking(move || wait_for_callback(listener, opts.timeout))
        .await?;
    let (code, returned_state) = join.map_err(|err| -> Box<dyn std::error::Error> {
        format!("{}", err).into()
    })?;

    if returned_state != state {
        return Err("OAuth state mismatch".into());
    }

    let client = build_http_client();
    let token =
        operations::exchange_code(&client, token_config, &code, &verifier, &redirect_uri).await?;

    Ok(token)
}

fn build_authorize_url(
    config: &TokenConfig,
    redirect_uri: &str,
    state: &str,
    code_challenge: &str,
) -> String {
    let params = [
        ("client_id", config.client_id.as_str()),
        ("redirect_uri", redirect_uri),
        ("response_type", "code"),
        ("provider", "authkit"),
        ("scope", "openid profile email offline_access"),
        ("state", state),
        ("code_challenge", code_challenge),
        ("code_challenge_method", "S256"),
    ];

    let query = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    format!("{}/user_management/authorize?{}", config.identity_url, query)
}

fn wait_for_callback(
    listener: TcpListener,
    timeout: Duration,
) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    listener.set_nonblocking(false)?;
    let deadline = std::time::Instant::now() + timeout;

    loop {
        if std::time::Instant::now() >= deadline {
            return Err("Timed out waiting for OAuth callback".into());
        }
        listener.set_nonblocking(true)?;
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream.set_read_timeout(Some(Duration::from_secs(5)))?;
                let mut buf = [0u8; 8192];
                let n = stream.read(&mut buf)?;
                let req = std::str::from_utf8(&buf[..n])?;
                let target = parse_request_target(req)
                    .ok_or("Could not parse callback request line")?;
                let (code, state) = parse_callback_query(target)?;

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    RESPONSE_BODY.len(),
                    RESPONSE_BODY,
                );
                stream.write_all(response.as_bytes())?;
                stream.flush()?;
                return Ok((code, state));
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(err) => return Err(Box::new(err)),
        }
    }
}

fn parse_request_target(req: &str) -> Option<&str> {
    let first_line = req.lines().next()?;
    let mut parts = first_line.split_whitespace();
    let _method = parts.next()?;
    parts.next()
}

fn parse_callback_query(
    target: &str,
) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    let query = target.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut code = None;
    let mut state = None;
    for pair in query.split('&') {
        let (k, v) = match pair.split_once('=') {
            Some(kv) => kv,
            None => continue,
        };
        let decoded = url_decode(v);
        match k {
            "code" => code = Some(decoded),
            "state" => state = Some(decoded),
            "error" => {
                return Err(format!("Authorization server returned error: {}", decoded).into())
            }
            _ => {}
        }
    }
    Ok((
        code.ok_or("missing `code` in callback")?,
        state.ok_or("missing `state` in callback")?,
    ))
}

fn url_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        let b = *byte;
        let unreserved = b.is_ascii_alphanumeric()
            || b == b'-'
            || b == b'_'
            || b == b'.'
            || b == b'~';
        if unreserved {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

fn url_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hi = hex_val(bytes[i + 1]);
                let lo = hex_val(bytes[i + 2]);
                match (hi, lo) {
                    (Some(h), Some(l)) => {
                        out.push((h << 4) | l);
                        i += 3;
                    }
                    _ => {
                        out.push(bytes[i]);
                        i += 1;
                    }
                }
            }
            other => {
                out.push(other);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn generate_verifier() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64_url_no_pad(&bytes)
}

fn code_challenge_s256(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    base64_url_no_pad(&hasher.finalize())
}

fn random_url_safe(num_bytes: usize) -> String {
    let mut bytes = vec![0u8; num_bytes];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64_url_no_pad(&bytes)
}

fn base64_url_no_pad(data: &[u8]) -> String {
    base64::encode_config(data, base64::URL_SAFE_NO_PAD)
}
