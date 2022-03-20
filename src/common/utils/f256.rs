use ethers::core::abi::ethereum_types::U256;
use qd::Quad;
use std::ops::{Div, Mul};
use std::str::FromStr;

pub trait ToF256Units {
    fn to_f256(&self) -> qd::Quad;
}

impl ToF256Units for U256 {
    fn to_f256(&self) -> qd::Quad {
        let self_str = &*self.to_string();

        qd::qd!(qd::Quad::from(self_str))
    }
}

impl ToF256Units for &str {
    fn to_f256(&self) -> qd::Quad {
        qd::qd!(*self)
    }
}

impl ToF256Units for String {
    fn to_f256(&self) -> qd::Quad {
        qd::qd!(self.as_str())
    }
}

impl ToF256Units for u32 {
    fn to_f256(&self) -> qd::Quad {
        let self_str = &*self.to_string();

        qd::qd!(qd::Quad::from(self_str))
    }
}

impl ToF256Units for u64 {
    fn to_f256(&self) -> qd::Quad {
        let self_str = &*self.to_string();

        qd::qd!(qd::Quad::from(self_str))
    }
}

impl ToF256Units for u8 {
    fn to_f256(&self) -> qd::Quad {
        let self_str = &*self.to_string();

        qd::qd!(qd::Quad::from(self_str))
    }
}

impl ToF256Units for f64 {
    fn to_f256(&self) -> qd::Quad {
        qd::qd!(qd::Quad::from(*self))
    }
}

impl ToF256Units for Quad {
    fn to_f256(&self) -> qd::Quad {
        *self
    }
}

pub fn to_f256<K>(value: K) -> qd::Quad
where
    K: ToF256Units,
    qd::Quad: std::convert::From<K>,
{
    qd::qd!(value)
}

pub fn divide_into_f256<K, Q>(value: &K, by: &Q) -> qd::Quad
where
    K: ToF256Units,
    Q: ToF256Units,
{
    let value_qq = ToF256Units::to_f256(value);
    let by_qq = ToF256Units::to_f256(by);

    value_qq.div(by_qq)
}

pub fn percentage_of_f256<K, Q>(total: &K, part: &Q) -> qd::Quad
where
    K: ToF256Units,
    Q: ToF256Units,
{
    let total_quad = ToF256Units::to_f256(total);
    let part_quad = ToF256Units::to_f256(part);
    let div_quad = ToF256Units::to_f256(&100_u8);

    (total_quad.mul(part_quad)).div(div_quad)
}
