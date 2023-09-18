use crate::codec::{
    message::{self, PutOptions},
    MessageId, Payload, Token,
};

use super::reliability::Reliability;

#[derive(Clone, Debug, PartialEq)]
pub struct Put {
    pub options: PutOptions,
    pub reliability: Reliability,
    pub payload: Payload,
}

impl Put {
    pub fn encode(self, message_id: MessageId, token: Token) -> Vec<u8> {
        message::Put::new(
            message_id,
            (&self.reliability).into(),
            token,
            self.options,
            self.payload,
        )
        .encode()
    }
}
