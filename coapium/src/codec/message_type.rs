use MessageType::*;

/// Mask for decoding
const MASK: u8 = 0b0011_0000;

/// Steps to shift to encode/decode a byte
const SHIFT: u8 = 4;

/// Numeric value of [`Acknowledgement`](`MessageType::Acknowledgement`)
const ACKNOWLEDGEMENT: u8 = 0b10;

/// Numeric value of [`Confirmable`](`MessageType::Confirmable`)
const CONFIRMABLE: u8 = 0b00;

/// Numeric value of [`NonConfirmable`](`MessageType::NonConfirmable`)
const NON_CONFIRMABLE: u8 = 0b01;

/// Numeric value of [`Reset`](`MessageType::Reset`)
const RESET: u8 = 0b11;

/// The message type of the [`Message`](`crate::codec::Message`).
///
/// The message type(`T`) consists of a 2-bit value following the [`Version`](`crate::codec::Version`)(`Ver`) in the first byte of the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
///  
/// ```markdown
/// 0                 
/// 0 1 2 3 4 5 6 7 8
/// +-+-+-+-+-+-+-+-+-
/// |Ver| T |  TKL  |
/// +-+-+-+-+-+-+-+-+-
/// ```
///
/// There are four message types:
/// - [`Acknowledgement(ACK)`](`MessageType::Acknowledgement`)
/// - [`Confirmable(CON)`](`MessageType::Confirmable`)
/// - [`Non-confirmable(NON)`](`MessageType::NonConfirmable`)
/// - [`Reset(RST)`](`MessageType::Reset`)
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MessageType {
    /// Value(`0b10`) defined by [`ACKNOWLEDGEMENT`](`ACKNOWLEDGEMENT`)
    Acknowledgement,
    /// Value(`0b00`) defined by [`CONFIRMABLE`](`CONFIRMABLE`)
    Confirmable,
    /// Value(`0b01`) defined by [`NON_CONFIRMABLE`](`NON_CONFIRMABLE`)
    NonConfirmable,
    /// Value(`0b11`) defined by [`RESET`](`RESET`)
    Reset,
}

impl MessageType {
    /// Decode the byte from the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
    pub const fn decode(byte: u8) -> Self {
        match (byte & MASK) >> SHIFT {
            ACKNOWLEDGEMENT => Acknowledgement,
            CONFIRMABLE => Confirmable,
            NON_CONFIRMABLE => NonConfirmable,
            RESET => Reset,
            _ => unreachable!(),
        }
    }

    /// Encode to a byte formatted to fit into the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
    pub const fn encode(self) -> u8 {
        self.value() << SHIFT
    }

    /// Get the numeric value of the message type.
    ///
    /// Possible values:
    /// - [`ACKNOWLEDGEMENT`](`ACKNOWLEDGEMENT`)
    /// - [`CONFIRMABLE`](`CONFIRMABLE`)
    /// - [`NON_CONFIRMABLE`](`NON_CONFIRMABLE`)
    /// - [`RESET`](`RESET`)
    pub const fn value(&self) -> u8 {
        match self {
            Acknowledgement => ACKNOWLEDGEMENT,
            Confirmable => CONFIRMABLE,
            NonConfirmable => NON_CONFIRMABLE,
            Reset => RESET,
        }
    }

    /// Returns `true` if the message type is [`Acknowledgement`](`MessageType::Acknowledgement`)
    pub const fn is_acknowledgement(&self) -> bool {
        match self {
            Acknowledgement => true,
            _ => false,
        }
    }

    /// Returns `true` if the message type is [`Confirmable`](`MessageType::Confirmable`)
    pub const fn is_confirmable(&self) -> bool {
        match self {
            Confirmable => true,
            _ => false,
        }
    }

    /// Returns `true` if the message type is [`NonConfirmable`](`MessageType::NonConfirmable`)
    pub const fn is_non_confirmable(&self) -> bool {
        match self {
            NonConfirmable => true,
            _ => false,
        }
    }

    /// Returns `true` if the message type is [`Reset`](`MessageType::Reset`)
    pub const fn is_reset(&self) -> bool {
        match self {
            Reset => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use quickcheck_macros::quickcheck;
    use rstest::rstest;
    use std::ops::RangeInclusive;

    use super::MessageType::{self, *};

    #[rstest]
    #[case(Confirmable, 0b0000_0000)]
    #[case(NonConfirmable, 0b0001_0000)]
    #[case(Acknowledgement, 0b0010_0000)]
    #[case(Reset, 0b0011_0000)]
    fn from_message_type(#[case] input: MessageType, #[case] expected: u8) {
        assert_eq!(expected, MessageType::encode(input));
    }

    #[rstest]
    #[case(0b0000_0000..=0b0000_1111, Confirmable)]
    #[case(0b0001_0000..=0b0001_1111, NonConfirmable)]
    #[case(0b0010_0000..=0b0010_1111, Acknowledgement)]
    #[case(0b0011_0000..=0b0011_1111, Reset)]
    fn from_byte(#[case] inputs: RangeInclusive<u8>, #[case] expected: MessageType) {
        for input in inputs {
            assert_eq!(expected, MessageType::decode(input));
        }
    }

    #[quickcheck]
    fn decode_and_encode(byte: u8) {
        let expected = byte & 0b0011_0000;

        let message_type = MessageType::decode(byte);

        let actual = MessageType::encode(message_type);

        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn handle_any_byte(byte: u8) {
        MessageType::decode(byte);
    }

    #[rstest]
    #[case(Acknowledgement, true)]
    #[case(Confirmable, false)]
    #[case(NonConfirmable, false)]
    #[case(Reset, false)]
    fn is_acknowledgement(#[case] message_type: MessageType, #[case] expected: bool) {
        assert_eq!(message_type.is_acknowledgement(), expected);
    }

    #[rstest]
    #[case(Acknowledgement, false)]
    #[case(Confirmable, true)]
    #[case(NonConfirmable, false)]
    #[case(Reset, false)]
    fn is_confirmable(#[case] message_type: MessageType, #[case] expected: bool) {
        assert_eq!(message_type.is_confirmable(), expected);
    }

    #[rstest]
    #[case(Acknowledgement, false)]
    #[case(Confirmable, false)]
    #[case(NonConfirmable, true)]
    #[case(Reset, false)]
    fn is_non_confirmable(#[case] message_type: MessageType, #[case] expected: bool) {
        assert_eq!(message_type.is_non_confirmable(), expected);
    }

    #[rstest]
    #[case(Acknowledgement, false)]
    #[case(Confirmable, false)]
    #[case(NonConfirmable, false)]
    #[case(Reset, true)]
    fn is_reset(#[case] message_type: MessageType, #[case] expected: bool) {
        assert_eq!(message_type.is_reset(), expected);
    }
}
