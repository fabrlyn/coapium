use crate::codec::{
    message::{self, GetOptions},
    MessageId, Token,
};

use super::reliability::Reliability;

#[derive(Clone, Debug, PartialEq)]
pub struct Get {
    pub options: GetOptions,
    pub reliability: Reliability,
}

impl Get {
    pub fn encode(self, message_id: MessageId, token: Token) -> Vec<u8> {
        message::Get::new(message_id, (&self.reliability).into(), token, self.options).encode()
    }
}
