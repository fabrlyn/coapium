use crate::codec::{self, Options, Payload, ResponseCode};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    AcknowledgementTimeout,
    Codec(codec::Error),
    Reset,
    Timeout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Response {
    pub response_code: ResponseCode,
    pub options: Options, // ResponseOptions
    pub payload: Payload,
}

impl From<codec::Response> for Response {
    fn from(value: codec::Response) -> Self {
        Self {
            response_code: value.response_code(),
            options: value.options().clone(),
            payload: value.payload().clone(),
        }
    }
}
