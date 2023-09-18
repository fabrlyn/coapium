use crate::codec::{Header, MessageId, Options, Payload, ResponseCode, Token, TokenLength};

use super::{Error, Reliability};

#[derive(Clone, Debug, PartialEq)]
pub struct Response {
    reliability: Reliability,
    message_id: MessageId,
    token: Token,
    response_code: ResponseCode,
    options: Options,
    payload: Payload,
}

impl Response {
    pub fn decode(
        reliability: Reliability,
        token_length: TokenLength,
        response_code: ResponseCode,
        message_id: MessageId,
        rest: &[u8],
    ) -> Result<Self, Error> {
        let (rest, token) = Token::parse(token_length, rest)?;

        let (rest, options) = Options::parse(rest)?;

        let payload = Payload::decode(rest)?;

        Ok(Self {
            reliability,
            response_code,
            message_id,
            token,
            options,
            payload,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let (token_length, encoded_token) = self.token.encode();

        Header::new(
            self.reliability.into(),
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
        reliability: Reliability,
        token: Token,
        response_code: ResponseCode,
        message_id: MessageId,
        options: Options,
        payload: Payload,
    ) -> Self {
        Self {
            reliability,
            response_code,
            message_id,
            token,
            options,
            payload,
        }
    }

    pub fn message_id(&self) -> MessageId {
        self.message_id
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn response_code(&self) -> ResponseCode {
        self.response_code
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub fn reliability(&self) -> Reliability {
        self.reliability
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::super::super::code::response_code::{ResponseCode, Success};
    use super::{Error, MessageId, Options, Payload, Reliability, Response, Token, TokenLength};

    #[rstest]
    #[case(
        Reliability::Confirmable, 
        TokenLength::from_value(1).unwrap(),
        ResponseCode::Success(Success::Content), 
        MessageId::from_value(21), 
        &[9, 0b1101_0100, 1, 0, 0, 0, 30, 0xff, 1, 2, 3],
        Ok(Response {
            reliability: Reliability::Confirmable,
            message_id: MessageId::from_value(21),
            token: Token::from_value(vec![9]).unwrap(),
            response_code: ResponseCode::Success(Success::Content), 
            options: {
                let mut options = Options::new(); 
                options.set_max_age(30.try_into().unwrap()); 
                options
            }, 
            payload: Payload::from_value(vec![1, 2, 3])
        })
    )]

    fn decode(
        #[case] reliability: Reliability,
        #[case] token_length: TokenLength,
        #[case] response_code: ResponseCode,
        #[case] message_id: MessageId,
        #[case] bytes: &[u8],
        #[case] expected: Result<Response, Error>,
    ) {
        assert_eq!(
            expected,
            Response::decode(reliability, token_length, response_code, message_id, bytes)
        )
    }

    #[rstest]
    #[case(
        Response {
            reliability: Reliability::Confirmable,
            message_id: MessageId::from_value(21),
            token: Token::from_value(vec![9]).unwrap(),
            response_code: ResponseCode::Success(Success::Content), 
            options: {
                let mut options = Options::new(); 
                options.set_max_age(30.try_into().unwrap()); 
                options
            }, 
            payload: Payload::from_value(vec![1, 2, 3])
        },
        &[0b01_00_0001, 0b010_00101, 0, 21, 9, 0b1101_0001, 1, 30, 0xff, 1, 2, 3]
        
    )]
    fn encode(#[case] response: Response, #[case] expected: &[u8]) {
        assert_eq!(expected, response.encode())
    }

    #[rstest]
    #[case(
        Reliability::Confirmable,
        ResponseCode::Success(Success::Content), 
        MessageId::from_value(21),
        Token::from_value(vec![9]).unwrap(),
        {
           let mut options = Options::new(); 
           options.set_max_age(30.try_into().unwrap()); 
           options
        }, 
        Payload::from_value(vec![1, 2, 3]),
        Response {
            reliability: Reliability::Confirmable,
            message_id: MessageId::from_value(21),
            token: Token::from_value(vec![9]).unwrap(),
            response_code: ResponseCode::Success(Success::Content), 
            options: {
                let mut options = Options::new(); 
                options.set_max_age(30.try_into().unwrap()); 
                options
            }, 
            payload: Payload::from_value(vec![1, 2, 3])
        }
    )]
    fn new(
        #[case] reliability: Reliability,
        #[case] response_code: ResponseCode,
        #[case] message_id: MessageId,
        #[case] token: Token,
        #[case] options: Options,
        #[case] payload: Payload,
        #[case] expected: Response,
    ) {
        assert_eq!(
            expected,
            Response::new(
                reliability,
                token,
                response_code,
                message_id,
                options,
                payload
            )
        )
    }

