use std::ops::Deref;

use ropey::Rope;

// TODO: ref-cast

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Document(Rope);

impl From<Rope> for Document {
    fn from(rope: Rope) -> Self {
        Self(rope)
    }
}

impl Deref for Document {
    type Target = Rope;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
