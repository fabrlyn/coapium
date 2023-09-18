use std::fmt::Display;

use url::Url;

use crate::codec::parsing::single_or_err;

use super::{decoded_option::DecodedOption, number::Number, value::Value, Delta};

#[derive(Clone, Debug, PartialEq)]
pub struct UriHost {
    host: Value,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecodeError {
    SingleValue,
    Value(ValueError),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ValueError {
    Format,
    Length(usize),
}

impl UriHost {
    const MAX_LENGTH: usize = 255;
    const NUMBER: u16 = 3;

    pub fn decode(values: Vec<Value>) -> Result<Self, DecodeError> {
        let value = single_or_err(values, || DecodeError::SingleValue)?;
        Ok(to_string(value).and_then(Self::from_value)?)
    }

    pub fn encode(self, delta_sum: Delta) -> Vec<u8> {
        DecodedOption {
            number: Self::number(),
            values: vec![self.host],
        }
        .encode(delta_sum)
    }

    pub fn from_value<S: Into<String>>(value: S) -> Result<Self, ValueError> {
        to_url(value.into())
            //.and_then(validate_length) TODO: validate that only host was given
            .and_then(to_host)
            .and_then(validate_length)
            .and_then(to_value)
            .map(to_uri_host)
    }

    pub fn number() -> Number {
        Number::from_value_or_panic(Self::NUMBER)
    }
}

fn to_host(url: Url) -> Result<String, ValueError> {
    url.host_str()
        .map(ToOwned::to_owned)
        .ok_or(ValueError::Format)
}

fn to_string(value: Value) -> Result<String, ValueError> {
    value.string().map_err(|_| ValueError::Format)
}

fn to_url(value: String) -> Result<Url, ValueError> {
    Url::parse(&format!("coap://{}", value)).map_err(|_| ValueError::Format)
}

fn to_uri_host(value: Value) -> UriHost {
    UriHost { host: value }
}

fn to_value(host: String) -> Result<Value, ValueError> {
    Value::from_string(host).map_err(|_| ValueError::Format)
}

fn validate_length(host: String) -> Result<String, ValueError> {
    if host.is_empty() || host.len() > UriHost::MAX_LENGTH {
        Err(ValueError::Length(host.len()))
    } else {
        Ok(host)
    }
}

impl Display for UriHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.host.clone().string().unwrap()) // TODO: Remove clone
    }
}

impl From<ValueError> for DecodeError {
    fn from(error: ValueError) -> Self {
        Self::Value(error)
    }
}

impl TryFrom<&str> for UriHost {
    type Error = ValueError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        UriHost::from_value(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{DecodeError, DecodedOption, Delta, Number, UriHost, Value, ValueError};

    #[rstest]
    #[case(
        vec![], 
        Err(DecodeError::SingleValue)
    )]
    #[case(
        vec![Value::from_str("").unwrap()], 
        Err(DecodeError::Value(ValueError::Format))
    )]
    #[case(
        vec![Value::from_str("robertbarl.in").unwrap()], 
        Ok(UriHost { host: Value::from_str("robertbarl.in").unwrap() })
    )]
    #[case(
        vec![Value::from_str("robertbarl.in").unwrap(),
        Value::from_str("robertbarl.in").unwrap()], Err(DecodeError::SingleValue)
    )]
    #[case(
        vec![Value::from_string(format!("{}.com", "a".repeat(255))).unwrap()], 
        Err(DecodeError::Value(ValueError::Length(259)))
    )]
    fn decode(#[case] values: Vec<Value>, #[case] expected: Result<UriHost, DecodeError>) {
        assert_eq!(expected, UriHost::decode(values));
    }

    #[rstest]
    #[case(
        UriHost { host: Value::from_str("robertbarl.in").unwrap() }, 
        DecodedOption { number: UriHost::number(), values: vec![Value::from_str("robertbarl.in").unwrap()] }
    )]
    fn encode(#[case] uri_host: UriHost, #[case] expected: DecodedOption) {
        assert_eq!(
            expected.encode(Delta::from_value(0)),
            uri_host.encode(Delta::from_value(0))
        );
    }

    #[rstest]
    #[case(
        "192.168.1.123",
        Ok(UriHost { host: Value::from_str("192.168.1.123").unwrap() })
    )]
    #[case(
        "robertbarl.in",
        Ok(UriHost { host: Value::from_str("robertbarl.in").unwrap() })
    )]
    #[case(
        "[2001:db8:aaaa:bbbb:cccc:dddd:eeee:aaaa]", 
        Ok(UriHost { host: Value::from_str("[2001:db8:aaaa:bbbb:cccc:dddd:eeee:aaaa]").unwrap() })
    )]
    #[case("", Err(ValueError::Format))]
    #[case("this is not a host", Err(ValueError::Format))]
    #[case(
        &format!("{}.com", "a".repeat(255)), 
        Err(ValueError::Length(259))
    )]
    fn from_value(#[case] value: &str, #[case] expected: Result<UriHost, ValueError>) {
        assert_eq!(expected, UriHost::from_value(value));
    }

    #[rstest]
    fn number() {
        assert_eq!(Number::from_value(3).unwrap(), UriHost::number())
    }
}
