use std::result;

use crate::codec::{self, message::Reliability, Code, Header, MessageId, Token};

use super::{
    response::{self, Response},
    transmission_parameters::ConfirmableParameters,
};

pub type Result = result::Result<(), Error>;

#[derive(Clone, Debug, PartialEq)]
pub struct Ping {
    pub confirmable_parameters: ConfirmableParameters,
}

impl Ping {
    pub fn encode(self, message_id: MessageId, token: Token) -> Vec<u8> {
        let (token_length, token) = token.encode();

        Header::new(
            Reliability::Confirmable.into(),
            token_length,
            Code::Empty,
            message_id,
        )
        .encode()
        .into_iter()
        .chain(token)
        .collect()
    }
}

#[derive(Clone, Debug)]
pub enum Error {
    UnexpectedResponse(Response),
    AcknowledgementTimeout,
    Codec(codec::Error),
    Timeout,
}

pub fn into_result(result: result::Result<Response, response::Error>) -> result::Result<(), Error> {
    match result {
        Ok(response) => Err(Error::UnexpectedResponse(response)),
        Err(error) => match error {
            response::Error::AcknowledgementTimeout => Err(Error::AcknowledgementTimeout),
            response::Error::Codec(error) => Err(Error::Codec(error)),
            response::Error::Reset => Ok(()),
            response::Error::Timeout => Err(Error::Timeout),
        },
    }
}
