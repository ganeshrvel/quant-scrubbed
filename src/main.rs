#![warn(clippy::all)]
#![warn(
    clippy::print_literal,
    clippy::print_with_newline,
    clippy::println_empty_string
)]

#[macro_use]
mod macros;

use crate::common::constants::strings::Strings;
use crate::common::helpers::parsers::setting_files::SettingFiles;
use crate::common::utils::logs::fern_log::setup_logging;
use crate::controllers::cli::entry_points::EntryPoints;
use crate::controllers::quant::quant::{Quant, QuantFeature};
use crate::features::trade::QuantTrade;
use dialoguer::console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;

mod common;
mod controllers;
mod features;

fn main() {
    println!("initializing the logger...");
    let s = setup_logging();

    if let Err(e) = s {
        paniq!("failed to initialize the logging (P00001): {}", e)
    }

    log::debug!("-----------------");
    log::debug!("Launching {}...", Strings::APP_NAME);

    let rt = tokio::runtime::Runtime::new().unwrap();
    if let Err(e) = rt.block_on(run()) {
        log::error!("{:?}", e);
    }
}

async fn run() -> anyhow::Result<()> {
    let settings = SettingFiles::new();
    let entry_points = EntryPoints::new()?;

    let quant = Quant::new(&settings, &entry_points).await?;

    if entry_points.config_check {
        log::info!("config check was successful, exiting...");

        return Ok(());
    }

    if entry_points.dry_run {
        log::info!("conducting a dry run...");
        log::info!("Note: an 'INSUFFICIENT_INPUT_AMOUNT' error from factory need not mean that there is a config error. It could be because there are no available tokens in the wallet and a dry run for BuySell could throw an error.");
    }

    // press enter to start the trade
    if entry_points.wait_to_continue {
        log::info!("received 'wait_to_continue' entry point");
        log::info!("waiting for the user response to continue with the trading..\n\n");

        let items = vec!["Press enter to continue with the trading..."];
        let _ = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact_on_opt(&Term::stderr())?;
    }

    match &quant.feature {
        QuantFeature::TokenTransfers(token_transfer_ctx) => {}

        QuantFeature::Trading(trading_ctx) => {
            QuantTrade::new(&quant, &entry_points, trading_ctx).await?;
        }
    }

    Ok(())
}
