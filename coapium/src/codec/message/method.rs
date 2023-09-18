use super::{Code, MethodCode, Payload};

#[derive(Clone, Debug, PartialEq)]
pub enum Method {
    Get,
    Post(Payload), // TODO: Try and see it this is doable without the payload
    Put(Payload),
    Delete,
}

impl Method {
    pub fn encode(self) -> (Code, Payload) {
        match self {
            Method::Get => (Code::Request(MethodCode::Get), Payload::empty()),
            Method::Post(payload) => (Code::Request(MethodCode::Post), payload),
            Method::Put(payload) => (Code::Request(MethodCode::Put), payload),
            Method::Delete => (Code::Request(MethodCode::Delete), Payload::empty()),
        }
    }
}
