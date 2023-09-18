use super::{option::Value, parsing::single};

#[derive(Clone, Debug, PartialEq)]
pub struct ExpertReview(u16);

impl ExpertReview {
    pub fn from_value(value: u16) -> Result<Self, ()> {
        if value <= 255 {
            Ok(Self(value))
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IetfOrIesg(u16);

impl IetfOrIesg {
    pub fn from_value(value: u16) -> Result<Self, ()> {
        if value > 255 && value < 10000 {
            Ok(Self(value))
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FirstComeFirstServe(u16);

impl FirstComeFirstServe {
    pub fn from_value(value: u16) -> Result<Self, ()> {
        if value > 9999 && value < 65000 {
            Ok(Self(value))
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Experimental(u16);

impl Experimental {
    pub fn from_value(value: u16) -> Result<Self, ()> {
        if value < 65000 {
            Err(())
        } else {
            Ok(Self(value))
        }
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum MediaType {
    TextPlain,
    CharsetUtf8,
    ApplicationLinkFormat,
    ApplicationXml,
    ApplicationOctetStream,
    ApplicationExi,
    ApplicationJson,
    ExpertReview(ExpertReview),
    IetfOrIesg(IetfOrIesg),
    FirstComeFirstServe(FirstComeFirstServe),
    Experimental(Experimental),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    SingleValue,
    Number,
}

impl MediaType {
    pub const TEXT_PLAIN: u16 = 0;
    pub const APPLICATION_LINK_FORMAT: u16 = 40;
    pub const APPLICATION_XML: u16 = 41;
    pub const APPLICATION_OCTET_STREAM: u16 = 42;
    pub const APPLICATION_EXI: u16 = 47;
    pub const APPLICATION_JSON: u16 = 50;

    pub fn decode(values: Vec<Value>) -> Result<Self, Error> {
        let value = single(values).map_err(|_| Error::SingleValue)?;

        value
            .to_owned()
            .u16()
            .map_err(|_| Error::Number)
            .map(MediaType::from_value)
    }

    pub fn from_value(value: u16) -> Self {
        match value {
            Self::TEXT_PLAIN => Self::TextPlain,
            Self::APPLICATION_LINK_FORMAT => Self::ApplicationLinkFormat,
            Self::APPLICATION_XML => Self::ApplicationXml,
            Self::APPLICATION_OCTET_STREAM => Self::ApplicationOctetStream,
            Self::APPLICATION_EXI => Self::ApplicationExi,
            Self::APPLICATION_JSON => Self::ApplicationJson,
            0..=255 => Self::ExpertReview(ExpertReview(value)),
            256..=9999 => Self::IetfOrIesg(IetfOrIesg(value)),
            10000..=64999 => Self::FirstComeFirstServe(FirstComeFirstServe(value)),
            experimental => Self::Experimental(Experimental(experimental)),
        }
    }

    pub fn value(&self) -> Option<u16> {
        match self {
            MediaType::TextPlain => Some(Self::TEXT_PLAIN),
            MediaType::CharsetUtf8 => None,
            MediaType::ApplicationLinkFormat => Some(Self::APPLICATION_LINK_FORMAT),
            MediaType::ApplicationXml => Some(Self::APPLICATION_XML),
            MediaType::ApplicationOctetStream => Some(Self::APPLICATION_OCTET_STREAM),
            MediaType::ApplicationExi => Some(Self::APPLICATION_EXI),
            MediaType::ApplicationJson => Some(Self::APPLICATION_JSON),
            MediaType::ExpertReview(ExpertReview(value)) => Some(*value),
            MediaType::IetfOrIesg(IetfOrIesg(value)) => Some(*value),
            MediaType::FirstComeFirstServe(FirstComeFirstServe(value)) => Some(*value),
            MediaType::Experimental(Experimental(value)) => Some(*value),
        }
    }
}

impl TryFrom<&str> for MediaType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_lowercase();
        match value.as_str() {
            "text/plain;" => Ok(MediaType::TextPlain),
            "charset=utf-8" => Ok(MediaType::CharsetUtf8),
            "application/link-format" => Ok(MediaType::ApplicationLinkFormat),
            "application/xml" => Ok(MediaType::ApplicationXml),
            "application/octet-stream " => Ok(MediaType::ApplicationOctetStream),
            "application/exi" => Ok(MediaType::ApplicationExi),
            "application/json" => Ok(MediaType::ApplicationJson),
            _ => Err(()),
        }
    }
}
