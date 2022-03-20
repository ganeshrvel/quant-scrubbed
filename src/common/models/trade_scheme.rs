use std::fmt;
use std::str::FromStr;
use ethers::abi::Address;
use ethers::types::U256;
use serde::{Deserialize, Serialize};

use crate::common::models::scheme_helpers::SchemeHelpers;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "name")]
pub enum TradeScheme {
    #[serde(rename = "buy")]
    Buy(BuyScheme),

    #[serde(rename = "sell")]
    Sell(SellScheme),
}

impl fmt::Display for TradeScheme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TradeName {
    fn buy() -> Self {
        TradeName::Buy
    }
    fn sell() -> Self {
        TradeName::Sell
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuyScheme {
    #[serde(default = "TradeName::buy")]
    pub name: TradeName,

    pub token_in_contract: String,

    pub token_in_symbol: String,

    pub token_out_contract: Option<String>,

    pub token_out_symbol: String,

    pub amount_of_token_in: String,

    pub slippage: u8,

    pub gas_price: String,

    pub gas_limit: u64,

    pub tx_timeout_in_ms: i64,

    pub min_liquidity_required: Option<String>,

    pub approve_gas_price: Option<String>,

    pub approve_gas_limit: Option<u64>,

    pub is_token_out_deflationary: bool,

    pub perc_increase_gas_price: Option<u32>,

    pub max_gas_price: Option<String>,

    pub retry_attempts: Option<u64>,

    pub time_between_retries_in_ms: Option<i64>,

    pub wait_time_before_first_tx_attempt_in_ms: Option<i64>,

    pub max_allowed_time_for_trading_in_ms: Option<i64>,

    pub trade_at_price: Option<String>,

    pub hold_trade_above_price: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SellScheme {
    #[serde(default = "TradeName::sell")]
    pub name: TradeName,

    pub token_in_contract: Option<String>,

    pub token_in_symbol: String,

    pub token_out_contract: String,

    pub token_out_symbol: String,

    pub amount_of_token_in: Option<String>,

    pub slippage: u8,

    pub gas_price: String,

    pub gas_limit: u64,

    pub tx_timeout_in_ms: i64,

    pub min_liquidity_required: Option<String>,

    pub approve_gas_price: Option<String>,

    pub approve_gas_limit: Option<u64>,

    pub is_token_out_deflationary: bool,

    pub perc_of_token_in: Option<u8>,

    pub perc_increase_gas_price: Option<u32>,

    pub max_gas_price: Option<String>,

    pub retry_attempts: Option<u64>,

    pub time_between_retries_in_ms: Option<i64>,

    pub wait_time_before_first_tx_attempt_in_ms: Option<i64>,

    pub max_allowed_time_for_trading_in_ms: Option<i64>,

    pub trade_at_price: Option<String>,

    pub hold_trade_below_price: Option<String>,

    pub trade_at_profit_perc: Option<u8>,

    pub hold_trade_below_profit_perc: Option<u8>,
}

pub trait TradeSchemeVariant {
    fn gas_limit_u256(&self) -> anyhow::Result<U256>;

    fn gas_price_gwei(&self) -> anyhow::Result<U256>;

    fn max_gas_price_gwei(&self) -> anyhow::Result<Option<U256>>;

    fn approve_gas_price_gwei(&self) -> anyhow::Result<Option<U256>>;

    fn approve_gas_limit_u256(&self) -> anyhow::Result<Option<U256>>;

    fn trade_at_price_u256(&self) -> anyhow::Result<Option<U256>>;

    fn min_liquidity_required_u256(&self) -> anyhow::Result<Option<U256>>;
}

impl TradeSchemeVariant for BuyScheme {
    fn gas_limit_u256(&self) -> anyhow::Result<U256> {
        Ok(U256::from(self.gas_limit))
    }

    fn gas_price_gwei(&self) -> anyhow::Result<U256> {
        SchemeHelpers::decimals_price_to_gwei(&self.gas_price)
    }

    fn max_gas_price_gwei(&self) -> anyhow::Result<Option<U256>> {
        SchemeHelpers::decimals_price_to_gwei_option(&self.max_gas_price)
    }

    fn approve_gas_price_gwei(&self) -> anyhow::Result<Option<U256>> {
        SchemeHelpers::decimals_price_to_gwei_option(&self.approve_gas_price)
    }

    fn approve_gas_limit_u256(&self) -> anyhow::Result<Option<U256>> {
        SchemeHelpers::convert_to_u256_option(&self.approve_gas_limit)
    }

    fn trade_at_price_u256(&self) -> anyhow::Result<Option<U256>> {
        SchemeHelpers::price_to_u256_option(&self.trade_at_price)
    }

    fn min_liquidity_required_u256(&self) -> anyhow::Result<Option<U256>> {
        SchemeHelpers::price_to_u256_option(&self.min_liquidity_required)
    }
}

impl TradeSchemeVariant for SellScheme {
    fn gas_limit_u256(&self) -> anyhow::Result<U256> {
        Ok(U256::from(self.gas_limit))
    }

    fn gas_price_gwei(&self) -> anyhow::Result<U256> {
        SchemeHelpers::decimals_price_to_gwei(&self.gas_price)
    }

    fn max_gas_price_gwei(&self) -> anyhow::Result<Option<U256>> {
        SchemeHelpers::decimals_price_to_gwei_option(&self.max_gas_price)
    }

    fn approve_gas_price_gwei(&self) -> anyhow::Result<Option<U256>> {
        SchemeHelpers::decimals_price_to_gwei_option(&self.approve_gas_price)
    }

    fn approve_gas_limit_u256(&self) -> anyhow::Result<Option<U256>> {
        SchemeHelpers::convert_to_u256_option(&self.approve_gas_limit)
    }

    fn trade_at_price_u256(&self) -> anyhow::Result<Option<U256>> {
        SchemeHelpers::price_to_u256_option(&self.trade_at_price)
    }

    fn min_liquidity_required_u256(&self) -> anyhow::Result<Option<U256>> {
        SchemeHelpers::price_to_u256_option(&self.min_liquidity_required)
    }
}

impl BuyScheme {
    pub fn token_in_h160(&self) -> anyhow::Result<Address> {
        Ok(Address::from_str(&*self.token_in_contract)?)
    }

    pub fn token_out_h160(&self) -> anyhow::Result<Option<Address>> {
        SchemeHelpers::contract_to_h160_option(&self.token_out_contract)
    }

    pub fn amount_of_token_in_u256(&self) -> anyhow::Result<U256> {
        SchemeHelpers::price_to_u256(&self.amount_of_token_in)
    }

    pub fn trade_actuator(&self) -> anyhow::Result<Option<TradeActuatorForBuy>> {
        // static trade actuator
        let s = SchemeHelpers::static_trade_actuator(
            &self.trade_at_price,
            &None,
            &self.hold_trade_above_price,
        )?;

        if let Some(d) = s {
            let st = StaticTradeActuatorForBuy {
                trade_at_price_u256: d.trade_at_price_u256,
                hold_trade_above_price_u256: d.hold_trade_above_price_u256,
            };

            return Ok(Some(TradeActuatorForBuy::Static(st)));
        }

        Ok(None)
    }
}

impl SellScheme {
    pub fn token_in_h160(&self) -> anyhow::Result<Option<Address>> {
        SchemeHelpers::contract_to_h160_option(&self.token_in_contract)
    }

    pub fn token_out_h160(&self) -> anyhow::Result<Address> {
        SchemeHelpers::contract_to_h160(&self.token_out_contract)
    }

    pub fn amount_of_token_in_u256(&self) -> anyhow::Result<Option<U256>> {
        SchemeHelpers::price_to_u256_option(&self.amount_of_token_in)
    }

    // we return either "static trade actuator" or "percentage trade actuator"
    // if static actuator is found then we return that
    // if percentage actuator is found then we return that
    // both cant be active at the same time. the program will panic if both are found active at the same time
    pub fn trade_actuator(&self) -> anyhow::Result<Option<TradeActuatorForSell>> {
        // static trade actuator
        let s = SchemeHelpers::static_trade_actuator(
            &self.trade_at_price,
            &self.hold_trade_below_price,
            &None,
        )?;

        if let Some(d) = s {
            let st = StaticTradeActuatorForSell {
                trade_at_price_u256: d.trade_at_price_u256,
                hold_trade_below_price_u256: d.hold_trade_below_price_u256,
            };

            return Ok(Some(TradeActuatorForSell::Static(st)));
        };

        // percentage trade actuator
        let p = SchemeHelpers::percentage_trade_actuator(
            self.trade_at_profit_perc,
            self.hold_trade_below_profit_perc,
        )?;

        if let Some(d) = p {
            let pt = PercentageTradeActuatorForSell {
                trade_at_profit_perc: d.trade_at_profit_perc,
                hold_trade_below_profit_perc: d.hold_trade_below_profit_perc,
            };

            return Ok(Some(TradeActuatorForSell::Percentage(pt)));
        }

        Ok(None)
    }
}

#[derive(Debug)]
pub enum TradeActuatorForBuy {
    Static(StaticTradeActuatorForBuy),
}

#[derive(Debug)]
pub enum TradeActuatorForSell {
    Static(StaticTradeActuatorForSell),
    Percentage(PercentageTradeActuatorForSell),
}

#[derive(Debug)]
pub struct StaticTradeActuatorForBuy {
    pub trade_at_price_u256: Option<U256>,
    pub hold_trade_above_price_u256: Option<U256>,
}

#[derive(Debug)]
pub struct StaticTradeActuatorForSell {
    pub trade_at_price_u256: Option<U256>,
    pub hold_trade_below_price_u256: Option<U256>,
}

#[derive(Debug)]
pub struct PercentageTradeActuatorForSell {
    pub trade_at_profit_perc: Option<u8>,
    pub hold_trade_below_profit_perc: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trades {
    pub trade: TradeScheme,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TradeName {
    #[serde(rename = "buy")]
    Buy,

    #[serde(rename = "sell")]
    Sell,
}

impl fmt::Display for TradeName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
