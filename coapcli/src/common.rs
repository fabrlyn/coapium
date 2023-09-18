use coapium::{
    client::url::Url,
    codec::{option::ContentFormat, MediaType},
};

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
