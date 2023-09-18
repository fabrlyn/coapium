use crate::codec::parsing::single;

use super::{decoded_option::DecodedOption, number::Number, value::Value, Delta};

#[derive(Clone, Debug, PartialEq)]
pub struct Size1 {
    value: Value,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    SingleValue,
    Format,
}

impl Size1 {
    pub fn decode(values: Vec<Value>) -> Result<Self, Error> {
        let value = single(values).map_err(|_| Error::SingleValue)?;

        let value = value.u32().map_err(|_| Error::Format)?;

        Ok(Self {
            value: Value::from_u32(value),
        })
    }

    pub fn encode(self, delta_sum: Delta) -> Vec<u8> {
        DecodedOption {
            number: Self::number(),
            values: vec![self.value],
        }
        .encode(delta_sum)
    }

    pub fn number() -> Number {
        Number::from_value_or_panic(60)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{Error, Size1, Value};

    #[rstest]
    #[case(vec![Value::from_opaque(vec![10]).unwrap()],                                      Ok(Size1 { value: Value::from_u32(10) }))]
    #[case(vec![Value::from_opaque(vec![1, 2]).unwrap()],                                    Ok(Size1 { value: Value::from_u32(258) }))]
    #[case(vec![Value::from_opaque(vec![]).unwrap()],                                        Ok(Size1 { value: Value::Empty } ))]
    #[case(vec![],                                                                           Err(Error::SingleValue))]
    #[case(vec![Value::from_opaque(vec![1, 2, 3, 4, 5]).unwrap()],                           Err(Error::Format))]
    #[case(vec![Value::from_opaque(vec![1]).unwrap(), Value::from_opaque(vec![2]).unwrap()], Err(Error::SingleValue))]
    fn decode(#[case] values: Vec<Value>, #[case] expected: Result<Size1, Error>) {
        assert_eq!(expected, Size1::decode(values));
    }
}
