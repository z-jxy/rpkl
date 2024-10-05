use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),

    PklSend,
    PklRecv,
    PklMalformedResponse { message: String },
    PklProcessStart,
    PklServerError { pkl_error: String },

    SerializeAst,
    ParseError(String),
    DeserializeError(String),
    MsgpackDecodeError(rmpv::decode::Error),

    Eof,
    Syntax,
    ExpectedBoolean,
    ExpectedInteger,
    ExpectedString,
    ExpectedNull,
    ExpectedArray,
    ExpectedArrayComma,
    ExpectedArrayEnd,
    ExpectedMap,
    ExpectedMapColon,
    ExpectedMapComma,
    ExpectedMapEnd,
    ExpectedEnum,
    TrailingCharacters,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg)
            | Error::DeserializeError(msg)
            | Error::ParseError(msg)
            | Error::PklServerError { pkl_error: msg }
            | Error::PklMalformedResponse { message: msg } => formatter.write_str(msg),

            Error::MsgpackDecodeError(e) => formatter.write_str(&e.to_string()),

            Error::Eof => formatter.write_str("unexpected end of input"),

            _ => formatter.write_str("unknown error"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Message(e.to_string())
    }
}

impl From<rmpv::decode::Error> for Error {
    fn from(e: rmpv::decode::Error) -> Self {
        Error::MsgpackDecodeError(e)
    }
}
