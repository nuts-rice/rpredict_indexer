use alloy::{primitives::Address, signers::local::PrivateKeySigner};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct PolymarketAccount {
    priv_key: String,
    proxy: Option<String>,
    address: String,
    pub proxy_address: Option<String>,
    #[serde(
        serialize_with = "serialize_arc_rwlock",
        deserialize_with = "deserialize_arc_rwlock"
    )]
    pub api_key: Arc<RwLock<Option<String>>>,
}

fn serialize_arc_rwlock<S>(
    data: &Arc<RwLock<Option<String>>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    data.read().unwrap().serialize(serializer)
}

fn deserialize_arc_rwlock<'de, D>(deserializer: D) -> Result<Arc<RwLock<Option<String>>>, D::Error>
where
    D: Deserializer<'de>,
{
    let data = Option::<String>::deserialize(deserializer)?;
    Ok(Arc::new(RwLock::new(data)))
}

impl PolymarketAccount {
    pub fn new(priv_key: &str, proxy: Option<String>, recipt_address: Option<String>) -> Self {
        let signer =
            Arc::new(PrivateKeySigner::from_str(priv_key).expect("Private key to be valid "));
        let recipt_address = recipt_address.unwrap_or_else(|| signer.address().to_string());
        let address = signer.address();
        unimplemented!()
        // let proxy_address = =
    }
}
