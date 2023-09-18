use crate::codec::{token, Header, MessageId, Token, TokenLength};

use super::delete_options::{self, DeleteOptions};
use super::{Method, Reliability};

#[derive(Clone, Debug, PartialEq)]
pub struct Delete {
    message_id: MessageId,
    reliability: Reliability,
    token: Token,
    options: DeleteOptions,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Token(token::Error),
    Options(delete_options::Error),
    ResidualData,
}

impl Delete {
    pub fn decode(
        message_id: MessageId,
        token_length: TokenLength,
        reliability: Reliability,
        remaining_bytes: &[u8],
    ) -> Result<Self, Error> {
        let (remaining_bytes, token) = Token::parse(token_length, remaining_bytes)?;

        let (remaining_bytes, options) = DeleteOptions::parse(remaining_bytes)?;

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
            Method::Delete.encode().0,
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
        options: DeleteOptions,
    ) -> Self {
        Self {
            message_id,
            reliability,
            token,
            options,
        }
    }

    pub fn options(&self) -> &DeleteOptions {
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

impl From<delete_options::Error> for Error {
    fn from(error: delete_options::Error) -> Self {
        Self::Options(error)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::codec::{
        self,
        message::{Delete, Reliability},
        option::{DecodedOption, Delta, Number},
        MessageId,
    };

    use super::{
        delete_options::{self, DeleteOptions},
        Error, Token, TokenLength,
    };

    #[rstest]
    #[case(
        MessageId::from_value(1), 
        TokenLength::from_value(1).unwrap(), 
        Reliability::Confirmable, 
        Token::from_value(vec![1]).unwrap().encode().1, 
        Ok(
            Delete{
                message_id: MessageId::from_value(1), 
                reliability: Reliability::Confirmable, 
                token: Token::from_value(vec![1]).unwrap(),
                options: DeleteOptions::new(),
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
            }
        .encode(Delta::from_value(0)),
        ].into_iter().flatten().collect::<Vec<_>>(),
        Err(
            Error::Options(
                delete_options::Error::Options(
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
            { DeleteOptions::new().encode() }
        ].into_iter().flatten().collect::<Vec<_>>(),
        Ok(Delete {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: {  DeleteOptions::new() }
        })
    )]
    fn decode(
        #[case] message_id: MessageId,
        #[case] token_length: TokenLength,
        #[case] reliability: Reliability,
        #[case] remaining_bytes: Vec<u8>,
        #[case] expected: Result<Delete, Error>,
    ) {
        assert_eq!(
            expected,
            Delete::decode(message_id, token_length, reliability, &remaining_bytes)
        )
    }

    #[rstest]
    #[case(
        Delete {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: { DeleteOptions::new() }
        },
        &[0b01_00_0001, 0b000_00100, 0, 4, 1]
    )]
    fn encode(#[case] delete: Delete, #[case] expected: &[u8]) {
        assert_eq!(expected, delete.encode())
    }

    #[rstest]
    #[case(
        Delete {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: { DeleteOptions::new() }
        },
       MessageId::from_value(4) 
    )]
    fn message_id(#[case] delete: Delete, #[case] expected: MessageId) {
        assert_eq!(expected, delete.message_id())
    }

    #[rstest]
    #[case(
             MessageId::from_value(4),
             Reliability::Confirmable,
             Token::from_value(vec![1]).unwrap(),
            { DeleteOptions::new() }
            ,
        Delete {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: { DeleteOptions::new() }
        },
    )]
    fn new(
        #[case] message_id: MessageId,
        #[case] reliability: Reliability,
        #[case] token: Token,
        #[case] options: DeleteOptions,
        #[case] expected: Delete,
    ) {
        assert_eq!(
            expected,
            Delete::new(message_id, reliability, token, options)
        )
    }

    #[rstest]
    #[case(
        Delete {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: { DeleteOptions::new() }
        },
        {  DeleteOptions::new() }
    )]
    fn options(#[case] delete: Delete, #[case] expected: DeleteOptions) {
        assert_eq!(&expected, delete.options())
    }

    #[rstest]
    #[case(
        Delete {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: { DeleteOptions::new() }
        },
             Reliability::Confirmable,

    )]
    fn reliability(#[case] delete: Delete, #[case] expected: Reliability) {
        assert_eq!(expected, delete.reliability())
    }

    #[rstest]
    #[case(
        Delete {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: { DeleteOptions::new() }
        },
             Token::from_value(vec![1]).unwrap(),

    )]
    fn token(#[case] delete: Delete, #[case] expected: Token) {
        assert_eq!(&expected, delete.token())
    }
}
