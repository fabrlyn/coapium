use super::{
    decoded_option::DecodedOption,
    number::Number,
    value::{self, Value},
    Delta,
};

#[derive(Clone, Debug, PartialEq)]
pub struct IfMatch {
    values: Vec<Value>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Length,
}

impl IfMatch {
    const MAX_LENGTH: usize = 8;

    fn decode_value(value: Value) -> Result<Value, Error> {
        if value.len() > Self::MAX_LENGTH {
            Err(Error::Length)
        } else {
            Ok(value)
        }
    }

    pub fn from_values(values: Vec<Vec<u8>>) -> Result<Self, Error> {
        let values = values
            .into_iter()
            .map(Value::from_opaque)
            .collect::<Result<_, _>>()?;

        Self::decode(values)
    }

    pub fn decode(values: Vec<Value>) -> Result<Self, Error> {
        values
            .into_iter()
            .map(Self::decode_value)
            .collect::<Result<_, _>>()
            .map(|values| Self { values })
    }

    pub fn encode(self, delta_sum: Delta) -> Vec<u8> {
        DecodedOption {
            number: Self::number(),
            values: self.values,
        }
        .encode(delta_sum)
    }

    pub fn number() -> Number {
        Number::from_value_or_panic(1)
    }
}

impl From<value::Error> for Error {
    fn from(_value: value::Error) -> Self {
        Self::Length
    }
}

impl TryFrom<Vec<Vec<u8>>> for IfMatch {
    type Error = Error;
    fn try_from(values: Vec<Vec<u8>>) -> Result<Self, Self::Error> {
        Self::from_values(values)
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use super::IfMatch;
    use rstest::rstest;

    use crate::codec::option::Value;

    #[rstest]
    #[case(vec![], Ok(IfMatch{ values: vec![] }))]
    #[case(vec![Value::from_opaque(vec![]).unwrap()], Ok(IfMatch{ values: vec![Value::from_opaque(vec![]).unwrap()] }))]
    #[case(vec![Value::from_opaque(vec![0, 1, 2]).unwrap()], Ok(IfMatch{ values: vec![Value::from_opaque(vec![0, 1, 2]).unwrap()] }))]
    #[case(vec![Value::from_opaque(vec![0, 1, 2]).unwrap(), Value::from_opaque(vec![3, 4, 5]).unwrap()], Ok(IfMatch{ values: vec![Value::from_opaque(vec![0, 1, 2]).unwrap(), Value::from_opaque(vec![3, 4, 5]).unwrap()] }))]
    #[case(vec![Value::from_opaque(vec![0, 1, 2]).unwrap(), Value::from_opaque(vec![3, 4, 5]).unwrap()], Ok(IfMatch{ values: vec![Value::from_opaque(vec![0, 1, 2]).unwrap(), Value::from_opaque(vec![3, 4, 5]).unwrap()] }))]
    #[case(vec![Value::from_opaque(vec![1].repeat(IfMatch::MAX_LENGTH + 1)).unwrap()], Err(Error::Length))]
    fn decode(#[case] values: Vec<Value>, #[case] expected: Result<IfMatch, Error>) {
        assert_eq!(expected, IfMatch::decode(values));
    }
}
