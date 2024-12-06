use crate::utils::math::{adjust_amount, ClobPrecision};
use alloy::{
    dyn_abi::Eip712Domain,
    primitives::{utils::parse_units, Address, Signature, U256},
    signers::Signer,
    sol,
    sol_types::eip712_domain,
};
use chrono::Utc;
use eyre::bail;
use rand::Rng;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_repr::Serialize_repr;
use std::{collections::HashMap, sync::LazyLock};
use std::{fmt::Display, ops::Div, str::FromStr, sync::Arc};

#[allow(unused)]
#[derive(Debug, Deserialize, Clone)]
pub struct OrderBookData {
    market: String,
    asset_id: String,
    timestamp: String,
    hash: String,
    pub bids: Vec<PlacedOrder>,
    pub asks: Vec<PlacedOrder>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    pub order: SignedOrder,
    owner: String,
    order_type: OrderType,
}

#[allow(unused)]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    Fok,
    Gtc,
    Gtd,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlacedOrder {
    #[serde(deserialize_with = "string_to_f64")]
    pub price: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub size: f64,
}

fn string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<f64>().map_err(de::Error::custom)
}

impl OrderRequest {
    pub fn new(signed_order: SignedOrder, owner: &str, order_type: Option<OrderType>) -> Self {
        let order_type = order_type.unwrap_or(OrderType::Fok);

        Self {
            order: signed_order,
            owner: owner.to_string(),
            order_type,
        }
    }
}
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct AccumulatedOrder {
    pub price: f64,
    pub size: f64,
    pub value: f64,
    pub net_value: f64,
    pub net_size: f64,
}

#[derive(Deserialize)]
pub struct NegRiskResponseBody {
    pub neg_risk: bool,
}

#[allow(unused)]
pub struct ContractConfig {
    pub exchange: &'static str,
    pub neg_risk_adapter: &'static str,
    pub neg_risk_exchange: &'static str,
    pub collateral: &'static str,
    pub conditional_tokens: &'static str,
}

pub const MATIC_CONTRACTS: ContractConfig = ContractConfig {
    exchange: "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E",
    neg_risk_adapter: "0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296",
    neg_risk_exchange: "0xC5d563A36AE78145C45a50134d48A1215220f80a",
    collateral: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",
    conditional_tokens: "0x4D97DCd97eC945f40cF65F87097ACe5EA0476045",
};

pub const AMOY_CONTRACTS: ContractConfig = ContractConfig {
    exchange: "0xdFE02Eb6733538f8Ea35D585af8DE5958AD99E40",
    neg_risk_adapter: "0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296",
    neg_risk_exchange: "0xC5d563A36AE78145C45a50134d48A1215220f80a",
    collateral: "0x9c4e1703476e875070ee25b56a58b008cfb8fa78",
    conditional_tokens: "0x69308FB512518e39F9b16112fA8d994F4e2Bf8bB",
};
pub const PROTOCOL_NAME: &str = "Polymarket CTF Exchange";
pub const PROTOCOL_VERSION: &str = "1";

pub fn get_contract_config(chain_id: u64) -> eyre::Result<&'static ContractConfig> {
    match chain_id {
        137 => Ok(&MATIC_CONTRACTS),
        80002 => Ok(&AMOY_CONTRACTS),
        _ => eyre::bail!("Invalid network"),
    }
}

pub struct RoundingConfig {
    pub price: u32,
    pub size: u32,
    pub amount: u32,
}

pub static ROUNDING_CONFIG: LazyLock<HashMap<&str, RoundingConfig>> = LazyLock::new(|| {
    [
        (
            "0.1",
            RoundingConfig {
                price: 1,
                size: 2,
                amount: 3,
            },
        ),
        (
            "0.01",
            RoundingConfig {
                price: 2,
                size: 2,
                amount: 4,
            },
        ),
        (
            "0.001",
            RoundingConfig {
                price: 3,
                size: 2,
                amount: 5,
            },
        ),
        (
            "0.0001",
            RoundingConfig {
                price: 4,
                size: 2,
                amount: 6,
            },
        ),
    ]
    .into_iter()
    .collect()
});

#[derive(Debug, Clone, Copy)]
pub enum TickSize {
    OneTenth,
    OneHundredth,
    OneThousandth,
    TenThousandth,
}

