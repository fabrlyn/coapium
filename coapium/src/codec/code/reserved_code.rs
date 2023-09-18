use super::{Class, Detail};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReservedCode {
    class: Class,
    detail: Detail,
}

impl ReservedCode {
    pub const fn encode(self) -> (Class, Detail) {
        (self.class, self.detail)
    }

    pub const fn new(class: Class, detail: Detail) -> Self {
        Self { class, detail }
    }
}
