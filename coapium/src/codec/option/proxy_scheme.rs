use super::{decoded_option::DecodedOption, number::Number, value::Value, Delta};

#[derive(Clone, Debug, PartialEq)]
pub struct ProxyScheme {
    value: Value,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Format,
    SingleValue,
    Length(usize),
}

impl ProxyScheme {
    const MAX_LENGTH: usize = 255;
    const MIN_LENGTH: usize = 1;

    pub fn decode(values: Vec<Value>) -> Result<Self, Error> {
        let [value] = &*values else {
            return Err(Error::SingleValue);
        };

        if !value.valid_as_string() {
            return Err(Error::Format);
        }

        if value.len() > Self::MAX_LENGTH || value.len() < Self::MIN_LENGTH {
            Err(Error::Length(value.len()))
        } else {
            Ok(Self {
                value: value.clone(),
            })
        }
    }

    pub fn encode(self, delta_sum: Delta) -> Vec<u8> {
        DecodedOption {
            number: Self::number(),
            values: vec![self.value],
        }
        .encode(delta_sum)
    }

    pub fn number() -> Number {
        Number::from_value_or_panic(39)
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use super::ProxyScheme;
    use crate::codec::option::Value;
    use rstest::rstest;

    #[rstest]
    #[case(vec![], Err(Error::SingleValue))]
    #[case(vec![Value::from_string("a".repeat(ProxyScheme::MIN_LENGTH - 1)).unwrap()], Err(Error::Length(ProxyScheme::MIN_LENGTH - 1)))]
    #[case(vec![Value::from_string("a".repeat(ProxyScheme::MAX_LENGTH + 1)).unwrap()], Err(Error::Length(ProxyScheme::MAX_LENGTH + 1)))]
    #[case(vec![Value::from_str("abc").unwrap()], Ok(ProxyScheme { value: Value::from_str("abc").unwrap() }))]
    #[case(vec![Value::from_str("a").unwrap(), Value::from_str("b").unwrap()], Err(Error::SingleValue))]
    #[case(vec![Value::from_opaque(vec![0xbf]).unwrap()], Err(Error::Format))]
    fn decode(#[case] values: Vec<Value>, #[case] expected: Result<ProxyScheme, Error>) {
        assert_eq!(expected, ProxyScheme::decode(values));
    }
}
