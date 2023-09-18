use super::length::{self, Length};
use Value::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Empty,
    Bytes(Length, Vec<u8>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Length(length::DecodeError),
    Value(ValueError),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ValueError {
    LengthOutOfBounds,
}

// TODO: Look at introducing typed values, like StringValue, U16Value, etc
impl Value {
    pub fn len(&self) -> usize {
        match self {
            Empty => 0,
            Bytes(length, _) => usize::from(length.value()),
        }
    }

    pub fn parse<'a>(header_byte: u8, bytes: &'a [u8]) -> Result<(&'a [u8], Value), Error> {
        let (bytes, length) = Length::parse(header_byte, bytes)?;

        let length = usize::from(length.value());

        if bytes.len() < length {
            Err(Error::Value(ValueError::LengthOutOfBounds))
        } else {
            Ok((&bytes[length..], Self::decode(bytes[..length].to_vec())?))
        }
    }

    pub fn decode(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.is_empty() {
            return Ok(Self::Empty);
        }

        let length = u16::try_from(bytes.len())
            .map(Length::from_value)
            .map_err(|_e| Error::Value(ValueError::LengthOutOfBounds))?;

        Ok(Self::Bytes(length, bytes))
    }

    pub fn u16(&self) -> Result<u16, ()> {
        match self {
            Empty => Ok(0),
            Bytes(_, bytes) => match bytes.len() {
                0 => Ok(0),
                1 => Ok(u16::from_be_bytes([0, bytes[0]])),
                2 => Ok(u16::from_be_bytes([bytes[0], bytes[1]])),
                _ => Err(()),
            },
        }
    }

    pub fn u32(&self) -> Result<u32, ()> {
        match self {
            Empty => Ok(0),
            Bytes(_, bytes) => match bytes.len() {
                0 => Ok(0),
                1 => Ok(u32::from_be_bytes([0, 0, 0, bytes[0]])),
                2 => Ok(u32::from_be_bytes([0, 0, bytes[0], bytes[1]])),
                3 => Ok(u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]])),
                4 => Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])),
                _ => Err(()),
            },
        }
    }

    pub fn empty() -> Self {
        Value::Empty
    }

    pub fn is_bytes(&self) -> bool {
        match self {
            Bytes(_, _) => true,
            _ => false,
        }
    }

    pub fn length(&self) -> Length {
        match self {
            Empty => Length::from_value(0),
            Bytes(length, _) => *length,
        }
    }

    pub fn encode(self) -> Vec<u8> {
        match self {
            Empty => vec![],
            Bytes(_, bytes) => bytes,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Empty => true,
            _ => false,
        }
    }

    pub fn from_u32(value: u32) -> Self {
        if value == 0 {
            return Self::Empty;
        }

        if let Ok(value) = u8::try_from(value) {
            return Self::from_u8(value);
        }

        if let Ok(value) = u16::try_from(value) {
            return Self::from_u16(value);
        }

        let value = value.to_be_bytes();
        Self::Bytes(Length::from_value(value.len() as u16), value.to_vec())
    }

    pub fn from_u16(value: u16) -> Self {
        if value == 0 {
            return Self::Empty;
        }

        if let Ok(value) = u8::try_from(value) {
            return Self::from_u8(value);
        }

        let value = value.to_be_bytes();
        Self::Bytes(Length::from_value(value.len() as u16), value.to_vec())
    }

    pub fn from_u8(value: u8) -> Self {
        if value == 0 {
            return Self::Empty;
        }

        let value = value.to_be_bytes();
        Self::Bytes(Length::from_value(value.len() as u16), value.to_vec())
    }

    pub fn from_str(value: &str) -> Result<Self, Error> {
        Self::from_string(value.to_owned())
    }

    pub fn from_string(value: String) -> Result<Self, Error> {
        Self::decode(value.into_bytes())
    }

    pub fn from_opaque(value: Vec<u8>) -> Result<Self, Error> {
        Self::decode(value)
    }

    pub fn valid_as_u32(&self) -> bool {
        self.u32().is_err()
    }

    pub fn valid_as_u16(&self) -> bool {
        self.u16().is_ok()
    }

    pub fn valid_as_string(&self) -> bool {
        match self {
            Empty => true,
            Bytes(_, bytes) => String::from_utf8(bytes.clone()).is_ok(),
        }
    }

    pub fn string(self) -> Result<String, ()> {
        match self {
            Empty => Ok("".to_owned()),
            Bytes(_, bytes) => String::from_utf8(bytes).map_err(|_| ()),
        }
    }

    pub fn opaque(self) -> Vec<u8> {
        match self {
            Empty => vec![],
            Bytes(_, bytes) => bytes,
        }
    }
}

impl From<length::DecodeError> for Error {
    fn from(error: length::DecodeError) -> Self {
        Self::Length(error)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use quickcheck_macros::quickcheck;
    use rstest::rstest;

    use super::{
        Length,
        Value::{self, *},
    };

    #[rstest]
    #[case(&[])]
    #[case(&[1, 2])]
    fn empty(#[case] input: &[u8]) {
        assert_eq!((input, Empty), Value::parse(0, input).unwrap());
    }

    #[rstest]
    #[case(5, &[])]
    #[case(5, &[1,2,3])]
    fn error(#[case] length: u8, #[case] bytes: &[u8]) {
        assert!(Value::parse(length, bytes).is_err());
    }

    #[rstest]
    #[case(3, &[1,2,3], &[], &[1,2,3])]
    #[case(2, &[1,2,3], &[3], &[1,2])]
    fn bytes(
        #[case] length: u8,
        #[case] input: &[u8],
        #[case] expected_rest: &[u8],
        #[case] expected_result: &[u8],
    ) {
        assert_eq!(
            (
                expected_rest,
                Value::Bytes(
                    Length::from_value(u16::from(length)),
                    Vec::from(expected_result)
                )
            ),
            Value::parse(length, input).unwrap()
        );
    }

    #[rstest]
    #[case(vec![],      Ok(0))]
    #[case(vec![15],    Ok(15))]
    #[case(vec![1, 0],  Ok((u8::MAX as u16) + 1))]
    #[case(vec![1,2,3], Err(()))]
    fn u16(#[case] value: Vec<u8>, #[case] expected: Result<u16, ()>) {
        assert_eq!(expected, Value::from_opaque(value).unwrap().u16());
    }

    #[quickcheck]
    fn u32(value: u32) {
        assert_eq!(
            Ok(value),
            Value::from_opaque(value.to_be_bytes().to_vec())
                .unwrap()
                .u32()
        );
    }
}
