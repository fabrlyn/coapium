use crate::codec::parsing::single;

use super::{decoded_option::DecodedOption, number::Number, value::Value, Delta};

#[derive(Clone, Debug, PartialEq)]
pub struct MaxAge {
    value: Value,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecodeError {
    SingleValue,
    Format,
}

impl MaxAge {
    const DEFAULT: u32 = 60;
    const NUMBER: u16 = 14;

    pub fn decode(values: Vec<Value>) -> Result<Self, DecodeError> {
        let value = single(values).map_err(|_| DecodeError::SingleValue)?;

        let value = value.u32().map_err(|_| DecodeError::Format)?;

        Ok(Self {
            value: Value::from_u32(value),
        })
    }

    pub fn default() -> Self {
        Self {
            value: Value::from_u32(Self::DEFAULT),
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
        Number::from_value_or_panic(Self::NUMBER)
    }
}

impl Default for MaxAge {
    fn default() -> Self {
        Self::default()
    }
}

impl From<u32> for MaxAge {
    fn from(value: u32) -> Self {
        Self {
            value: Value::from_u32(value),
        }
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{DecodeError, Delta, MaxAge, Number, Value};

    #[rstest]
    #[case(vec![Value::from_opaque(vec![]).unwrap()],                                        Ok(MaxAge { value: Value::Empty } ))]
    #[case(vec![Value::from_u32(10)],                                                        Ok(MaxAge { value: Value::from_u32(10) }))]
    #[case(vec![],                                                                           Err(DecodeError::SingleValue))]
    #[case(vec![Value::from_opaque(vec![1, 2, 3, 4, 5]).unwrap()],                           Err(DecodeError::Format))]
    #[case(vec![Value::from_opaque(vec![1]).unwrap(), Value::from_opaque(vec![2]).unwrap()], Err(DecodeError::SingleValue))]
    fn decode(#[case] values: Vec<Value>, #[case] expected: Result<MaxAge, DecodeError>) {
        assert_eq!(expected, MaxAge::decode(values));
    }

    #[rstest]
    fn default() {
        assert_eq!(
            MaxAge {
                value: Value::from_u32(60)
            },
            MaxAge::default()
        )
    }

    #[rstest]
    #[case(MaxAge { value: Value::from_u32(132) }, vec![0b1101_0001, 1, 132])]
    fn encode(#[case] max_age: MaxAge, #[case] expected: Vec<u8>) {
        assert_eq!(expected, max_age.encode(Delta::from_value(0)))
    }

    #[rstest]
    fn number() {
        assert_eq!(Number::from_value(14).unwrap(), MaxAge::number())
    }
}

// Happiness of could-be dreams eclipse late hours of accomplishment
