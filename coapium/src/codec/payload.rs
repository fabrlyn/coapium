const MARKER: u8 = 0xff;

#[derive(Clone, Debug, PartialEq)]
pub struct Payload {
    value: Option<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Format,
}

impl Payload {
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        match (bytes.first(), bytes.len()) {
            (None, _) => Ok(Self { value: None }),
            (Some(&MARKER), 1) => Err(Error::Format),
            (Some(&MARKER), _) => Ok(Self {
                value: Some(bytes[1..].to_vec()),
            }),
            _ => Err(Error::Format),
        }
    }

    pub fn empty() -> Self {
        Self { value: None }
    }

    pub fn encode(self) -> Vec<u8> {
        match self.value {
            None => vec![],
            Some(bytes) => [MARKER].into_iter().chain(bytes).collect(),
        }
    }

    pub fn from_value(value: Vec<u8>) -> Self {
        if value.is_empty() {
            Self { value: None }
        } else {
            Self { value: Some(value) }
        }
    }

    pub fn value(&self) -> &[u8] {
        match &self.value {
            None => &[],
            Some(bytes) => &bytes,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_none()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{Error, Payload, MARKER};

    #[rstest]
    #[case(&[], &[], Payload{ value: None })]
    #[case(&[0xff, 1], &[1], Payload{ value: Some(vec![1]) })]
    fn decode_value_encode(
        #[case] bytes: &[u8],
        #[case] expected_value: &[u8],
        #[case] expected_decoded: Payload,
    ) {
        let decoded = Payload::decode(&bytes).unwrap();
        let value = decoded.value();
        let encoded = decoded.clone().encode();

        assert_eq!(bytes, encoded);
        assert_eq!(expected_value, value);
        assert_eq!(expected_decoded, decoded);
    }

    #[rstest]
    fn decode_invalid_marker() {
        for marker in 0..=u8::MAX {
            if marker == MARKER {
                continue;
            }

            assert_eq!(Err(Error::Format), Payload::decode(&[marker]));
        }
    }

    #[rstest]
    fn decode_only_marker() {
        assert_eq!(Err(Error::Format), Payload::decode(&[0xff]));
    }

    #[rstest]
    fn empty() {
        assert_eq!(Payload { value: None }, Payload::decode(&[]).unwrap());
    }

    #[rstest]
    #[case(vec![], Payload{ value: None })]
    #[case(vec![1, 2, 3], Payload{ value: Some(vec![1, 2, 3]) })]
    fn from_value(#[case] value: Vec<u8>, #[case] expected: Payload) {
        assert_eq!(expected, Payload::from_value(value));
    }

    #[rstest]
    fn is_empty() {
        assert!(Payload::empty().is_empty())
    }
}
