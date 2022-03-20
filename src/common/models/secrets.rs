use crate::common::models::config::{ExchangeName, NetworkName};
use ethers::abi::Address;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretsSettings {
    pub settings: SecretsExchanges,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretsExchanges {
    pub exchanges: Vec<Exchanges>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Exchanges {
    pub exchange: ExchangeEntity,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExchangeEntity {
    pub name: ExchangeName,
    pub networks: Vec<Networks>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Networks {
    pub network: NetworkEntity,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkEntity {
    pub name: NetworkName,
    pub accounts: Vec<Accounts>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Accounts {
    pub account: AccountEntity,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountEntity {
    pub name: String,

    pub address: String,

    pub mnemonic: String,
}

impl AccountEntity {
    pub fn address_h160(&self) -> anyhow::Result<Address> {
        Ok(Address::from_str(&*self.address)?)
    }
}
