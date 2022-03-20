use crate::common::constants::strings::Strings;
use crate::common::errors::CliError;
use crate::common::models::config::NetworkName;
use crate::controllers::cli::entry_points::{EntryPoints, TradeType};
use clap::{App, Arg};
use std::str::FromStr;

pub struct CliEntryPointsR {
    pub network_type: Option<NetworkName>,
    pub trade_type: Option<TradeType>,
    pub config_check: bool,
    pub dry_run: bool,
    pub wait_to_continue: bool,
}

impl EntryPoints {
    pub fn cli_args() -> anyhow::Result<CliEntryPointsR> {
        let matches = App::new(Strings::APP_NAME)
            .args(&[
                Arg::new("networktype")
                    .about("Sets the network type")
                    .takes_value(true)
                    .short('n')
                    .long("networktype")
                    .case_insensitive(true)
                    .possible_values(&[
                        NetworkName::Mainnet.to_string().to_lowercase().as_str(),
                        NetworkName::Testnet.to_string().to_lowercase().as_str(),
                    ]),
                Arg::new("tradetype")
                    .about("Sets the trade type")
                    .takes_value(true)
                    .short('t')
                    .long("tradetype")
                    .case_insensitive(true)
                    .possible_values(&[
                        TradeType::BuySell.to_string().to_lowercase().as_str(),
                        TradeType::Buy.to_string().to_lowercase().as_str(),
                        TradeType::Sell.to_string().to_lowercase().as_str(),
                    ]),
                Arg::new("configcheck")
                    .about("Checks the config files")
                    .takes_value(false)
                    .short('c')
                    .long("configcheck")
                    .required(false),
                Arg::new("dryrun")
                    .about("Conduct a dry run. It will approve, check minimum liquidity and print the token info but it will stop just before the token swapping.")
                    .takes_value(false)
                    .short('d')
                    .long("dryrun")
                    .required(false),
                Arg::new("wait-to-continue")
                    .about("Show a confirmation action before starting the trade. The program will initialize and do all sanity checks before showing the confirmation action on the screen.")
                    .takes_value(false)
                    .short('w')
                    .long("wait-to-continue")
                    .required(false),
            ])
            .get_matches();

        let mut network_type: Option<NetworkName> = None;
        let mut trade_type: Option<TradeType> = None;
        let mut config_check = false;
        let mut dry_run = false;
        let mut wait_to_continue = false;

        if let Some(c) = matches.value_of("networktype") {
            log::debug!("received 'networktype={}' from command line arguments", c);

            network_type = match NetworkName::from_str(c) {
                Ok(d) => Some(d),
                Err(e) => return Err(CliError::Invalid(e).into()),
            };
        }

        if let Some(c) = matches.value_of("tradetype") {
            log::debug!("received 'tradetype={}' from command line arguments", c);

            trade_type = match TradeType::from_str(c) {
                Ok(d) => Some(d),
                Err(e) => return Err(CliError::Invalid(e).into()),
            };
        }

        if matches.is_present("configcheck") {
            log::debug!("received 'configcheck' from command line arguments");

            config_check = true
        }

        if matches.is_present("dryrun") {
            log::debug!("received 'dryrun' from command line arguments");

            dry_run = true
        }

        if matches.is_present("wait-to-continue") {
            log::debug!("received 'wait-to-continue' from command line arguments");

            wait_to_continue = true
        }

        let ep = CliEntryPointsR {
            network_type,
            trade_type,
            config_check,
            dry_run,
            wait_to_continue,
        };

        Ok(ep)
    }
}