    #[rstest]
    #[case(
        Response {
            reliability: Reliability::Confirmable,
            message_id: MessageId::from_value(21),
            token: Token::from_value(vec![9]).unwrap(),
            response_code: ResponseCode::Success(Success::Content), 
            options: {
                let mut options = Options::new(); 
                options.set_max_age(30.try_into().unwrap()); 
                options
            }, 
            payload: Payload::from_value(vec![1, 2, 3])
        },
        MessageId::from_value(21),
    )]
    fn message_id(#[case] response: Response, #[case] expected: MessageId) {
        assert_eq!(expected, response.message_id())
    }

    #[rstest]
    #[case(
        Response {
            reliability: Reliability::Confirmable,
            message_id: MessageId::from_value(21),
            token: Token::from_value(vec![9]).unwrap(),
            response_code: ResponseCode::Success(Success::Content), 
            options: {
                let mut options = Options::new(); 
                options.set_max_age(30.try_into().unwrap()); 
                options
            }, 
            payload: Payload::from_value(vec![1, 2, 3])
        },
        Token::from_value(vec![9]).unwrap()
    )]
    fn token(#[case] response: Response, #[case] expected: Token) {
        assert_eq!(&expected, response.token())
    }

    #[rstest]
    #[case(
        Response {
            reliability: Reliability::Confirmable,
            message_id: MessageId::from_value(21),
            token: Token::from_value(vec![9]).unwrap(),
            response_code: ResponseCode::Success(Success::Content), 
            options: {
                let mut options = Options::new(); 
                options.set_max_age(30.try_into().unwrap()); 
                options
            }, 
            payload: Payload::from_value(vec![1, 2, 3])
        },
        ResponseCode::Success(Success::Content), 
    )]
    fn response_code(#[case] response: Response, #[case] expected: ResponseCode) {
        assert_eq!(expected, response.response_code())
    }

    #[rstest]
    #[case(
        Response {
            reliability: Reliability::Confirmable,
            message_id: MessageId::from_value(21),
            token: Token::from_value(vec![9]).unwrap(),
            response_code: ResponseCode::Success(Success::Content), 
            options: {
                let mut options = Options::new(); 
                options.set_max_age(30.try_into().unwrap()); 
                options
            }, 
            payload: Payload::from_value(vec![1, 2, 3])
        },
        {
            let mut options = Options::new(); 
            options.set_max_age(30.try_into().unwrap()); 
            options
        }, 
    )]
    fn options(#[case] response: Response, #[case] expected: Options) {
        assert_eq!(&expected, response.options())
    }

    #[rstest]
    #[case(
        Response {
            reliability: Reliability::Confirmable,
            message_id: MessageId::from_value(21),
            token: Token::from_value(vec![9]).unwrap(),
            response_code: ResponseCode::Success(Success::Content), 
            options: {
                let mut options = Options::new(); 
                options.set_max_age(30.try_into().unwrap()); 
                options
            }, 
            payload: Payload::from_value(vec![1, 2, 3])
        },
        Reliability::Confirmable,
    )]
    fn reliability(#[case] response: Response, #[case] expected: Reliability) {
        assert_eq!(expected, response.reliability())
    }

    #[rstest]
    #[case(
        Response {
            reliability: Reliability::Confirmable,
            message_id: MessageId::from_value(21),
            token: Token::from_value(vec![9]).unwrap(),
            response_code: ResponseCode::Success(Success::Content), 
            options: {
                let mut options = Options::new(); 
                options.set_max_age(30.try_into().unwrap()); 
                options
            }, 
            payload: Payload::from_value(vec![1, 2, 3])
        },
        Payload::from_value(vec![1, 2, 3])
    )]
    fn payload(#[case] response: Response, #[case] expected: Payload) {
        assert_eq!(&expected, response.payload())
    }
}
