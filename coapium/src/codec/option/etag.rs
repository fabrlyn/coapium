use super::{decoded_option::DecodedOption, number::Number, value::Value, Delta};

#[derive(Clone, Debug, PartialEq)]
pub struct ETag {
    values: Vec<Value>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Length(usize),
}

impl ETag {
    const MIN_LENGTH: usize = 1;
    const MAX_LENGTH: usize = 8;

    fn decode_value(value: Value) -> Result<Value, Error> {
        if value.len() < Self::MIN_LENGTH {
            return Err(Error::Length(value.len()));
        }

        if value.len() > Self::MAX_LENGTH {
            return Err(Error::Length(value.len()));
        }

        Ok(value)
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
        Number::from_value_or_panic(4)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::ETag;
    use super::Error;
    use crate::codec::option::Value;

    #[rstest]
    #[case(vec![], Ok(ETag{ values: vec![] }))]
    #[case(vec![Value::from_opaque(vec![]).unwrap()], Err(Error::Length(0)))]
    #[case(vec![Value::from_opaque(vec![1]).unwrap()], Ok(ETag{ values: vec![Value::from_opaque(vec![1]).unwrap()] }))]
    #[case(vec![Value::from_opaque(vec![1]).unwrap(), Value::from_opaque(vec![2, 3]).unwrap()], Ok(ETag{ values: vec![Value::from_opaque(vec![1]).unwrap(), Value::from_opaque(vec![2, 3]).unwrap()] }))]
    #[case(vec![Value::from_opaque(vec![1].repeat(ETag::MAX_LENGTH + 1)).unwrap()], Err(Error::Length(ETag::MAX_LENGTH + 1)))]
    fn decode(#[case] values: Vec<Value>, #[case] expected: Result<ETag, Error>) {
        assert_eq!(expected, ETag::decode(values));
    }
}
