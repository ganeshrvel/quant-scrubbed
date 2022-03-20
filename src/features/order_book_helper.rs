use crate::common::helpers::date::get_elapsed_time_ms;
use crate::features::trade_helper::{AmountInCtx, TokenInCTx};
use ethers::core::types::TransactionReceipt;
use std::time::Instant;

pub struct OrderBookHelper;

pub struct OrderBookR {
    pub transaction_receipt: Option<TransactionReceipt>,
    pub token_in_ctx: TokenInCTx,
    pub amount_in_ctx: AmountInCtx,
}

impl OrderBookHelper {
    pub fn time_since_token_paired(token_paired_time: &Instant) -> i64 {
        let time_elapsed_since_token_pairing_ms = get_elapsed_time_ms(token_paired_time);

        log::debug!(
            "[wait_time_before_first_tx_attempt_in_ms]  the elapsed time since pairing: {}ms",
            time_elapsed_since_token_pairing_ms
        );

        time_elapsed_since_token_pairing_ms as i64
    }
}