impl TickSize {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "0.1" => Some(TickSize::OneTenth),
            "0.01" => Some(TickSize::OneHundredth),
            "0.001" => Some(TickSize::OneThousandth),
            "0.0001" => Some(TickSize::TenThousandth),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TickSize::OneTenth => "0.1",
            TickSize::OneHundredth => "0.01",
            TickSize::OneThousandth => "0.001",
            TickSize::TenThousandth => "0.0001",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateOrderOptions {
    pub tick_size: TickSize,
    pub neg_risk: Option<bool>,
}

pub struct OrderBuilder<'a, S>
where
    S: Signer + Send + Sync,
{
    pub signer: Arc<S>,
    chain_id: u64,
    signature_type: Option<SignatureType>,
    funder_address: Option<&'a str>,
}

impl<'a, S> OrderBuilder<'a, S>
where
    S: Signer + Send + Sync,
{
    pub fn new(
        signer: Arc<S>,
        chain_id: u64,
        signature_type: Option<SignatureType>,
        funder_address: Option<&'a str>,
    ) -> Self {
        Self {
            signer: signer.clone(),
            chain_id,
            signature_type,
            funder_address,
        }
    }

    pub async fn build_signed_order(
        &self,
        user_order: UserOrder,
        options: CreateOrderOptions,
    ) -> eyre::Result<SignedOrder> {
        let signer_address = self.signer.address().to_string();

        let maker = match self.funder_address {
            Some(address) => address,
            None => &signer_address,
        };

        let contract_config = get_contract_config(self.chain_id).unwrap_or(&MATIC_CONTRACTS);

        let order_data = self.build_order_creation_args(
            &signer_address,
            maker,
            self.signature_type,
            &user_order,
            &ROUNDING_CONFIG[options.tick_size.as_str()],
        );

        let exchange_contract = match options.neg_risk.unwrap_or(false) {
            true => contract_config.neg_risk_exchange,
            false => contract_config.exchange,
        };

        self.create_signed_order(order_data, Address::from_str(exchange_contract)?)
            .await
    }
    fn build_order_creation_args(
        &self,
        signer: &str,
        maker: &str,
        signature_type: Option<SignatureType>,
        user_order: &UserOrder,
        round_config: &RoundingConfig,
    ) -> OrderData {
        let OrderRawAmounts {
            side,
            raw_maker_amount,
            raw_taker_amount,
        } = self.get_order_raw_amounts(
            &user_order.side,
            user_order.size,
            user_order.price,
            round_config,
        );

        let maker_amount = parse_units(&raw_maker_amount.to_string(), "MWEI")
            .unwrap()
            .to_string();
        let taker_amount = parse_units(&raw_taker_amount.to_string(), "MWEI")
            .unwrap()
            .to_string();

        let taker = match user_order.taker.clone() {
            Some(taker) => taker,
            None => Address::ZERO.to_string(),
        };

        let fee_rate_bps = match user_order.fee_rate_bps {
            Some(fee_rate) => fee_rate.to_string(),
            None => "0".to_string(),
        };

        let nonce = match user_order.nonce {
            Some(nonce) => nonce.to_string(),
            None => "0".to_string(),
        };

        let expiration = match user_order.expiration {
            Some(exp) => exp.to_string(),
            None => "0".to_string(),
        };

        OrderData {
            maker: maker.to_string(),
            taker,
            token_id: user_order.token_id.clone(),
            maker_amount,
            taker_amount,
            side,
            fee_rate_bps,
            nonce,
            signer: Some(signer.to_string()),
            expiration: Some(expiration),
            signature_type,
        }
    }

    fn get_order_raw_amounts(
        &self,
        side: &Side,
        size: f64,
        price: f64,
        round_config: &RoundingConfig,
    ) -> OrderRawAmounts {
        let raw_price = price.round_normal(round_config.price);
        let (raw_maker_amount, raw_taker_amount) = match side {
            Side::Buy => {
                let raw_taker_amount = size.round_down(round_config.size);
                let raw_maker_amount =
                    adjust_amount(raw_taker_amount * raw_price, round_config.amount);
                (raw_maker_amount, raw_taker_amount)
            }
            Side::Sell => {
                let raw_maker_amount = size.round_down(round_config.size);
                let raw_taker_amount =
                    adjust_amount(raw_maker_amount * raw_price, round_config.amount);
                (raw_maker_amount, raw_taker_amount)
            }
        };

        OrderRawAmounts::new(side, raw_maker_amount, raw_taker_amount)
    }

    pub async fn build_signed_market_buy_order(
        &self,
        user_market_order: UserMarketOrder,
        options: CreateOrderOptions,
    ) -> eyre::Result<SignedOrder> {
        let signer_address = self.signer.address().to_string();

        let maker = match self.funder_address {
            Some(address) => address,
            None => &signer_address,
        };

        let contract_config = get_contract_config(self.chain_id).unwrap_or(&MATIC_CONTRACTS);

        let order_data = self.build_market_buy_order_creation_args(
            &signer_address,
            maker,
            self.signature_type,
            user_market_order,
            &ROUNDING_CONFIG[options.tick_size.as_str()],
        );

        let exchange_contract = match options.neg_risk.unwrap_or(false) {
            true => contract_config.neg_risk_exchange,
            false => contract_config.exchange,
        };

        self.create_signed_order(order_data, Address::from_str(exchange_contract)?)
            .await
    }

    async fn create_signed_order(
        &self,
        order_data: OrderData,
        verifying_contract: Address,
    ) -> eyre::Result<SignedOrder> {
        let order = self.build_order(order_data)?;
        let order_domain = self.get_order_domain(verifying_contract);

        let order_signature = self.signer.sign_typed_data(&order, &order_domain).await?;
        unimplemented!()
        //SignedOrder::new(order, order_signature)
    }

    fn build_order(&self, mut order_data: OrderData) -> eyre::Result<Order> {
        if order_data.signer.is_none() || order_data.signer.as_ref().unwrap().is_empty() {
            order_data.signer = Some(order_data.maker.clone());
        }

        let signer_address = self.signer.address().to_string();

        if order_data.signer.as_ref().unwrap() != &signer_address {
            eyre::bail!("Signer does not match");
        }

        if order_data.expiration.is_none() || order_data.expiration.as_ref().unwrap().is_empty() {
            order_data.expiration = Some("0".to_string());
        }

        if order_data.signature_type.is_none() {
            order_data.signature_type = Some(SignatureType::PolyGnosisSafe);
        }

        let order = Order::new(&self.generate_salt(), order_data)?;

        Ok(order)
    }

    fn generate_salt(&self) -> String {
        let now = Utc::now().timestamp_millis() as u128;

        let random_value: u128 = rand::thread_rng().gen_range(0..now);

        random_value.to_string()
    }

    fn get_order_domain(&self, verifying_contract: Address) -> Eip712Domain {
        eip712_domain! {
            name: PROTOCOL_NAME,
            version: PROTOCOL_VERSION,
            chain_id: self.chain_id,
            verifying_contract: verifying_contract,
        }
    }

    fn build_market_buy_order_creation_args(
        &self,
        signer: &str,
        maker: &str,
        signature_type: Option<SignatureType>,
        user_market_order: UserMarketOrder,
        round_config: &RoundingConfig,
    ) -> OrderData {
        let price = user_market_order.price.unwrap_or(1f64);

        let BuyOrderRawAmounts {
            raw_maker_amount,
            raw_taker_amount,
        } = self.get_market_buy_order_raw_amounts(user_market_order.amount, price, round_config);

        let maker_amount = parse_units(&raw_maker_amount.to_string(), "MWEI")
            .unwrap()
            .to_string();
        let taker_amount = parse_units(&raw_taker_amount.to_string(), "MWEI")
            .unwrap()
            .to_string();

        let taker = match user_market_order.taker.clone() {
            Some(taker) => taker,
            None => Address::ZERO.to_string(),
        };

        let fee_rate_bps = match user_market_order.fee_rate_bps {
            Some(fee_rate) => fee_rate.to_string(),
            None => "0".to_string(),
        };

        let nonce = match user_market_order.nonce {
            Some(nonce) => nonce.to_string(),
            None => "0".to_string(),
        };

        OrderData {
            maker: maker.to_string(),
            taker,
            token_id: user_market_order.token_id,
            maker_amount,
            taker_amount,
            side: Side::Buy,
            fee_rate_bps,
            nonce,
            signer: Some(signer.to_string()),
            expiration: Some("0".to_string()),
            signature_type,
        }
    }
    fn get_market_buy_order_raw_amounts(
        &self,
        amount: f64,
        price: f64,
        round_config: &RoundingConfig,
    ) -> BuyOrderRawAmounts {
        let raw_maker_amount = amount.round_down(round_config.size);
        let raw_price = price.round_down(round_config.price);

        let raw_taker_amount = adjust_amount(raw_maker_amount.div(raw_price), round_config.amount);

        BuyOrderRawAmounts::new(raw_maker_amount, raw_taker_amount)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[repr(u8)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy = 0,
    Sell = 1,
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Buy => write!(f, "BUY"),
            Self::Sell => write!(f, "SELL"),
        }
    }
}

impl Default for Side {
    fn default() -> Self {
        Self::Buy
    }
}

impl TryFrom<u8> for Side {
    type Error = eyre::Report;

