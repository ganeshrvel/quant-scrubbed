use crate::common::constants::default_values::DefaultValues;
use crate::common::errors::{OrderBookError, TradingError};
use crate::common::helpers::date::get_elapsed_time_ms;
use crate::common::helpers::timer::{interruptable_sleep, tokio_sleep};
use crate::common::helpers::urls::get_tx_hash_url;
use crate::common::models::trade_scheme::{SellScheme, TradeSchemeVariant};
use crate::common::utils::ether::decimals_to_ethers;
use crate::common::utils::f256::percentage_of_f256;
use crate::controllers::cli::entry_points::EntryPoints;
use crate::controllers::quant::quant::{Quant, TradeContext};
use crate::features::order_book_helper::{OrderBookHelper, OrderBookR};
use crate::features::trade_helper::{
    AmountInCtx, AmountOutCtx, GasCtx, QuantTradeHelper, TokenPairR,
};
use ethers::middleware::SignerMiddleware;
use ethers::prelude::builders::ContractCall;
use ethers::providers::Provider;
use ethers::signers::LocalWallet;
use ethers::types::{TransactionReceipt, U256};
use min_max::max;
use std::time::Instant;

pub struct QuantSell {
    transaction_receipt: Option<TransactionReceipt>,
}

impl QuantSell {
    async fn amount_token_in(
        ctx: &TradeContext<SellScheme>,
        q: &Quant,
    ) -> anyhow::Result<AmountInCtx> {
        let amount_of_token_in = &ctx.scheme.amount_of_token_in.to_owned();
        let amount_of_token_in_u256 = ctx.scheme.amount_of_token_in_u256()?;
        let perc_of_token_in = &ctx.scheme.perc_of_token_in.to_owned();

        let mut amount_of_token_in_computed: String = Default::default();
        let mut amount_of_token_in_u256_computed = Default::default();

        // check if [amount_of_token_in] is available
        // if it exists then assign the [amount_of_token_in] value
        // else fetch the token balance from the account and use it along with [perc_of_token_in].
        match amount_of_token_in_u256 {
            Some(amount) => {
                log::debug!("received 'amount_of_token_in_u256': {:?}", amount);

                amount_of_token_in_u256_computed = amount;

                if let Some(amount_str) = amount_of_token_in {
                    log::debug!("received 'amount_of_token_in': {}", amount_str);

                    amount_of_token_in_computed = amount_str.to_owned();
                }
            }
            None => match perc_of_token_in {
                Some(perc) => {
                    log::debug!("received 'perc_of_token_in': {}", perc);

                    let (token_balance_in_account_quad, token_balance_in_account_u256) =
                        QuantTradeHelper::get_token_balance_in_account(q, &ctx.contracts).await?;

                    log::debug!(
                        "total available {} balance in the account: {}",
                        ctx.scheme.token_in_symbol,
                        token_balance_in_account_quad
                    );

                    log::debug!(
                        "total available {} balance in the account in u256: {}",
                        ctx.scheme.token_in_symbol,
                        token_balance_in_account_u256
                    );

                    let amount_of_token_after_perc_correction =
                        percentage_of_f256(&token_balance_in_account_quad, perc);
                    let amount_of_token_after_perc_human_readable_format =
                        decimals_to_ethers(&amount_of_token_after_perc_correction)?;

                    amount_of_token_in_u256_computed =
                        amount_of_token_after_perc_human_readable_format;
                    amount_of_token_in_computed = amount_of_token_after_perc_correction.to_string();
                }
                None => {
                    log::error!("both 'amount_of_token_in' and 'perc_of_token_in' are unavailable in the 'Sell' function, the trade may fail, continuing anyway...");
                }
            },
        }

        let a = AmountInCtx {
            amount_of_token_in: amount_of_token_in_computed,
            amount_of_token_in_u256: amount_of_token_in_u256_computed,
            slippage: ctx.scheme.slippage as u64,
            token_in_contract: ctx.feed.token_in_contract.to_owned(),
            token_in_h160: ctx.feed.token_in_h160,
            token_in_symbol: ctx.scheme.token_in_symbol.to_owned(),
        };

        Ok(a)
    }

    async fn amount_token_out(
        ctx: &TradeContext<SellScheme>,
        amount_of_token_in_u256: U256,
    ) -> anyhow::Result<AmountOutCtx> {
        let amount_out_list_contract_call: ContractCall<
            SignerMiddleware<Provider<_>, LocalWallet>,
            Vec<U256>,
        > = ctx.contracts.router.get_amounts_out(
            amount_of_token_in_u256,
            vec![ctx.feed.token_in_h160, ctx.feed.token_out_h160],
        );

        let amount_out_list = amount_out_list_contract_call.call().await?;

        let a = AmountOutCtx {
            amount_out_list,
            token_out_contract: ctx.feed.token_out_contract.to_owned(),
            token_out_symbol: ctx.scheme.token_out_symbol.to_owned(),
            token_out_h160: ctx.feed.token_out_h160,
        };

        Ok(a)
    }

