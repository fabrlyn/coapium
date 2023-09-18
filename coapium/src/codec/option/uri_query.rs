use super::{
    decoded_option::DecodedOption,
    number::Number,
    value::{self, Value},
    Delta,
};

#[derive(Clone, Debug, PartialEq)]
pub struct UriQuery {
    // Values here are stored in a encoded format, so when reading from this as a user we need to decode from url-encoding first.
    // This is in order to allow a user to do both key=value and also just non-key-value things like: '==3=='
    queries: Vec<Value>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Length(usize),
    String,
    Value(value::Error),
}

impl UriQuery {
    const MAX_LENGTH: usize = 255;
    const NUMBER: u16 = 15;

    fn add<S: AsRef<str>>(&mut self, value: S) -> Result<(), Error> {
        let value = Value::from_str(value.as_ref())?;

        if value.len() > Self::MAX_LENGTH {
            return Err(Error::Length(value.len()));
        }

        self.queries.push(value);

        Ok(())
    }

    pub fn add_key_value<S: AsRef<str>>(&mut self, key: S, value: S) -> Result<(), Error> {
        let value = format!(
            "{}={}",
            urlencoding::encode(key.as_ref()),
            urlencoding::encode(value.as_ref())
        );
        self.add(value)
    }

    pub fn add_value<S: AsRef<str>>(&mut self, value: S) -> Result<(), Error> {
        self.add(urlencoding::encode(value.as_ref()))
    }

    pub fn decode(values: Vec<Value>) -> Result<Self, Error> {
        values
            .into_iter()
            .map(Self::decode_value)
            .collect::<Result<_, _>>()
            .map(|values| Self { queries: values })
    }

    fn decode_value(value: Value) -> Result<Value, Error> {
        if !value.valid_as_string() {
            return Err(Error::String);
        }

        if value.len() > Self::MAX_LENGTH {
            return Err(Error::Length(value.len()));
        }

        Ok(value)
    }

    pub fn encode(self, delta_sum: Delta) -> Vec<u8> {
        DecodedOption {
            number: Self::number(),
            values: self.queries,
        }
        .encode(delta_sum)
    }

    pub fn new() -> Self {
        Self { queries: vec![] }
    }

    pub fn number() -> Number {
        Number::from_value_or_panic(Self::NUMBER)
    }
}

impl TryFrom<&str> for UriQuery {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut uri_query = Self::new();

        uri_query.add_value(value)?;

        Ok(uri_query)
    }
}

impl TryFrom<(&str, &str)> for UriQuery {
    type Error = Error;

    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        let mut uri_query = Self::new();

        uri_query.add_key_value(value.0, value.1)?;

        Ok(uri_query)
    }
}

impl From<value::Error> for Error {
    fn from(error: value::Error) -> Self {
        Self::Value(error)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{value, Delta, Error, Number, UriQuery, Value};

    #[rstest]
    #[case("", "", Ok(()), Some(vec![Value::from_str("=").unwrap()]))]
    #[case("a", "", Ok(()), Some(vec![Value::from_str("a=").unwrap()]))]
    #[case("a", "b", Ok(()), Some(vec![Value::from_str("a=b").unwrap()]))]
    #[case("a", " ", Ok(()), Some(vec![Value::from_str("a=%20").unwrap()]))]
    #[case(" ", "b", Ok(()), Some(vec![Value::from_str("%20=b").unwrap()]))]
    #[case("a", &"b".repeat(256), Err(Error::Length(258)), None)]
    #[case("a", &"b".repeat((u16::MAX as usize) + 1), Err(Error::Value(value::Error::Value(value::ValueError::LengthOutOfBounds))), None)]
    fn add_key_value(
        #[case] key: &str,
        #[case] value: &str,
        #[case] expected: Result<(), Error>,
        #[case] queries: Option<Vec<Value>>,
    ) {
        let mut uri_query = UriQuery::new();

        let actual = uri_query.add_key_value(key, value);

        assert_eq!(expected, actual);
        if let Ok(()) = actual {
            assert_eq!(queries, Some(uri_query.queries));
        }
    }

    #[rstest]
    #[case("", Ok(()), Some(vec![Value::Empty]))]
    #[case("a", Ok(()), Some(vec![Value::from_str("a").unwrap()]))]
    #[case(" ", Ok(()), Some(vec![Value::from_str("%20").unwrap()]))]
    #[case(&"a".repeat(256), Err(Error::Length(256)), None)]
    #[case(&"a".repeat((u16::MAX as usize) + 1), Err(Error::Value(value::Error::Value(value::ValueError::LengthOutOfBounds))), None)]
    fn add_value(
        #[case] value: &str,
        #[case] expected: Result<(), Error>,
        #[case] queries: Option<Vec<Value>>,
    ) {
        let mut uri_query = UriQuery::new();

        let actual = uri_query.add_value(value);

        assert_eq!(expected, actual);
        if let Ok(()) = actual {
            assert_eq!(queries, Some(uri_query.queries));
        }
    }

    #[rstest]
    #[case(vec![], Ok(UriQuery{queries: vec![]}))]
    #[case(vec![Value::from_str("").unwrap()], Ok(UriQuery{queries: vec![Value::from_str("").unwrap()]}))]
    #[case(vec![Value::from_str("abc").unwrap()], Ok(UriQuery{queries: vec![Value::from_str("abc").unwrap()]}))]
    #[case(vec![Value::from_str("foo=bar").unwrap(), Value::from_str("abc=def").unwrap()], Ok(UriQuery{queries: vec![Value::from_str("foo=bar").unwrap(), Value::from_str("abc=def").unwrap()]}))]
    #[case(vec![Value::from_string("a".repeat(256)).unwrap()], Err(Error::Length(256)))]
    #[case(vec![Value::from_opaque(vec![0xE0, 0x80, 0x80]).unwrap()], Err(Error::String))]
    fn decode(#[case] values: Vec<Value>, #[case] expected: Result<UriQuery, Error>) {
        assert_eq!(expected, UriQuery::decode(values));
    }

    #[rstest]
    #[case(UriQuery { queries: vec![] }, vec![0b1101_0000, 2])]
    #[case(UriQuery { queries: vec![Value::Empty] }, vec![0b1101_0000, 2])]
    #[case(UriQuery { queries: vec![Value::from_opaque(vec!['a' as u8]).unwrap()] }, vec![0b1101_0001, 2, 97])]
    #[case(UriQuery { queries: vec![Value::from_opaque(vec!['a' as u8, 'b' as u8]).unwrap()] }, vec![0b1101_0010, 2, 97, 98])]
    fn encode(#[case] uri_query: UriQuery, #[case] expected: Vec<u8>) {
        assert_eq!(expected, uri_query.encode(Delta::from_value(0)))
    }

    #[rstest]
    fn new() {
        let actual = UriQuery::new();
        assert_eq!(UriQuery { queries: vec![] }, actual)
    }

    #[rstest]
    fn number() {
        assert_eq!(Number::from_value(15).unwrap(), UriQuery::number())
    }
}
