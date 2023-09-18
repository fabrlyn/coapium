use crate::codec::{MessageId, Token};

use super::{delete::Delete, get::Get, post::Post, put::Put, reliability::Reliability};

// TODO: Fix this, weird naming, what do you mean "new request", are there old requests? :P
#[derive(Clone, Debug, PartialEq)]
pub enum NewRequest {
    Get(Get),
    Post(Post),
    Put(Put),
    Delete(Delete),
}

impl NewRequest {
    pub fn reliability(&self) -> Reliability {
        match self {
            NewRequest::Get(get) => get.reliability,
            NewRequest::Post(post) => post.reliability,
            NewRequest::Put(put) => put.reliability,
            NewRequest::Delete(delete) => delete.reliability,
        }
    }

    pub fn encode(self, message_id: MessageId, token: Token) -> Vec<u8> {
        match self {
            NewRequest::Get(request) => request.encode(message_id, token),
            NewRequest::Post(request) => request.encode(message_id, token),
            NewRequest::Put(request) => request.encode(message_id, token),
            NewRequest::Delete(request) => request.encode(message_id, token),
        }
    }
}