    fn gas(ctx: &TradeContext<SellScheme>) -> anyhow::Result<GasCtx> {
        let g = GasCtx {
            gas_price: ctx.scheme.gas_price.to_owned(),
            gas_price_gwei: ctx.scheme.gas_price_gwei()?,
            gas_limit: ctx.scheme.gas_limit,
            gas_limit_u256: ctx.scheme.gas_limit_u256()?,
            tx_timeout_in_ms: ctx.scheme.tx_timeout_in_ms,
            max_gas_price: ctx.scheme.max_gas_price.to_owned(),
            max_gas_price_gwei: ctx.scheme.max_gas_price_gwei()?,
            perc_increase_gas_price: ctx.scheme.perc_increase_gas_price,
        };

        Ok(g)
    }

    async fn order_book(
        ctx: &TradeContext<SellScheme>,
        q: &Quant,
        entry_points: &EntryPoints,
        token_pair: &TokenPairR,
    ) -> anyhow::Result<OrderBookR> {
        log::debug!("initializing the order book...");

        let wait_time_before_first_tx_attempt_in_ms =
            ctx.scheme.wait_time_before_first_tx_attempt_in_ms;
        let time_between_retries_in_ms = ctx.scheme.time_between_retries_in_ms;
        let retry_attempts = ctx.scheme.retry_attempts;
        let max_allowed_time_for_trading_in_ms = ctx.scheme.max_allowed_time_for_trading_in_ms;
        let token_paired_time = token_pair.token_paired_time;

        if let Some(attempt_after_ms) = wait_time_before_first_tx_attempt_in_ms {
            log::debug!(
                "found 'wait_time_before_first_tx_attempt_in_ms': {}",
                attempt_after_ms
            );

            let time_elapsed_since_token_pairing_ms =
                OrderBookHelper::time_since_token_paired(&token_paired_time);

            if attempt_after_ms > time_elapsed_since_token_pairing_ms {
                let sleep_for_ms = attempt_after_ms - time_elapsed_since_token_pairing_ms;
                let sleep_for_ms = max!(sleep_for_ms, 0) as u64;

                log::debug!(
                    "[wait_time_before_first_tx_attempt_in_ms] delaying the 'First Sell Attempt' for another {} ms",
                    sleep_for_ms
                );

                //////////////////////////////
                //////////////////////////////
                //////////////////////////////
                // todo: this is a temporary hack to interrupt the the 'First Sell Attempt'
                //  remove this while converting this whole thing into a micro service
                interruptable_sleep(
                    DefaultValues::SELL_INTERRUPTER_KEYWORD.parse()?,
                    ctx.scheme.name.clone(),
                    sleep_for_ms,
                )
                .await?;
                //////////////////////////////
                //////////////////////////////
                //////////////////////////////

                log::debug!("[wait_time_before_first_tx_attempt_in_ms] waking up from the 'First Sell Attempt' sleep");
            } else {
                log::debug!("[wait_time_before_first_tx_attempt_in_ms] didn't have to delay the 'First Sell Attempt'");
            }
        }

        // [amount_in_ctx] and [token_in_ctx] are fetched after the [wait_time_before_first_tx_attempt_in_ms] sleep in the 'Sell' function because we could manually sell off some tokens in between the BuySell function
        // create token in amount context
        let amount_in_ctx = Self::amount_token_in(ctx, q).await?;

        // create token in context
        let token_in_ctx = QuantTradeHelper::token_in(&amount_in_ctx)?;

        let mut trade_attempt_count = 1_u64;
        let first_trade_attempt_instant = Instant::now();

        loop {
            log::debug!("attempting to sell #{}...", trade_attempt_count);
            let current_trade_attempt_instant = Instant::now();

            // token amount out handler
            let amount_out_ctx =
                Self::amount_token_out(ctx, token_in_ctx.amount_of_token_in_u256).await?;
            let token_out_ctx = QuantTradeHelper::token_out(&amount_in_ctx, &amount_out_ctx)?;

            // gas price handler
            let gas_ctx = Self::gas(ctx)?;
            let gas_tx_ctx = QuantTradeHelper::gas(&gas_ctx, trade_attempt_count)?;

            log::debug!("\n");
            QuantTradeHelper::print_info(&token_in_ctx, &token_out_ctx, &gas_tx_ctx, q);
            log::debug!("\n\n\n");

            if entry_points.dry_run {
                log::info!("dry run successfull...");
                log::debug!("\n\n\n");

                let order_r = OrderBookR {
                    transaction_receipt: None,
                    token_in_ctx,
                    amount_in_ctx,
                };

                return Ok(order_r);
            }

            let swap_result = QuantTradeHelper::swap_tokens(
                ctx,
                &token_in_ctx,
                &token_out_ctx,
                &gas_tx_ctx,
                q,
                ctx.scheme.is_token_out_deflationary,
            )
            .await;

            let elapsed_time_since_first_trade_attempt =
                get_elapsed_time_ms(&first_trade_attempt_instant);
            let elapsed_time_since_the_current_trade_attempt =
                get_elapsed_time_ms(&current_trade_attempt_instant);

            match swap_result {
                Ok(swap_tx_receipt) => {
                    let tx_url = get_tx_hash_url(
                        swap_tx_receipt.transaction_hash,
                        q.variables.network_name.clone(),
                    );

                    log::info!("YAY!!! The Sell trade was successful!");
                    log::info!(
                        "tx hash ({:?}) {}",
                        swap_tx_receipt.transaction_hash,
                        tx_url
                    );
                    log::debug!("finishing up the Sell trade");
                    log::info!(
                        "this successful transaction attempt (#{}) took {:?} ms to execute",
                        trade_attempt_count,
                        elapsed_time_since_the_current_trade_attempt
                    );
                    log::info!(
                        "the elapsed time to execute this transaction since the first attempt: {:?} ms",
                        elapsed_time_since_first_trade_attempt
                    );

                    let order_r = OrderBookR {
                        transaction_receipt: Some(swap_tx_receipt),
                        token_in_ctx,
                        amount_in_ctx,
                    };

                    return Ok(order_r);
                }
                Err(swap_result_err) => {
                    log::info!(
                        "this failed transaction attempt (#{}) took {:?} ms to execute",
                        trade_attempt_count,
                        elapsed_time_since_the_current_trade_attempt
                    );
                    log::info!(
                        "the elapsed time to execute this transaction since the first attempt: {:?} ms",
                        elapsed_time_since_first_trade_attempt
                    );

                    if let Some(e) = swap_result_err.downcast_ref::<TradingError>() {
                        if let TradingError::SwapToken(_) = e {
                            log::error!(
                                "the swap_tokens method returned a 'SwapToken' error: {:?}",
                                e
                            );

                            log::warn!("will attempt to retry the Sell trade again")
                        }
                    } else {
                        log::error!(
                            "the swap_tokens method returned an unknown error : {:?}",
                            swap_result_err
                        );

                        log::debug!("terminating the Sell trade...");

                        return Err(swap_result_err);
                    }
                }
            }

            // if the [retry_attempts] is available then check whether the number of attempts has reached
            if let Some(r) = retry_attempts {
                log::debug!("received 'retry_attempts': {}", r);

                if r < trade_attempt_count {
                    log::debug!(
                        "retry attempt to sell has ended at #{}",
                        trade_attempt_count
                    );

                    return Err(OrderBookError::Sell(
                        "exhausted the retry attempts to carry out the Sell trade",
                    )
                    .into());
                }
            }

            // if the [max_allowed_time_for_trading_in_ms] is available then check whether the max allowed trading time has reached
            if let Some(max_allowed_time_for_trading_in_ms_ok) = max_allowed_time_for_trading_in_ms
            {
                log::debug!(
                    "received 'max_allowed_time_for_trading_in_ms': {}",
                    max_allowed_time_for_trading_in_ms_ok
                );
                log::debug!("terminating the Sell trade...");

                let time_elapsed_since_token_pairing_ms =
                    OrderBookHelper::time_since_token_paired(&token_paired_time);

                if time_elapsed_since_token_pairing_ms >= max_allowed_time_for_trading_in_ms_ok {
                    return Err(OrderBookError::Sell(
                        "exhausted the max allowed time to carry out the Sell trade",
                    )
                    .into());
                }
            }

            // increment [trade_attempt_count] to keep track of the atempts
            trade_attempt_count += 1;

            // if [time_between_retries_in_ms] is present then delay the next attempt for the said time
            if let Some(time_between_retries) = time_between_retries_in_ms {
                log::debug!("delaying the retry attempt for {} ms", time_between_retries);

                tokio_sleep(time_between_retries as u64).await;
            }
        }
    }

    pub async fn new(
        ctx: &TradeContext<SellScheme>,
        q: &Quant,
        entry_points: &EntryPoints,
    ) -> anyhow::Result<Self> {
        log::debug!("initializing the 'Sell' function...");

        // create token pair address
        // the method will wait until the token pair address is created if not found
        let pair_address = QuantTradeHelper::create_pair(ctx).await?;

        // create token pair value
        // the method will wait until the minimum required liquidity is found
        let token_pair = QuantTradeHelper::token_pair_value(
            &ctx.contracts,
            pair_address,
            &ctx.scheme.min_liquidity_required_u256()?,
        )
        .await?;

        // approve token if it wasn't already approved
        let _ = QuantTradeHelper::approve(
            &ctx.contracts,
            q,
            ctx.scheme.approve_gas_price_gwei()?,
            ctx.scheme.approve_gas_limit_u256()?,
            ctx.scheme.token_in_symbol.to_owned(),
        )
        .await?;

        // create the order sell
        let o = Self::order_book(ctx, q, entry_points, &token_pair).await?;

        let q_sell = QuantSell {
            transaction_receipt: o.transaction_receipt,
        };

        Ok(q_sell)
    }
}
