#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MessageId {
    value: u16,
}

impl MessageId {
    pub const fn decode(bytes: [u8; 2]) -> Self {
        Self::from_value(u16::from_be_bytes(bytes))
    }

    pub const fn encode(self) -> [u8; 2] {
        self.value.to_be_bytes()
    }

    pub const fn from_value(value: u16) -> Self {
        MessageId { value }
    }

    pub const fn next(&self) -> Self {
        Self::from_value(self.value.overflowing_add(1).0)
    }

    pub const fn value(&self) -> u16 {
        self.value
    }
}

impl From<u16> for MessageId {
    fn from(value: u16) -> Self {
        Self::from_value(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::MessageId;

    #[rstest]
    #[case([0, 2], 2)]
    #[case([1, 2], 258)]
    fn decode_value_encode(#[case] bytes: [u8; 2], #[case] expected_value: u16) {
        let decoded = MessageId::decode(bytes);
        let value = decoded.value();
        let encoded = decoded.encode();

        assert_eq!(
            MessageId {
                value: expected_value
            },
            decoded
        );
        assert_eq!(expected_value, value);
        assert_eq!(bytes, encoded);
    }

    #[rstest]
    fn from_value() {
        for value in 0..=u16::MAX {
            assert_eq!(MessageId { value }, MessageId::from_value(value));
        }
    }

    #[rstest]
    #[case(MessageId{ value: 0 }, MessageId{ value: 1 })]
    #[case(MessageId{ value: u16::MAX }, MessageId{ value: 0 })]
    fn next(#[case] message_id: MessageId, #[case] expected: MessageId) {
        assert_eq!(expected, message_id.next())
    }
}
