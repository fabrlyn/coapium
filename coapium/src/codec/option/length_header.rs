const MASK: u8 = 0b00001111;

pub const EXTENDED_8_BIT: u8 = 13;
pub const EXTENDED_16_BIT: u8 = 14;
pub const RESERVED_FOR_FUTURE: u8 = 15;

const MAX_LENGTH_VALUE: u8 = 12;
const MAX_HEADER_VALUE: u8 = 15;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Value(u8);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LengthHeader {
    Length(Value),
    Extended8Bit,
    Extended16Bit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Range(u8),
    Reserved,
}

impl LengthHeader {
    pub const fn decode(byte: u8) -> Result<Self, Error> {
        Self::from_value(byte & MASK)
    }

    pub const fn encode(self) -> u8 {
        self.value()
    }

    pub const fn from_value(value: u8) -> Result<Self, Error> {
        if value > MAX_HEADER_VALUE {
            return Err(Error::Range(value));
        }

        match value & MASK {
            EXTENDED_8_BIT => Ok(Self::Extended8Bit),
            EXTENDED_16_BIT => Ok(Self::Extended16Bit),
            RESERVED_FOR_FUTURE => Err(Error::Reserved),
            value => match Value::from_value(value) {
                Ok(value) => Ok(Self::Length(value)),
                Err(_) => Err(Error::Range(value)),
            },
        }
    }

    pub const fn from_value_or_panic(value: u8) -> Self {
        match Self::from_value(value) {
            Ok(length_header) => length_header,
            Err(_) => panic!("Invalid value"),
        }
    }

    pub const fn value(&self) -> u8 {
        match self {
            Self::Length(value) => value.0,
            Self::Extended8Bit => EXTENDED_8_BIT,
            Self::Extended16Bit => EXTENDED_16_BIT,
        }
    }
}

impl Value {
    pub const fn from_value(value: u8) -> Result<Self, ()> {
        if value > MAX_LENGTH_VALUE {
            Err(())
        } else {
            Ok(Self(value))
        }
    }

    pub const fn from_value_or_panic(value: u8) -> Self {
        match Self::from_value(value) {
            Ok(value) => value,
            Err(_) => panic!("Invalid value"),
        }
    }

    pub const fn value(&self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{Error, LengthHeader};

    #[rstest]
    fn decode_encode() {
        for byte in 0..=14 {
            assert_eq!(byte, LengthHeader::decode(byte).unwrap().encode());
        }
    }

    #[rstest]
    fn decode_reserved_for_payload() {
        assert_eq!(Err(Error::Reserved), LengthHeader::decode(15))
    }

    #[rstest]
    #[case(13, LengthHeader::Extended8Bit)]
    #[case(14, LengthHeader::Extended16Bit)]
    fn decode_extended(#[case] byte: u8, #[case] expected: LengthHeader) {
        assert_eq!(expected, LengthHeader::decode(byte).unwrap())
    }

    #[rstest]
    fn isomorphic_value() {
        for value in 0..=12 {
            let delta_header = LengthHeader::from_value(value).unwrap();
            assert_eq!(value, delta_header.value())
        }
    }

    #[rstest]
    fn from_value_extended_8bit() {
        assert_eq!(
            LengthHeader::Extended8Bit,
            LengthHeader::from_value(13).unwrap()
        )
    }

    #[rstest]
    fn from_value_extended_16bit() {
        assert_eq!(
            LengthHeader::Extended16Bit,
            LengthHeader::from_value(14).unwrap()
        )
    }

    #[rstest]
    fn from_value_reserved_for_payload() {
        assert_eq!(Err(Error::Reserved), LengthHeader::from_value(15))
    }

    #[rstest]
    fn from_value_value_out_of_bounds() {
        for value in 16..=u8::MAX {
            assert_eq!(Err(Error::Range(value)), LengthHeader::from_value(value))
        }
    }
}
