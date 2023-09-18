use super::{decoded_option::DecodedOption, number::Number, value::Value, Delta};

#[derive(Clone, Debug, PartialEq)]
pub struct IfNoneMatch;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    SingleValue,
    NotEmpty,
}

impl IfNoneMatch {
    pub fn decode(values: Vec<Value>) -> Result<Self, Error> {
        let [value] = &*values else {
            return Err(Error::SingleValue);
        };

        if !value.is_empty() {
            return Err(Error::NotEmpty);
        }

        Ok(Self)
    }

    pub fn encode(self, delta_sum: Delta) -> Vec<u8> {
        DecodedOption {
            number: Self::number(),
            values: vec![Value::empty()],
        }
        .encode(delta_sum)
    }
    pub fn number() -> Number {
        Number::from_value_or_panic(5)
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use super::IfNoneMatch;
    use crate::codec::option::Value;
    use rstest::rstest;

    #[rstest]
    #[case(vec![], Err(Error::SingleValue))]
    #[case(vec![Value::empty()], Ok(IfNoneMatch))]
    #[case(vec![Value::from_opaque(vec![]).unwrap()], Ok(IfNoneMatch))]
    #[case(vec![Value::from_opaque(vec![0]).unwrap()], Err(Error::NotEmpty))]
    #[case(vec![Value::from_opaque(vec![0]).unwrap(), Value::from_opaque(vec![1]).unwrap()], Err(Error::SingleValue))]
    fn decode(#[case] values: Vec<Value>, #[case] expected: Result<IfNoneMatch, Error>) {
        assert_eq!(expected, IfNoneMatch::decode(values));
    }
}
