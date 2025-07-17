//! State tracking for decoding and encoding

mod state;
mod structure_error;
mod token;

pub use self::token::Token;
pub(crate) use self::{state::StateTracker, structure_error::StructureError};
