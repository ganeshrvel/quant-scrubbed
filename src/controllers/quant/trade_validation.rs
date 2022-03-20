use std::cmp::Ordering;
use std::ops::Add;

use ethers::abi::ethereum_types::{Address, U256};
use qd::Quad;

use crate::common::helpers::ethers::{get_account_balance, get_network_gas_price};
use crate::common::models::config::ProviderEntity;
use crate::common::models::trade_scheme::{SellScheme, TradeActuatorForSell, TradeName, TradeScheme, TradeSchemeVariant};
use crate::common::utils::ether::ether_to_human_display;
use crate::controllers::cli::entry_points::TradeType;
use crate::controllers::middleware::QuantMiddleware;
use crate::controllers::quant::trade_validation_helper::{
    CalculateGasPriceArgs, TradeValidationHelper,
};

pub struct TradeValidation;

pub struct CheckGasFeesBalanceVariables {
    pub account_address_h160: Address,
    pub buy_approve_gas_price: Option<String>,
    pub buy_approve_gas_limit: Option<u64>,
    pub buy_gas_price: Option<String>,
    pub buy_gas_limit: Option<u64>,
    pub buy_retry_attempts: Option<u64>,
    pub buy_perc_increase_gas_price: Option<u32>,

    pub sell_approve_gas_price: Option<String>,
    pub sell_approve_gas_limit: Option<u64>,
    pub sell_gas_price: Option<String>,
    pub sell_gas_limit: Option<u64>,
    pub sell_retry_attempts: Option<u64>,
    pub sell_perc_increase_gas_price: Option<u32>,
    pub native_token_symbol: String,
}

impl TradeValidation {
    // [gas_price], [max_gas_price] and [perc_increase_gas_price]
    fn check_gas_price(
        trade_scheme: &TradeScheme,
        perc_increase_gas_price: Option<u32>,
        max_gas_price_gwei: Option<U256>,
        gas_price_gwei: U256,
    ) -> anyhow::Result<()> {
        if perc_increase_gas_price.is_some() && max_gas_price_gwei.is_none() {
            paniq!("config file error in {}:\n\n'max_gas_price' is mandatory when 'perc_increase_gas_price' is active (P00015a)", trade_scheme)
        }

        if let Some(price) = max_gas_price_gwei {
            if gas_price_gwei > price {
                paniq!("config file error in {}:\n\n'max_gas_price' should be greater than or equal to 'gas_price' (P00015c)", trade_scheme)
            }
        }

        Ok(())
    }

    // [approve_gas_price] and [approve_gas_limit]
    fn check_approve_gas_fees(
        trade_scheme: &TradeScheme,
        approve_gas_price: Option<String>,
        approve_gas_limit: Option<u64>,
    ) -> anyhow::Result<()> {
        if approve_gas_price.is_some() && approve_gas_limit.is_none() {
            paniq!("config file error in {}:\n\n'approve_gas_limit' is mandatory when 'approve_gas_price' is active (P00019a)", trade_scheme)
        }

        Ok(())
    }

    // Check if the wallet has enough base token to spend on the gas
    pub async fn check_gas_fees_balance(
        middleware: &QuantMiddleware,
        v: &CheckGasFeesBalanceVariables,
    ) -> anyhow::Result<()> {
        log::debug!("initializing gas fees checker...");

        log::debug!("fetching account balance...");
        let account_balance =
            get_account_balance(&middleware.client, &v.account_address_h160).await?;

        log::debug!("fetching the network gas price...");
        let network_gas_price = get_network_gas_price(&middleware.client).await?;

        let mut min_required_gas_price_human_readable_format: Quad = Quad::from(0);
        let mut active_gas_fees_variables: Vec<String> = vec![];

        let buy_calculate_gas_price_args = CalculateGasPriceArgs {
            trade_name: TradeName::Buy,
            network_gas_price,
            native_token_symbol: v.native_token_symbol.to_owned(),
            approve_gas_price: v.buy_approve_gas_price.to_owned(),
            approve_gas_limit: v.buy_approve_gas_limit,
            gas_price: v.buy_gas_price.to_owned(),
            gas_limit: v.buy_gas_limit,
            retry_attempts: v.buy_retry_attempts,
            perc_increase_gas_price: v.buy_perc_increase_gas_price,
        };

        let (buy_min_required_gas_price_human_readable_format, mut buy_active_gas_fees_variables) =
            TradeValidationHelper::calculate_gas_price(buy_calculate_gas_price_args)?;

        // sell ->>
        let sell_calculate_gas_price_args = CalculateGasPriceArgs {
            trade_name: TradeName::Sell,
            network_gas_price,
            native_token_symbol: v.native_token_symbol.to_owned(),
            approve_gas_price: v.sell_approve_gas_price.to_owned(),
            approve_gas_limit: v.sell_approve_gas_limit,
            gas_price: v.sell_gas_price.to_owned(),
            gas_limit: v.sell_gas_limit,
            retry_attempts: v.sell_retry_attempts,
            perc_increase_gas_price: v.sell_perc_increase_gas_price,
        };

        let (sell_min_required_gas_price_human_readable_format, mut sell_active_gas_fees_variables) =
            TradeValidationHelper::calculate_gas_price(sell_calculate_gas_price_args)?;

        min_required_gas_price_human_readable_format = min_required_gas_price_human_readable_format
            .add(buy_min_required_gas_price_human_readable_format)
            .add(sell_min_required_gas_price_human_readable_format);
        active_gas_fees_variables.append(&mut buy_active_gas_fees_variables);
        active_gas_fees_variables.append(&mut sell_active_gas_fees_variables);

        let account_balance_human_readable = ether_to_human_display(account_balance);

        log::info!(
            "account balance: {} {}",
            account_balance_human_readable,
            v.native_token_symbol
        );

        log::debug!(
            "minimum gas required : {} {}",
            min_required_gas_price_human_readable_format,
            v.native_token_symbol
        );

        // if the account balance is lower than the minimum required gas price then return an error
        if min_required_gas_price_human_readable_format > account_balance_human_readable {
            paniq!("config file error: you need atleast {} {} in your wallet to spend on the gas fees (P00020a).\nyou only have {} {} as your wallet balance.\nactive gas variables in the config file: \"{}\"", min_required_gas_price_human_readable_format, v.native_token_symbol.to_owned(), account_balance_human_readable,v.native_token_symbol.to_owned(), active_gas_fees_variables.join(", "));
        }

        Ok(())
    }

