use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::observer::Observer;
use crate::errors::{ApiResponseError, CommunicationError, EscError, ProblemDetails, Result};

/// Wraps a reqwest::Client and an observer with a helper function which accepts
/// a token to make authenticated requests. This makes it possible to observe
/// all parts of the request / response apart from authentication.
#[derive(Clone)]
pub struct Sender {
    pub client: reqwest::Client,
    pub observer: Option<Arc<dyn Observer + Send + Sync>>,
}

impl Sender {
    pub fn new(client: reqwest::Client, observer: Option<Arc<dyn Observer + Send + Sync>>) -> Self {
        Self { client, observer }
    }

    pub async fn send_request<B: Serialize + ?Sized, R: DeserializeOwned>(
        &self,
        authorization_header: &str,
        method: reqwest::Method,
        url: String,
        body: Option<&B>,
        use_return_value: Option<R>,
    ) -> Result<R> {
        if let Some(o) = &self.observer {
            let body_string: String = match body {
                Some(b) => match serde_json::to_string(b) {
                    Ok(s) => s,
                    _ => "<err!>".to_string(),
                },
                None => "".to_string(),
            };
            o.on_request(method.as_str(), &url, &body_string);
        }

        let req = self
            .client
            .request(method.clone(), url.as_str())
            .header("Authorization", authorization_header)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("User-Agent", &crate::version::user_agent());
        let req = match body {
            Some(b) => req.json(b),
            None => req,
        };

        let resp = req.send().await.map_err(|r| CommunicationError {
            debug: format!("method={}, url={}", method, url),
            message: "Error making request".to_string(),
            source: Box::new(r),
        })?;

        let status_code = resp.status();

        if status_code.is_success() {
            return match &self.observer {
                Some(o) => {
                    let text = resp.text().await.map_err(|r| CommunicationError {
                        debug: format!(
                            "method={}, url={}, status code={}",
                            method, url, status_code
                        ),
                        message: "Error reading request response".to_string(),
                        source: Box::new(r),
                    })?;
                    o.on_response(status_code.as_str(), &text);
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
                            let from_str_result: serde_json::Result<serde_json::Value> =
                                serde_json::from_str(&text);
                            match from_str_result {
                                Err(err) => {
                                    return Err(EscError::Other(CommunicationError {
                                        debug: format!(
                                            "method={}, url={}, status code={} response text={}",
                                            method, url, status_code, text
                                        ),
                                        message: "Error deserializing response text".to_string(),
                                        source: Box::new(err),
                                    }));
                                }
                                Ok(d) => {
                                    let from_value_result: serde_json::Result<R> =
                                        serde_json::from_value(d);
                                    match from_value_result {
                                        Err(err) => {
                                            return Err(EscError::Other(CommunicationError{
                                                debug: format!("method={}, url={}, status code={} response text={}", method, url, status_code, text),
                                                message: "Error deserializing response text into specific client type".to_string(),
                                                source: Box::new(err),
                                            }));
                                        }
                                        Ok(r) => r,
                                    }
                                }
                            }
                        }
                    };
                    Ok(r)
                }
                None => {
                    let r: R = match use_return_value {
                        Some(r) => r,
                        None => {
                            let d: serde_json::Value =
                                resp.json().await.map_err(|s| CommunicationError {
                                    debug: format!(
                                        "method={}, url={}, status code={}",
                                        method, url, status_code
                                    ),
                                    message: "Error reading or deserializing the request response"
                                        .to_string(),
                                    source: Box::new(s),
                                })?;
                            let r: R = serde_json::from_value(d).map_err(|s| CommunicationError {
                                debug: format!(
                                    "method={}, url={}, status code={}",
                                    method, url, status_code
                                ),
                                message: "Error deserializing response text into specific client type"
                                    .to_string(),
                                source: Box::new(s),
                            })?;
                            r
                        }
                    };
                    Ok(r)
                }
            };
        }

        let problem_details: ProblemDetails = match &self.observer {
            Some(o) => {
                let text = resp.text().await.map_err(|r| CommunicationError {
                    debug: format!(
                        "method={}, url={}, status code={}",
                        method, url, status_code
                    ),
                    message: "Error reading request response".to_string(),
                    source: Box::new(r),
                })?;
                o.on_response(status_code.as_str(), &text);
                serde_json::from_str(&text).map_err(|s| CommunicationError {
                    debug: format!(
                        "method={}, url={}, status code={}, response text={}",
                        method, url, status_code, text
                    ),
                    message: "Error deserializing response text into specific client type"
                        .to_string(),
                    source: Box::new(s),
                })?
            }
            None => resp.json().await.map_err(|s| CommunicationError {
                debug: format!(
                    "method={}, url={}, status code={}",
                    method, url, status_code
                ),
                message: "Error reading or deserializing the request response".to_string(),
                source: Box::new(s),
            })?,
        };
        Err(EscError::ApiResponse(Box::new(ApiResponseError {
            problem_details,
            status_code,
        })))
    }
}
