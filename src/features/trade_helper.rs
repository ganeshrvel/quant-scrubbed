use crate::common::constants::default_values::DefaultValues;
use crate::common::errors::TradingError;
use crate::common::helpers::gas::increase_gas_price_by;
use crate::common::helpers::urls::get_tx_hash_url;
use crate::common::models::trade_scheme::TradeSchemeVariant;
use crate::common::types::ChronoDuration;
use crate::common::utils::ether::ether_to_human_display;
use crate::common::utils::f256::{divide_into_f256, to_f256};
use crate::common::utils::u256::percentage_of_u256;
use crate::controllers::contracts::{Erc20Contract, QuantContracts};
use crate::controllers::quant::quant::{Quant, TradeContext};
use ethers::abi::ethereum_types::U256;
use ethers::abi::Address;
use ethers::contract::builders::ContractCall;
use ethers::core::types::TransactionReceipt;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{LocalWallet, Middleware, Provider, Ws};
use min_max::max;
use qd::Quad;
use std::ops::{Add, Sub};
use std::time::Instant;

pub struct QuantTradeHelper;

/// inputs for various methods
pub struct AmountInCtx {
    pub amount_of_token_in: String,
    pub amount_of_token_in_u256: U256,
    pub slippage: u64,
    pub token_in_contract: String,
    pub token_in_h160: Address,
    pub token_in_symbol: String,
}

pub struct AmountOutCtx {
    pub amount_out_list: Vec<U256>,
    pub token_out_contract: String,
    pub token_out_symbol: String,
    pub token_out_h160: Address,
}

pub struct GasCtx {
    pub gas_price: String,
    pub gas_price_gwei: U256,
    pub gas_limit: u64,
    pub gas_limit_u256: U256,
    pub tx_timeout_in_ms: i64,
    pub max_gas_price: Option<String>,
    pub max_gas_price_gwei: Option<U256>,
    pub perc_increase_gas_price: Option<u32>,
}

////////////////////////////////////////////////////////////

/// outputs for various methods
pub struct TokenInCTx {
    pub token_in_contract: String,
    pub token_in_symbol: String,
    pub token_in_h160: Address,
    pub amount_of_token_in: String,
    pub amount_of_token_in_u256: U256,
    pub slippage: u64,
}

pub struct TokenOutCTx {
    pub amount_out_min_u256: U256,
    pub amount_out_max_u256: U256,
    pub token_out_symbol: String,
    pub token_out_contract: String,
    pub token_out_h160: Address,
    pub amount_out_min_in_human_readable_format: Quad,
    pub amount_out_max_in_human_readable_format: Quad,
    pub price_of_token_out_per_token_in_human_readable_format: Quad,
}

pub struct GasTxCtx {
    pub gas_price: String,
    pub gas_price_gwei: U256,
    pub gas_limit: u64,
    pub gas_limit_u256: U256,
    pub tx_timeout_in_ms: i64,
    pub tx_deadline_u256: U256,
    pub tx_deadline: i64,
}

pub struct TokenPairR {
    pub paired_tokens_value: U256,
    pub paired_tokens_value_in_human_readable_format: Quad,
    pub token_paired_time: Instant,
}

////////////////////////////////////////////////////////////