    // [perc_of_token_in] && [amount_of_token_in]
    fn perc_of_token_in(
        trade_scheme: &TradeScheme,
        amount_of_token_in: Option<String>,
        perc_of_token_in: Option<u8>,
    ) -> anyhow::Result<()> {
        let mut token_in_scheme_count = 0;

        if amount_of_token_in.is_some() {
            token_in_scheme_count += 1;
        }

        if let Some(p) = perc_of_token_in {
            if p > 100 {
                paniq!("config file error in {}:\n\n'perc_of_token_in' cannot be greater than 100 percentage (P00012c)", trade_scheme)
            }

            token_in_scheme_count += 1;
        }

        // [1] is used as the comparing factor here because at most only either of the [amount_of_token_in] or [perc_of_token_in] should be used.
        // we panic the otherwise
        match token_in_scheme_count.cmp(&1) {
            Ordering::Greater => paniq!("config file error in {}:\n\nonly either one of 'perc_of_token_in' or 'amount_of_token_in' is allowed in the Sell function (P00012a)", trade_scheme),
            Ordering::Less => paniq!("config file error in {}:\n\natleast either of 'perc_of_token_in' or 'amount_of_token_in' is required in the Sell function (P00012b)", trade_scheme),
            _  => {}
        }

        Ok(())
    }

    // [max_allowed_time_for_trading_in_ms], [wait_time_before_first_tx_attempt_in_ms], [time_between_retries_in_ms] && [retry_attempts]
    fn trading_time(
        trade_scheme: &TradeScheme,
        max_allowed_time_for_trading_in_ms: Option<i64>,
        wait_time_before_first_tx_attempt_in_ms: Option<i64>,
        time_between_retries_in_ms: Option<i64>,
        retry_attempts: Option<u64>,
    ) -> anyhow::Result<()> {
        // check whether the [max_allowed_time_for_trading_in_ms] is greater then the other time limiting params
        if let Some(max_allowed_time_for_trading_in_ms_ok) = max_allowed_time_for_trading_in_ms {
            let mut wait_time_before_first_tx_attempt_in_ms_ok = 0_i64;
            let mut time_between_retries_in_ms_ok = 0_i64;
            let mut retry_attempts_ok = 0_u64;

            if let Some(v) = wait_time_before_first_tx_attempt_in_ms {
                wait_time_before_first_tx_attempt_in_ms_ok = v;
            }

            if let Some(v) = time_between_retries_in_ms {
                time_between_retries_in_ms_ok = v;
            }

            if let Some(v) = retry_attempts {
                retry_attempts_ok = v;
            }

            let min_req_time_in_ms = wait_time_before_first_tx_attempt_in_ms_ok
                + (time_between_retries_in_ms_ok * retry_attempts_ok as i64);

            // if [max_allowed_time_for_trading_in_ms_ok] is lesser than the time limiting params then throw an error
            if max_allowed_time_for_trading_in_ms_ok < min_req_time_in_ms {
                paniq!("config file error in {}:\n\n'max_allowed_time_for_trading_in_ms' should be at least: {} (P00018a)", trade_scheme, min_req_time_in_ms);
            }
        }

        Ok(())
    }

