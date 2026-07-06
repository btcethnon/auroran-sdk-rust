//! HTTP exchange logging and response decoding shared by sync/async clients.

use std::sync::{Arc, RwLock};

use crate::error::ClientError;

/// One HTTP round-trip observed by the JSON-RPC client (for CLI/debug logging).
#[derive(Debug, Clone)]
pub struct HttpExchange {
    pub url: String,
    pub method: &'static str,
    pub request_json: Option<String>,
    pub response_status: Option<u16>,
    pub response_body: Option<String>,
    pub error: Option<String>,
}

pub(crate) type HttpExchangeLogger = Arc<dyn Fn(&HttpExchange) + Send + Sync>;

pub(crate) struct HttpAttempt {
    url: String,
    method: &'static str,
    request_json: Option<String>,
    http_log: Arc<RwLock<Option<HttpExchangeLogger>>>,
}

impl HttpAttempt {
    pub fn new(
        url: String,
        method: &'static str,
        request_json: Option<String>,
        http_log: Arc<RwLock<Option<HttpExchangeLogger>>>,
    ) -> Self {
        Self {
            url,
            method,
            request_json,
            http_log,
        }
    }

    fn emit(&self, exchange: HttpExchange) {
        if let Ok(log) = self.http_log.read() {
            if let Some(log) = log.as_ref() {
                log(&exchange);
            }
        }
    }

    fn exchange(
        &self,
        response_status: Option<u16>,
        response_body: Option<String>,
        error: Option<String>,
    ) -> HttpExchange {
        HttpExchange {
            url: self.url.clone(),
            method: self.method,
            request_json: self.request_json.clone(),
            response_status,
            response_body,
            error,
        }
    }

    pub fn log_transport_error(self, err: reqwest::Error) -> ClientError {
        let msg = crate::client_common::format_http_error(&err);
        self.emit(self.exchange(None, None, Some(msg)));
        err.into()
    }

    pub fn log_body_read_error(self, status: u16, err: reqwest::Error) -> ClientError {
        self.emit(self.exchange(
            Some(status),
            None,
            Some(format!("failed to read response body: {err}")),
        ));
        err.into()
    }

    pub fn log_serialize_error(self, err: serde_json::Error) -> ClientError {
        self.emit(self.exchange(
            None,
            None,
            Some(format!("failed to serialize request JSON: {err}")),
        ));
        ClientError::Json(err)
    }

    pub fn decode_json<T: serde::de::DeserializeOwned>(
        self,
        status: u16,
        text: String,
    ) -> Result<T, ClientError> {
        if !(200..300).contains(&status) {
            self.emit(self.exchange(
                Some(status),
                Some(text.clone()),
                Some(format!("HTTP {status}")),
            ));
            return Err(ClientError::Api {
                status,
                body: text,
            });
        }
        match serde_json::from_str::<T>(&text) {
            Ok(val) => {
                self.emit(self.exchange(Some(status), Some(text), None));
                Ok(val)
            }
            Err(e) => {
                self.emit(self.exchange(
                    Some(status),
                    Some(text),
                    Some(format!("JSON decode failed: {e}")),
                ));
                Err(e.into())
            }
        }
    }
}
