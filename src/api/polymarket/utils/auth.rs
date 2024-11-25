use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub struct ApiCreds {
    pub api_key: String,
    pub api_passphrase: String,
    pub api_secret: String,
}

pub struct AmpCookie {}
pub struct ClobKeyResponse {}

pub trait HeaderMapSerializeable {
    fn to_headermap(&self) -> HeaderMap
    where
        Self: Serialize,
    {
        let mut headers = HeaderMap::new();
        let value = serde_json::to_value(self).unwrap();

        if let Value::Object(map) = value {
            for (k, v) in map {
                if let Value::String(s) = v {
                    let header_name = HeaderName::from_bytes(k.as_bytes()).unwrap();
                    let header_value = HeaderValue::from_str(&s).unwrap();
                    headers.insert(header_name, header_value);
                }
            }
        }

        headers
    }
}

#[derive(Serialize)]
pub struct LayerOneAuthHeader {
    poly_address: String,
    poly_nonce: String,
    poly_signature: String,
    poly_timestamp: String,
}

#[derive(Serialize, Debug)]
pub struct LayerTwoAuthHeader {
    poly_address: String,
    poly_nonce: String,
    poly_signature: String,
    poly_timestamp: String,
    poly_api_key: String,
    poly_passphrase: String,
}

#[cfg(test)]
mod test {}
