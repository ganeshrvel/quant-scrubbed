use crate::common::helpers::date::get_elapsed_time_ms;
use crate::controllers::cli::entry_points::EntryPoints;
use crate::controllers::quant::quant::{Quant, QuantTrading};
use crate::features::buy::QuantBuy;
use crate::features::sell::QuantSell;
use std::time::Instant;

pub struct QuantTrade;

impl QuantTrade {
    pub async fn new(
        q: &Quant,
        entry_points: &EntryPoints,
        trading_ctx: &QuantTrading,
    ) -> anyhow::Result<Self> {
        let trade_start_time = Instant::now();
        let trade_start_time_locale = chrono::Local::now();
        log::debug!(
            "trading start time: {}",
            trade_start_time_locale.format("%Y-%m-%d %H:%M:%S.%f")
        );

        // <dry run print info>
        if entry_points.dry_run {
            log::debug!("printing dry run...");

            if let Some(bucket) = &trading_ctx.buy_context {
                if let Some(d) = bucket.first() {
                    log::info!(
                        "'token_in_contract' for Buy function: {:?} ({})",
                        d.feed.token_in_contract,
                        d.scheme.token_in_symbol
                    );
                    log::info!(
                        "'token_out_contract' for Buy function: {:?} ({})",
                        d.feed.token_out_contract,
                        d.scheme.token_in_symbol
                    );
                }
            }

            if let Some(bucket) = &trading_ctx.sell_context {
                if let Some(d) = bucket.first() {
                    log::info!(
                        "'token_in_contract' for Sell function: {:?} ({})",
                        d.feed.token_in_contract,
                        d.scheme.token_in_symbol
                    );
                    log::info!(
                        "'token_out_contract' for Sell function: {:?} ({})",
                        d.feed.token_out_contract,
                        d.scheme.token_in_symbol
                    );
                }
            }
        }
        // <!dry run print info>

        if let Some(bucket) = &trading_ctx.buy_context {
            if let Some(d) = bucket.first() {
                let buy = QuantBuy::new(d, q, entry_points).await?;
            }

            let elapsed_trade_time = get_elapsed_time_ms(&trade_start_time);
            log::debug!(
                "elapsed Buy trade time: {:?} milliseconds",
                elapsed_trade_time
            );
        }

        if let Some(bucket) = &trading_ctx.sell_context {
            if let Some(d) = bucket.first() {
                let sell = QuantSell::new(d, q, entry_points).await?;
            }
            let elapsed_trade_time = get_elapsed_time_ms(&trade_start_time);
            log::debug!(
                "elapsed Sell trade time: {:?} milliseconds",
                elapsed_trade_time
            );
        }

        let elapsed_trade_time = get_elapsed_time_ms(&trade_start_time);
        log::debug!(
            "elapsed trading time: {:?} milliseconds",
            elapsed_trade_time
        );

        Ok(Self)
    }
}
