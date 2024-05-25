use std::ops::{Add, Div, Mul, Sub};

const DEBUG_MODE: bool = true; // set to false for release mode

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Integer {
    Int8(i8),
    Int(i32),
    Int64(i64),
    Int128(i128),
}

pub fn parse_large_integer(num_str: &str) -> Result<i128, &'static str> {
    let mut result = 0i128;

    for (_idx, ch) in num_str.chars().enumerate() {
        let digit = (ch as i128) - ('0' as i128);
        if digit < 0 || digit > 9 {
            return Err("Invalid character in number string");
        }

        result = checked_mul(result, 10);
        result = checked_add(result, digit);
    }

    Ok(result)
}

fn checked_add(a: i128, b: i128) -> i128 {
    let (result, overflowed) = a.overflowing_add(b);
    if DEBUG_MODE && overflowed {
        panic!("Integer overflow detected in addition");
    }
    result
}

fn checked_mul(a: i128, b: i128) -> i128 {
    let (result, overflowed) = a.overflowing_mul(b);
    if DEBUG_MODE && overflowed {
        panic!("Integer overflow detected in multiplication");
    }
    result
}

impl Integer {
    fn promote(self) -> (Integer, i128) {
        match self {
            Integer::Int8(a) => (Integer::Int8(a), a as i128),
            Integer::Int(a) => (Integer::Int(a), a as i128),
            Integer::Int64(a) => (Integer::Int64(a), a as i128),
            Integer::Int128(a) => (Integer::Int128(a), a),
        }
    }

    fn demote(val: i128) -> Integer {
        if let Some(result) = i8::try_from(val).ok() {
            Integer::Int8(result)
        } else if let Some(result) = i32::try_from(val).ok() {
            Integer::Int(result)
        } else if let Some(result) = i64::try_from(val).ok() {
            Integer::Int64(result)
        } else {
            Integer::Int128(val)
        }
    }

    fn add(self, other: Integer) -> Integer {
        let (_s1, val1) = self.promote();
        let (_s2, val2) = other.promote();
        let result = Self::checked_add(val1, val2); // Use the checked_add method here
        Integer::demote(result)
    }

    fn checked_add(a: i128, b: i128) -> i128 {
        let (result, overflowed) = a.overflowing_add(b);
        if DEBUG_MODE && overflowed {
            panic!("Integer overflow detected");
        }
        result
    }
}

impl Add for Integer {
    type Output = Integer;

    fn add(self, other: Self) -> Self::Output {
        self.add(other)
    }
}

impl From<i32> for Integer {
    fn from(item: i32) -> Self {
        Integer::Int(item)
    }
}

impl Sub for Integer {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::Int8(l), Integer::Int8(r)) => Integer::Int8(l - r),
            (Integer::Int(l), Integer::Int(r)) => Integer::Int(l - r),
            (Integer::Int64(l), Integer::Int64(r)) => Integer::Int64(l - r),
            (Integer::Int128(l), Integer::Int128(r)) => Integer::Int128(l - r),
            _ => panic!("Subtraction mismatched types"),
        }
    }
}

impl Mul for Integer {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::Int8(l), Integer::Int8(r)) => Integer::Int8(l * r),
            (Integer::Int(l), Integer::Int(r)) => Integer::Int(l * r),
            (Integer::Int64(l), Integer::Int64(r)) => Integer::Int64(l * r),
            (Integer::Int128(l), Integer::Int128(r)) => Integer::Int128(l * r),
            _ => panic!("Multiplication mismatched types"),
        }
    }
}

impl Div for Integer {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::Int8(l), Integer::Int8(r)) => Integer::Int8(l / r),
            (Integer::Int(l), Integer::Int(r)) => Integer::Int(l / r),
            (Integer::Int64(l), Integer::Int64(r)) => Integer::Int64(l / r),
            (Integer::Int128(l), Integer::Int128(r)) => Integer::Int128(l / r),
            _ => panic!("Division mismatched types"),
        }
    }
}

impl std::fmt::Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Integer::Int8(n) => write!(f, "{}", n),
            Integer::Int(n) => write!(f, "{}", n),
            Integer::Int64(n) => write!(f, "{}", n),
            Integer::Int128(n) => write!(f, "{}", n),
        }
    }
}

pub fn try_parse_number<T: std::str::FromStr>(s: &str) -> Option<T> {
    s.parse::<T>().ok()
}
