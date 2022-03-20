use crate::common::errors::QuantError;
use ethers::abi::ethereum_types::U256;
use std::ops::{Div, Mul};

pub trait ToU256Units {
    fn stringify(&self) -> String;
}

impl ToU256Units for qd::Quad {
    fn stringify(&self) -> String {
        self.to_string()
    }
}

impl ToU256Units for qd::Double {
    fn stringify(&self) -> String {
        self.to_string()
    }
}

impl ToU256Units for U256 {
    fn stringify(&self) -> String {
        format!("{:?}", self)
    }
}

impl ToU256Units for u32 {
    fn stringify(&self) -> String {
        self.to_string()
    }
}

impl ToU256Units for u64 {
    fn stringify(&self) -> String {
        self.to_string()
    }
}

impl ToU256Units for f64 {
    fn stringify(&self) -> String {
        self.to_string()
    }
}

impl ToU256Units for f32 {
    fn stringify(&self) -> String {
        self.to_string()
    }
}

impl ToU256Units for String {
    fn stringify(&self) -> String {
        self.to_string()
    }
}

pub fn decimals_to_u256<K>(value: &K) -> anyhow::Result<(U256, usize)>
where
    K: ToU256Units,
{
    let value_str = ToU256Units::stringify(value);
    let value_str_decimal_split: Vec<&str> = value_str.split('.').collect();
    let value_str_decimal_split_len = value_str_decimal_split.len();

    let mut decimal_units = 0;
    let value_str_u256: U256;

    if value_str_decimal_split_len > 2 {
        return Err(QuantError::Utils(
            r#"invalid floating point; "value" should only have one decimal point"#,
        )
        .into());
    }

    if value_str_decimal_split_len > 1 {
        let value_str_no_decimals = value_str.replace('.', "");

        decimal_units = value_str_decimal_split[1].len();

        value_str_u256 = U256::from_dec_str(&*value_str_no_decimals)?;
    } else {
        value_str_u256 = U256::from_dec_str(&*value_str)?;
    }

    Ok((value_str_u256, decimal_units))
}

pub fn to_u256<K>(value: &K) -> anyhow::Result<U256>
where
    K: ToU256Units,
{
    let (r, _) = decimals_to_u256(value)?;

    Ok(r)
}

pub fn percentage_of_u256(total: U256, part: u64) -> U256 {
    (total.mul(part)).div(100)
}
