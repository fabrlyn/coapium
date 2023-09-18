use crate::codec::{token, Header, MessageId, Token, TokenLength};

use super::get_options::{self, GetOptions};
use super::{Method, Reliability};

#[derive(Clone, Debug, PartialEq)]
pub struct Get {
    message_id: MessageId,
    reliability: Reliability,
    token: Token,
    options: GetOptions,
}

// Option notes regarding GET-requests
// -----------------------------------------
// If the request includes an Accept Option, that indicates the
// preferred content-format of a response.  If the request includes an
// ETag Option, the GET method requests that ETag be validated and that
// the representation be transferred only if validation failed.  Upon
// success, a 2.05 (Content) or 2.03 (Valid) Response Code SHOULD be
// present in the response.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Token(token::Error),
    Options(get_options::Error),
    ResidualData,
}

impl Get {
    pub fn decode(
        message_id: MessageId,
        token_length: TokenLength,
        reliability: Reliability,
        remaining_bytes: &[u8],
    ) -> Result<Self, Error> {
        let (remaining_bytes, token) = Token::parse(token_length, remaining_bytes)?;

        let (remaining_bytes, options) = GetOptions::parse(remaining_bytes)?;

        if !remaining_bytes.is_empty() {
            return Err(Error::ResidualData);
        }

        Ok(Self::new(message_id, reliability, token, options))
    }

    pub fn encode(self) -> Vec<u8> {
        let (token_length, token) = self.token.encode();

        let header = Header::new(
            self.reliability.into(),
            token_length,
            Method::Get.encode().0,
            self.message_id,
        );

        let options = self.options.encode();

        header
            .encode()
            .into_iter()
            .chain(token)
            .chain(options)
            .collect()
    }

    pub fn message_id(&self) -> MessageId {
        self.message_id
    }

    pub fn new(
        message_id: MessageId,
        reliability: Reliability,
        token: Token,
        options: GetOptions,
    ) -> Self {
        Self {
            message_id,
            reliability,
            token,
            options,
        }
    }

    pub fn options(&self) -> &GetOptions {
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

impl From<get_options::Error> for Error {
    fn from(error: get_options::Error) -> Self {
        Self::Options(error)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::codec::{
        self,
        message::{Get, Reliability},
        option::{DecodedOption, Delta, Number},
        MessageId,
    };

    use super::{
        get_options::{self, GetOptions},
        Error, Token, TokenLength,
    };

    #[rstest]
    #[case(
        MessageId::from_value(1), 
        TokenLength::from_value(1).unwrap(), 
        Reliability::Confirmable, 
        Token::from_value(vec![1]).unwrap().encode().1, 
        Ok(
            Get{
                message_id: MessageId::from_value(1), 
                reliability: Reliability::Confirmable, 
                token: Token::from_value(vec![1]).unwrap(),
                options: GetOptions::new(),
            }
        )
    )]
    #[case(
        MessageId::from_value(2), 
        TokenLength::from_value(1).unwrap(), 
        Reliability::Confirmable, 
        vec![Token::from_value(vec![1]).unwrap().encode().1, vec![0xff]].into_iter().flatten().collect::<Vec<_>>(), 
        Err(Error::ResidualData)
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
            }.encode(Delta::from_value(0)),
        ].into_iter().flatten().collect::<Vec<_>>(),
        Err(
            Error::Options(
                get_options::Error::Options(
                    codec::options::Error::Option(
                        codec::option::Error::Unrecognized(
                            Number::from_value_or_panic(2049)
                        )
                    )
                )
            )
        )
    )]
    #[case(
        MessageId::from_value(4),
        TokenLength::from_value(1).unwrap(),
        Reliability::Confirmable,
        vec![
            Token::from_value(vec![1]).unwrap().encode().1,
            {
               let mut options = GetOptions::new();
               options.set_uri_path("a/b/c".try_into().unwrap());
               options.encode()
            }
        ].into_iter().flatten().collect::<Vec<_>>(),
        Ok(Get {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: {
                let mut options = GetOptions::new();
                options.set_uri_path("a/b/c".try_into().unwrap());
                options
            }
        })
    )]
    fn decode(
        #[case] message_id: MessageId,
        #[case] token_length: TokenLength,
        #[case] reliability: Reliability,
        #[case] remaining_bytes: Vec<u8>,
        #[case] expected: Result<Get, Error>,
    ) {
        assert_eq!(
            expected,
            Get::decode(message_id, token_length, reliability, &remaining_bytes)
        )
    }

    #[rstest]
    #[case(
        Get {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: {
                let mut options = GetOptions::new();
                options.set_uri_path("a/b/c".try_into().unwrap());
                options
            }
        },
        &[0b01_00_0001, 0b000_00001, 0, 4, 1, 0b1011_0001, 97, 0b0000_0001, 98, 0b0000_0001, 99]
    )]
    fn encode(#[case] get: Get, #[case] expected: &[u8]) {
        assert_eq!(expected, get.encode())
    }

    #[rstest]
    #[case(
        Get {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: {
                let mut options = GetOptions::new();
                options.set_uri_path("a/b/c".try_into().unwrap());
                options
            }
        },
       MessageId::from_value(4) 
    )]
    fn message_id(#[case] get: Get, #[case] expected: MessageId) {
        assert_eq!(expected, get.message_id())
    }

    #[rstest]
    #[case(
             MessageId::from_value(4),
             Reliability::Confirmable,
             Token::from_value(vec![1]).unwrap(),
            {
                let mut options = GetOptions::new();
                options.set_uri_path("a/b/c".try_into().unwrap());
                options
            }

,
        Get {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: {
                let mut options = GetOptions::new();
                
                options.set_uri_path("a/b/c".try_into().unwrap());

                options
            }
        },
    )]
    fn new(
        #[case] message_id: MessageId,
        #[case] reliability: Reliability,
        #[case] token: Token,
        #[case] options: GetOptions,
        #[case] expected: Get,
    ) {
        assert_eq!(expected, Get::new(message_id, reliability, token, options))
    }

    #[rstest]
    #[case(
        Get {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: {
                let mut options = GetOptions::new();
                options.set_uri_path("a/b/c".try_into().unwrap());
                options
            }
        },
        {
                let mut options = GetOptions::new();
                
                options.set_uri_path("a/b/c".try_into().unwrap());

                options
            
        }
    )]
    fn options(#[case] get: Get, #[case] expected: GetOptions) {
        assert_eq!(&expected, get.options())
    }

    #[rstest]
    #[case(
        Get {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: {
                let mut options = GetOptions::new();
                
                options.set_uri_path("a/b/c".try_into().unwrap());

                options
            }
        },
             Reliability::Confirmable,

    )]
    fn reliability(#[case] get: Get, #[case] expected: Reliability) {
        assert_eq!(expected, get.reliability())
    }

    #[rstest]
    #[case(
        Get {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: {
                let mut options = GetOptions::new();
                
                options.set_uri_path("a/b/c".try_into().unwrap());

                options
            }
        },
             Token::from_value(vec![1]).unwrap(),

    )]
    fn token(#[case] get: Get, #[case] expected: Token) {
        assert_eq!(&expected, get.token())
    }
}
