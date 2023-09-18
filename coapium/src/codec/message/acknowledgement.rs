use crate::codec::{Code, Header, MessageId, MessageType, TokenLength};

use super::{Error, FormatError};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Acknowledgement {
    message_id: MessageId,
}

impl Acknowledgement {
    pub fn decode(
        message_id: MessageId,
        token_length: TokenLength,
        remaining_bytes: &[u8],
    ) -> Result<Self, Error> {
        if !token_length.is_zero_length() {
            return Err(FormatError::TokenLengthNonZero)?;
        }

        if !remaining_bytes.is_empty() {
            return Err(FormatError::ExcessiveData)?;
        }

        Ok(Acknowledgement { message_id })
    }

    pub fn encode(self) -> Vec<u8> {
        Header::new(
            MessageType::Acknowledgement,
            TokenLength::zero_length(),
            Code::Empty,
            self.message_id,
        )
        .encode()
    }

    pub fn message_id(&self) -> MessageId {
        self.message_id
    }

    pub fn new(message_id: MessageId) -> Self {
        Self { message_id }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{Acknowledgement, Error, FormatError, MessageId, TokenLength};

    #[rstest]
    #[case(
        MessageId::from_value(5), 
        TokenLength::zero_length(), 
        &[], 
        Ok(Acknowledgement{ message_id: MessageId::from_value(5) })
    )]
    #[case(
        MessageId::from_value(5), 
        TokenLength::from_value(4).unwrap(), 
        &[], 
        Err(Error::Format(FormatError::TokenLengthNonZero))
        
    )]
    #[case(
        MessageId::from_value(5), 
        TokenLength::zero_length(), 
        &[1, 2, 3], 
        Err(Error::Format(FormatError::ExcessiveData))
        
    )]
    fn decode(
        #[case] message_id: MessageId,
        #[case] token_length: TokenLength,
        #[case] remaining_bytes: &[u8],
        #[case] expected: Result<Acknowledgement, Error>,
    ) {
        assert_eq!(
            expected,
            Acknowledgement::decode(message_id, token_length, remaining_bytes)
        )
    }

    #[rstest]
    #[case(Acknowledgement{ message_id: MessageId::from_value(6) }, &[0b01_10_0000, 0b00000000, 0b00000000, 0b00000110])]
    fn encode(#[case] acknowledgement: Acknowledgement, #[case] expected: &[u8]) {
        assert_eq!(expected, acknowledgement.encode())
    }

    #[rstest]
    #[case(Acknowledgement {message_id: MessageId::from_value(13)}, MessageId::from_value(13))]
    fn message_id(#[case] acknowledgement: Acknowledgement, #[case] expected: MessageId) {
        assert_eq!(expected, acknowledgement.message_id())
    }

    #[rstest]
    #[case(MessageId::from_value(22), Acknowledgement {message_id: MessageId::from_value(22)})]
    fn new(#[case] message_id: MessageId, #[case] expected: Acknowledgement) {
        assert_eq!(expected, Acknowledgement::new(message_id))
    }
}
