use std::fmt;
use std::str::FromStr;

use ethers::types::Address;
use serde::{Deserialize, Serialize};

use crate::common::models::token_transfer_scheme::TokenTransfers;
use crate::common::utils::network_request::RestfulAuthorization;
use crate::common::models::trade_scheme::Trades;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigSettings {
    pub settings: ConfigExchanges,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigExchanges {
    pub exchanges: Vec<Exchanges>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Exchanges {
    pub exchange: ExchangeEntity,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExchangeEntity {
    pub name: ExchangeName,

    pub networks: Vec<Networks>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExchangeName {
    #[serde(rename = "pancakeswap")]
    Pancakeswap,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Networks {
    pub network: NetworkEntity,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkEntity {
    pub name: NetworkName,

    pub factory: String,

    pub router: String,

    pub usd_token_contract: String,

    pub native_token_contract: String,

    pub providers: Vec<Providers>,

    pub feature: Feature,

    pub native_token_symbol: String,

    pub check_gas_fees: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum NetworkName {
    #[serde(rename = "mainnet")]
    Mainnet,

    #[serde(rename = "testnet")]
    Testnet,
}

impl fmt::Display for NetworkName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for NetworkName {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(NetworkName::Mainnet),
            "testnet" => Ok(NetworkName::Testnet),
            _ => Err("an error occured while trying to convert string to 'NetworkName'"),
        }
    }
}

impl NetworkEntity {
    pub fn factory_h160(&self) -> anyhow::Result<Address> {
        Ok(Address::from_str(&*self.factory)?)
    }

    pub fn router_h160(&self) -> anyhow::Result<Address> {
        Ok(Address::from_str(&*self.router)?)
    }

    pub fn usd_token_h160(&self) -> anyhow::Result<Address> {
        Ok(Address::from_str(&*self.usd_token_contract)?)
    }

    pub fn native_token_h160(&self) -> anyhow::Result<Address> {
        Ok(Address::from_str(&*self.native_token_contract)?)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Feature {
    pub trades: Option<Vec<Trades>>,

    pub token_transfers: Option<Vec<TokenTransfers>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Providers {
    pub provider: ProviderEntity,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProviderEntity {
    pub name: String,

    pub api: String,

    pub protocol: Protocol,

    pub username: Option<String>,

    pub password: Option<String>,
}

impl ProviderEntity {
    pub fn get_authorization(&self) -> Option<RestfulAuthorization> {
        let mut username = String::default();
        let mut password = String::default();

        let mut field_count = 0;

        if let Some(v) = &self.username {
            username = v.to_owned();

            field_count += 1;
        }
        if let Some(v) = &self.password {
            password = v.to_owned();

            field_count += 1;
        }

        if field_count < 2 {
            return None;
        }

        Some(RestfulAuthorization { username, password })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Protocol {
    #[serde(rename = "wss")]
    Wss,
}
