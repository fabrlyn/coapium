use crate::codec::{
    message::{self, DeleteOptions},
    MessageId, Token,
};

use super::reliability::Reliability;

#[derive(Clone, Debug, PartialEq)]
pub struct Delete {
    pub options: DeleteOptions,
    pub reliability: Reliability,
}

impl Delete {
    pub fn encode(self, message_id: MessageId, token: Token) -> Vec<u8> {
        message::Delete::new(message_id, (&self.reliability).into(), token, self.options).encode()
    }
}
