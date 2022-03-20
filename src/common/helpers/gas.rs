use crate::common::utils::ether::decimals_to_gwei;
use crate::common::utils::f256::{percentage_of_f256, to_f256};
use ethers::abi::ethereum_types::U256;
use qd::Quad;
use std::ops::{Add, Mul};

pub fn increase_gas_price_by(
    gas_price: Quad,
    perc_increase_by: u64,
    attempts: Option<u64>,
) -> anyhow::Result<(Quad, U256)> {
    let gas_price_perc = percentage_of_f256(&gas_price, &perc_increase_by);

    let mut gas_price_perc_add_by_factor = gas_price.add(gas_price_perc);

    if let Some(a) = attempts {
        gas_price_perc_add_by_factor = gas_price_perc_add_by_factor.mul(to_f256(a));
    }

    Ok((
        gas_price_perc_add_by_factor,
        decimals_to_gwei(&gas_price_perc_add_by_factor.to_string())?,
    ))
}

pub fn gas_price_calc(gas_price: &str, gas_limit: &str) -> anyhow::Result<Quad> {
    let gas_price_quad = to_f256(gas_price);
    let gas_limit_quad = to_f256(gas_limit);

    let r = gas_price_quad.mul(gas_limit_quad);

    Ok(r)
}
