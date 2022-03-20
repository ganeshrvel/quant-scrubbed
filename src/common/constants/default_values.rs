use ethers::abi::ethereum_types::U256;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct DefaultValues<'a> {
    /// https://stackoverflow.com/questions/40484154/parameter-a-is-never-used-error-when-a-is-used-in-type-parameter-bound
    // Causes the type to function *as though* it has a `&'a ()` field,
    // despite not *actually* having one.
    _marker: PhantomData<&'a ()>,
}

impl DefaultValues<'static> {
    pub const PROVIDER_TIMEOUT: u64 = 10000;

    pub const TOKEN_ALLOWANCE_MIN_AMOUNT: &'static str =
        "11579208923731619542357098500868790700000000000000000000000000000000000000000";

    pub const TOKEN_ALLOWANCE_MAX_AMOUNT: U256 = U256::MAX;

    pub fn token_allowance_min_amount() -> anyhow::Result<U256> {
        Ok(U256::from_dec_str(
            DefaultValues::TOKEN_ALLOWANCE_MIN_AMOUNT,
        )?)
    }

    pub const BUY_INTERRUPTER_KEYWORD: &'static str = "by";

    pub const SELL_INTERRUPTER_KEYWORD: &'static str = "sl";
}
