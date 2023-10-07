use crate::codec::{message::Reliability, Code, Header, MessageId, Token};

use super::transmission_parameters::ConfirmableParameters;

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
