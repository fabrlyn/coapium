use version::Version;

use crate::codec::{parsing::take, version, Code, MessageId, MessageType, TokenLength};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
    message_type: MessageType,
    token_length: TokenLength,
    code: Code,
    message_id: MessageId,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    DataLength,
    Version(version::Error),
}

impl Header {
    pub fn code(&self) -> Code {
        self.code
    }

    pub fn encode(self) -> Vec<u8> {
        [Version::V1.encode() | self.message_type.encode() | self.token_length.encode()]
            .into_iter()
            .chain([self.code.encode()])
            .chain(self.message_id.encode())
            .collect()
    }

    pub fn message_id(&self) -> MessageId {
        self.message_id
    }

    pub fn message_type(&self) -> MessageType {
        self.message_type
    }

    pub fn new(
        message_type: MessageType,
        token_length: TokenLength,
        code: Code,
        message_id: MessageId,
    ) -> Self {
        Self {
            message_type,
            token_length,
            code,
            message_id,
        }
    }

    pub fn parse(bytes: &[u8]) -> Result<(&[u8], Self), Error> {
        let Ok((_, header_bytes)) = take::<4>(bytes) else {
            return Err(Error::DataLength);
        };
        let rest = &bytes[4..];

        Version::decode(header_bytes[0])?;

        let message_type = MessageType::decode(header_bytes[0]);
        let token_length = TokenLength::decode(header_bytes[0]);
        let code = Code::decode(header_bytes[1]);
        let message_id = MessageId::decode([header_bytes[2], header_bytes[3]]);

        Ok((
            rest,
            Header {
                message_id,
                message_type,
                token_length,
                code,
            },
        ))
    }

    pub fn token_length(&self) -> TokenLength {
        self.token_length
    }
}

impl From<version::Error> for Error {
    fn from(value: version::Error) -> Self {
        Self::Version(value)
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{
        super::code::response_code::{ClientError, Success},
        super::ResponseCode,
        version, Code, Error, Header, MessageId, MessageType, TokenLength,
    };

    #[rstest]
    #[case(
        Header {
            message_type: MessageType::Acknowledgement,
            token_length: TokenLength::from_value(1).unwrap(),
            code: Code::Response(ResponseCode::Success(Success::Created)),
            message_id: MessageId::from_value(2), 
        },
        Code::Response(ResponseCode::Success(Success::Created)),
    )]
    fn get_code(#[case] header: Header, #[case] expected: Code) {
        assert_eq!(expected, header.code())
    }

    #[rstest]
    #[case(
        Header {
            message_type: MessageType::Acknowledgement,
            token_length: TokenLength::from_value(1).unwrap(),
            code: Code::Response(ResponseCode::Success(Success::Created)),
            message_id: MessageId::from_value(2), 
        },
        vec![0b01_10_0001, 0b010_00001, 0, 2]
    )]
    fn encode(#[case] header: Header, #[case] expected: Vec<u8>) {
        assert_eq!(expected, header.encode())
    }

    #[rstest]
    #[case(
        Header {
            message_type: MessageType::Acknowledgement,
            token_length: TokenLength::from_value(1).unwrap(),
            code: Code::Response(ResponseCode::Success(Success::Created)),
            message_id: MessageId::from_value(2), 
        },
        MessageId::from_value(2), 
    )]
    fn get_message_id(#[case] header: Header, #[case] expected: MessageId) {
        assert_eq!(expected, header.message_id())
    }

    #[rstest]
    #[case(
        Header {
            message_type: MessageType::Acknowledgement,
            token_length: TokenLength::from_value(1).unwrap(),
            code: Code::Response(ResponseCode::Success(Success::Created)),
            message_id: MessageId::from_value(2), 
        },
       MessageType::Acknowledgement,
    )]
    fn get_message_type(#[case] header: Header, #[case] expected: MessageType) {
        assert_eq!(expected, header.message_type())
    }

    #[rstest]
    #[case(
        MessageType::NonConfirmable,
        TokenLength::from_value(2).unwrap(),
        Code::Response(ResponseCode::ClientError(ClientError::BadOption)),
        MessageId::from_value(4), 
        Header {
            message_type: MessageType::NonConfirmable,
            token_length: TokenLength::from_value(2).unwrap(),
            code: Code::Response(ResponseCode::ClientError(ClientError::BadOption)),
            message_id: MessageId::from_value(4), 
        },
    )]
    fn new(
        #[case] message_type: MessageType,
        #[case] token_length: TokenLength,
        #[case] code: Code,
        #[case] message_id: MessageId,
        #[case] expected: Header,
    ) {
        let header = Header::new(message_type, token_length, code, message_id);
        assert_eq!(expected, header)
    }

    #[rstest]
    #[case(&[], &[], Err(Error::DataLength))]
    #[case(
        &[0b10_10_0001, 0b010_00001, 0, 2, 3, 4],
        &[],
        Err(Error::Version(version::Error::Unsupported(2)))
    )]
    #[case(
        &[0b01_10_0001, 0b010_00001, 0, 2, 3, 4],
        &[3, 4],
        Ok(Header {
            message_type: MessageType::Acknowledgement,
            token_length: TokenLength::from_value(1).unwrap(),
            code: Code::Response(ResponseCode::Success(Success::Created)),
            message_id: MessageId::from_value(2), 
        }),
    )]
    fn parse(
        #[case] bytes: &[u8],
        #[case] expected_rest: &[u8],
        #[case] expected: Result<Header, Error>,
    ) {
        assert_eq!(expected.map(|v| (expected_rest, v)), Header::parse(bytes))
    }

    #[rstest]
    #[case(
        Header {
            message_type: MessageType::Acknowledgement,
            token_length: TokenLength::from_value(1).unwrap(),
            code: Code::Response(ResponseCode::Success(Success::Created)),
            message_id: MessageId::from_value(2), 
        },
        TokenLength::from_value(1).unwrap(),
    )]
    fn get_token_length(#[case] header: Header, #[case] expected: TokenLength) {
        assert_eq!(expected, header.token_length())
    }
}
