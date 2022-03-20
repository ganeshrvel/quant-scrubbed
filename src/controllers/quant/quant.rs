use std::str::FromStr;

use ethers::abi::ethereum_types::Address;

use crate::common::constants::chain_ids::ChainIds;
use crate::common::errors::SetupError;
use crate::common::helpers::parsers::setting_files::SettingFiles;
use crate::common::models::config::{NetworkEntity, NetworkName};
use crate::common::models::secrets::AccountEntity;
use crate::common::models::trade_scheme::{
    BuyScheme, SellScheme, TradeScheme, TradeSchemeVariant, Trades,
};
use crate::common::models::{config, secrets};
use crate::controllers::cli::entry_points::{EntryPoints, TradeType};
use crate::controllers::cli::trade_inputs::TradeInputs;
use crate::controllers::contracts::{QuantContracts, QuantContractsArgs};
use crate::controllers::middleware::QuantMiddleware;
use crate::controllers::providers::QuantProvider;
use crate::controllers::quant::trade_validation::{CheckGasFeesBalanceVariables, TradeValidation};

type TradingContexts = (
    Option<Vec<TradeContext<BuyScheme>>>,
    Option<Vec<TradeContext<SellScheme>>>,
);

#[derive(Debug)]
pub struct Quant {
    pub variables: QuantVariables,
    pub middleware: QuantMiddleware,
    pub provider: QuantProvider,
    pub feature: QuantFeature,
}

#[derive(Debug)]
pub enum QuantFeature {
    TokenTransfers(QuantTokenTransfer),
    Trading(QuantTrading),
}

#[derive(Debug)]
pub struct QuantTokenTransfer {}

#[derive(Debug)]
pub struct QuantTrading {
    pub buy_context: Option<Vec<TradeContext<BuyScheme>>>,
    pub sell_context: Option<Vec<TradeContext<SellScheme>>>,
}

#[derive(Debug)]
pub struct QuantVariables {
    pub account_address_h160: Address,
    pub account_address: String,
    pub factory_addr_h160: Address,
    pub native_token_h160: Address,
    pub router_in_h160: Address,
    pub usd_token_h160: Address,
    pub mnemonic: String,
    pub chain_id: u64,
    pub network_name: NetworkName,
    pub check_gas_fees: bool,
    pub native_token_symbol: String,
}

#[derive(Debug)]
pub struct TradeContext<T>
where
    T: TradeSchemeVariant,
{
    pub contracts: QuantContracts,
    pub scheme: T,

    /// some of the fields in the trade schemes are optional.
    /// these fields requires user's input via CLI
    /// use [feed] field when you need to access those fields
    pub feed: TradeCliParsed,
}

#[derive(Debug)]
pub struct TradeCliParsed {
    pub token_in_contract: String,
    pub token_in_h160: Address,
    pub token_out_contract: String,
    pub token_out_h160: Address,
}

