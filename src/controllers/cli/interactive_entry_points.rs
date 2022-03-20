use crate::common::models::config::NetworkName;
use crate::controllers::cli::entry_points::{EntryPoints, TradeType};
use dialoguer::console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;

impl EntryPoints {
    pub fn interactive_network_name() -> anyhow::Result<NetworkName> {
        let selections = vec![
            NetworkName::Mainnet.to_string(),
            NetworkName::Testnet.to_string(),
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select your network type")
            .default(0)
            .items(&selections)
            .interact_on_opt(&Term::stderr())?;

        let network_name: NetworkName = match selection {
            Some(index) => {
                let t: NetworkName;

                if index == 0 {
                    t = NetworkName::Mainnet;
                } else if index == 1 {
                    t = NetworkName::Testnet;
                } else {
                    paniq!("unknown interative cli input for network name (P00008)")
                }

                t
            }
            None => paniq!("unknown interative cli input for network name (P00009)"),
        };

        Ok(network_name)
    }

    pub fn interactive_trade_name() -> anyhow::Result<TradeType> {
        let selections = vec![
            TradeType::BuySell.to_string(),
            TradeType::Buy.to_string(),
            TradeType::Sell.to_string(),
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select your trade type")
            .default(0)
            .items(&selections)
            .interact_on_opt(&Term::stderr())?;

        let trade_type: TradeType = match selection {
            Some(index) => {
                let t: TradeType;

                if index == 0 {
                    t = TradeType::BuySell;
                } else if index == 1 {
                    t = TradeType::Buy;
                } else if index == 2 {
                    t = TradeType::Sell;
                } else {
                    paniq!("unknown interative cli input for trade type (P00010)")
                }

                t
            }
            None => paniq!("unknown interative cli input for trade type (P00011)"),
        };

        Ok(trade_type)
    }
}
