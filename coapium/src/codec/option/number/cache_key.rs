const MASK: u8 = 0b00011100;
const SHIFT: u8 = 2;

const NOT_SET: u8 = 0b00011100;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CacheKey {
    NotSet,
    Set(u8),
}

impl CacheKey {
    pub const fn decode(byte: u8) -> Self {
        let cache_key = byte & MASK;

        if cache_key == NOT_SET {
            Self::NotSet
        } else {
            Self::Set(cache_key >> SHIFT)
        }
    }

    pub const fn encode(self) -> u8 {
        match self {
            CacheKey::NotSet => NOT_SET,
            CacheKey::Set(cache_key) => cache_key << SHIFT,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::CacheKey;

    #[rstest]
    #[case(0b0011100, CacheKey::NotSet)]
    #[case(0b0000000, CacheKey::Set(0))]
    #[case(0b0000100, CacheKey::Set(1))]
    #[case(0b0001000, CacheKey::Set(2))]
    #[case(0b0001100, CacheKey::Set(3))]
    #[case(0b0010000, CacheKey::Set(4))]
    #[case(0b0010100, CacheKey::Set(5))]
    #[case(0b0011000, CacheKey::Set(6))]
    fn decode_encode(#[case] byte: u8, #[case] expected: CacheKey) {
        assert_eq!(expected, CacheKey::decode(byte));
        assert_eq!(byte, CacheKey::decode(byte).encode());
    }
}
