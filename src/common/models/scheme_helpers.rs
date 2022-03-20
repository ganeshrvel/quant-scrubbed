use crate::common::utils::ether::{decimals_to_ethers, decimals_to_gwei};
use crate::common::utils::u256::ToU256Units;
use ethers::abi::ethereum_types::U256;
use ethers::abi::Address;
use ethers::core::utils::parse_units;
use std::str::FromStr;

pub struct SchemeHelpers;

pub struct StaticTradeActuatorR {
    pub trade_at_price_u256: Option<U256>,
    pub hold_trade_below_price_u256: Option<U256>,
    pub hold_trade_above_price_u256: Option<U256>,
}

pub struct PercentageTradeActuatorR {
    pub trade_at_profit_perc: Option<u8>,
    pub hold_trade_below_profit_perc: Option<u8>,
}

impl SchemeHelpers {
    pub fn static_trade_actuator(
        trade_at_price: &Option<String>,
        hold_trade_below_price: &Option<String>,
        hold_trade_above_price: &Option<String>,
    ) -> anyhow::Result<Option<StaticTradeActuatorR>> {
        let trade_at_price_u256 = Self::price_to_u256_option(trade_at_price)?;
        let hold_trade_below_price_u256 = Self::price_to_u256_option(hold_trade_below_price)?;
        let hold_trade_above_price_u256 = Self::price_to_u256_option(hold_trade_above_price)?;

        if trade_at_price_u256.is_some()
            || hold_trade_below_price_u256.is_some()
            || hold_trade_above_price_u256.is_some()
        {
            let s = StaticTradeActuatorR {
                trade_at_price_u256,
                hold_trade_below_price_u256,
                hold_trade_above_price_u256,
            };

            return Ok(Some(s));
        }

        Ok(None)
    }

    pub fn percentage_trade_actuator(
        trade_at_profit_perc: Option<u8>,
        hold_trade_below_profit_perc: Option<u8>,
    ) -> anyhow::Result<Option<PercentageTradeActuatorR>> {
        if trade_at_profit_perc.is_some() || hold_trade_below_profit_perc.is_some() {
            let s = PercentageTradeActuatorR {
                trade_at_profit_perc,
                hold_trade_below_profit_perc,
            };

            return Ok(Some(s));
        }

        Ok(None)
    }

    pub fn price_to_gwei(value: u64) -> anyhow::Result<U256> {
        Ok(parse_units(value, "gwei")?)
    }

    pub fn price_to_gwei_option(value: &Option<u64>) -> anyhow::Result<Option<U256>> {
        match value {
            None => Ok(None),
            Some(d) => Ok(Some(parse_units(*d, "gwei")?)),
        }
    }

    pub fn price_to_u256(value: &str) -> anyhow::Result<U256> {
        let e = decimals_to_ethers(&value.to_owned())?;

        Ok(e)
    }

    pub fn price_to_u256_option(value: &Option<String>) -> anyhow::Result<Option<U256>> {
        match value {
            None => Ok(None),
            Some(d) => {
                let e = decimals_to_ethers(d)?;

                Ok(Some(e))
            }
        }
    }

    pub fn decimals_price_to_gwei<K>(value: &K) -> anyhow::Result<U256>
    where
        K: ToU256Units,
    {
        decimals_to_gwei(value)
    }

    pub fn decimals_price_to_gwei_option<K>(value: &Option<K>) -> anyhow::Result<Option<U256>>
    where
        K: ToU256Units,
    {
        match value {
            None => Ok(None),
            Some(d) => Ok(Some(decimals_to_gwei(d)?)),
        }
    }

    pub fn contract_to_h160(contract: &str) -> anyhow::Result<Address> {
        Ok(Address::from_str(contract)?)
    }

    pub fn contract_to_h160_option(contract: &Option<String>) -> anyhow::Result<Option<Address>> {
        match &contract {
            None => Ok(None),
            Some(d) => Ok(Some(Address::from_str(&*d)?)),
        }
    }

    pub fn convert_to_u256_option(value: &Option<u64>) -> anyhow::Result<Option<U256>> {
        match value {
            None => Ok(None),
            Some(d) => Ok(Some(U256::from(*d))),
        }
    }
}
