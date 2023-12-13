use alloc::format;
use alloc::string::String;
use core::fmt::{Display, Formatter};
use core::ops::Deref;
use num_bigint::BigUint;

/// A Game Value is a value of ~inf size that has a nice iso multiple display.
/// intended for use with any key game value that might be displayed or used to store state

const MAGNITUDES: [&'static str; 11] = ["", "k", "M", "G", "T", "P", "E", "Z", "Y", "R", "Q"];
const MAGNITUDES_FULL: [&'static str; 11] = [
    "", "kilo", "mega", "giga", "tera", "peta", "exa", "zetta", "yotta", "ronna", "quanto",
];

pub trait GameValue {
    fn to_string_hum(&self) -> String;
}

#[derive(Clone, PartialEq, PartialOrd, Eq)]
pub struct GameUInt {
    value: BigUint,
}

impl Deref for GameUInt {
    type Target = BigUint;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<T> From<T> for GameUInt
where
    T: Into<BigUint>,
{
    fn from(value: T) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl GameValue for GameUInt {
    fn to_string_hum(&self) -> String {
        let mut value_s = format!("{}", self.value);
        let magnitude = value_s.len() / 3;
        let magnitude_s = MAGNITUDES[magnitude];
        if value_s.len() > 3 {
            let _ = value_s.split_at(3);
        }
        format!("{}{}", value_s, magnitude_s)
    }
}
