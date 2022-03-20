use crate::common::models::scheme_helpers::SchemeHelpers;
use ethers::abi::Address;
use ethers::types::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenTransfers {
    pub token_transfer: TokenTransfer,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenTransfer {
    pub recipient_wallet_address: String,
    pub wallet_name: String,
    pub amount_of_tokens: String,
    pub gas_price: Option<String>,
    pub gas_limit: Option<u64>,
}

impl TokenTransfer {
    pub fn recipient_wallet_address_h160(&self) -> anyhow::Result<Address> {
        SchemeHelpers::contract_to_h160(&self.recipient_wallet_address)
    }

    pub fn amount_of_token_u256(&self) -> anyhow::Result<U256> {
        SchemeHelpers::price_to_u256(&self.amount_of_tokens)
    }

    pub fn gas_price_gwei(&self) -> anyhow::Result<Option<U256>> {
        SchemeHelpers::decimals_price_to_gwei_option(&self.gas_price)
    }

    pub fn gas_limit_u256(&self) -> anyhow::Result<Option<U256>> {
        SchemeHelpers::convert_to_u256_option(&self.gas_limit)
    }
}
