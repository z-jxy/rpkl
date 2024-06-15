use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

// This is a bare-bones implementation.
#[derive(Debug)]
pub enum Error {
    Message(String),

    PklSend,
    PklRecv,
    PklMalformedResponse { message: String },
    PklProcessStart,
    PklServerError { pkl_error: String },

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
            Error::Message(msg) => formatter.write_str(msg),
            Error::Eof => formatter.write_str("unexpected end of input"),
            _ => formatter.write_str("unknown error"),
            /* and so forth */
        }
    }
}

impl std::error::Error for Error {}

// std::io::Error

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
