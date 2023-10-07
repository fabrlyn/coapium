use crate::codec::{MessageId, Token};

use super::{delete::Delete, get::Get, ping::Ping, post::Post, put::Put, reliability::Reliability};

// TODO: Fix this, weird naming, what do you mean "new request", are there old requests? :P
#[derive(Clone, Debug, PartialEq)]
pub enum NewRequest {
    Delete(Delete),
    Get(Get),
    Ping(Ping),
    Post(Post),
    Put(Put),
}

impl NewRequest {
    pub fn encode(self, message_id: MessageId, token: Token) -> Vec<u8> {
        match self {
            NewRequest::Ping(request) => request.encode(message_id, token),
            NewRequest::Get(request) => request.encode(message_id, token),
            NewRequest::Post(request) => request.encode(message_id, token),
            NewRequest::Put(request) => request.encode(message_id, token),
            NewRequest::Delete(request) => request.encode(message_id, token),
        }
    }

    pub fn reliability(&self) -> Reliability {
        match self {
            NewRequest::Ping(ping) => Reliability::Confirmable(ping.confirmable_parameters),
            NewRequest::Get(get) => get.reliability,
            NewRequest::Post(post) => post.reliability,
            NewRequest::Put(put) => put.reliability,
            NewRequest::Delete(delete) => delete.reliability,
        }
    }
}
