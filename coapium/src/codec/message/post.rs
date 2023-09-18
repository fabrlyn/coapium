use crate::codec::{payload, token, Header, MessageId, Payload, Token, TokenLength};

use super::post_options::{self, PostOptions};
use super::{Method, Reliability};

#[derive(Clone, Debug, PartialEq)]
pub struct Post {
    message_id: MessageId,
    reliability: Reliability,
    token: Token,
    options: PostOptions,
    payload: Payload,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Token(token::Error),
    Options(post_options::Error),
    Payload(payload::Error),
}

impl Post {
    pub fn decode(
        message_id: MessageId,
        token_length: TokenLength,
        reliability: Reliability,
        remaining_bytes: &[u8],
    ) -> Result<Self, Error> {
        let (remaining_bytes, token) = Token::parse(token_length, remaining_bytes)?;

        let (remaining_bytes, options) = PostOptions::parse(remaining_bytes)?;

        let payload = Payload::decode(remaining_bytes)?;

        Ok(Self::new(message_id, reliability, token, options, payload))
    }

    pub fn encode(self) -> Vec<u8> {
        let (token_length, token) = self.token.encode();

        let header = Header::new(
            self.reliability.into(),
            token_length,
            Method::Post(Payload::empty()).encode().0,
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
        options: PostOptions,
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

    pub fn options(&self) -> &PostOptions {
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

impl From<payload::Error> for Error {
    fn from(value: payload::Error) -> Self {
        Self::Payload(value)
    }
}

impl From<post_options::Error> for Error {
    fn from(error: post_options::Error) -> Self {
        Self::Options(error)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::codec::{
        self,
        message::{Post, Reliability},
        option::{DecodedOption, Delta, Number},
        MessageId, Payload,
    };

    use super::{
        post_options::{self, PostOptions},
        Error, Token, TokenLength,
    };

    #[rstest]
    #[case(
        MessageId::from_value(1),
        TokenLength::from_value(1).unwrap(),
        Reliability::Confirmable,
        Token::from_value(vec![1]).unwrap().encode().1,
        Ok(
            Post{
                message_id: MessageId::from_value(1),
                reliability: Reliability::Confirmable,
                token: Token::from_value(vec![1]).unwrap(),
                options: PostOptions::new(),
                payload: Payload::empty(),
            }
        )
    )]
    #[case(
        MessageId::from_value(2),
        TokenLength::from_value(1).unwrap(),
        Reliability::Confirmable,
        vec![Token::from_value(vec![1]).unwrap().encode().1, vec![0xff, 97]].into_iter().flatten().collect::<Vec<_>>(),
        Ok(
            Post{
                message_id: MessageId::from_value(2),
                reliability: Reliability::Confirmable,
                token: Token::from_value(vec![1]).unwrap(),
                options: PostOptions::new(),
                payload: Payload::from_value(vec![97]),
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
            }.encode(Delta::from_value(0)),
        ].into_iter().flatten().collect::<Vec<_>>(),
        Err(
            Error::Options(
                post_options::Error::Options(
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
            { PostOptions::new().encode() }
        ].into_iter().flatten().collect::<Vec<_>>(),
        Ok(Post {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: { PostOptions::new() },
            payload: Payload::empty(),
        }),
    )]
    fn decode(
        #[case] message_id: MessageId,
        #[case] token_length: TokenLength,
        #[case] reliability: Reliability,
        #[case] remaining_bytes: Vec<u8>,
        #[case] expected: Result<Post, Error>,
    ) {
        assert_eq!(
            expected,
            Post::decode(message_id, token_length, reliability, &remaining_bytes)
        )
    }

    #[rstest]
    #[case(
        Post {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: {  PostOptions::new() },
            payload: Payload::empty(),
        },
        &[0b01_00_0001, 0b000_00010, 0, 4, 1, ]
    )]
    fn encode(#[case] post: Post, #[case] expected: &[u8]) {
        assert_eq!(expected, post.encode())
    }

    #[rstest]
    #[case(
        Post {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: {  PostOptions::new() },
            payload: Payload::empty(),
        },
       MessageId::from_value(4)
    )]
    fn message_id(#[case] post: Post, #[case] expected: MessageId) {
        assert_eq!(expected, post.message_id())
    }

    #[rstest]
    #[case(
        MessageId::from_value(4),
        Reliability::Confirmable,
        Token::from_value(vec![1]).unwrap(),
        { PostOptions::new() },
        Payload::empty(),
        Post {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: { PostOptions::new() },
            payload: Payload::empty(),
        },
    )]
    fn new(
        #[case] message_id: MessageId,
        #[case] reliability: Reliability,
        #[case] token: Token,
        #[case] options: PostOptions,
        #[case] payload: Payload,
        #[case] expected: Post,
    ) {
        assert_eq!(
            expected,
            Post::new(message_id, reliability, token, options, payload)
        )
    }

    #[rstest]
    #[case(
        Post {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: { PostOptions::new() },
            payload: Payload::empty(),
        },
        { PostOptions::new() }
    )]
    fn options(#[case] post: Post, #[case] expected: PostOptions) {
        assert_eq!(&expected, post.options())
    }

    #[rstest]
    #[case(
        Post {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: { PostOptions::new() },
            payload: Payload::empty(),
        },
             Reliability::Confirmable,

    )]
    fn reliability(#[case] post: Post, #[case] expected: Reliability) {
        assert_eq!(expected, post.reliability())
    }

    #[rstest]
    #[case(
        Post {
            message_id: MessageId::from_value(4),
            reliability: Reliability::Confirmable,
            token: Token::from_value(vec![1]).unwrap(),
            options: { PostOptions::new() },
            payload: Payload::empty(),
        },
             Token::from_value(vec![1]).unwrap(),

    )]
    fn token(#[case] post: Post, #[case] expected: Token) {
        assert_eq!(&expected, post.token())
    }
}
