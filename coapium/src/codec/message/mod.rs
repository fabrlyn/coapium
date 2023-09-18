pub mod acknowledgement;
pub mod delete;
pub mod delete_options;
pub mod get;
pub mod get_options;
pub mod method;
pub mod piggyback;
pub mod post;
pub mod post_options;
pub mod put;
pub mod put_options;
pub mod reliability;
pub mod request;
pub mod reserved;
pub mod reset;
pub mod response;

pub use acknowledgement::Acknowledgement;
pub use delete::Delete;
pub use delete_options::DeleteOptions;
pub use get::Get;
pub use get_options::GetOptions;
pub use method::Method;
pub use piggyback::Piggyback;
pub use post::Post;
pub use post_options::PostOptions;
pub use put::Put;
pub use put_options::PutOptions;
pub use reliability::Reliability;
pub use request::Request;
pub use reserved::Reserved;
pub use reset::Reset;
pub use response::Response;

use crate::codec::{
    header,
    option::{self, encoded_option},
    options, payload, token, token_length, version, Code, Header, MessageType, MethodCode, Payload,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Acknowledgement(Acknowledgement),
    Piggyback(Piggyback),
    Request(Request),
    Reset(Reset),
    Response(Response),
    Reserved(Reserved),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FormatError {
    TokenLengthNonZero,
    ExcessiveData,
    InvalidTypeAndCode(MessageType, Code),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Version(version::Error),
    Format(FormatError),
    HeaderMissing,
    DataLength,
    EncodedOption(encoded_option::Error),
    Option(option::Error),
    Options(options::Error),
    Payload(payload::Error),
    Token(token::Error),
    TokenLength(token_length::Error),
    Header(header::Error),
}

impl Message {
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        let (bytes, header) = Header::parse(bytes)?;

        match header.message_type() {
            MessageType::Acknowledgement => Self::decode_acknowledgement(header, bytes),
            MessageType::Confirmable => Self::decode_confirmable(header, bytes),
            MessageType::NonConfirmable => Self::decode_non_confirmable(header, bytes),
            MessageType::Reset => Self::decode_reset(header, bytes),
        }
    }

    fn decode_acknowledgement(header: Header, bytes: &[u8]) -> Result<Self, Error> {
        match header.code() {
            Code::Empty => {
                Acknowledgement::decode(header.message_id(), header.token_length(), bytes)
                    .map(Self::Acknowledgement)
            }
            Code::Response(response_code) => {
                Piggyback::decode(header, response_code, bytes).map(Self::Piggyback)
            }
            code => Err(Error::Format(FormatError::InvalidTypeAndCode(
                MessageType::Acknowledgement,
                code,
            ))),
        }
    }

    fn decode_confirmable(header: Header, bytes: &[u8]) -> Result<Self, Error> {
        match header.code() {
            Code::Request(method_code) => {
                Request::decode(header, method_code, Reliability::Confirmable, bytes)
                    .map(Self::Request)
            }
            Code::Response(response_code) => Response::decode(
                Reliability::Confirmable,
                header.token_length(),
                response_code,
                header.message_id(),
                bytes,
            )
            .map(Self::Response),
            Code::Reserved(reserved) => Reserved::decode(
                Reliability::Confirmable,
                header.token_length(),
                reserved,
                header.message_id(),
                bytes,
            )
            .map(Self::Reserved),
            code => Err(Error::Format(FormatError::InvalidTypeAndCode(
                MessageType::Confirmable,
                code,
            ))),
        }
    }

    fn decode_non_confirmable(header: Header, bytes: &[u8]) -> Result<Self, Error> {
        match header.code() {
            Code::Request(method_code) => {
                Request::decode(header, method_code, Reliability::NonConfirmable, bytes)
                    .map(Self::Request)
            }
            Code::Response(response_code) => Response::decode(
                Reliability::NonConfirmable,
                header.token_length(),
                response_code,
                header.message_id(),
                bytes,
            )
            .map(Self::Response),
            Code::Reserved(reserved) => Reserved::decode(
                Reliability::NonConfirmable,
                header.token_length(),
                reserved,
                header.message_id(),
                bytes,
            )
            .map(Self::Reserved),
            code => Err(Error::Format(FormatError::InvalidTypeAndCode(
                MessageType::NonConfirmable,
                code,
            ))),
        }
    }

    fn decode_reset(header: Header, bytes: &[u8]) -> Result<Self, Error> {
        match header.code() {
            Code::Empty => {
                Reset::decode(header.message_id(), header.token_length(), bytes).map(Self::Reset)
            }
            code => Err(Error::Format(FormatError::InvalidTypeAndCode(
                MessageType::Reset,
                code,
            ))),
        }
    }
}

impl From<options::Error> for Error {
    fn from(value: options::Error) -> Self {
        Self::Options(value)
    }
}

impl From<encoded_option::Error> for Error {
    fn from(value: encoded_option::Error) -> Self {
        Self::EncodedOption(value)
    }
}

impl From<header::Error> for Error {
    fn from(value: header::Error) -> Self {
        Self::Header(value)
    }
}

impl From<version::Error> for Error {
    fn from(value: version::Error) -> Self {
        Self::Version(value)
    }
}

impl From<FormatError> for Error {
    fn from(value: FormatError) -> Self {
        Self::Format(value)
    }
}

impl From<option::Error> for Error {
    fn from(value: option::Error) -> Self {
        Self::Option(value)
    }
}

impl From<payload::Error> for Error {
    fn from(value: payload::Error) -> Self {
        Self::Payload(value)
    }
}

impl From<token::Error> for Error {
    fn from(value: token::Error) -> Self {
        Self::Token(value)
    }
}

impl From<token_length::Error> for Error {
    fn from(value: token_length::Error) -> Self {
        Self::TokenLength(value)
    }
}
