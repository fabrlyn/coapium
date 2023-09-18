use super::{
    delta::{self, Delta},
    value::{self, Value},
};

#[derive(Debug, PartialEq, Clone)]
pub struct EncodedOption {
    delta: Delta,
    value: Value,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    HeaderMissing,
    Delta(delta::DecodeError),
    Value(value::Error),
}

impl EncodedOption {
    pub const fn decode(delta: Delta, value: Value) -> Self {
        Self { delta, value }
    }

    pub const fn delta(&self) -> &Delta {
        &self.delta
    }

    pub fn encode(self) -> Vec<u8> {
        let (delta_header, delta_extended) = self.delta.encode();
        let (length_header, length_extended) = self.value.length().encode();

        let flags = delta_header.encode() | length_header.encode();
        let data = self.value.encode();

        let mut bytes = vec![flags];
        bytes.extend(delta_extended);
        bytes.extend(length_extended);
        bytes.extend(data);

        bytes
    }

    pub fn new(delta: Delta, value: Value) -> Self {
        Self::decode(delta, value)
    }

    pub fn parse(bytes: &[u8]) -> Result<(&[u8], Self), Error> {
        let (rest, flags) = Self::parse_header(bytes)?;
        let (rest, delta) = Delta::parse(flags, rest)?;
        let (rest, value) = Value::parse(flags, rest)?;

        Ok((rest, Self::decode(delta, value)))
    }

    fn parse_header(bytes: &[u8]) -> Result<(&[u8], u8), Error> {
        bytes
            .first()
            .ok_or(Error::HeaderMissing)
            .map(|flags| (&bytes[1..], *flags))
    }

    pub fn to_value(self) -> Value {
        self.value
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

impl From<delta::DecodeError> for Error {
    fn from(error: delta::DecodeError) -> Self {
        Self::Delta(error)
    }
}

impl From<value::Error> for Error {
    fn from(value: value::Error) -> Self {
        Self::Value(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{
        super::delta_header, super::length, super::length_header, delta, value, Delta,
        EncodedOption, Error, Value,
    };

    #[rstest]
    #[case(Delta::from_value(0), Value::from_str("a").unwrap(), vec![1, 97])]
    #[case(Delta::from_value(11), Value::from_str("abc").unwrap(), vec![179, 97, 98, 99])]
    fn decode_encode(#[case] delta: Delta, #[case] value: Value, #[case] expected: Vec<u8>) {
        let decoded = EncodedOption::decode(delta, value);
        let encoded = decoded.encode();

        assert_eq!(expected, encoded);
    }

    #[rstest]
    fn delta() {
        assert_eq!(
            &Delta::from_value(11),
            EncodedOption::new(Delta::from_value(11), Value::from_str("a").unwrap()).delta()
        );
    }

    #[rstest]
    fn new() {
        let encoded_option = EncodedOption::new(
            Delta::from_value(2),
            Value::from_opaque(vec![1, 2, 3]).unwrap(),
        );

        assert_eq!(&Delta::from_value(2), encoded_option.delta());
        assert_eq!(
            &Value::from_opaque(vec![1, 2, 3]).unwrap(),
            encoded_option.value()
        );
    }

    #[rstest]
    #[case(&[], Err(Error::HeaderMissing))]
    #[case(&[0b1111_0001], Err(Error::Delta(delta::DecodeError::Header(delta_header::Error::Reserved))))]
    #[case(&[0b0001_1111], Err(Error::Value(value::Error::Length(length::DecodeError::Header(length_header::Error::Reserved)))))]
    #[case(
        &[0b0010_0011, 97, 98, 99],
        Ok((
            [].as_ref(),
            EncodedOption::new(Delta::from_value(2), Value::from_str("abc").unwrap())
        ))
    )]
    #[case(
        &[0b0010_0011, 97, 98, 99, 101],
        Ok((
            [101].as_ref(),
            EncodedOption::new(Delta::from_value(2), Value::from_str("abc").unwrap())
        ))
    )]
    fn parse(#[case] bytes: &[u8], #[case] expected: Result<(&[u8], EncodedOption), Error>) {
        assert_eq!(expected, EncodedOption::parse(bytes))
    }

    #[rstest]
    fn to_value() {
        let encoded_option = EncodedOption::new(
            Delta::from_value(2),
            Value::from_opaque(vec![1, 2, 3]).unwrap(),
        );
        assert_eq!(
            Value::from_opaque(vec![1, 2, 3]).unwrap(),
            encoded_option.to_value()
        );
    }

    #[rstest]
    fn value() {
        let encoded_option = EncodedOption::new(
            Delta::from_value(2),
            Value::from_opaque(vec![1, 2, 3]).unwrap(),
        );
        assert_eq!(
            &Value::from_opaque(vec![1, 2, 3]).unwrap(),
            encoded_option.value()
        );
    }
}
