use failure::Fail;

use crate::state_tracker::StructureError;

#[derive(Debug, Fail, Ord, PartialOrd, Eq, PartialEq)]
#[fail(display = "encoding failed: {}", _0)]
pub struct Error(StructureError);

impl From<StructureError> for Error {
    fn from(error: StructureError) -> Self {
        Self(error)
    }
}

impl Into<StructureError> for Error {
    fn into(self) -> StructureError {
        self.0
    }
}
