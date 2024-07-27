use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
/// Implementation of Pkl [Data Size](https://pkl-lang.org/main/current/language-reference/index.html#data-sizes)
///
/// Decimal units: KB, MB, GB, TB, PB
///
/// Binary units: KiB, MiB, GiB, TiB, PiB
///
/// The `Bytes` variant is the smallest unit and does not have a binary or decimal counterpart.
/// Calling `is_binary` or `is_decimal` on `DataSizeUnit::Bytes` will return `true` in both cases.
pub enum DataSizeUnit {
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
    Terabytes,
    Petabytes,
    Kibibytes,
    Mebibytes,
    Gibibytes,
    Tebibytes,
    Pebibytes,
}

impl From<&str> for DataSizeUnit {
    fn from(unit: &str) -> Self {
        match unit {
            "b" => DataSizeUnit::Bytes,
            "kb" => DataSizeUnit::Kilobytes,
            "mb" => DataSizeUnit::Megabytes,
            "gb" => DataSizeUnit::Gigabytes,
            "tb" => DataSizeUnit::Terabytes,
            "pb" => DataSizeUnit::Petabytes,
            "kib" => DataSizeUnit::Kibibytes,
            "mib" => DataSizeUnit::Mebibytes,
            "gib" => DataSizeUnit::Gibibytes,
            "tib" => DataSizeUnit::Tebibytes,
            "pib" => DataSizeUnit::Pebibytes,
            _ => panic!("invalid data size unit: {}", unit),
        }
    }
}

impl Display for DataSizeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        match self {
            DataSizeUnit::Bytes => write!(f, "b"),
            DataSizeUnit::Kilobytes => write!(f, "kb"),
            DataSizeUnit::Megabytes => write!(f, "mb"),
            DataSizeUnit::Gigabytes => write!(f, "gb"),
            DataSizeUnit::Terabytes => write!(f, "tb"),
            DataSizeUnit::Petabytes => write!(f, "pb"),
            DataSizeUnit::Kibibytes => write!(f, "kib"),
            DataSizeUnit::Mebibytes => write!(f, "mib"),
            DataSizeUnit::Gibibytes => write!(f, "gib"),
            DataSizeUnit::Tebibytes => write!(f, "tib"),
            DataSizeUnit::Pebibytes => write!(f, "pib"),
        }
    }
}

impl DataSizeUnit {
    /// Returns true if the unit is a binary unit (e.g. KiB, MiB, GiB, TiB, PiB) or `DataSizeUnit::Bytes`.
    fn is_binary(&self) -> bool {
        match self {
            DataSizeUnit::Bytes
            | DataSizeUnit::Kibibytes
            | DataSizeUnit::Mebibytes
            | DataSizeUnit::Gibibytes
            | DataSizeUnit::Tebibytes
            | DataSizeUnit::Pebibytes => true,

            DataSizeUnit::Kilobytes
            | DataSizeUnit::Megabytes
            | DataSizeUnit::Gigabytes
            | DataSizeUnit::Terabytes
            | DataSizeUnit::Petabytes => false,
        }
    }

    /// Returns true if the unit is a decimal unit (e.g. KB, MB, GB, TB, PB) or `DataSizeUnit::Bytes`.
    fn is_decimal(&self) -> bool {
        match self {
            DataSizeUnit::Bytes
            | DataSizeUnit::Kilobytes
            | DataSizeUnit::Megabytes
            | DataSizeUnit::Gigabytes
            | DataSizeUnit::Terabytes
            | DataSizeUnit::Petabytes => true,

            DataSizeUnit::Kibibytes
            | DataSizeUnit::Mebibytes
            | DataSizeUnit::Gibibytes
            | DataSizeUnit::Tebibytes
            | DataSizeUnit::Pebibytes => false,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DataSizeUnit::Bytes => "b",
            DataSizeUnit::Kilobytes => "kb",
            DataSizeUnit::Megabytes => "mb",
            DataSizeUnit::Gigabytes => "gb",
            DataSizeUnit::Terabytes => "tb",
            DataSizeUnit::Petabytes => "pb",
            DataSizeUnit::Kibibytes => "kib",
            DataSizeUnit::Mebibytes => "mib",
            DataSizeUnit::Gibibytes => "gib",
            DataSizeUnit::Tebibytes => "tib",
            DataSizeUnit::Pebibytes => "pib",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DataSize {
    value: f64,
    unit: DataSizeUnit,
}

use serde::de::{self, Visitor};
use std::fmt::{self, Display};

impl<'de> Deserialize<'de> for DataSizeUnit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DataSizeUnitVisitor;

        impl<'de> Visitor<'de> for DataSizeUnitVisitor {
            type Value = DataSizeUnit;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid data size unit")
            }

            fn visit_str<E>(self, value: &str) -> Result<DataSizeUnit, E>
            where
                E: de::Error,
            {
                match value {
                    "b" => Ok(DataSizeUnit::Bytes),
                    "kb" => Ok(DataSizeUnit::Kilobytes),
                    "mb" => Ok(DataSizeUnit::Megabytes),
                    "gb" => Ok(DataSizeUnit::Gigabytes),
                    "tb" => Ok(DataSizeUnit::Terabytes),
                    "pb" => Ok(DataSizeUnit::Petabytes),
                    "kib" => Ok(DataSizeUnit::Kibibytes),
                    "mib" => Ok(DataSizeUnit::Mebibytes),
                    "gib" => Ok(DataSizeUnit::Gibibytes),
                    "tib" => Ok(DataSizeUnit::Tebibytes),
                    "pib" => Ok(DataSizeUnit::Pebibytes),
                    _ => Err(E::custom(format!("invalid data size unit: {}", value))),
                }
            }
        }

        deserializer.deserialize_str(DataSizeUnitVisitor)
    }
}

impl DataSize {
    pub fn new(value: f64, unit: DataSizeUnit) -> Self {
        DataSize { value, unit }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn unit(&self) -> DataSizeUnit {
        self.unit
    }

    pub fn is_binary(&self) -> bool {
        self.unit.is_binary()
    }

    pub fn is_decimal(&self) -> bool {
        self.unit.is_decimal()
    }
}

use serde::{
    de::{Deserializer, MapAccess},
    forward_to_deserialize_any,
};

use crate::pkl::de::KeyDeserializer;

pub struct DataSizeDeserializer<'a> {
    pub input: &'a DataSize,
}

impl<'a, 'de> Deserializer<'de> for DataSizeDeserializer<'a> {
    type Error = crate::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 u8 u16 u32 f32 char string str
        bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map enum struct identifier ignored_any

        i64 u64 f64
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor
            .visit_map(DataSizeMapAccess {
                input: self.input,
                state: 0,
            })
            .map_err(|_| crate::Error::Message("failed to deserialize datasize".to_string()))
    }
}

pub(crate) struct DataSizeMapAccess<'a> {
    pub(crate) input: &'a DataSize,
    pub(crate) state: u8,
}

impl<'a, 'de> MapAccess<'de> for DataSizeMapAccess<'a> {
    type Error = crate::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.state {
            0 => {
                self.state += 1;
                seed.deserialize(KeyDeserializer("value")).map(Some)
            }
            1 => {
                self.state += 1;
                seed.deserialize(KeyDeserializer("unit")).map(Some)
            }
            _ => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.state {
            1 => seed.deserialize(de::value::F64Deserializer::new(self.input.value())),
            2 => seed.deserialize(de::value::StrDeserializer::new(self.input.unit().as_str())),
            _ => Err(de::Error::custom("unexpected state")),
        }
    }
}
