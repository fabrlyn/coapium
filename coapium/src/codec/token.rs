use rand::{rngs::StdRng, RngCore, SeedableRng};

use crate::codec::TokenLength;

use super::token_length;

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    length: TokenLength,
    value: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    LengthOutOfRange,
}

impl Token {
    pub fn decode(bytes: Vec<u8>) -> Result<Self, Error> {
        let token_length = u8::try_from(bytes.len())
            .map(TokenLength::from_value)
            .map_err(|_| Error::LengthOutOfRange)??;

        Ok(Self {
            length: token_length,
            value: bytes,
        })
    }

    pub fn empty() -> Self {
        Self::decode(vec![]).expect("create empty token")
    }

    pub fn encode(self) -> (TokenLength, Vec<u8>) {
        (self.length, self.value)
    }

    pub fn from_value(value: Vec<u8>) -> Result<Self, Error> {
        Self::decode(value)
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub const fn length(&self) -> u8 {
        self.length.value()
    }

    pub fn new() -> Result<Self, Error> {
        let mut rng = StdRng::from_entropy();
        let mut bytes = [0; TokenLength::MAX as usize];
        rng.fill_bytes(&mut bytes);

        Self::decode(bytes.to_vec())
    }

    pub fn parse<'a>(
        token_length: TokenLength,
        bytes: &'a [u8],
    ) -> Result<(&'a [u8], Self), Error> {
        let length = usize::from(token_length.value());

        if bytes.len() < length {
            return Err(Error::LengthOutOfRange);
        }

        let token = Self::decode(bytes[..length].to_vec())?;

        Ok((&bytes[length..], token))
    }

    pub fn value(&self) -> Vec<u8> {
        self.value.clone()
    }
}

impl From<token_length::Error> for Error {
    fn from(_: token_length::Error) -> Self {
        Self::LengthOutOfRange
    }
}
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{Error, Token, TokenLength};

    #[rstest]
    #[case(vec![], Token { length: TokenLength::from_value(0).unwrap(), value: vec![] })]
    #[case(vec![1], Token { length: TokenLength::from_value(1).unwrap(), value: vec![1] })]
    #[case(vec![1, 2], Token { length: TokenLength::from_value(2).unwrap(), value: vec![1, 2] })]
    #[case(vec![1, 2, 3], Token { length: TokenLength::from_value(3).unwrap(), value: vec![1, 2, 3] })]
    #[case(vec![1, 2, 3, 4], Token { length: TokenLength::from_value(4).unwrap(), value: vec![1, 2, 3, 4] })]
    #[case(vec![1, 2, 3, 4, 5], Token { length: TokenLength::from_value(5).unwrap(), value: vec![1, 2, 3, 4, 5] })]
    #[case(vec![1, 2, 3, 4, 5, 6], Token { length: TokenLength::from_value(6).unwrap(), value: vec![1, 2, 3, 4, 5, 6] })]
    #[case(vec![1, 2, 3, 4, 5, 6, 7], Token { length: TokenLength::from_value(7).unwrap(), value: vec![1, 2, 3, 4, 5, 6, 7] })]
    #[case(vec![1, 2, 3, 4, 5, 6, 7, 8], Token { length: TokenLength::from_value(8).unwrap(), value: vec![1, 2, 3, 4, 5, 6, 7, 8] })]
    fn decode_value_encode(#[case] bytes: Vec<u8>, #[case] expected: Token) {
        let decoded = Token::decode(bytes.clone()).unwrap();
        let value = decoded.value();
        let (encoded_length, encoded_bytes) = decoded.clone().encode();

        assert_eq!(expected, decoded);
        assert_eq!(bytes, value);
        assert_eq!(bytes, encoded_bytes);
        assert_eq!(
            TokenLength::from_value(u8::try_from(decoded.length()).unwrap()).unwrap(),
            encoded_length
        );
    }

    #[rstest]
    #[case(vec![1, 2, 3, 4, 5, 6, 7, 8, 9], Err(Error::LengthOutOfRange))]
    fn decode_error(#[case] bytes: Vec<u8>, #[case] expected: Result<Token, Error>) {
        assert_eq!(expected, Token::decode(bytes))
    }

    #[rstest]
    fn empty() {
        let empty = Token::empty();

        assert!(empty.is_empty());
        assert_eq!(Vec::<u8>::new(), empty.value());
    }

    #[rstest]
    #[case(vec![], 0)]
    #[case(vec![1, 2, 3, 4], 4)]
    #[case(vec![1, 2, 3, 4, 5, 6, 7, 8], 8)]
    fn length(#[case] value: Vec<u8>, #[case] expected: u8) {
        assert_eq!(expected, Token::from_value(value).unwrap().length())
    }

    #[rstest]
    #[case(vec![], true)]
    #[case(vec![1], false)]
    fn is_empty(#[case] value: Vec<u8>, #[case] expected: bool) {
        assert_eq!(expected, Token::from_value(value).unwrap().is_empty())
    }

    #[rstest]
    fn new() {
        let token = Token::new().unwrap();
        assert!(!token.is_empty());
        assert_eq!(8, token.length())
    }

    #[rstest]
    #[case(0, &[1,2,3,4,5,6,7,8], &[1,2,3,4,5,6,7,8], &[0; 0])]
    #[case(1, &[1,2,3,4,5,6,7,8], &[2,3,4,5,6,7,8], &[1])]
    #[case(2, &[1,2,3,4,5,6,7,8], &[3,4,5,6,7,8], &[1, 2])]
    #[case(3, &[1,2,3,4,5,6,7,8], &[4,5,6,7,8], &[1, 2, 3])]
    #[case(4, &[1,2,3,4,5,6,7,8], &[5,6,7,8], &[1, 2, 3, 4])]
    #[case(5, &[1,2,3,4,5,6,7,8], &[6,7,8], &[1, 2, 3, 4, 5])]
    #[case(6, &[1,2,3,4,5,6,7,8], &[7,8], &[1, 2, 3, 4, 5, 6])]
    #[case(7, &[1,2,3,4,5,6,7,8], &[8], &[1, 2, 3, 4, 5, 6, 7])]
    #[case(8, &[1,2,3,4,5,6,7,8], &[0; 0], &[1, 2, 3, 4, 5, 6, 7, 8])]
    #[case(8, &[1,2,3,4,5,6,7,8, 9, 10], &[9, 10], &[1, 2, 3, 4, 5, 6, 7, 8])]
    fn parse(
        #[case] token_length: u8,
        #[case] input: &[u8],
        #[case] expected_rest: &[u8],
        #[case] expected_token: &[u8],
    ) {
        let token_length = TokenLength::decode(token_length);
        let (actual_rest, actual_token) = Token::parse(token_length, input).unwrap();

        assert_eq!(expected_rest, actual_rest);
        assert_eq!(expected_token, actual_token.value);
    }

    #[rstest]
    #[case(4, &[1,2,3])]
    fn error(#[case] token_length: u8, #[case] input: &[u8]) {
        let token_length = TokenLength::decode(token_length);
        assert!(Token::parse(token_length, input).is_err());
    }
}
