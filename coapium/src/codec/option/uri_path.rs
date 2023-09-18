use url::Url;

use super::{decoded_option::DecodedOption, number::Number, value::Value, Delta};

#[derive(Clone, Debug, PartialEq)]
pub struct UriPath {
    segments: Vec<Value>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Format,
    Length(usize),
}

impl UriPath {
    const MAX_LENGTH: usize = 255;
    const NUMBER: u16 = 11;

    pub fn decode(encoded_options: Vec<Value>) -> Result<Self, Error> {
        let segments = encoded_options
            .into_iter()
            .map(|v| v.string())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Error::Format)?;

        Self::from_value(segments.join("/"))
    }

    pub fn encode(self, delta_sum: Delta) -> Vec<u8> {
        DecodedOption {
            number: Self::number(),
            values: self.segments,
        }
        .encode(delta_sum)
    }

    pub fn from_value<S: AsRef<str>>(value: S) -> Result<Self, Error> {
        let value = value.as_ref();

        let url = if value.starts_with("/") {
            format!("coap://127.0.0.1{}", value)
        } else {
            format!("coap://127.0.0.1/{}", value)
        };

        let url = Url::parse(&url).map_err(|_| Error::Format)?;

        if url.fragment().is_some() {
            return Err(Error::Format);
        }

        if url.query().is_some() {
            return Err(Error::Format);
        }

        let Some(path_segments) = url.path_segments() else {
            return Ok(UriPath { segments: vec![] });
        };

        let segments = path_segments
            .into_iter()
            .map(to_value)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .enumerate()
            .filter(is_tail_segment)
            .map(|(_, value)| value)
            .collect::<Vec<_>>();

        Ok(UriPath { segments })
    }

    pub fn number() -> Number {
        Number::from_value_or_panic(Self::NUMBER)
    }
}

fn is_tail_segment(element: &(usize, Value)) -> bool {
    match element {
        (0, Value::Empty) => false,
        _ => true,
    }
}

fn to_value(path_segment: &str) -> Result<Value, Error> {
    if path_segment.len() > UriPath::MAX_LENGTH {
        Err(Error::Length(path_segment.len()))
    } else {
        Value::from_str(path_segment).map_err(|_| Error::Format)
    }
}

impl TryFrom<&str> for UriPath {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_value(value)
    }
}

impl TryFrom<String> for UriPath {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_value(&value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{Delta, Error, Number, UriPath, Value};

    #[rstest]
    #[case(vec![], Ok(UriPath { segments: vec![] } ))]
    #[case(vec![Value::Empty], Ok(UriPath { segments: vec![] } ))]
    #[case(vec![Value::from_str("").unwrap()], Ok(UriPath { segments: vec![] } ))]
    #[case(vec![Value::from_str("a").unwrap()], Ok(UriPath { segments: vec![Value::from_str("a").unwrap()] } ))]
    #[case(vec![Value::from_str(&format!("{}", "a".repeat(256))).unwrap()], Err(Error::Length(256)))]
    fn decode(#[case] values: Vec<Value>, #[case] expected: Result<UriPath, Error>) {
        assert_eq!(expected, UriPath::decode(values))
    }

    #[rstest]
    #[case(
        UriPath { segments: vec![] },
        vec![0b1011_0000]
    )]
    #[case(
        UriPath { segments: vec![Value::Empty] }, 
        vec![0b1011_0000])]
    #[case(
        UriPath { segments: vec![Value::Empty, Value::Empty] }, 
        vec![0b1011_0000]
    )]
    #[case(
        UriPath { segments: vec![Value::Empty, Value::from_str("a").unwrap(), Value::Empty, Value::Empty] }, 
        vec![0b1011_0001, 'a' as u8]
    )]
    #[case(
        UriPath { segments: vec![Value::Empty, Value::from_str("a").unwrap(), Value::Empty, Value::Empty, Value::from_str("b").unwrap()] }, 
        vec![0b1011_0001, 'a' as u8, 0b0000_0001, 'b' as u8]
    )]
    fn encode(#[case] uri_path: UriPath, #[case] expected: Vec<u8>) {
        assert_eq!(expected, uri_path.encode(Delta::from_value(0)))
    }

    #[rstest]
    #[case("", Ok(UriPath { segments: vec![] } ))]
    #[case("/", Ok(UriPath { segments: vec![] } ))]
    #[case("//", Ok(UriPath { segments: vec![Value::Empty] } ))]
    #[case("a/b", Ok(UriPath { segments: vec![Value::from_str("a").unwrap(), Value::from_str("b").unwrap()] } ))]
    #[case("/abc", Ok(UriPath { segments: vec![Value::from_str("abc").unwrap()] } ))]
    #[case("/abc/", Ok(UriPath { segments: vec![Value::from_str("abc").unwrap(), Value::Empty] } ))]
    #[case("/a/b/", Ok(UriPath { segments: vec![Value::from_str("a").unwrap(), Value::from_str("b").unwrap(), Value::Empty] } ))]
    #[case("a/#ac", Err(Error::Format))]
    #[case("a/?b=c", Err(Error::Format))]
    #[case(&format!("/a/{}", "c".repeat(256)),  Err(Error::Length(256)))]
    fn from_value(#[case] value: &str, #[case] expected: Result<UriPath, Error>) {
        assert_eq!(expected, UriPath::from_value(value))
    }

    #[rstest]
    fn number() {
        assert_eq!(Number::from_value(11).unwrap(), UriPath::number())
    }
}
