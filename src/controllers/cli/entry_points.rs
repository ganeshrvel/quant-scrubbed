use crate::common::models::config::NetworkName;
use crate::controllers::cli::cli_entry_points::CliEntryPointsR;
use core::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum TradeType {
    BuySell,
    Buy,
    Sell,
}

impl fmt::Display for TradeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for TradeType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "buysell" => Ok(TradeType::BuySell),
            "buy" => Ok(TradeType::Buy),
            "sell" => Ok(TradeType::Sell),
            _ => Err("an error occured while trying to convert string to 'TradeType'"),
        }
    }
}

#[derive(Debug)]
pub struct EntryPoints {
    pub network_name: NetworkName,
    pub trade_type: TradeType,
    pub config_check: bool,
    pub dry_run: bool,
    pub wait_to_continue: bool,
}

impl EntryPoints {
    fn network_name(from_cli_arg: Option<NetworkName>) -> anyhow::Result<NetworkName> {
        let network_name = match from_cli_arg {
            None => Self::interactive_network_name()?,
            Some(d) => d,
        };

        Ok(network_name)
    }

    fn trade_name(from_cli_arg: Option<TradeType>) -> anyhow::Result<TradeType> {
        let trade_name = match from_cli_arg {
            None => Self::interactive_trade_name()?,
            Some(d) => d,
        };

        Ok(trade_name)
    }

    pub fn new() -> anyhow::Result<EntryPoints> {
        let CliEntryPointsR {
            network_type: cli_arg_network_name,
            trade_type: cli_arg_trade_type,
            config_check,
            dry_run,
            wait_to_continue,
        } = Self::cli_args()?;

        let network_name = Self::network_name(cli_arg_network_name)?;
        let trade_type = Self::trade_name(cli_arg_trade_type)?;

        let ep = EntryPoints {
            network_name,
            trade_type,
            config_check,
            dry_run,
            wait_to_continue,
        };

        Ok(ep)
    }
}
