use crate::codec::{
    code::reserved_code::ReservedCode, MessageId, Options, Payload, Token, TokenLength,
};

use super::{Error, Reliability};

#[derive(Clone, Debug, PartialEq)]
pub struct Reserved {
    reliability: Reliability,
    code: ReservedCode,
    message_id: MessageId,
    token: Token,
    options: Options,
    payload: Payload,
}

impl Reserved {
    pub fn decode(
        reliability: Reliability,
        token_length: TokenLength,
        reserved_code: ReservedCode,
        message_id: MessageId,
        remaining_bytes: &[u8],
    ) -> Result<Self, Error> {
        let (bytes, token) = Token::parse(token_length, remaining_bytes)?;

        let (bytes, options) = Options::parse(bytes)?;

        let payload = Payload::decode(bytes)?;

        Ok(Self {
            reliability,
            code: reserved_code,
            message_id,
            token,
            options,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::codec::Options;

    use super::{
        super::super::code::Class, super::super::code::Detail, Error, MessageId, Payload,
        Reliability, Reserved, ReservedCode, Token, TokenLength,
    };

    #[rstest]
    #[case(
        MessageId::from_value(5),
        TokenLength::from_value(1).unwrap(),
        ReservedCode::new(Class::Reserved { value: 1 }, Detail::from_value(5).unwrap()),
        Reliability::Confirmable,
        &[4, 0b1011_0001, 97, 0xff, 1, 2, 3],
        Ok(Reserved {
           message_id:  MessageId::from_value(5),
           reliability: Reliability::Confirmable,
           token: Token::from_value(vec![4]).unwrap(),
         code: ReservedCode::new(Class::Reserved { value: 1 }, Detail::from_value(5).unwrap()),
         options: {
            let mut options = Options::new();
            options.set_uri_path("a".try_into().unwrap());
            options
         },
         payload: Payload::from_value(vec![1, 2, 3])
        })
    )]
    fn decode(
        #[case] message_id: MessageId,
        #[case] token_length: TokenLength,
        #[case] reserved_code: ReservedCode,
        #[case] reliability: Reliability,
        #[case] remaining_bytes: &[u8],
        #[case] expected: Result<Reserved, Error>,
    ) {
        assert_eq!(
            expected,
            Reserved::decode(
                reliability,
                token_length,
                reserved_code,
                message_id,
                remaining_bytes
            )
        )
    }
}
