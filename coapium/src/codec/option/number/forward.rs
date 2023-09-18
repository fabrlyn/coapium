use super::cache_key::CacheKey;

const MASK: u8 = 2;

const SAFE: u8 = 0;
const UNSAFE: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Forward {
    Safe(CacheKey),
    Unsafe,
}

impl Forward {
    pub const fn decode(byte: u8) -> Self {
        if byte & MASK == UNSAFE {
            Self::Unsafe
        } else {
            Self::Safe(CacheKey::decode(byte))
        }
    }

    pub const fn encode(self) -> u8 {
        match self {
            Forward::Safe(cache_key) => cache_key.encode() | SAFE,
            Forward::Unsafe => UNSAFE,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{CacheKey, Forward};

    #[rstest]
    #[case(0b111_0_0, Forward::Safe(CacheKey::NotSet))]
    #[case(0b001_0_1, Forward::Safe(CacheKey::Set(1)))]
    #[case(0b111_1_0, Forward::Unsafe)]
    #[case(0b000_1_1, Forward::Unsafe)]
    fn decode(#[case] byte: u8, #[case] expected: Forward) {
        assert_eq!(expected, Forward::decode(byte));
    }

    #[rstest]
    #[case(Forward::Safe(CacheKey::NotSet), 0b111_0_0)]
    #[case(Forward::Safe(CacheKey::Set(2)), 0b010_0_0)]
    #[case(Forward::Unsafe, 0b000_1_0)]
    fn encode(#[case] forward: Forward, #[case] expected: u8) {
        assert_eq!(expected, forward.encode())
    }
}
