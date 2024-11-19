use base64::{engine::general_purpose::URL_SAFE, prelude::BASE64_STANDARD, Engine};
use chrono::Utc;
use rand::{thread_rng, Rng};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use crate::utils::orders;
use alloy::{
    primitives::{Address, U256},
    signers::Signer,
    sol,
    sol_types::eip712_domain,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
pub struct ApiCreds {
    pub api_key: String,
    pub api_passphrase: String,
    pub api_secret: String,
}

#[derive(Serialize)]
pub struct AmpCookie {
    device_id: String,
    user_id: Option<String>,
    session_id: i64,
    opt_out: bool,
    last_event_time: i64,
    last_event_id: u32,
}

impl AmpCookie {
    pub fn new() -> Self {
        let device_id = Uuid::new_v4().to_string();
        let session_id = Utc::now().timestamp_millis();
        let last_event_time = session_id + thread_rng().gen_range(50..=1000);
        let last_event_id = thread_rng().gen_range(5..=40);

        Self {
            device_id,
            user_id: None,
            session_id,
            opt_out: false,
            last_event_time,
            last_event_id: last_event_id as u32,
        }
    }
    pub fn set_user_id(&mut self, user_id: Option<String>) {
        self.user_id = user_id
    }

    fn tick_last_event_id(&mut self) {
        self.last_event_id = thread_rng().gen_range(5..=40);
    }

    fn tick_last_event_time(&mut self) {
        self.last_event_time += thread_rng().gen_range(500..=3000);
    }

    pub fn tick(&mut self) {
        self.tick_last_event_id();
        self.tick_last_event_time();
    }

    pub fn to_base64_url_encoded(&self) -> String {
        let header_json_str = serde_json::to_string(self).unwrap();
        let url_encoded = urlencoding::encode(&header_json_str).to_string();
        BASE64_STANDARD.encode(url_encoded)
    }
}

sol! {
    #[derive(Debug)]
    struct ClobAuth {
        address address;
        string timestamp;
        uint256 nonce;
        string message;
    }
}


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

impl HeaderMapSerializeable for LayerOneAuthHeader {}


impl LayerOneAuthHeader {
        pub async fn new<S: Signer + Send + Sync>(signer: Arc<S>) -> Self {
        let timestamp = Utc::now().timestamp().to_string();
        let signature = Self::sign_clob_auth_message(signer.clone(), &timestamp).await;

        Self {
            poly_address: signer.address().to_string(),
            poly_nonce: "0".to_string(),
            poly_signature: signature,
            poly_timestamp: timestamp,
        }
    }

    pub async fn sign_clob_auth_message<S: Signer + Send + Sync>(
        signer: Arc<S>,
        timestamp: &str,
    ) -> String {
        let message = "This message attests that I control the given wallet";

        let auth = ClobAuth {
            address: signer.address(),
            timestamp: timestamp.to_string(),
            nonce: U256::ZERO,
            message: message.to_string(),
        };

        let domain = eip712_domain! {
            name: "ClobAuthDomain",
            version: "1",
            chain_id: 137,
        };

        let signed_message = signer.sign_typed_data(&auth, &domain).await.unwrap();

        const_hex::encode_prefixed(signed_message.as_bytes())
    }
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
