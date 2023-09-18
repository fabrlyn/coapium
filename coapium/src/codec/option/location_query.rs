use super::{decoded_option::DecodedOption, number::Number, value::Value, Delta};

#[derive(Clone, Debug, PartialEq)]
pub struct LocationQuery {
    values: Vec<Value>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Format,
    Length(usize),
}

impl LocationQuery {
    const MAX_LENGTH: usize = 255;

    fn decode_value(value: Value) -> Result<Value, Error> {
        if !value.valid_as_string() {
            return Err(Error::Format);
        }

        if value.len() > Self::MAX_LENGTH {
            Err(Error::Length(value.len()))
        } else {
            Ok(value)
        }
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
        Number::from_value_or_panic(20)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::Error;
    use super::LocationQuery;
    use crate::codec::option::Value;

    #[rstest]
    #[case(vec![], Ok(LocationQuery { values: vec![] }))]
    #[case(vec![Value::from_str("abc").unwrap()], Ok(LocationQuery { values: vec![Value::from_str("abc").unwrap()] }))]
    #[case(vec![Value::from_opaque(vec![0xbf]).unwrap()], Err(Error::Format))]
    #[case(vec![Value::from_string("a".repeat(LocationQuery::MAX_LENGTH + 1)).unwrap()], Err(Error::Length(LocationQuery::MAX_LENGTH + 1)))]
    fn decode(#[case] values: Vec<Value>, #[case] expected: Result<LocationQuery, Error>) {
        assert_eq!(expected, LocationQuery::decode(values));
    }
}
