use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait RequestObserver {
    fn on_request(&self, method: &str, url: &str, body: &str);
    fn on_response(&self, status: &str, body: &str);
}

#[derive(Clone)]
pub struct RequestSender {
    pub client: reqwest::Client,
    pub observer: Option<Arc<dyn RequestObserver + Send + Sync>>,
}

impl RequestSender {
    pub fn new(
        client: reqwest::Client,
        observer: Option<Arc<dyn RequestObserver + Send + Sync>>,
    ) -> Self {
        Self { client, observer }
    }

    pub async fn send_request<B: Serialize + ?Sized, R: DeserializeOwned>(
        &self,
        token: &crate::Token,
        method: reqwest::Method,
        url: String,
        body: Option<&B>,
        use_return_value: Option<R>,
    ) -> crate::Result<R> {
        use crate::http::Failure;

        if let Some(o) = &self.observer {
            let body_string: String = match body {
                Some(b) => match serde_json::to_string(b) {
                    Ok(s) => s,
                    _ => "<err!>".to_string(),
                },
                None => "".to_string(),
            };
            o.on_request(&method.as_str(), &url, &body_string);
        }

        let req = self
            .client
            .request(method, url.as_str())
            .header(
                "Authorization",
                format!("{} {}", token.token_type, token.access_token),
            )
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("User-Agent", &crate::version::user_agent());
        let req = match body {
            Some(b) => req.json(b),
            None => req,
        };

        let resp = req.send().await?;

        let status = resp.status();

        if status.is_success() {
            return match &self.observer {
                Some(o) => {
                    let text = resp.text().await?;
                    o.on_response(status.as_str(), &text);
                    let r: R = match use_return_value {
                        Some(r) => r,
                        None => {
                            // `from_str` must use an intermediary serde_json::Value
                            // here to avoid an esoteric failure which is seen
                            // in some cases, see:
                            // https://github.com/serde-rs/json/issues/505
                            // let r: R = serde_json::from_str(&text)?;
                            // When revisiting the code generator this work around
                            // should only be applied to problematic models as
                            // it's a little inefficient.
                            let d: serde_json::Value = serde_json::from_str(&text)?;
                            let r: R = serde_json::from_value(d)?;
                            r
                        }
                    };
                    Ok(r)
                }
                None => {
                    let d: serde_json::Value = resp.json().await?;
                    let r: R = serde_json::from_value(d)?;
                    Ok(r)
                }
            };
        }

        let message = resp.text().await?;

        if let Some(o) = &self.observer {
            o.on_response(status.as_str(), &message);
        }

        if status.is_client_error() {
            if status.as_u16() == 401 {
                return Err(Failure::Client("Action not authorized".to_string()).into());
            }

            if status.as_u16() == 404 {
                return Err(Failure::Client("Resource doesn't exist".to_string()).into());
            }

            return Err(Failure::Client(message).into());
        }

        Err(Failure::Server(message).into())
    }
}

#[derive(Clone)]
pub struct EscRequestSender {
    pub base_url: String,
    pub sender: RequestSender,
    pub token: crate::Token,
}

impl EscRequestSender {
    pub async fn send_request<B: Serialize + ?Sized, R: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        relative_url: String,
        body: Option<&B>,
        use_return_value: Option<R>,
    ) -> crate::Result<R> {
        let url = if relative_url.starts_with("/") {
            format!("{}{}", self.base_url, relative_url)
        } else {
            format!("{}/{}", self.base_url, relative_url)
        };
        self.sender
            .send_request(&self.token, method, url, body, use_return_value)
            .await
    }
}
