use crate::codec::{payload, token, Header, MessageId, Payload, Token, TokenLength};

use super::put_options::{self, PutOptions};
use super::{Method, Reliability};

#[derive(Clone, Debug, PartialEq)]
pub struct Put {
    message_id: MessageId,
    reliability: Reliability,
    token: Token,
    options: PutOptions,
    payload: Payload,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Token(token::Error),
    Options(put_options::Error),
    Payload(payload::Error),
}

impl Put {
    pub fn decode(
        message_id: MessageId,
        token_length: TokenLength,
        reliability: Reliability,
        remaining_bytes: &[u8],
    ) -> Result<Self, Error> {
        let (remaining_bytes, token) = Token::parse(token_length, remaining_bytes)?;

        let (remaining_bytes, options) = PutOptions::parse(remaining_bytes)?;

        let payload = Payload::decode(remaining_bytes)?;

        Ok(Self::new(message_id, reliability, token, options, payload))
    }

    pub fn encode(self) -> Vec<u8> {
        let (token_length, token) = self.token.encode();

        let header = Header::new(
            self.reliability.into(),
            token_length,
            Method::Put(Payload::empty()).encode().0,
            self.message_id,
        );

        header
            .encode()
            .into_iter()
            .chain(token)
            .chain(self.options.encode())
            .chain(self.payload.encode())
            .collect()
    }

    pub fn message_id(&self) -> MessageId {
        self.message_id
    }

    pub fn new(
        message_id: MessageId,
        reliability: Reliability,
        token: Token,
        options: PutOptions,
        payload: Payload,
    ) -> Self {
        Self {
            message_id,
            reliability,
            token,
            options,
            payload,
        }
    }

    pub fn options(&self) -> &PutOptions {
        &self.options
    }

    pub fn reliability(&self) -> Reliability {
        self.reliability
    }

    pub fn token(&self) -> &Token {
        &self.token
    }
}

impl From<token::Error> for Error {
    fn from(value: token::Error) -> Self {
        Error::Token(value)
    }
}

impl From<put_options::Error> for Error {
    fn from(error: put_options::Error) -> Self {
        Self::Options(error)
    }
}

impl From<payload::Error> for Error {
    fn from(value: payload::Error) -> Self {
        Self::Payload(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::codec::{
        self,
        message::{Put, Reliability},
        option::{DecodedOption, Delta, Number},
        MessageId,
    };

    use super::{
        put_options::{self, PutOptions},
        Error, Payload, Token, TokenLength,
    };

    #[rstest]
    #[case(
        MessageId::from_value(1),
        TokenLength::from_value(1).unwrap(),
        Reliability::Confirmable,
        Token::from_value(vec![1]).unwrap().encode().1,
        Ok(
            Put{
                message_id: MessageId::from_value(1),
                reliability: Reliability::Confirmable,
                token: Token::from_value(vec![1]).unwrap(),
                options: PutOptions::new(),
                payload: Payload::empty(),
            }
        )
    )]
    #[case(
        MessageId::from_value(2),
        TokenLength::from_value(1).unwrap(),
        Reliability::Confirmable,
        vec![Token::from_value(vec![1]).unwrap().encode().1, vec![0xff, 97, 98, 99]].into_iter().flatten().collect::<Vec<_>>(),
        Ok(
            Put{
                message_id: MessageId::from_value(2),
                reliability: Reliability::Confirmable,
                token: Token::from_value(vec![1]).unwrap(),
                options: PutOptions::new(),
                payload: Payload::from_value(vec![97, 98, 99]),
            }
        )
    )]
    #[case(
        MessageId::from_value(3),
        TokenLength::from_value(1).unwrap(),
        Reliability::Confirmable,
        vec![
            Token::from_value(vec![1]).unwrap().encode().1,
            DecodedOption {
                number: Number::from_value_or_panic(2049),
                values: vec![]
            }
        .encode(Delta::from_value(0)),
        ].into_iter().flatten().collect::<Vec<_>>(),
        Err(
            Error::Options(
                put_options::Error::Options(
                    codec::options::Error::Option(
                        codec::option::Error::Unrecognized(
                            Number::from_value_or_panic(2049)
                        )
                    )
                )
            )
        )
    )]
    fn decode(
        #[case] message_id: MessageId,
        #[case] token_length: TokenLength,
        #[case] reliability: Reliability,
        #[case] remaining_bytes: Vec<u8>,
        #[case] expected: Result<Put, Error>,
    ) {
        assert_eq!(
            expected,
            Put::decode(message_id, token_length, reliability, &remaining_bytes)
        )
    }

    #[rstest]
    #[case(
        Put {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: { PutOptions::new() },
            payload: Payload::empty(),
        },
       MessageId::from_value(4)
    )]
    fn message_id(#[case] put: Put, #[case] expected: MessageId) {
        assert_eq!(expected, put.message_id())
    }

    #[rstest]
    #[case(
       MessageId::from_value(4),
       Reliability::Confirmable,
       Token::from_value(vec![1]).unwrap(),
       PutOptions::new(),
       Payload::empty(),
       Put {
         message_id: MessageId::from_value(4),
         reliability: Reliability::Confirmable,
         token: Token::from_value(vec![1]).unwrap(),
         options: PutOptions::new(),
         payload: Payload::empty(),
       },
    )]
    fn new(
        #[case] message_id: MessageId,
        #[case] reliability: Reliability,
        #[case] token: Token,
        #[case] options: PutOptions,
        #[case] payload: Payload,
        #[case] expected: Put,
    ) {
        assert_eq!(
            expected,
            Put::new(message_id, reliability, token, options, payload)
        )
    }

    #[rstest]
    #[case(
        Put {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: PutOptions::new(),
            payload: Payload::empty(),
        },
        { PutOptions::new() }
    )]
    fn options(#[case] put: Put, #[case] expected: PutOptions) {
        assert_eq!(&expected, put.options())
    }

    #[rstest]
    #[case(
        Put {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: PutOptions::new(),
            payload: Payload::empty(),
        },
             Reliability::Confirmable,

    )]
    fn reliability(#[case] put: Put, #[case] expected: Reliability) {
        assert_eq!(expected, put.reliability())
    }

    #[rstest]
    #[case(
        Put {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: PutOptions::new(),
            payload: Payload::empty(),
        },
             Token::from_value(vec![1]).unwrap(),

    )]
    fn token(#[case] put: Put, #[case] expected: Token) {
        assert_eq!(&expected, put.token())
    }
}