    pub async fn sanity_check_trade_schemes(s: &TradeScheme) -> anyhow::Result<()> {
        match s {
            TradeScheme::Buy(d) => {
                // [gas_price], [max_gas_price] and [perc_increase_gas_price]
                Self::check_gas_price(
                    s,
                    d.perc_increase_gas_price,
                    d.max_gas_price_gwei()?,
                    d.gas_price_gwei()?,
                )?;

                // [approve_gas_price] and [approve_gas_limit]
                Self::check_approve_gas_fees(
                    s,
                    d.approve_gas_price.to_owned(),
                    d.approve_gas_limit,
                )?;

                Self::trading_time(
                    s,
                    d.max_allowed_time_for_trading_in_ms,
                    d.wait_time_before_first_tx_attempt_in_ms,
                    d.time_between_retries_in_ms,
                    d.retry_attempts,
                )?;
            }
            TradeScheme::Sell(d) => {
                // [perc_of_token_in] && [amount_of_token_in]
                Self::perc_of_token_in(s, d.amount_of_token_in.to_owned(), d.perc_of_token_in)?;

                // [gas_price], [max_gas_price] and [perc_increase_gas_price]
                Self::check_gas_price(
                    s,
                    d.perc_increase_gas_price,
                    d.max_gas_price_gwei()?,
                    d.gas_price_gwei()?,
                )?;

                // [approve_gas_price] and [approve_gas_limit]
                Self::check_approve_gas_fees(
                    s,
                    d.approve_gas_price.to_owned(),
                    d.approve_gas_limit,
                )?;

                Self::trading_time(
                    s,
                    d.max_allowed_time_for_trading_in_ms,
                    d.wait_time_before_first_tx_attempt_in_ms,
                    d.time_between_retries_in_ms,
                    d.retry_attempts,
                )?;
            }
        }

        Ok(())
    }

    // check and confirm whether in a BuySell trade both 'token_out_contract' of Sell and 'token_in_contract' of Buy are the same
    pub fn validate_buysell_fn_tokens(
        buy_token_out_contract: Option<String>,
        sell_token_in_contract: Option<String>,
    ) -> anyhow::Result<()> {
        if buy_token_out_contract.is_none() || sell_token_in_contract.is_none() {
            paniq!("config file error: 'token_out_contract' of Sell and 'token_in_contract' of Buy needs to be the same (P00016a)", )
        }

        if sell_token_in_contract != buy_token_out_contract {
            paniq!("config file error: 'token_out_contract' of Sell and 'token_in_contract' of Buy needs to be the same (P00016a)", )
        }

        Ok(())
    }

    // check sell scheme actuator
    fn trade_actuator_sell_scheme(
        trade_scheme: &TradeScheme,
        sell_scheme: &SellScheme,
        t: &TradeType,
    ) -> anyhow::Result<()> {
        // <--- check and confirm that only either one of 'trade_at_price|hold_trade_below_price' or 'trade_at_profit_perc|hold_trade_below_profit_perc' combination is used in the trade actuator --->
        let mut trade_actuator_count = 0;

        if sell_scheme.trade_at_price.is_some() || sell_scheme.hold_trade_below_price.is_some() {
            trade_actuator_count += 1;
        }

        if sell_scheme.trade_at_profit_perc.is_some()
            || sell_scheme.hold_trade_below_profit_perc.is_some()
        {
            trade_actuator_count += 1;
        }

        if trade_actuator_count.cmp(&1) == Ordering::Greater {
            paniq!("config file error in {}:\n\nonly either one of 'trade_at_price|hold_trade_below_price' or 'trade_at_profit_perc|hold_trade_below_profit_perc' combination is allowed in the Sell function (P00013a)", trade_scheme)
        }
        // </--- check and confirm that only either one of 'trade_at_price|hold_trade_below_price' or 'trade_at_profit_perc|hold_trade_below_profit_perc' combination is used in the trade actuator --->

        ////////////////////////////////////////////////////////
        ////////////////////////////////////////////////////////
        // <--- check and confirm that either one of 'trade_at_profit_perc' and 'hold_trade_below_profit_perc' are ONLY used in the [TradeType::BuySell] --->
        if *t != TradeType::BuySell {
            let a = sell_scheme.trade_actuator()?;

            if let Some(TradeActuatorForSell::Percentage(_)) = a {
                paniq!("config file error in {}:\n\neither of 'trade_at_profit_perc' and 'hold_trade_below_profit_perc' are only allowed in the BuySell function (P00014a)", trade_scheme)
            };
        }
        // </--- check and confirm that either of 'trade_at_profit_perc' and 'hold_trade_below_profit_perc' are ONLY used in the [TradeType::BuySell] --->

        Ok(())
    }

    pub fn sanity_check_trade_schemes_for_trade_types(
        s: &TradeScheme,
        t: &TradeType,
    ) -> anyhow::Result<()> {
        match s {
            TradeScheme::Buy(_) => {}
            TradeScheme::Sell(sell_scheme) => {
                // check scheme actuator
                Self::trade_actuator_sell_scheme(s, sell_scheme, t)?;
            }
        }

        Ok(())
    }

    pub fn provider_validation(provider: &ProviderEntity) -> anyhow::Result<()> {
        let mut basic_auth_fields_count = 0;

        if provider.username.is_some() {
            basic_auth_fields_count += 1;
        }
        if provider.password.is_some() {
            basic_auth_fields_count += 1;
        }

        if basic_auth_fields_count.cmp(&1) == Ordering::Equal {
            paniq!("config file error: both 'username' and 'password' fields are mandatory if Basic Authorization has to be be used in the provider (P00017a)")
        }

        Ok(())
    }
}
