use crate::codec::{parsing::single, MediaType};

use super::{decoded_option::DecodedOption, number::Number, value::Value, Delta};

#[derive(Clone, Debug, PartialEq)]
pub struct Accept {
    value: Value,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    SingleValue,
    Format,
}

impl Accept {
    pub fn decode(values: Vec<Value>) -> Result<Self, Error> {
        let value = single(values).map_err(|_| Error::SingleValue)?;

        value
            .to_owned()
            .u16()
            .map_err(|_| Error::Format)
            .map(MediaType::from_value)?;

        Ok(Self { value })
    }

    pub fn encode(self, delta_sum: Delta) -> Vec<u8> {
        DecodedOption {
            number: Self::number(),
            values: vec![self.value],
        }
        .encode(delta_sum)
    }

    pub fn number() -> Number {
        Number::from_value_or_panic(17)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{Accept, Error, MediaType, Value};

    #[rstest]
    #[case(
        vec![],
        Err(Error::SingleValue)
    )]
    #[case(
        vec![Value::from_u16(MediaType::ApplicationJson.value().unwrap())], 
        Ok(Accept { value: Value::from_u16(MediaType::ApplicationJson.value().unwrap()) }))]
    #[case(
        vec![Value::from_opaque(vec![]).unwrap()], 
        Ok(Accept {value: Value::from_u16(MediaType::TextPlain.value().unwrap()) } )
        //Err(Error::Format)
    )]
    #[case(
        vec![Value::from_opaque(vec![1]).unwrap(), Value::from_opaque(vec![2]).unwrap()], 
        Err(Error::SingleValue)
    )]
    #[case(
        vec![Value::from_opaque(vec![1, 2, 3]).unwrap()], 
        Err(Error::Format)
    )]
    fn decode(#[case] values: Vec<Value>, #[case] expected: Result<Accept, Error>) {
        assert_eq!(expected, Accept::decode(values));
    }
}
