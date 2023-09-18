use crate::codec::{media_type, MediaType};

use super::{decoded_option::DecodedOption, number::Number, value::Value, Delta};

#[derive(Clone, Debug, PartialEq)]
pub struct ContentFormat {
    media_type: MediaType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    MediaType(media_type::Error),
}

impl ContentFormat {
    const NUMBER: u16 = 12;

    pub fn decode(values: Vec<Value>) -> Result<Self, Error> {
        Ok(Self {
            media_type: MediaType::decode(values)?,
        })
    }

    pub fn encode(self, delta_sum: Delta) -> Vec<u8> {
        if let Some(value) = self.media_type.value() {
            DecodedOption {
                number: Self::number(),
                values: vec![Value::from_u16(value)],
            }
            .encode(delta_sum)
        } else {
            vec![]
        }
    }

    pub fn number() -> Number {
        Number::from_value_or_panic(Self::NUMBER)
    }
}

impl From<media_type::Error> for Error {
    fn from(error: media_type::Error) -> Self {
        Self::MediaType(error)
    }
}

impl From<MediaType> for ContentFormat {
    fn from(media_type: MediaType) -> Self {
        Self { media_type }
    }
}

impl TryFrom<&str> for ContentFormat {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(TryInto::<MediaType>::try_into(value)?.into())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{media_type, ContentFormat, Delta, Error, MediaType, Number, Value};
    use crate::codec::media_type::{Experimental, ExpertReview, FirstComeFirstServe, IetfOrIesg};

    #[rstest]
    #[case(
        vec![], 
        Err(Error::MediaType(media_type::Error::SingleValue))
    )]
    #[case(
        vec![Value::from_opaque(vec![]).unwrap()], 
        Ok(ContentFormat { media_type: MediaType::TextPlain} )
    )]
    #[case(
        vec![Value::from_u16(MediaType::APPLICATION_JSON)], 
        Ok(ContentFormat { media_type: MediaType::ApplicationJson })
    )]
    #[case(
        vec![Value::from_u16(254u16)], 
        Ok(ContentFormat { media_type: MediaType::ExpertReview(ExpertReview::from_value(254).unwrap()) })
    )]
    #[case(
        vec![Value::from_opaque(270u16.to_be_bytes().to_vec()).unwrap()], 
        Ok(ContentFormat { media_type: MediaType::IetfOrIesg(IetfOrIesg::from_value(270).unwrap()) })
    )]
    #[case(
        vec![Value::from_opaque(10001u16.to_be_bytes().to_vec()).unwrap()], 
        Ok(ContentFormat { media_type: MediaType::FirstComeFirstServe(FirstComeFirstServe::from_value(10001).unwrap()) })
    )]
    #[case(
        vec![Value::from_opaque(65001u16.to_be_bytes().to_vec()).unwrap()], 
        Ok(ContentFormat { media_type: MediaType::Experimental(Experimental::from_value(65001).unwrap()) })
    )]
    #[case(
        vec![Value::from_opaque(vec![1, 2, 3]).unwrap()], 
        Err(Error::MediaType(media_type::Error::Number))
    )]
    #[case(
        vec![Value::from_opaque(vec![]).unwrap(), Value::from_opaque(vec![]).unwrap()], 
        Err(Error::MediaType(media_type::Error::SingleValue))
    )]
    fn decode(#[case] values: Vec<Value>, #[case] expected: Result<ContentFormat, Error>) {
        assert_eq!(expected, ContentFormat::decode(values));
    }

    #[rstest]
    #[case(ContentFormat { media_type: MediaType::ApplicationXml }, vec![0b1100_0001, 41])]
    fn encode(#[case] content_format: ContentFormat, #[case] expected: Vec<u8>) {
        assert_eq!(expected, content_format.encode(Delta::from_value(0)))
    }

    #[rstest]
    fn number() {
        assert_eq!(Number::from_value(12).unwrap(), ContentFormat::number())
    }
}
