use std::{cmp::Ordering, ops::Sub};

use super::delta_header::{self, DeltaHeader};

const EXTENDED_8_BIT_OFFSET: u16 = 13;
const EXTENDED_16_BIT_OFFSET: u16 = 269;

const EXTENDED_8_BIT_MAX_VALUE: u16 = (u8::MAX as u16) + EXTENDED_8_BIT_OFFSET;
const EXTENDED_16_BIT_MAX_VALUE: u16 = u16::MAX - EXTENDED_16_BIT_OFFSET;

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub struct Extended8Bit(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub struct Extended16Bit(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delta {
    Length(delta_header::Value),
    Extended8Bit(Extended8Bit),
    Extended16Bit(Extended16Bit),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecodeError {
    Combination(DeltaHeader, usize),
    Header(delta_header::Error),
    OutOfRange(u16),
}

impl Delta {
    pub fn decode(delta_header: DeltaHeader, extended: &[u8]) -> Result<Self, DecodeError> {
        match (delta_header, extended) {
            (DeltaHeader::Length(value), &[]) => Ok(Self::Length(value)),
            (DeltaHeader::Extended8Bit, &[byte]) => Self::decode_extended_8bit(byte),
            (DeltaHeader::Extended16Bit, &[first, second]) => {
                Self::decode_extended_16bit([first, second])
            }
            _ => Err(DecodeError::Combination(delta_header, extended.len())),
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

    pub fn encode(self) -> (DeltaHeader, Vec<u8>) {
        match self {
            Self::Length(delta_header) => (DeltaHeader::Length(delta_header), vec![]),
            Self::Extended8Bit(value) => {
                (DeltaHeader::Extended8Bit, value.0.to_be_bytes().to_vec())
            }
            Self::Extended16Bit(value) => {
                (DeltaHeader::Extended16Bit, value.0.to_be_bytes().to_vec())
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
        use delta_header::Value;

        if value > (u8::MAX as u16) {
            Self::from_extended_value(value)
        } else {
            match Value::from_value(value as u8) {
                Ok(length) => Self::Length(length),
                Err(_) => Self::from_extended_value(value),
            }
        }
    }

    pub fn is_repeating(&self) -> bool {
        *self == Self::repeating()
    }

    pub fn parse(header_byte: u8, bytes: &[u8]) -> Result<(&[u8], Self), DecodeError> {
        match DeltaHeader::decode(header_byte)? {
            header @ DeltaHeader::Length(_) => Ok((bytes, Self::decode(header, &[])?)),
            header @ DeltaHeader::Extended8Bit => {
                Ok((&bytes[1..], Self::decode(header, &bytes[..1])?))
            }
            header @ DeltaHeader::Extended16Bit => {
                Ok((&bytes[2..], Self::decode(header, &bytes[..2])?))
            }
        }
    }

    pub const fn repeating() -> Self {
        Self::Length(delta_header::Value::from_value_or_panic(0))
    }

    pub fn sub(self, other: Self) -> Self {
        Self::from_value(self.value() - other.value())
    }

    pub const fn value(&self) -> u16 {
        match *self {
            Self::Length(length) => length.value() as u16,
            Self::Extended8Bit(length) => (length.0 as u16) + EXTENDED_8_BIT_OFFSET,
            Self::Extended16Bit(length) => length.0 + EXTENDED_16_BIT_OFFSET,
        }
    }
}

impl PartialOrd for Delta {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value().partial_cmp(&other.value())
    }
}

impl Ord for Delta {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(&other.value())
    }
}

impl Sub for Delta {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.sub(rhs)
    }
}

impl From<delta_header::Error> for DecodeError {
    fn from(error: delta_header::Error) -> Self {
        Self::Header(error)
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{delta_header, DecodeError, Delta, DeltaHeader, Extended16Bit, Extended8Bit};

    #[rstest]
    #[case(Delta::from_value(5), Delta::from_value(6), Ordering::Less)]
    #[case(Delta::from_value(5), Delta::from_value(15), Ordering::Less)]
    #[case(Delta::from_value(5), Delta::from_value(65000), Ordering::Less)]
    #[case(Delta::from_value(15), Delta::from_value(6), Ordering::Greater)]
    #[case(Delta::from_value(15), Delta::from_value(16), Ordering::Less)]
    #[case(Delta::from_value(15), Delta::from_value(65000), Ordering::Less)]
    #[case(Delta::from_value(65000), Delta::from_value(5), Ordering::Greater)]
    #[case(Delta::from_value(65000), Delta::from_value(15), Ordering::Greater)]
    #[case(Delta::from_value(65000), Delta::from_value(65001), Ordering::Less)]
    fn ord(#[case] a: Delta, #[case] b: Delta, #[case] ordering: Ordering) {
        assert_eq!(a.cmp(&b), ordering)
    }

    #[rstest]
    #[case(vec![], vec![])]
    #[case(vec![Delta::from_value(15)], vec![Delta::from_value(15)])]
    #[case(vec![Delta::from_value(15), Delta::from_value(5)], vec![Delta::from_value(5), Delta::from_value(15)])]
    #[case(vec![Delta::from_value(15), Delta::from_value(5)], vec![Delta::from_value(5), Delta::from_value(15)])]
    #[case(vec![Delta::from_value(65000), Delta::from_value(15), Delta::from_value(5)], vec![Delta::from_value(5), Delta::from_value(15), Delta::from_value(65000)])]
    fn sort(#[case] mut deltas: Vec<Delta>, #[case] expected: Vec<Delta>) {
        deltas.sort();
        assert_eq!(expected, deltas)
    }

    #[rstest]
    fn decode_value_encode_length() {
        for byte in 0..=12 {
            let header = DeltaHeader::from_value_or_panic(byte);

            let decoded = Delta::decode(header, &[]).unwrap();
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
            let decoded = Delta::decode(DeltaHeader::Extended8Bit, &[extended]).unwrap();
            let value = decoded.value();
            let (encoded_header, encoded_value) = decoded.encode();

            assert_eq!(u16::from(extended) + 13, value);
            assert_eq!(DeltaHeader::Extended8Bit, encoded_header);
            assert_eq!(encoded_value, encoded_value);
        }
    }

    #[rstest]
    fn decode_value_encode_extended_16bit() {
        for extended in 0..=(u16::MAX - 269) {
            let extended_bytes = extended.to_be_bytes().to_vec();

            let decoded = Delta::decode(DeltaHeader::Extended16Bit, &extended_bytes).unwrap();
            let value = decoded.value();
            let (encoded_header, encoded_value) = decoded.encode();

            assert_eq!(u16::from(extended) + 269, value);
            assert_eq!(DeltaHeader::Extended16Bit, encoded_header);
            assert_eq!(extended_bytes, encoded_value);
        }
    }

    #[rstest]
    #[case(&[], Err(DecodeError::Combination(DeltaHeader::Extended8Bit, 0)))]
    #[case(&[1], Ok(Delta::Extended8Bit(Extended8Bit(1))))]
    #[case(&[1, 2], Err(DecodeError::Combination(DeltaHeader::Extended8Bit, 2)))]
    fn decode_extended_8bit(#[case] extended: &[u8], #[case] expected: Result<Delta, DecodeError>) {
        assert_eq!(expected, Delta::decode(DeltaHeader::Extended8Bit, extended));
    }

    #[rstest]
    #[case(&[], Err(DecodeError::Combination(DeltaHeader::Extended16Bit, 0)))]
    #[case(&[1], Err(DecodeError::Combination(DeltaHeader::Extended16Bit, 1)))]
    #[case(&[1, 2], Ok(Delta::Extended16Bit(Extended16Bit(258))))]
    #[case(&[1, 2, 3], Err(DecodeError::Combination(DeltaHeader::Extended16Bit, 3)))]
    fn decode_extended_16bit(
        #[case] extended: &[u8],
        #[case] expected: Result<Delta, DecodeError>,
    ) {
        assert_eq!(
            expected,
            Delta::decode(DeltaHeader::Extended16Bit, extended)
        );
    }

    #[rstest]
    fn decode_out_of_range() {
        for extended in (u16::MAX - 268)..=u16::MAX {
            assert_eq!(
                Err(DecodeError::OutOfRange(extended)),
                Delta::decode(DeltaHeader::Extended16Bit, &extended.to_be_bytes())
            );
        }
    }

    #[rstest]
    fn isomorphic_value() {
        for value in 0..=u16::MAX {
            assert_eq!(value, Delta::from_value(value).value());
        }
    }

    #[rstest]
    fn is_repeating() {
        assert!(Delta::repeating().is_repeating())
    }

    #[rstest]
    fn is_not_repeating() {
        for value in 1..=u16::MAX {
            assert!(!Delta::from_value(value).is_repeating())
        }
    }

    #[rstest]
    #[case(3 << 4, &[1, 2], Ok(([1, 2].as_ref(), Delta::Length(delta_header::Value::from_value_or_panic(3)))))]
    #[case(13 << 4, &[1, 2], Ok(([2].as_ref(), Delta::Extended8Bit(Extended8Bit(1)))))]
    #[case(14 << 4, &[1, 2, 3], Ok(([3].as_ref(), Delta::Extended16Bit(Extended16Bit(258)))))]
    fn parse(
        #[case] header: u8,
        #[case] rest: &[u8],
        #[case] expected: Result<(&[u8], Delta), DecodeError>,
    ) {
        assert_eq!(expected, Delta::parse(header, rest))
    }

    #[rstest]
    #[case(Delta::from_value(0), Delta::from_value(0), Delta::from_value(0))]
    #[case(Delta::from_value(3), Delta::from_value(1), Delta::from_value(2))]
    fn sub(#[case] delta: Delta, #[case] other: Delta, #[case] expected: Delta) {
        assert_eq!(expected, delta.sub(other))
    }
}
