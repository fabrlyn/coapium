/// Mask for decoding.
const MASK: u8 = 0b0000_1111;

/// The token length indicating the length of the [`Token`](`crate::codec::Token`) in the [`Message`](`crate::codec::Message`).
///
/// The token length(`TKL`) consists of a 4-bit value following the [`MessageType`](`crate::codec::MessageType`)(`T`) in the first byte of the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
///  
/// ```markdown
/// 0                 
/// 0 1 2 3 4 5 6 7 8
/// +-+-+-+-+-+-+-+-+-
/// |Ver| T |  TKL  |
/// +-+-+-+-+-+-+-+-+-
/// ```
///
/// The token length value can be between `0 - 8` and any value larger than `8` is reserved for future use and **must** be parsed as an error.
///
/// A reserved value will treated as a parsing error and will result in [`OutOfBounds`](`Error::OutOfBounds`).
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TokenLength {
    value: u8,
}

/// Possible errors when decoding [`TokenLength`](`TokenLength`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    /// The byte value exceeds [`MAX_VALUE`](`MAX_VALUE`).
    ///
    /// Contains the out of range value.
    OutOfRange(u8),
}

impl TokenLength {
    /// Max token length value.
    pub const MAX: u8 = 8;

    /// Parse the byte from the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
    pub const fn decode(byte: u8) -> Self {
        Self {
            value: (byte & MASK),
        }
    }

    /// Encode to a byte formatted to fit into the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
    pub const fn encode(self) -> u8 {
        self.value & MASK
    }

    /// Create a [`TokenLength`](`TokenLength`) from a `u8`.
    pub const fn from_value(value: u8) -> Result<Self, Error> {
        if value > Self::MAX {
            Err(Error::OutOfRange(value))
        } else {
            Ok(Self::decode(value))
        }
    }

    /// Returns `true` if the token length is indicating that the [`Token`](`crate::codec::Token`) is empty.
    pub const fn is_zero_length(&self) -> bool {
        self.value == 0
    }

    /// Get the numeric value of the token length.
    pub const fn value(&self) -> u8 {
        self.value
    }

    /// Create a [`TokenLength`](`TokenLength`) configured to indicate an empty [`Token`](`crate::codec::Token`)
    pub const fn zero_length() -> Self {
        TokenLength { value: 0 }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use std::ops::RangeInclusive;

    use super::{Error, TokenLength};

    #[rstest]
    #[case(0b000..=0b1111,  0b0000, 0)]
    #[case(0b000..=0b1111,  0b0001, 1)]
    #[case(0b000..=0b1111,  0b0010, 2)]
    #[case(0b000..=0b1111,  0b0011, 3)]
    #[case(0b000..=0b1111,  0b0100, 4)]
    #[case(0b000..=0b1111,  0b0101, 5)]
    #[case(0b000..=0b1111,  0b0110, 6)]
    #[case(0b000..=0b1111,  0b0111, 7)]
    #[case(0b000..=0b1111,  0b1000, 8)]
    fn decode(
        #[case] upper_ranges: RangeInclusive<u8>,
        #[case] lower_range: u8,
        #[case] expected: u8,
    ) {
        for upper_range in upper_ranges {
            let input = (upper_range << 4) | lower_range;
            assert_eq!(expected, TokenLength::decode(input).value);
        }
    }

    #[rstest]
    fn encode() {
        for value in 0..=8 {
            assert_eq!(value, TokenLength::from_value(value).unwrap().encode());
        }
    }

    #[rstest]
    fn from_value() {
        for value in 0..=8 {
            assert_eq!(Ok(TokenLength { value }), TokenLength::from_value(value));
        }
    }

    #[rstest]
    fn from_value_out_of_range() {
        for value in 9..=u8::MAX {
            assert_eq!(
                Err(Error::OutOfRange(value)),
                TokenLength::from_value(value)
            );
        }
    }

    #[rstest]
    fn is_zero_length() {
        assert!(TokenLength::from_value(0).unwrap().is_zero_length())
    }

    #[rstest]
    fn is_not_zero_length() {
        for value in 1..=8 {
            assert!(!TokenLength::from_value(value).unwrap().is_zero_length())
        }
    }

    #[rstest]
    #[case(0..=TokenLength::MAX)]
    fn value(#[case] inputs: RangeInclusive<u8>) {
        for input in inputs {
            assert_eq!(input, TokenLength::decode(input).value())
        }
    }

    #[rstest]
    fn zero_length() {
        assert!(TokenLength::zero_length().is_zero_length())
    }
}