    fn try_from(value: u8) -> eyre::Result<Self> {
        match value {
            0 => Ok(Side::Buy),
            1 => Ok(Side::Sell),
            _ => bail!("expected: 0 => Side::Buy\n1 => Side::Sell"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr)]
#[repr(u8)]
pub enum SignatureType {
    Eoa = 0,
    PolyProxy = 1,
    PolyGnosisSafe = 2,
}

impl TryFrom<u8> for SignatureType {
    type Error = eyre::Report;

    fn try_from(value: u8) -> eyre::Result<Self> {
        match value {
            0 => Ok(SignatureType::Eoa),
            1 => Ok(SignatureType::PolyProxy),
            2 => Ok(SignatureType::PolyGnosisSafe),
            _ => bail!("expected: 0 => SignatureType::Eoa\n1 => SignatureType::PolyProxy\n2 => SignatureType::PolyGnosisSafe, but got {}", value),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct OrderData {
    pub maker: String,
    pub taker: String,
    pub token_id: String,
    pub maker_amount: String,
    pub taker_amount: String,
    pub side: Side,
    pub fee_rate_bps: String,
    pub nonce: String,
    pub signer: Option<String>,
    pub expiration: Option<String>,
    pub signature_type: Option<SignatureType>,
}

sol! {
    #[derive(Debug)]
    struct Order {
        uint256 salt;
        address maker;
        address signer;
        address taker;
        uint256 tokenId;
        uint256 makerAmount;
        uint256 takerAmount;
        uint256 expiration;
        uint256 nonce;
        uint256 feeRateBps;
        uint8 side;
        uint8 signatureType;
    }
}

impl Order {
    pub fn new(salt: &str, order_data: OrderData) -> eyre::Result<Self> {
        Ok(Self {
            salt: U256::from_str_radix(salt, 10)?,
            maker: Address::from_str(&order_data.maker)?,
            signer: Address::from_str(&order_data.signer.unwrap())?,
            taker: Address::from_str(&order_data.taker)?,
            tokenId: U256::from_str_radix(&order_data.token_id, 10)?,
            makerAmount: U256::from_str_radix(&order_data.maker_amount, 10)?,
            takerAmount: U256::from_str_radix(&order_data.taker_amount, 10)?,
            expiration: U256::from_str_radix(&order_data.expiration.unwrap(), 10)?,
            nonce: U256::from_str_radix(&order_data.nonce, 10)?,
            feeRateBps: U256::from_str_radix(&order_data.fee_rate_bps, 10)?,
            side: order_data.side as u8,
            signatureType: order_data.signature_type.unwrap() as u8,
        })
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignedOrder {
    pub salt: usize,
    pub maker: String,
    pub signer: String,
    pub taker: String,
    pub token_id: String,
    pub maker_amount: String,
    pub taker_amount: String,
    pub side: Side,
    pub expiration: String,
    pub nonce: String,
    pub fee_rate_bps: String,
    pub signature_type: SignatureType,
    pub signature: String,
}

impl SignedOrder {
    pub fn new(order: Order, signature: Signature) -> eyre::Result<Self> {
        Ok(Self {
            salt: order.salt.try_into().unwrap(),
            maker: order.maker.to_string(),
            signer: order.signer.to_string(),
            taker: order.taker.to_string(),
            token_id: order.tokenId.to_string(),
            maker_amount: order.makerAmount.to_string(),
            taker_amount: order.takerAmount.to_string(),
            expiration: order.expiration.to_string(),
            nonce: order.nonce.to_string(),
            fee_rate_bps: order.feeRateBps.to_string(),
            side: Side::try_from(order.side)?,
            signature_type: SignatureType::try_from(order.signatureType)?,
            signature: const_hex::encode_prefixed(signature.as_bytes()),
        })
    }
}

pub struct OrderRawAmounts {
    pub side: Side,
    pub raw_maker_amount: f64,
    pub raw_taker_amount: f64,
}

impl OrderRawAmounts {
    pub fn new(side: &Side, raw_maker_amount: f64, raw_taker_amount: f64) -> Self {
        Self {
            side: side.clone(),
            raw_maker_amount,
            raw_taker_amount,
        }
    }
}

pub struct BuyOrderRawAmounts {
    pub raw_maker_amount: f64,
    pub raw_taker_amount: f64,
}

impl BuyOrderRawAmounts {
    pub fn new(raw_maker_amount: f64, raw_taker_amount: f64) -> Self {
        Self {
            raw_maker_amount,
            raw_taker_amount,
        }
    }
}

#[derive(Default)]
pub struct UserOrder {
    pub token_id: String,
    pub price: f64,
    pub size: f64,
    pub side: Side,
    pub fee_rate_bps: Option<f64>,
    pub nonce: Option<u64>,
    pub expiration: Option<u64>,
    pub taker: Option<String>,
}

#[allow(unused)]
impl UserOrder {
    pub fn set_token_id(&mut self, token_id: &str) {
        self.token_id = token_id.to_string();
    }

    pub fn with_token_id(mut self, token_id: &str) -> Self {
        self.set_token_id(token_id);
        self
    }

    pub fn set_price(&mut self, price: f64) {
        self.price = price;
    }

    pub fn with_price(mut self, price: f64) -> Self {
        self.set_price(price);
        self
    }

    pub fn set_size(&mut self, size: f64) {
        self.size = size;
    }

    pub fn with_size(mut self, size: f64) -> Self {
        self.set_size(size);
        self
    }

    pub fn set_side(&mut self, side: Side) {
        self.side = side;
    }

    pub fn with_side(mut self, side: Side) -> Self {
        self.set_side(side);
        self
    }

    pub fn set_fee_rate_bps(&mut self, fee_rate_bps: f64) {
        self.fee_rate_bps = Some(fee_rate_bps);
    }

    pub fn with_fee_rate_bps(mut self, fee_rate_bps: f64) -> Self {
        self.set_fee_rate_bps(fee_rate_bps);
        self
    }

    pub fn set_nonce(&mut self, nonce: u64) {
        self.nonce = Some(nonce);
    }

    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.set_nonce(nonce);
        self
    }

    pub fn set_expiration(&mut self, expiration: u64) {
        self.expiration = Some(expiration);
    }

    pub fn with_expiration(mut self, expiration: u64) -> Self {
        self.set_expiration(expiration);
        self
    }

    pub fn set_taker(&mut self, taker: String) {
        self.taker = Some(taker);
    }

    pub fn with_taker(mut self, taker: String) -> Self {
        self.set_taker(taker);
        self
    }
}

pub struct UserMarketOrder {
    pub token_id: String,
    pub price: Option<f64>,
    pub amount: f64,
    pub fee_rate_bps: Option<f64>,
    pub nonce: Option<u64>,
    pub taker: Option<String>,
}

impl UserMarketOrder {
    pub fn new(
        token_id: String,
        amount: f64,
        price: Option<f64>,
        fee_rate_bps: Option<f64>,
        nonce: Option<u64>,
        taker: Option<String>,
    ) -> Self {
        Self {
            token_id,
            price,
            amount,
            fee_rate_bps,
            nonce,
            taker,
        }
    }
}

pub async fn place_order() {
    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::*;
    use tracing_subscriber::prelude::*;

    #[tokio::test]
    pub async fn test_build_polymarket_order() {
        unimplemented!()
    }
}
