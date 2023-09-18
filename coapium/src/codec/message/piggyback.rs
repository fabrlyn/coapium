use crate::codec::{
    Header, MessageId, MessageType, Options, Payload, Response, ResponseCode, Token,
};

use super::{Error, Reliability};

#[derive(Clone, Debug, PartialEq)]
pub struct Piggyback {
    response_code: ResponseCode,
    message_id: MessageId,
    token: Token,
    options: Options,
    payload: Payload,
}

impl Piggyback {
    pub fn decode(header: Header, response_code: ResponseCode, rest: &[u8]) -> Result<Self, Error> {
        let (rest, token) = Token::parse(header.token_length(), rest)?;
        let (rest, options) = Options::parse(rest)?;
        let payload = Payload::decode(rest)?;

        Ok(Self {
            message_id: header.message_id(),
            response_code,
            token,
            options,
            payload,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let (token_length, encoded_token) = self.token.encode();

        Header::new(
            MessageType::Acknowledgement,
            token_length,
            self.response_code.into(),
            self.message_id,
        )
        .encode()
        .into_iter()
        .chain(encoded_token)
        .chain(self.options.encode())
        .chain(self.payload.encode())
        .collect()
    }

    pub fn new(
        token: Token,
        response_code: ResponseCode,
        message_id: MessageId,
        options: Options,
        payload: Payload,
    ) -> Self {
        Self {
            token,
            response_code,
            message_id,
            options,
            payload,
        }
    }
}

impl From<Piggyback> for Response {
    fn from(value: Piggyback) -> Self {
        Self::new(
            Reliability::NonConfirmable,
            value.token,
            value.response_code,
            value.message_id,
            value.options,
            value.payload,
        )
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::codec::code::response_code::Success;

    use super::{
        super::token_length::TokenLength, super::Code, super::MessageType, Error, Header,
        MessageId, Options, Payload, Piggyback, ResponseCode, Token,
    };

    #[rstest]
    #[case(
        &[2, 0b1101_0100, 1, 0, 0, 0, 30, 0xff, 97, 98, 99],
        Ok(Piggyback {
        message_id: MessageId::from_value(4),
        token: Token::from_value(vec![2]).unwrap(),
        response_code: ResponseCode::Success(Success::Content),
        options: {
            let mut options = Options::new();
            
            options.set_max_age(30.into());
            
            options
        },
        payload: Payload::from_value(vec![97, 98, 99])
    }))]
    fn decode(#[case] bytes: &[u8], #[case] expected: Result<Piggyback, Error>) {
        let response_code = ResponseCode::Success(Success::Content);
        let header = Header::new(
            MessageType::Acknowledgement,
            TokenLength::from_value(1).unwrap(),
            Code::Response(response_code),
            MessageId::from_value(4),
        );
        assert_eq!(expected, Piggyback::decode(header, response_code, bytes))
    }
}
