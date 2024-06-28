use std::fmt::Display;

use crate::{Error, Result};

pub trait Context<T> {
    fn context<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static;
}

impl<T> Context<T> for Error {
    fn context<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        Err(Error::Message(format!("{}: {}", context, self)))
    }
}

impl<T> Context<T> for Option<T> {
    fn context<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        self.ok_or_else(|| Error::Message(format!("{}", context)))
    }
}
