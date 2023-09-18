const SHIFT: u8 = 4;

const EXTENDED_8_BIT: u8 = 13;
const EXTENDED_16_BIT: u8 = 14;
const RESERVED_FOR_PAYLOAD: u8 = 15;

const MAX_LENGTH_VALUE: u8 = 12;
const MAX_HEADER_VALUE: u8 = 15;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Value(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeltaHeader {
    Length(Value),
    Extended8Bit,
    Extended16Bit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Range(u8),
    Reserved,
}

impl DeltaHeader {
    pub const fn decode(byte: u8) -> Result<Self, Error> {
        Self::from_value(byte >> SHIFT)
    }

    pub const fn encode(self) -> u8 {
        self.value() << SHIFT
    }

    pub const fn from_value(value: u8) -> Result<Self, Error> {
        if value > MAX_HEADER_VALUE {
            return Err(Error::Range(value));
        }

        match value {
            EXTENDED_8_BIT => Ok(Self::Extended8Bit),
            EXTENDED_16_BIT => Ok(Self::Extended16Bit),
            RESERVED_FOR_PAYLOAD => Err(Error::Reserved),
            value => match Value::from_value(value) {
                Ok(value) => Ok(Self::Length(value)),
                Err(_) => Err(Error::Range(value)),
            },
        }
    }

    pub const fn from_value_or_panic(value: u8) -> Self {
        match Self::from_value(value) {
            Ok(delta_header) => delta_header,
            Err(_) => panic!("Invalid value"),
        }
    }

    pub const fn value(&self) -> u8 {
        match self {
            Self::Length(length) => length.0,
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
            Ok(length) => length,
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

    use super::{DeltaHeader, Error};

    #[rstest]
    fn decode_encode() {
        for value in 0..=14 {
            let byte = value << 4;
            assert_eq!(byte, DeltaHeader::decode(byte).unwrap().encode());
        }
    }

    #[rstest]
    fn decode_reserved_for_payload() {
        assert_eq!(Err(Error::Reserved), DeltaHeader::decode(15 << 4))
    }

    #[rstest]
    #[case(13 << 4, DeltaHeader::Extended8Bit)]
    #[case(14 << 4, DeltaHeader::Extended16Bit)]
    fn decode_extended(#[case] byte: u8, #[case] expected: DeltaHeader) {
        assert_eq!(expected, DeltaHeader::decode(byte).unwrap())
    }

    #[rstest]
    fn isomorphic_value() {
        for value in 0..=12 {
            let delta_header = DeltaHeader::from_value(value).unwrap();
            assert_eq!(value, delta_header.value())
        }
    }

    #[rstest]
    fn from_value_extended_8bit() {
        assert_eq!(
            DeltaHeader::Extended8Bit,
            DeltaHeader::from_value(13).unwrap()
        )
    }

    #[rstest]
    fn from_value_extended_16bit() {
        assert_eq!(
            DeltaHeader::Extended16Bit,
            DeltaHeader::from_value(14).unwrap()
        )
    }

    #[rstest]
    fn from_value_reserved_for_payload() {
        assert_eq!(Err(Error::Reserved), DeltaHeader::from_value(15))
    }

    #[rstest]
    fn from_value_value_out_of_bounds() {
        for value in 16..=u8::MAX {
            assert_eq!(Err(Error::Range(value)), DeltaHeader::from_value(value))
        }
    }
}
