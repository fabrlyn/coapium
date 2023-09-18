const MASK: u8 = 1;

const ELECTIVE: u8 = 0;
const CRITICAL: u8 = 1;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Class {
    Elective,
    Critical,
}

impl Class {
    pub const fn decode(byte: u8) -> Self {
        if byte & MASK == CRITICAL {
            Self::Critical
        } else {
            Self::Elective
        }
    }

    pub const fn encode(self) -> u8 {
        match self {
            Class::Elective => ELECTIVE,
            Class::Critical => CRITICAL,
        }
    }

    pub const fn is_critical(&self) -> bool {
        match self {
            Class::Critical => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::Class;

    #[rstest]
    fn decode() {
        for byte in 0..=u8::MAX {
            if byte % 2 == 0 {
                assert_eq!(Class::Elective, Class::decode(byte));
            } else {
                assert_eq!(Class::Critical, Class::decode(byte));
            }
        }
    }

    #[rstest]
    #[case(Class::Elective, 0)]
    #[case(Class::Critical, 1)]
    fn encode_decode(#[case] class: Class, #[case] byte: u8) {
        assert_eq!(byte, class.encode());
        assert_eq!(class, Class::decode(byte));
    }

    #[rstest]
    #[case(Class::Critical, true)]
    #[case(Class::Elective, false)]
    fn is_critical(#[case] class: Class, #[case] expected: bool) {
        assert_eq!(expected, class.is_critical());
    }
}
