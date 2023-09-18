pub mod timeout;

pub use timeout::Timeout;

use crate::{
    codec::Token,
    protocol::response::{self, Response},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    CreateTimeout(Timeout),
    TransactionResolved(Token, Result<Response, response::Error>),
    Transmit(Vec<u8>),
}

pub type Effects = Vec<Effect>;

impl<T> From<T> for Effect
where
    T: Into<Timeout>,
{
    fn from(value: T) -> Self {
        Self::CreateTimeout(value.into())
    }
}
