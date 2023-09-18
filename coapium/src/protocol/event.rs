use crate::codec::token::Token;

use super::{effect::Timeout, new_request::NewRequest};

#[derive(Debug)]
pub enum Event {
    TransactionRequested(NewRequest, Token),
    TransactionCanceled(Token),
    TimeoutReached(Timeout),
    DataReceived(Vec<u8>),
}

pub type Events = Vec<Event>;

impl<T> From<T> for Event
where
    T: Into<Timeout>,
{
    fn from(value: T) -> Self {
        Self::TimeoutReached(value.into())
    }
}
