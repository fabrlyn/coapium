use coapium::{
    client::url::Url,
    codec::{option::ContentFormat, MediaType},
};

#[derive(Clone, Debug)]
pub enum PayloadType {
    String,
    Octets,
    UnsignedInteger,
}

pub fn parse_payload_type(s: &str) -> Result<PayloadType, String> {
    match s.to_lowercase().as_str() {
        "string" => Ok(PayloadType::String),
        "octets" => Ok(PayloadType::Octets),
        "unsigned-integer" => Ok(PayloadType::UnsignedInteger),
        _ => Err("invalid payload type".to_owned()),
    }
}

pub fn parse_url(s: &str) -> Result<Url, String> {
    Ok(s.try_into().map_err(|e| format!("{:?}", e))?)
}

pub fn parse_content_format(s: &str) -> Result<ContentFormat, String> {
    if let Ok(content_format) = s.try_into() {
        return Ok(content_format);
    }

    let Ok(number) = s.parse::<u16>() else {
        return Err("invalid content format".to_owned());
    };

    Ok(MediaType::from_value(number).into())
}
