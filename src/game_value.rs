use alloc::format;
use alloc::string::String;
use core::fmt::{Display, Formatter};
use core::ops::{AddAssign, Deref, DerefMut, Mul, SubAssign};
use crankstart::Game;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

/// A Game Value is a value of ~inf size that has a nice iso multiple display.
/// intended for use with any key game value that might be displayed or used to store state

const MAGNITUDES: [&'static str; 11] = ["", "k", "M", "G", "T", "P", "E", "Z", "Y", "R", "Q"];
const MAGNITUDES_FULL: [&'static str; 11] = [
    "", "kilo", "mega", "giga", "tera", "peta", "exa", "zetta", "yotta", "ronna", "quanto",
];

pub trait GameValue {
    fn to_string_hum(&self) -> String;
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Default, Serialize, Deserialize)]
pub struct GameUInt {
    value: BigUint,
}

impl Deref for GameUInt {
    type Target = BigUint;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for GameUInt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl AddAssign<usize> for GameUInt {
    fn add_assign(&mut self, rhs: usize) {
        self.value += rhs;
    }
}

impl AddAssign<GameUInt> for GameUInt {
    fn add_assign(&mut self, rhs: GameUInt) {
        self.value += rhs.value;
    }
}

impl SubAssign<GameUInt> for GameUInt {
    fn sub_assign(&mut self, rhs: GameUInt) {
        self.value -= rhs.value;
    }
}

impl Mul for GameUInt {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value * rhs.value,
        }
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
impl GameUInt {
    pub fn new(value: BigUint) -> Self {
        Self { value }
    }

    pub fn get(&self) -> BigUint {
        self.value.clone()
    }
    pub fn take(self) -> BigUint {
        self.value
    }

    pub fn one() -> Self {
        Self::from(1usize)
    }
}

impl GameValue for GameUInt {
    fn to_string_hum(&self) -> String {
        let mut value_s = format!("{}", self.value);
        if value_s.len() < 4 {
            return value_s;
        }
        let magnitude = value_s.len() / 3;
        let magnitude_s = MAGNITUDES[magnitude];
        // If it's "1,500"
        // Magnitude is 1 = 3
        // so decimal_places = 2 = 3 - (len - magnitude * 3) = 3 - (4 - 3) = 2
        // If its 150,000
        // magnitude is 1
        // so decimal_places = 0 = 3 - (len - magnitude * 3) = 3 - (6 - 3) = 0
        let decimal_places = 3 - (value_s.len() - (magnitude * 3));
        let trunc = value_s.split_at(3).0;
        let (pre, post) = trunc.split_at(decimal_places);
        if post == "" {
            return format!("{}{}", pre, magnitude_s);
        } else {
            return format!("{}.{}{}", pre, post, magnitude_s);
        }
    }
}