impl<'a> QuantTradeHelper {
    // create token pair address
    // the method will wait until the token pair address is created if not found
    pub async fn create_pair<T>(ctx: &TradeContext<T>) -> anyhow::Result<Address>
    where
        T: TradeSchemeVariant,
    {
        let factory_get_pair_contract_call: ContractCall<
            SignerMiddleware<Provider<_>, LocalWallet>,
            Address,
        > = ctx
            .contracts
            .factory
            .get_pair(ctx.feed.token_in_h160, ctx.feed.token_out_h160);

        let mut pair_address: Address = Default::default();

        // loop until we fetch a valid pair address and we hit liquidity
        'pair_address_loop: loop {
            let pair_addr = factory_get_pair_contract_call.call().await;

            if let Ok(pa) = pair_addr {
                let pa_str = format!("{:?}", pa);

                log::debug!("\n",);
                log::debug!("detected a pair address: {}", pa_str);

                if pa_str.starts_with("0x0000000000000") {
                    log::warn!("no liquidity pool found, retrying...");
                    log::debug!("\n",);
                } else {
                    pair_address = pa;

                    log::debug!("valid token pair address found: {:?}", pair_address);

                    break 'pair_address_loop;
                }
            };
        }

        Ok(pair_address)
    }

    // create token pair value
    // the method will wait until the minimum required liquidity is found
    pub async fn token_pair_value(
        contracts: &QuantContracts,
        pair_address: Address,
        min_liquidity_required: &Option<U256>,
    ) -> anyhow::Result<TokenPairR> {
        log::debug!("fetching paired token value...");

        let erc20_balance_of_contract_call: ContractCall<
            SignerMiddleware<Provider<_>, LocalWallet>,
            U256,
        > = contracts.token_in_erc20.balance_of(pair_address);

        let mut check_min_liquidity_hit = false;
        let mut paired_tokens_value: U256 = Default::default();
        let mut token_value_to_eth_to_human_display: Quad = Quad::from(0);

        if min_liquidity_required.is_some() {
            check_min_liquidity_hit = true;
            log::debug!("checking minimum liquidity required...");
        }

        'min_liquidity_hit_loop: loop {
            let erc20_balance = erc20_balance_of_contract_call.call().await;

            match erc20_balance {
                Ok(token_value) => {
                    paired_tokens_value = token_value;
                    token_value_to_eth_to_human_display =
                        ether_to_human_display(paired_tokens_value);

                    log::debug!("\n");
                    log::debug!("erc20 balance: {}", token_value);
                    log::debug!(
                        "detected some liquidity for the token pair: {}",
                        token_value_to_eth_to_human_display
                    );

                    if let Some(min_liquidity_required_u256) = min_liquidity_required {
                        if token_value >= *min_liquidity_required_u256 {
                            log::info!(
                                "hit the minimum required liquidity: {}",
                                token_value_to_eth_to_human_display
                            );

                            break 'min_liquidity_hit_loop;
                        } else {
                            log::warn!(
                                "min required liquidity not available for the token pair, retrying..."
                            );
                            log::debug!("\n");
                        }
                    } else {
                        if check_min_liquidity_hit {
                            log::info!(
                                "hit the minimum required liquidity: {:?}",
                                paired_tokens_value
                            );
                        }

                        break 'min_liquidity_hit_loop;
                    }
                }
                Err(e) => {
                    log::error!("{:?}", e);
                    log::warn!("liquidity checking call was unsuccessful, retrying...");
                }
            }
        }

        let token_paired_time = Instant::now();

        Ok(TokenPairR {
            token_paired_time,
            paired_tokens_value,
            paired_tokens_value_in_human_readable_format: token_value_to_eth_to_human_display,
        })
    }

    pub(crate) async fn get_token_balance_in_account(
        q: &Quant,
        contracts: &QuantContracts,
    ) -> anyhow::Result<(Quad, U256)> {
        let erc20_balance_of_in_account_contract_call: ContractCall<
            SignerMiddleware<Provider<_>, LocalWallet>,
            U256,
        > = contracts
            .token_in_erc20
            .balance_of(q.variables.account_address_h160);

        let erc20_balance_in_account_u256 =
            erc20_balance_of_in_account_contract_call.call().await?;

        let erc20_balance_in_account_human_readable_format =
            ether_to_human_display(erc20_balance_in_account_u256);

        Ok((
            erc20_balance_in_account_human_readable_format,
            erc20_balance_in_account_u256,
        ))
    }

    async fn start_token_approval(
        ecr20_contract: &Erc20Contract,
        q: &Quant,
        approve_gas_price_gwei: U256,
        approve_gas_limit_u256: U256,
    ) -> anyhow::Result<()> {
        log::debug!("approving the token...");

        'approve_token_loop: loop {
            let approve_token_contract_call: ContractCall<
                SignerMiddleware<Provider<Ws>, LocalWallet>,
                bool,
            > = ecr20_contract.approve(
                q.variables.router_in_h160,
                DefaultValues::TOKEN_ALLOWANCE_MAX_AMOUNT,
            );

            let approve_token_contract_call = approve_token_contract_call
                .gas(approve_gas_limit_u256)
                .gas_price(approve_gas_price_gwei);

            let approve_token_pending_tx_res = q
                .middleware
                .client
                .send_transaction(approve_token_contract_call.tx, None)
                .await;

            match approve_token_pending_tx_res {
                Ok(t) => {
                    let tx_hash = *t;
                    let tx_url = get_tx_hash_url(tx_hash, q.variables.network_name.clone());

                    log::info!("tx hash ({:?}) {}", tx_hash, tx_url);
                    log::debug!("waiting for the tx receipt...");

                    let tx_receipt_call = &t.await;

                    match tx_receipt_call {
                        Ok(tx_receipt) => match tx_receipt {
                            Some(r) => match r.status {
                                None => {
                                    log::error!("the approve token tx status did not return anything, trying to approve the token again...");
                                }
                                Some(s) => {
                                    let status_code = s.as_u32();

                                    if status_code == 1 {
                                        log::debug!("approve token tx receipt received");
                                        log::info!("token approved!");

                                        break 'approve_token_loop;
                                    } else {
                                        log::error!("the approve token tx status returned failure, trying to approve the token again...");
                                    }
                                }
                            },
                            None => {
                                log::error!("approve token tx receipt did not return anything, trying to approve the token again...");
                            }
                        },
                        Err(e) => {
                            log::error!("{:?}", e);
                            log::warn!("approve token tx receipt returned an error, trying to approve the token again...");
                        }
                    }
                }
                Err(e) => {
                    log::error!("{:?}", e);
                    log::warn!("approve token tx call was unsuccessful, retrying...");
                }
            }
        }

        Ok(())
    }

    // approve token if it wasn't already approved
    pub async fn approve(
        contracts: &QuantContracts,
        q: &Quant,
        approve_gas_price_gwei_option: Option<U256>,
        approve_gas_limit_u256_option: Option<U256>,
        token_in_symbol: String,
    ) -> anyhow::Result<Option<()>> {
        let approve_gas_price_gwei: U256;
        let approve_gas_limit_u256: U256;

        if let Some(a) = approve_gas_price_gwei_option {
            approve_gas_price_gwei = a;
        } else {
            return Ok(None);
        }

        if let Some(a) = approve_gas_limit_u256_option {
            approve_gas_limit_u256 = a;
        } else {
            return Ok(None);
        }

        log::debug!(
            "checking the approval status of {} token...",
            token_in_symbol
        );

        let erc20_allowance_contract_call: ContractCall<
            SignerMiddleware<Provider<_>, LocalWallet>,
            U256,
        > = contracts
            .token_in_erc20
            .allowance(q.variables.account_address_h160, q.variables.router_in_h160);

        log::debug!("fetching allowance of the token...");

        let allowance = erc20_allowance_contract_call.call().await?;

        log::debug!("allowance found: {:?}", allowance);

        if allowance >= DefaultValues::token_allowance_min_amount()? {
            log::debug!("the token was already approved, exiting the approval...");

            return Ok(None);
        }

        Self::start_token_approval(
            &contracts.token_in_erc20,
            q,
            approve_gas_price_gwei,
            approve_gas_limit_u256,
        )
        .await?;

        Ok(Some(()))
    }

    pub fn token_in(amount_in_ctx: &AmountInCtx) -> anyhow::Result<TokenInCTx> {
        // we have to strip the decimals off the [amount_of_token_in] in order to carry out arithemetic the operations related to ether
        let amount_of_token_in = amount_in_ctx.amount_of_token_in.to_string();
        let amount_of_token_in_u256 = amount_in_ctx.amount_of_token_in_u256;

        let t = TokenInCTx {
            amount_of_token_in_u256,
            amount_of_token_in,
            slippage: amount_in_ctx.slippage,
            token_in_contract: amount_in_ctx.token_in_contract.to_owned(),
            token_in_symbol: amount_in_ctx.token_in_symbol.to_owned(),
            token_in_h160: amount_in_ctx.token_in_h160,
        };

        Ok(t)
    }

    pub fn token_out(
        amount_in_ctx: &AmountInCtx,
        amount_out_ctx: &AmountOutCtx,
    ) -> anyhow::Result<TokenOutCTx> {
        let amount_out_max_u256 = amount_out_ctx.amount_out_list[1];
        let amount_out_max_in_human_readable_format = ether_to_human_display(amount_out_max_u256);

        // the correction is done here because Quad library doesn't accept decimals as input for [amount_out_min]
        // we first strip the decimal points out of it to convert [amount_out_min] into units of ethers
        let amount_out_min_correction_u256 = amount_out_max_u256.sub(percentage_of_u256(
            amount_out_max_u256,
            amount_in_ctx.slippage,
        ));
        let amount_out_min_in_human_readable_format =
            ether_to_human_display(amount_out_min_correction_u256);

        let price_of_token_out_per_token_in_human_readable_format = divide_into_f256(
            &amount_in_ctx.amount_of_token_in,
            &amount_out_max_in_human_readable_format,
        );

        let t = TokenOutCTx {
            amount_out_min_in_human_readable_format,
            amount_out_min_u256: amount_out_min_correction_u256,
            amount_out_max_u256,
            amount_out_max_in_human_readable_format,
            token_out_symbol: amount_out_ctx.token_out_symbol.to_owned(),
            price_of_token_out_per_token_in_human_readable_format,
            token_out_h160: amount_out_ctx.token_out_h160,
            token_out_contract: amount_out_ctx.token_out_contract.to_owned(),
        };

        Ok(t)
    }

    pub fn gas(gas_ctx: &GasCtx, trade_attempt_count: u64) -> anyhow::Result<GasTxCtx> {
        let tx_timeout_in_ms = gas_ctx.tx_timeout_in_ms;
        let tx_deadline = chrono::offset::Utc::now()
            .add(ChronoDuration::milliseconds(tx_timeout_in_ms))
            .timestamp_millis();
        let tx_deadline_u256 = U256::from(tx_deadline);

        let mut gas_price = gas_ctx.gas_price.to_owned();
        let mut gas_price_gwei = gas_ctx.gas_price_gwei;
        let trade_retry_attempt_count = max!(trade_attempt_count - 1, 0);

        let max_gas_price_str = match &gas_ctx.max_gas_price {
            None => "",
            Some(d) => d.as_str(),
        };

        if trade_retry_attempt_count > 0 {
            if let Some(increase_in_perc) = gas_ctx.perc_increase_gas_price {
                log::debug!("received 'perc_increase_gas_price' {}", increase_in_perc);
                log::debug!(
                    "attempting to increase the gas price by: {}%",
                    increase_in_perc
                );

                let increase_by_factor = increase_in_perc as u64 * trade_retry_attempt_count;
                let (gas_price_new, gas_price_gwei_new) =
                    increase_gas_price_by(to_f256(gas_price.to_owned().as_str()), increase_by_factor, None)?;

                let gas_price_new_str = gas_price_new.to_string();

                log::debug!("newly computed gas price: {} gwei", gas_price_new_str);

                match gas_ctx.max_gas_price_gwei {
                    None => {
                        log::debug!("DID NOT receive the 'max_gas_price_gwei'");

                        gas_price = gas_price_new_str;
                        gas_price_gwei = gas_price_gwei_new;

                        log::info!("increasing the gas price to: {} gwei", gas_price);
                    }
                    Some(m) => {
                        log::debug!("received 'max_gas_price_gwei' {}", m);

                        if gas_price_gwei_new > m {
                            log::warn!("the newly computed gas price ({} gwei) has crossed the max allowed gas price ({} gwei), will continue without increasing the gas price...",gas_price_new_str, max_gas_price_str);
                        } else {
                            gas_price = gas_price_new_str;
                            gas_price_gwei = gas_price_gwei_new;

                            log::info!("increasing the gas price to: {} gwei", gas_price);
                        }
                    }
                }
            }
        }

        let g = GasTxCtx {
            gas_price,
            gas_price_gwei,
            gas_limit: gas_ctx.gas_limit,
            gas_limit_u256: gas_ctx.gas_limit_u256,
            tx_timeout_in_ms,
            tx_deadline_u256,
            tx_deadline,
        };

        Ok(g)
    }

    pub fn print_info(
        token_in_ctx: &TokenInCTx,
        token_out_ctx: &TokenOutCTx,
        gas_tx_ctx: &GasTxCtx,
        quant: &Quant,
    ) {
        log::debug!("Transaction details:");

        log::info!(
            "Token Input: {} ({})",
            token_in_ctx.token_in_symbol,
            token_in_ctx.token_in_contract
        );
        log::info!(
            "Token Output: {} ({})",
            token_out_ctx.token_out_symbol,
            token_out_ctx.token_out_contract
        );
        log::info!(
            "Amount of token in: {}({})",
            token_in_ctx.amount_of_token_in,
            token_in_ctx.token_in_symbol
        );

        log::info!(
            "Price: {:.14} {} per {}",
            token_out_ctx.price_of_token_out_per_token_in_human_readable_format,
            token_in_ctx.token_in_symbol,
            token_out_ctx.token_out_symbol,
        );

        log::info!(
            "Amount of {} tokens you will get for {} {}s: {}",
            token_out_ctx.token_out_symbol,
            token_in_ctx.amount_of_token_in,
            token_in_ctx.token_in_symbol,
            token_out_ctx.amount_out_max_in_human_readable_format,
        );

        log::info!(
            "Minimum amount of {} you will get: {}",
            token_out_ctx.token_out_symbol,
            token_out_ctx.amount_out_min_in_human_readable_format
        );

        log::info!("Slippage Tolerance: {}%", token_in_ctx.slippage);
        log::info!("Account address: {}", quant.variables.account_address);
        log::info!("Gas Price (GWEI): {}", gas_tx_ctx.gas_price);
        log::info!("Gas Limit: {}", gas_tx_ctx.gas_limit);
    }

    pub fn swap_deflationary_tokens_contract_call<T>(
        ctx: &TradeContext<T>,
        token_in_ctx: &TokenInCTx,
        token_out_ctx: &TokenOutCTx,
        gas_tx_ctx: &GasTxCtx,
        quant: &Quant,
    ) -> ContractCall<SignerMiddleware<Provider<Ws>, LocalWallet>, Vec<U256>>
    where
        T: TradeSchemeVariant,
    {
        ctx.contracts.router.swap_exact_tokens_for_tokens(
            token_in_ctx.amount_of_token_in_u256,
            token_out_ctx.amount_out_min_u256,
            vec![token_in_ctx.token_in_h160, token_out_ctx.token_out_h160],
            quant.variables.account_address_h160,
            gas_tx_ctx.tx_deadline_u256,
        )
    }

    pub fn swap_non_deflationary_tokens_contract_call<T>(
        ctx: &TradeContext<T>,
        token_in_ctx: &TokenInCTx,
        token_out_ctx: &TokenOutCTx,
        gas_tx_ctx: &GasTxCtx,
        quant: &Quant,
    ) -> ContractCall<SignerMiddleware<Provider<Ws>, LocalWallet>, Vec<U256>>
    where
        T: TradeSchemeVariant,
    {
        ctx.contracts
            .router
            .swap_exact_tokens_for_tokens_supporting_fee_on_transfer_tokens(
                token_in_ctx.amount_of_token_in_u256,
                token_out_ctx.amount_out_min_u256,
                vec![token_in_ctx.token_in_h160, token_out_ctx.token_out_h160],
                quant.variables.account_address_h160,
                gas_tx_ctx.tx_deadline_u256,
            )
    }

    pub async fn swap_tokens<T>(
        ctx: &TradeContext<T>,
        token_in_ctx: &TokenInCTx,
        token_out_ctx: &TokenOutCTx,
        gas_tx_ctx: &GasTxCtx,
        quant: &Quant,
        is_deflationary_token: bool,
    ) -> anyhow::Result<TransactionReceipt>
    where
        T: TradeSchemeVariant,
    {
        log::debug!("initializing token swapping...");

        let contract_call;

        if is_deflationary_token {
            log::debug!("found deflationary token");

            contract_call = Self::swap_deflationary_tokens_contract_call(
                ctx,
                token_in_ctx,
                token_out_ctx,
                gas_tx_ctx,
                quant,
            )
        } else {
            log::debug!("found non deflationary token");

            contract_call = Self::swap_non_deflationary_tokens_contract_call(
                ctx,
                token_in_ctx,
                token_out_ctx,
                gas_tx_ctx,
                quant,
            )
        }

        let swap_tx = contract_call
            .gas(gas_tx_ctx.gas_limit_u256)
            .gas_price(gas_tx_ctx.gas_price_gwei);

        log::debug!("attempting to send the swap tokens transaction...");

        let pending_tx = quant
            .middleware
            .client
            .send_transaction(swap_tx.tx, None)
            .await;

        match pending_tx {
            Ok(t) => {
                let tx_hash = *t;
                let tx_url = get_tx_hash_url(tx_hash, quant.variables.network_name.clone());

                log::info!("tx hash ({:?}) {}", tx_hash, tx_url);
                log::debug!("waiting for the tx receipt...");

                let tx_receipt_call = &t.await;
                match tx_receipt_call {
                    Ok(tx_receipt) => match tx_receipt {
                        Some(r) => match r.status {
                            None => {
                                return Err(TradingError::SwapToken(
                                    "the swap tokens tx status did not return anything",
                                )
                                .into());
                            }
                            Some(s) => {
                                let status_code = s.as_u32();

                                // SUCCESS!!
                                if status_code == 1 {
                                    log::debug!("swap tokens tx receipt received");
                                    log::debug!("swap tokens tx receipt: {:?}", r);

                                    Ok(r.clone())
                                } else {
                                    return Err(TradingError::SwapToken(
                                        "the swap tokens tx status returned failure",
                                    )
                                    .into());
                                }
                            }
                        },
                        None => {
                            return Err(TradingError::SwapToken(
                                "the swap tokens tx receipt did not return anything",
                            )
                            .into());
                        }
                    },
                    Err(e) => {
                        let err_text = "swap tokens tx receipt returned an error";
                        log::error!("{:?}", e);

                        return Err(TradingError::SwapToken(err_text).into());
                    }
                }
            }
            Err(e) => {
                let err_text = "some error occured while sending the swap tokens transaction";
                log::error!("{:?}", e);

                return Err(TradingError::SwapToken(err_text).into());
            }
        }
    }
}
