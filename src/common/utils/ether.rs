use crate::common::utils::f256::divide_into_f256;
use crate::common::utils::u256::{decimals_to_u256, ToU256Units};
use ethers::abi::ethereum_types::U256;
use ethers::core::utils::{parse_units, WEI_IN_ETHER};
use std::ops::Div;

pub fn ether_to_human_display(value: U256) -> qd::Quad {
    divide_into_f256(&value, &WEI_IN_ETHER)
}

pub fn decimals_to_ethers<K>(value: &K) -> anyhow::Result<U256>
where
    K: ToU256Units,
{
    let (value_decimal_stripped_u256, value_decimals_unit) = decimals_to_u256(value)?;

    let value_parsed_to_eth = parse_units(value_decimal_stripped_u256, "ether")?;

    // the correction is done here because ethers-js library doesnt accept decimals as input for [amount_of_token_in]
    // we first strip the decimal points out of it and then later divide the number by the 10^deciman_units
    let value_corrected_u256: U256;
    if value_decimals_unit > 0 {
        value_corrected_u256 = value_parsed_to_eth.div(10_u64.pow(value_decimals_unit as u32));
    } else {
        value_corrected_u256 = value_parsed_to_eth;
    }

    Ok(value_corrected_u256)
}

pub fn decimals_to_gwei<K>(value: &K) -> anyhow::Result<U256>
where
    K: ToU256Units,
{
    let (value_decimal_stripped_u256, value_decimals_unit) = decimals_to_u256(value)?;

    let value_parsed_to_gwei = parse_units(value_decimal_stripped_u256, "gwei")?;

    // the correction is done here because ethers-js library doesnt accept decimals as input for [amount_of_token_in]
    // we first strip the decimal points out of it and then later divide the number by the 10^deciman_units
    let value_corrected_u256: U256;
    if value_decimals_unit > 0 {
        value_corrected_u256 = value_parsed_to_gwei.div(10_u64.pow(value_decimals_unit as u32));
    } else {
        value_corrected_u256 = value_parsed_to_gwei;
    }

    Ok(value_corrected_u256)
}
