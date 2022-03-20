use std::marker::PhantomData;

#[non_exhaustive]
#[derive(Debug)]
pub struct Urls<'a> {
    /// https://stackoverflow.com/questions/40484154/parameter-a-is-never-used-error-when-a-is-used-in-type-parameter-bound
    // Causes the type to function *as though* it has a `&'a ()` field,
    // despite not *actually* having one.
    _marker: PhantomData<&'a ()>,
}

impl Urls<'static> {
    pub const BSC_TESTNET: &'static str = "https://testnet.bscscan.com/";
    pub const BSC_MAINNET: &'static str = "https://www.bscscan.com/";
}
