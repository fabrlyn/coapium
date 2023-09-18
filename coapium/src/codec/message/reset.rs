use crate::codec::{Code, Header, MessageId, MessageType, Token, TokenLength};

use super::{Error, FormatError};

#[derive(Clone, Debug, PartialEq)]
pub struct Reset {
    message_id: MessageId,
}

impl Reset {
    pub fn decode(
        message_id: MessageId,
        token_length: TokenLength,
        remaining_bytes: &[u8],
    ) -> Result<Self, Error> {
        if !token_length.is_zero_length() {
            return Err(Error::Format(FormatError::TokenLengthNonZero));
        }

        if !remaining_bytes.is_empty() {
            return Err(Error::Format(FormatError::ExcessiveData));
        }

        Ok(Self { message_id })
    }

    // TODO: test this
    pub fn encode(self) -> Vec<u8> {
        let (token_length, _) = Token::empty().encode();
        Header::new(
            MessageType::Reset,
            token_length,
            Code::Empty,
            self.message_id,
        )
        .encode()
    }

    // TODO: test this
    pub fn from_message_id(message_id: MessageId) -> Self {
        Self { message_id }
    }

    pub fn message_id(&self) -> MessageId {
        self.message_id
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{Error, FormatError, MessageId, Reset, TokenLength};

    #[rstest]
    #[case(MessageId::from_value(10), TokenLength::from_value(0).unwrap(), &[], Ok(Reset {message_id: MessageId::from_value(10)}))]
    #[case(MessageId::from_value(10), TokenLength::from_value(1).unwrap(), &[3], Err(Error::Format(FormatError::TokenLengthNonZero)))]
    #[case(MessageId::from_value(10), TokenLength::from_value(0).unwrap(), &[0xff, 1, 2, 3 ], Err(Error::Format(FormatError::ExcessiveData)))]
    fn decode(
        #[case] message_id: MessageId,
        #[case] token_length: TokenLength,
        #[case] remaining_bytes: &[u8],
        #[case] expected: Result<Reset, Error>,
    ) {
        assert_eq!(
            expected,
            Reset::decode(message_id, token_length, remaining_bytes)
        )
    }

    #[rstest]
    #[case(Reset{message_id: MessageId::from_value(15)}, MessageId::from_value(15))]
    fn message_id(#[case] reset: Reset, #[case] expected: MessageId) {
        assert_eq!(expected, reset.message_id())
    }
}
