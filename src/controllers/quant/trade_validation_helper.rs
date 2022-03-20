use crate::common::helpers::gas::{gas_price_calc, increase_gas_price_by};
use crate::common::models::trade_scheme::TradeName;
use crate::common::utils::ether::ether_to_human_display;
use crate::common::utils::u256::to_u256;
use ethers::core::types::U256;
use qd::Quad;
use std::ops::{Add, Mul};

pub struct TradeValidationHelper;

pub struct CalculateGasPriceArgs {
    pub trade_name: TradeName,
    pub network_gas_price: U256,
    pub native_token_symbol: String,
    pub approve_gas_price: Option<String>,
    pub approve_gas_limit: Option<u64>,
    pub gas_price: Option<String>,
    pub gas_limit: Option<u64>,
    pub retry_attempts: Option<u64>,
    pub perc_increase_gas_price: Option<u32>,
}

impl TradeValidationHelper {
    pub fn multiply_network_gas_price(
        network_gas_price: U256,
        gas_fee: Quad,
    ) -> anyhow::Result<(U256, Quad)> {
        let gas_fee_factor = network_gas_price.mul(to_u256(&gas_fee)?);
        let gas_human_readable = ether_to_human_display(gas_fee_factor);

        Ok((gas_fee_factor, gas_human_readable))
    }

    pub fn calculate_gas_price(args: CalculateGasPriceArgs) -> anyhow::Result<(Quad, Vec<String>)> {
        let mut min_required_gas_price: Quad = Quad::from(0);
        let mut active_gas_fees_variables: Vec<String> = vec![];

        if let Some(p) = args.approve_gas_price {
            active_gas_fees_variables.push(format!("{}: 'approve_gas_price'", args.trade_name));

            if let Some(l) = args.approve_gas_limit {
                active_gas_fees_variables.push(format!("{}: 'approve_gas_limit'", args.trade_name));

                let gas_fee = gas_price_calc(p.as_str(), l.to_string().as_str())?;
                min_required_gas_price = min_required_gas_price.add(gas_fee);

                // print info
                let (_, approve_gas_human_readable) =
                    TradeValidationHelper::multiply_network_gas_price(
                        args.network_gas_price,
                        gas_fee,
                    )?;
                log::debug!(
                    "minimum gas required for approving the token in a '{}' function: {} {}",
                    args.trade_name,
                    approve_gas_human_readable,
                    args.native_token_symbol,
                );
            }
        }

        if let Some(p) = args.gas_price {
            active_gas_fees_variables.push(format!("{}: 'gas_price'", args.trade_name));

            if let Some(l) = args.gas_limit {
                active_gas_fees_variables.push(format!("{}: 'gas_limit'", args.trade_name));

                let pre_retry_gas_fee_quad = gas_price_calc(p.as_str(), l.to_string().as_str())?;
                let mut gas_fee = pre_retry_gas_fee_quad;

                // equation: (gas_price * gas_limit) + ((gas_price * gas_limit) * retry_attempts)
                if let Some(retry_count) = args.retry_attempts {
                    active_gas_fees_variables
                        .push(format!("{}: 'retry_attempts'", args.trade_name));

                    let mut perc_increase_gas_price: u64 = 0;
                    if let Some(perc_incr) = args.perc_increase_gas_price {
                        active_gas_fees_variables
                            .push(format!("{}: 'perc_increase_gas_price'", args.trade_name));

                        perc_increase_gas_price = perc_incr as u64;
                    }

                    let (retry_gas_fee, _) = increase_gas_price_by(
                        pre_retry_gas_fee_quad,
                        perc_increase_gas_price,
                        Some(retry_count),
                    )?;

                    gas_fee = gas_fee.add(retry_gas_fee);
                }

                min_required_gas_price = min_required_gas_price.add(gas_fee);

                // print info
                let (_, gas_human_readable) = TradeValidationHelper::multiply_network_gas_price(
                    args.network_gas_price,
                    gas_fee,
                )?;
                log::debug!(
                    "minimum gas required for purchasing the token in a '{}' function: {} {}",
                    args.trade_name,
                    gas_human_readable,
                    args.native_token_symbol,
                );
            }
        }

        // print info
        let (_, min_required_gas_price_human_readable) =
            TradeValidationHelper::multiply_network_gas_price(
                args.network_gas_price,
                min_required_gas_price,
            )?;
        log::debug!(
            "gross minimum gas required for purchasing the token in a '{}' function: {} {}",
            args.trade_name,
            min_required_gas_price_human_readable,
            args.native_token_symbol,
        );

        Ok((
            min_required_gas_price_human_readable,
            active_gas_fees_variables,
        ))
    }
}
