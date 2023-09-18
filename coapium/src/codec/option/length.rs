use std::cmp::Ordering;

use super::length_header::{self, LengthHeader};

const EXTENDED_8_BIT_OFFSET: u16 = 13;
const EXTENDED_16_BIT_OFFSET: u16 = 269;

const EXTENDED_8_BIT_MAX_VALUE: u16 = (u8::MAX as u16) + EXTENDED_8_BIT_OFFSET;
const EXTENDED_16_BIT_MAX_VALUE: u16 = u16::MAX - EXTENDED_16_BIT_OFFSET;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extended8Bit(u8);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extended16Bit(u16);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Length(length_header::Value),
    Extended8Bit(Extended8Bit),
    Extended16Bit(Extended16Bit),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecodeError {
    Combination(LengthHeader, usize),
    Header(length_header::Error),
    OutOfRange(u16),
}

impl Length {
    pub fn decode(length_header: LengthHeader, extended: &[u8]) -> Result<Self, DecodeError> {
        match (length_header, extended) {
            (LengthHeader::Length(value), &[]) => Ok(Self::Length(value)),
            (LengthHeader::Extended8Bit, &[byte]) => Self::decode_extended_8bit(byte),
            (LengthHeader::Extended16Bit, &[first, second]) => {
                Self::decode_extended_16bit([first, second])
            }
            _ => Err(DecodeError::Combination(length_header, extended.len())),
        }
    }

    fn decode_extended_8bit(byte: u8) -> Result<Self, DecodeError> {
        Ok(Self::Extended8Bit(Extended8Bit(byte)))
    }

    fn decode_extended_16bit(bytes: [u8; 2]) -> Result<Self, DecodeError> {
        let value = u16::from_be_bytes(bytes);

        if value > EXTENDED_16_BIT_MAX_VALUE {
            Err(DecodeError::OutOfRange(value))
        } else {
            Ok(Self::Extended16Bit(Extended16Bit(value)))
        }
    }

    pub fn encode(self) -> (LengthHeader, Vec<u8>) {
        match self {
            Length::Length(length_header) => (LengthHeader::Length(length_header), vec![]),
            Length::Extended8Bit(value) => {
                (LengthHeader::Extended8Bit, value.0.to_be_bytes().to_vec())
            }
            Length::Extended16Bit(value) => {
                (LengthHeader::Extended16Bit, value.0.to_be_bytes().to_vec())
            }
        }
    }

    const fn from_extended_value(value: u16) -> Self {
        if value <= EXTENDED_8_BIT_MAX_VALUE {
            Self::Extended8Bit(Extended8Bit((value - EXTENDED_8_BIT_OFFSET) as u8))
        } else {
            Self::Extended16Bit(Extended16Bit(value - EXTENDED_16_BIT_OFFSET))
        }
    }

    pub const fn from_value(value: u16) -> Self {
        use length_header::Value;

        if value > (u8::MAX as u16) {
            Self::from_extended_value(value)
        } else {
            match Value::from_value(value as u8) {
                Ok(length) => Self::Length(length),
                Err(_) => Self::from_extended_value(value),
            }
        }
    }

    pub fn parse(header_byte: u8, bytes: &[u8]) -> Result<(&[u8], Self), DecodeError> {
        match LengthHeader::decode(header_byte)? {
            header @ LengthHeader::Length(_) => Ok((bytes, Self::decode(header, &[])?)),
            header @ LengthHeader::Extended8Bit => {
                Ok((&bytes[1..], Self::decode(header, &bytes[..1])?))
            }
            header @ LengthHeader::Extended16Bit => {
                Ok((&bytes[2..], Self::decode(header, &bytes[..2])?))
            }
        }
    }

    pub const fn value(&self) -> u16 {
        match *self {
            Self::Length(length) => length.value() as u16,
            Self::Extended8Bit(length) => (length.0 as u16) + EXTENDED_8_BIT_OFFSET,
            Self::Extended16Bit(length) => length.0 + EXTENDED_16_BIT_OFFSET,
        }
    }
}

impl From<length_header::Error> for DecodeError {
    fn from(error: length_header::Error) -> Self {
        Self::Header(error)
    }
}

impl PartialOrd for Length {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        u16::partial_cmp(&self.value(), &other.value())
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{length_header, DecodeError, Extended16Bit, Extended8Bit, Length, LengthHeader};

