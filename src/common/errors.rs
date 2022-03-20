use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError<'a> {
    #[error("a cli error has occured: {0:?}")]
    Invalid(&'a str),
}

#[derive(Error, Debug)]
pub enum SetupError<'a> {
    #[error("a setting error has occured: {0:?}")]
    Settings(&'a str),
}

#[derive(Error, Debug)]
pub enum TradingError<'a> {
    #[error("a trading error occured while trying to create the address pair")]
    CreatingPair(),

    #[error("a trading error occured while trying to check the liquidity of the token: {0:?}")]
    LiquidityCheck(&'a str),

    #[error("a trading error occured while trying to approve the token: {0:?}")]
    ApprovingToken(&'a str),

    #[error("a trading error occured while trying to swap the token: {0:?}")]
    SwapToken(&'a str),
}

#[derive(Error, Debug)]
pub enum OrderBookError<'a> {
    #[error("a sell order book error occured: {0:?}")]
    Buy(&'a str),

    #[error("a buy order book error occured: {0:?}")]
    Sell(&'a str),
}

#[derive(Error, Debug)]
pub enum QuantError<'a> {
    #[error("a utils error has occured: {0:?}")]
    Utils(&'a str),
}

#[derive(Error, Debug)]
pub enum SpawnError {
    #[error("a spawn error has occured")]
    Closure(),
}