impl<'a> Quant {
    fn variables(
        selected_secrets_network_account: &'a AccountEntity,
        selected_config_network: &'a NetworkEntity,
        entry_points: &'a EntryPoints,
    ) -> anyhow::Result<QuantVariables> {
        let account_address_h160 = selected_secrets_network_account.address_h160()?;
        let account_address = &selected_secrets_network_account.address;
        let factory_addr_h160 = selected_config_network.factory_h160()?;
        let native_token_h160 = selected_config_network.native_token_h160()?;
        let usd_token_h160 = selected_config_network.usd_token_h160()?;
        let router_in_h160 = selected_config_network.router_h160()?;
        let mnemonic = &selected_secrets_network_account.mnemonic;
        let network_name = &entry_points.network_name;
        let chain_id = match entry_points.network_name {
            NetworkName::Mainnet => ChainIds::BSC_MAINNET,
            NetworkName::Testnet => ChainIds::BSC_TESTNET,
        };
        let check_gas_fees = &selected_config_network.check_gas_fees;
        let native_token_symbol = &selected_config_network.native_token_symbol;

        let v = QuantVariables {
            account_address_h160,
            account_address: account_address.to_string(),
            factory_addr_h160,
            native_token_h160,
            router_in_h160,
            usd_token_h160,
            mnemonic: mnemonic.to_string(),
            chain_id,
            network_name: network_name.clone(),
            check_gas_fees: *check_gas_fees,
            native_token_symbol: native_token_symbol.to_string(),
        };

        Ok(v)
    }

    async fn provider(selected_config_network: &NetworkEntity) -> anyhow::Result<QuantProvider> {
        let provider = match selected_config_network.providers.first() {
            None => return Err(SetupError::Settings("no provider found (E00007)").into()),
            Some(d) => d.clone().provider,
        };

        TradeValidation::provider_validation(&provider)?;

        let providers = QuantProvider::new(&provider).await?;

        Ok(providers)
    }

    fn middleware(
        providers: &QuantProvider,
        chain_id: u64,
        mnemonic: &str,
    ) -> anyhow::Result<QuantMiddleware> {
        let m = QuantMiddleware::new(providers.ws.clone(), chain_id, mnemonic)?;

        Ok(m)
    }

    fn contracts(args: QuantContractsArgs) -> anyhow::Result<QuantContracts> {
        let c = QuantContracts::new(args);

        Ok(c)
    }

    async fn set_buy_queue(
        ctx: &Option<Vec<TradeContext<BuyScheme>>>,
        s: BuyScheme,
        variables: &QuantVariables,
        middleware: &QuantMiddleware,
    ) -> anyhow::Result<TradeContext<BuyScheme>> {
        log::debug!("setting up the Buy function");

        if ctx.is_some() {
            paniq!("config file error: a maximum of one trade entity for 'buy' in 'config.trades.trade' is allowed (P00010a)");
        }

        TradeValidation::sanity_check_trade_schemes(&TradeScheme::Buy(s.clone())).await?;

        let token_out_contract: String;
        let token_out_h160: Address;

        // Why we are doing this:
        // if a token has to be Sniped, the devs usually give the token at the time of liquidity
        // this wont give us much time to edit the config files
        // we instead try to input the token straight from the CLI.
        // [token_out_contract] of Buy will be [token_in_contract] of Sell
        match &s.token_out_contract {
            // if [token_out_contract] is present in the config.yaml file then
            // pick that
            Some(d) => {
                token_out_contract = d.to_string();
                token_out_h160 = Address::from_str(&*d)?
            }
            // if [token_out_contract] is NOT present in the config.yaml file then
            // then ask the user for input from the cli
            None => {
                token_out_contract =
                    TradeInputs::token_out_contract(s.token_out_symbol.to_owned())?;
                token_out_h160 = Address::from_str(&*token_out_contract)?
            }
        }

        let cli_parsed = TradeCliParsed {
            token_out_contract,
            token_out_h160,
            token_in_contract: s.token_in_contract.to_owned(),
            token_in_h160: s.token_in_h160()?,
        };

        let q_args = QuantContractsArgs {
            client: &middleware.client,
            factory_addr_h160: variables.factory_addr_h160,
            token_in_h160: cli_parsed.token_in_h160,
            router_in_h160: variables.router_in_h160,
            native_token_h160: variables.native_token_h160,
        };

        let q_contracts = Self::contracts(q_args)?;

        let c = TradeContext {
            contracts: q_contracts,
            scheme: s,
            feed: cli_parsed,
        };

        Ok(c)
    }

    async fn set_sell_queue(
        ctx: &Option<Vec<TradeContext<SellScheme>>>,
        s: SellScheme,
        variables: &QuantVariables,
        middleware: &QuantMiddleware,
        buy_preselected_cli_parsed: Option<&TradeCliParsed>,
    ) -> anyhow::Result<TradeContext<SellScheme>> {
        log::debug!("setting up the Sell function");

        if ctx.is_some() {
            paniq!("config file error: a maximum of one trade entity for 'sell' in 'config.trades.trade' is allowed (P00010b)");
        }

        TradeValidation::sanity_check_trade_schemes(&TradeScheme::Sell(s.clone())).await?;

        let token_in_contract: String;
        let token_in_h160: Address;

        match &s.token_in_contract {
            // if [token_in_contract] is present in the config.yaml file then
            // pick that
            Some(d) => {
                token_in_contract = d.to_string();
                token_in_h160 = Address::from_str(&*d)?
            }
            // if [token_in_contract] is NOT present in the config.yaml file then
            // check if [buy_preselected_cli_parsed] has the [token_out_contract] value from the Buy trade function
            // Why we are doing this:
            // if a token has to be Sniped, the devs usually give the token at the time of liquidity
            // this wont give us much time to edit the config files
            // we instead try to input the token straight from the CLI.
            // [token_in_contract] of Sell will be [token_out_contract] of Buy
            None => match buy_preselected_cli_parsed {
                // if [token_out_contract] is present in the [buy_preselected_cli_parsed]
                // pick that
                Some(d) => {
                    token_in_contract = d.token_out_contract.clone();
                    token_in_h160 = d.token_out_h160;
                }

                // if [token_out_contract] is NOT present in the [buy_preselected_cli_parsed]
                // then ask the user for input from the cli
                None => {
                    token_in_contract =
                        TradeInputs::token_in_contract(s.token_in_symbol.to_owned())?;
                    token_in_h160 = Address::from_str(&*token_in_contract)?
                }
            },
        }

        let cli_parsed = TradeCliParsed {
            token_out_contract: s.token_out_contract.to_owned(),
            token_out_h160: s.token_out_h160()?,
            token_in_contract,
            token_in_h160,
        };

        let q_args = QuantContractsArgs {
            client: &middleware.client,
            factory_addr_h160: variables.factory_addr_h160,
            token_in_h160: cli_parsed.token_in_h160,
            router_in_h160: variables.router_in_h160,
            native_token_h160: variables.native_token_h160,
        };

        let q_contracts = Self::contracts(q_args)?;

        let c = TradeContext {
            contracts: q_contracts,
            scheme: s,
            feed: cli_parsed,
        };

        Ok(c)
    }

    async fn trading_contexts(
        entry_points: &'a EntryPoints,
        trades: &[Trades],
        variables: &QuantVariables,
        middleware: &QuantMiddleware,
    ) -> anyhow::Result<TradingContexts> {
        let mut buy_context: Option<Vec<TradeContext<BuyScheme>>> = None;
        let mut sell_context: Option<Vec<TradeContext<SellScheme>>> = None;

        let mut sell_token_in_contract: Option<String> = None;
        let mut buy_token_out_contract: Option<String> = None;

        match entry_points.trade_type {
            TradeType::BuySell => {
                for t in trades {
                    match t.clone().trade {
                        TradeScheme::Buy(s) => {
                            TradeValidation::sanity_check_trade_schemes_for_trade_types(
                                &t.trade,
                                &entry_points.trade_type,
                            )?;

                            let o =
                                Self::set_buy_queue(&buy_context, s, variables, middleware).await?;

                            buy_token_out_contract = Some(o.feed.token_out_contract.to_owned());

                            buy_context = Some(vec![o]);
                        }
                        TradeScheme::Sell(s) => {
                            TradeValidation::sanity_check_trade_schemes_for_trade_types(
                                &t.trade,
                                &entry_points.trade_type,
                            )?;

                            let cli_parsed: Option<&TradeCliParsed>;

                            // todo: by default we are selecting the first item in the [buy_context_bucket] to check whether a [TradeCliParsed] entity already exists or not which needs to be parsed into the Buy function
                            cli_parsed = match &buy_context {
                                None => None,
                                Some(v) => {
                                    let buy_ctx_first = v.first();

                                    match buy_ctx_first {
                                        None => None,
                                        Some(d) => Some(&d.feed),
                                    }
                                }
                            };

                            let o = Self::set_sell_queue(
                                &sell_context,
                                s,
                                variables,
                                middleware,
                                cli_parsed,
                            )
                            .await?;

                            sell_token_in_contract = Some(o.feed.token_in_contract.to_owned());

                            sell_context = Some(vec![o]);
                        }
                    };
                }
            }
            TradeType::Buy => {
                for t in trades {
                    if let TradeScheme::Buy(s) = t.clone().trade {
                        TradeValidation::sanity_check_trade_schemes_for_trade_types(
                            &t.trade,
                            &entry_points.trade_type,
                        )?;

                        let o = Self::set_buy_queue(&buy_context, s, variables, middleware).await?;

                        buy_context = Some(vec![o]);
                    };
                }
            }
            TradeType::Sell => {
                for t in trades {
                    if let TradeScheme::Sell(s) = t.clone().trade {
                        TradeValidation::sanity_check_trade_schemes_for_trade_types(
                            &t.trade,
                            &entry_points.trade_type,
                        )?;

                        let o = Self::set_sell_queue(&sell_context, s, variables, middleware, None)
                            .await?;

                        sell_context = Some(vec![o]);
                    };
                }
            }
        }

        match entry_points.trade_type {
            TradeType::BuySell => {
                let mut buy_approve_gas_price: Option<String> = None;
                let mut buy_approve_gas_limit: Option<u64> = None;
                let mut buy_gas_price: Option<String> = None;
                let mut buy_gas_limit: Option<u64> = None;
                let mut buy_retry_attempts: Option<u64> = None;
                let mut buy_perc_increase_gas_price: Option<u32> = None;

                let mut sell_approve_gas_price: Option<String> = None;
                let mut sell_approve_gas_limit: Option<u64> = None;
                let mut sell_gas_price: Option<String> = None;
                let mut sell_gas_limit: Option<u64> = None;
                let mut sell_retry_attempts: Option<u64> = None;
                let mut sell_perc_increase_gas_price: Option<u32> = None;

                match &buy_context {
                    None => {
                        return Err(SetupError::Settings(
                            "no trade entity found for 'buy' (E00006aa)",
                        )
                        .into());
                    }
                    Some(d) => {
                        // todo: we are only picking up the first item from here. we need to improve the validation here
                        if let Some(v) = d.first() {
                            buy_approve_gas_price = v.scheme.approve_gas_price.to_owned();
                            buy_gas_price = Some(v.scheme.gas_price.to_owned());
                            buy_approve_gas_limit = v.scheme.approve_gas_limit;
                            buy_gas_limit = Some(v.scheme.gas_limit);
                            buy_retry_attempts = v.scheme.retry_attempts;
                            buy_perc_increase_gas_price = v.scheme.perc_increase_gas_price;
                        }
                    }
                }

                match &sell_context {
                    None => {
                        return Err(SetupError::Settings(
                            "no trade entity found for 'sell' (E00006ab)",
                        )
                        .into());
                    }
                    Some(d) => {
                        // todo: we are only picking up the first item from here. we need to improve the validation here
                        if let Some(v) = d.first() {
                            sell_approve_gas_price = v.scheme.approve_gas_price.to_owned();
                            sell_gas_price = Some(v.scheme.gas_price.to_owned());
                            sell_approve_gas_limit = v.scheme.approve_gas_limit;
                            sell_gas_limit = Some(v.scheme.gas_limit);
                            sell_retry_attempts = v.scheme.retry_attempts;
                            sell_perc_increase_gas_price = v.scheme.perc_increase_gas_price;
                        }
                    }
                }

                TradeValidation::validate_buysell_fn_tokens(
                    buy_token_out_contract,
                    sell_token_in_contract,
                )?;

                if variables.check_gas_fees {
                    let v = CheckGasFeesBalanceVariables {
                        account_address_h160: variables.account_address_h160,
                        buy_approve_gas_price,
                        buy_approve_gas_limit,
                        buy_gas_price,
                        buy_gas_limit,
                        buy_retry_attempts,
                        buy_perc_increase_gas_price,
                        sell_approve_gas_price,
                        sell_gas_price,
                        sell_gas_limit,
                        sell_approve_gas_limit,
                        sell_retry_attempts,
                        sell_perc_increase_gas_price,
                        native_token_symbol: variables.native_token_symbol.to_owned(),
                    };

                    TradeValidation::check_gas_fees_balance(middleware, &v).await?;
                }
            }
            TradeType::Buy => {
                match &buy_context {
                    None => {
                        return Err(SetupError::Settings(
                            "no trade entity found for 'buy' (E00006ac)",
                        )
                        .into());
                    }
                    Some(d) => {
                        if variables.check_gas_fees {
                            // todo: we are only picking up the first item from here. we need to improve the validation here
                            if let Some(i) = d.first() {
                                let v = CheckGasFeesBalanceVariables {
                                    account_address_h160: variables.account_address_h160,
                                    buy_approve_gas_price: i.scheme.approve_gas_price.to_owned(),
                                    buy_approve_gas_limit: i.scheme.approve_gas_limit,
                                    buy_gas_price: Some(i.scheme.gas_price.to_owned()),
                                    buy_gas_limit: Some(i.scheme.gas_limit),
                                    buy_retry_attempts: i.scheme.retry_attempts,
                                    buy_perc_increase_gas_price: i.scheme.perc_increase_gas_price,

                                    sell_approve_gas_price: None,
                                    sell_gas_price: None,
                                    sell_gas_limit: None,
                                    sell_approve_gas_limit: None,
                                    sell_retry_attempts: None,
                                    sell_perc_increase_gas_price: None,
                                    native_token_symbol: variables.native_token_symbol.to_owned(),
                                };

                                TradeValidation::check_gas_fees_balance(middleware, &v).await?;
                            }
                        }
                    }
                }
            }
            TradeType::Sell => {
                match &sell_context {
                    None => {
                        return Err(SetupError::Settings(
                            "no trade entity found for 'sell' (E00006ad)",
                        )
                        .into());
                    }
                    Some(d) => {
                        if variables.check_gas_fees {
                            // todo: we are only picking up the first item from here. we need to improve the validation here
                            if let Some(i) = d.first() {
                                let v = CheckGasFeesBalanceVariables {
                                    account_address_h160: variables.account_address_h160,
                                    sell_approve_gas_price: i.scheme.approve_gas_price.to_owned(),
                                    sell_approve_gas_limit: i.scheme.approve_gas_limit,
                                    sell_gas_price: Some(i.scheme.gas_price.to_owned()),
                                    sell_gas_limit: Some(i.scheme.gas_limit),
                                    sell_retry_attempts: i.scheme.retry_attempts,
                                    sell_perc_increase_gas_price: i.scheme.perc_increase_gas_price,

                                    buy_approve_gas_price: None,
                                    buy_gas_price: None,
                                    buy_gas_limit: None,
                                    buy_approve_gas_limit: None,
                                    buy_retry_attempts: None,
                                    buy_perc_increase_gas_price: None,

                                    native_token_symbol: variables.native_token_symbol.to_owned(),
                                };

                                TradeValidation::check_gas_fees_balance(middleware, &v).await?;
                            }
                        }
                    }
                }
            }
        }

        Ok((buy_context, sell_context))
    }

    async fn feature(
        selected_config_network: &NetworkEntity,
        entry_points: &EntryPoints,
        q_variables: &QuantVariables,
        q_middleware: &QuantMiddleware,
    ) -> anyhow::Result<Option<QuantFeature>> {
        let mut feature: Option<QuantFeature> = None;

        if let Some(trades) = &selected_config_network.feature.trades {
            let (buy_context, sell_context) =
                Self::trading_contexts(entry_points, trades, q_variables, q_middleware).await?;

            let q_trading = QuantTrading {
                buy_context,
                sell_context,
            };

            feature = Some(QuantFeature::Trading(q_trading));
        }

        Ok(feature)
    }

    pub async fn new(
        settings: &'a SettingFiles,
        entry_points: &'a EntryPoints,
    ) -> anyhow::Result<Quant> {
        let selected_exchange_networks = match &settings.secrets.settings.exchanges.first() {
            None => return Err(SetupError::Settings("invalid exchange name (E00001)").into()),
            Some(d) => &d.exchange.networks,
        };

        let selected_config_networks = match &settings.config.settings.exchanges.first() {
            None => return Err(SetupError::Settings("invalid exchange name (E00002)").into()),
            Some(d) => &d.exchange.networks,
        };

        let mut selected_secrets_network_account_option: Option<&secrets::AccountEntity> = None;
        let mut selected_config_network_option: Option<&config::NetworkEntity> = None;

        'secrets_network_iteration: for sn in selected_exchange_networks {
            if sn.network.name == entry_points.network_name {
                selected_secrets_network_account_option = match sn.network.accounts.first() {
                    None => {
                        return Err(SetupError::Settings(
                            "unable to find the account matching the input (E00003)",
                        )
                        .into())
                    }
                    Some(d) => Some(&d.account),
                };

                'config_network_iteration: for cn in selected_config_networks {
                    if cn.network.name == entry_points.network_name {
                        selected_config_network_option = Some(&cn.network);

                        break 'config_network_iteration;
                    }
                }

                break 'secrets_network_iteration;
            }
        }

        let selected_secrets_network_account = match selected_secrets_network_account_option {
            None => {
                return Err(
                    SetupError::Settings("no secrets network account found (E00004)").into(),
                )
            }
            Some(d) => d,
        };

        let selected_config_network = match selected_config_network_option {
            None => {
                return Err(SetupError::Settings("no config network entity found (E00005)").into())
            }
            Some(d) => d,
        };

        let q_provider = Self::provider(selected_config_network).await?;

        let q_variables = Self::variables(
            selected_secrets_network_account,
            selected_config_network,
            entry_points,
        )?;

        let q_middleware =
            Self::middleware(&q_provider, q_variables.chain_id, &q_variables.mnemonic)?;

        let q_feature_option = Self::feature(
            selected_config_network,
            entry_points,
            &q_variables,
            &q_middleware,
        )
        .await?;

        let q_feature: QuantFeature;
        match q_feature_option {
            Some(e) => {
                q_feature = e;
            }
            None => return Err(SetupError::Settings("no feature entity found (E00008)").into()),
        }

        let q: Quant = Quant {
            variables: q_variables,
            middleware: q_middleware,
            provider: q_provider,
            feature: q_feature,
        };

        Ok(q)
    }
}
