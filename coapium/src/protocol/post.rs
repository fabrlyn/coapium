use crate::codec::{
    message::{self, PostOptions},
    MessageId, Payload, Token,
};

use super::reliability::Reliability;

#[derive(Clone, Debug, PartialEq)]
pub struct Post {
    pub options: PostOptions,
    pub reliability: Reliability,
    pub payload: Payload,
}

impl Post {
    pub fn encode(self, message_id: MessageId, token: Token) -> Vec<u8> {
        message::Post::new(
            message_id,
            (&self.reliability).into(),
            token,
            self.options,
            self.payload,
        )
        .encode()
    }
}
