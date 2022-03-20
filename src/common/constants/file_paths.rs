use crate::common::helpers::qenv::Qenv;
use std::marker::PhantomData;

#[non_exhaustive]
#[derive(Debug)]
pub struct FilePaths<'a> {
    /// https://stackoverflow.com/questions/40484154/parameter-a-is-never-used-error-when-a-is-used-in-type-parameter-bound
    // Causes the type to function *as though* it has a `&'a ()` field,
    // despite not *actually* having one.
    _marker: PhantomData<&'a ()>,
}

impl FilePaths<'static> {
    pub const CONFIG: &'static str = "./config.yaml";
    pub const SECRETS: &'static str = "./secrets.yaml";
    pub const LOG: &'static str = if Qenv::IS_RELEASE {
        "./logs/logging-release.log"
    } else {
        "./logs/logging-debug.log"
    };
}