    #[rstest]
    fn decode_value_encode_length() {
        for byte in 0..=12 {
            let header = LengthHeader::from_value_or_panic(byte);

            let decoded = Length::decode(header, &[]).unwrap();
            let value = decoded.value();
            let (encoded_header, encoded_extended) = decoded.encode();

            assert_eq!(u16::from(byte), value);
            assert_eq!(header, encoded_header);
            assert!(encoded_extended.is_empty());
        }
    }

    #[rstest]
    fn decode_value_encode_extended_8bit() {
        for extended in 0..=u8::MAX {
            let decoded = Length::decode(LengthHeader::Extended8Bit, &[extended]).unwrap();
            let value = decoded.value();
            let (encoded_header, encoded_value) = decoded.encode();

            assert_eq!(u16::from(extended) + 13, value);
            assert_eq!(LengthHeader::Extended8Bit, encoded_header);
            assert_eq!(encoded_value, encoded_value);
        }
    }

    #[rstest]
    fn decode_value_encode_extended_16bit() {
        for extended in 0..=(u16::MAX - 269) {
            let extended_bytes = extended.to_be_bytes().to_vec();

            let decoded = Length::decode(LengthHeader::Extended16Bit, &extended_bytes).unwrap();
            let value = decoded.value();
            let (encoded_header, encoded_value) = decoded.encode();

            assert_eq!(u16::from(extended) + 269, value);
            assert_eq!(LengthHeader::Extended16Bit, encoded_header);
            assert_eq!(extended_bytes, encoded_value);
        }
    }

    #[rstest]
    #[case(&[], Err(DecodeError::Combination(LengthHeader::Extended16Bit, 0)))]
    #[case(&[1], Err(DecodeError::Combination(LengthHeader::Extended16Bit, 1)))]
    #[case(&[1, 2], Ok(Length::Extended16Bit(Extended16Bit(258))))]
    #[case(&[1, 2, 3], Err(DecodeError::Combination(LengthHeader::Extended16Bit, 3)))]
    fn decode_extended_16bit(
        #[case] extended: &[u8],
        #[case] expected: Result<Length, DecodeError>,
    ) {
        assert_eq!(
            expected,
            Length::decode(LengthHeader::Extended16Bit, extended)
        );
    }

    #[rstest]
    #[case(&[], Err(DecodeError::Combination(LengthHeader::Extended8Bit, 0)))]
    #[case(&[1], Ok(Length::Extended8Bit(Extended8Bit(1))))]
    #[case(&[1, 2], Err(DecodeError::Combination(LengthHeader::Extended8Bit, 2)))]
    fn decode_extended_8bit(
        #[case] extended: &[u8],
        #[case] expected: Result<Length, DecodeError>,
    ) {
        assert_eq!(
            expected,
            Length::decode(LengthHeader::Extended8Bit, extended)
        );
    }

    #[rstest]
    fn decode_out_of_range() {
        for extended in (u16::MAX - 268)..=u16::MAX {
            assert_eq!(
                Err(DecodeError::OutOfRange(extended)),
                Length::decode(LengthHeader::Extended16Bit, &extended.to_be_bytes())
            );
        }
    }

    #[rstest]
    fn isomorphic_value() {
        for value in 0..=u16::MAX {
            assert_eq!(value, Length::from_value(value).value());
        }
    }

    #[rstest]
    #[case(3, &[1, 2], Ok(([1, 2].as_ref(), Length::Length(length_header::Value::from_value_or_panic(3)))))]
    #[case(13, &[1, 2], Ok(([2].as_ref(), Length::Extended8Bit(Extended8Bit(1)))))]
    #[case(14, &[1, 2, 3], Ok(([3].as_ref(), Length::Extended16Bit(Extended16Bit(258)))))]
    fn parse(
        #[case] header: u8,
        #[case] rest: &[u8],
        #[case] expected: Result<(&[u8], Length), DecodeError>,
    ) {
        assert_eq!(expected, Length::parse(header, rest))
    }

    #[rstest]
    #[case(Length::from_value(2), Length::from_value(1), Ordering::Greater)]
    #[case(Length::from_value(2), Length::from_value(2), Ordering::Equal)]
    #[case(Length::from_value(2), Length::from_value(3), Ordering::Less)]
    fn partial_cmp(#[case] left: Length, #[case] right: Length, #[case] expected: Ordering) {
        assert_eq!(Some(expected), left.partial_cmp(&right))
    }
}
