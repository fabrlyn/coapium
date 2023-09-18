use crate::codec::parsing::single;

use super::{decoded_option::DecodedOption, number::Number, value::Value, Delta};

#[derive(Clone, Debug, PartialEq)]
pub struct UriPort {
    value: Value,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecodeError {
    SingleValue,
    Format,
}

impl UriPort {
    const NUMBER: u16 = 7;

    pub fn decode(values: Vec<Value>) -> Result<Self, DecodeError> {
        let value = single(values).map_err(|_| DecodeError::SingleValue)?;

        if !value.valid_as_u16() {
            return Err(DecodeError::Format);
        }

        Ok(Self { value })
    }

    pub fn encode(self, delta_sum: Delta) -> Vec<u8> {
        DecodedOption {
            number: Self::number(),
            values: vec![self.value],
        }
        .encode(delta_sum)
    }

    pub fn from_u16(value: u16) -> Self {
        Self {
            value: Value::from_u16(value),
        }
    }

    pub fn number() -> Number {
        Number::from_value_or_panic(Self::NUMBER)
    }

    pub fn value(&self) -> u16 {
        self.value.u16().unwrap()
    }
}

impl Default for UriPort {
    fn default() -> Self {
        Self::from_u16(5683)
    }
}

impl From<u16> for UriPort {
    fn from(value: u16) -> Self {
        Self::from_u16(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{DecodeError, Delta, Number, UriPort, Value};

    #[rstest]
    #[case(vec![Value::from_opaque(vec![]).unwrap()],                                        Ok(UriPort { value: Value::Empty }))]
    #[case(vec![Value::from_opaque(vec![5]).unwrap()],                                       Ok(UriPort { value: Value::from_u16(5) } ))]
    #[case(vec![Value::from_opaque(vec![1, 2]).unwrap()],                                    Ok(UriPort { value: Value::from_u16(258) } ))]
    #[case(vec![Value::from_u16(1337)],                                                      Ok(UriPort { value: Value::from_u16(1337) }))]
    #[case(vec![],                                                                           Err(DecodeError::SingleValue))]
    #[case(vec![Value::from_opaque(vec![1,2,3]).unwrap()],                                   Err(DecodeError::Format))]
    #[case(vec![Value::from_opaque(vec![1]).unwrap(), Value::from_opaque(vec![2]).unwrap()], Err(DecodeError::SingleValue))]
    fn decode(#[case] values: Vec<Value>, #[case] expected: Result<UriPort, DecodeError>) {
        assert_eq!(expected, UriPort::decode(values));
    }

    #[rstest]
    #[case(UriPort { value: Value::from_u16(0) }, vec![7<<4] )]
    #[case(UriPort { value: Value::from_u16(13) }, vec![(7<<4) | 1, 13] )]
    #[case(UriPort { value: Value::from_u16(258) }, vec![(7<<4) | 2, 1, 2] )]
    fn encode(#[case] uri_port: UriPort, #[case] expected: Vec<u8>) {
        assert_eq!(expected, uri_port.encode(Delta::from_value(0)))
    }

    #[rstest]
    #[case(0,    UriPort { value: Value::from_u16(0) } )]
    #[case(3456, UriPort { value: Value::from_u16(3456) } )]
    fn from_u16(#[case] value: u16, #[case] expected: UriPort) {
        assert_eq!(expected, UriPort::from_u16(value))
    }

    #[rstest]
    fn number() {
        assert_eq!(Number::from_value(7).unwrap(), UriPort::number())
    }
}
