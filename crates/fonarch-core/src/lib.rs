//! Headless GATHER core: find Adobe livetype, name fonts from
//! `.c/entitlements.xml` child tags, copy real OpenType to a dated Desktop folder.

mod catalog;
mod error;
mod gather;
mod hunt;
mod names;
mod opentype;

pub use error::{Error, Result};
pub use gather::{archive, gather, GatherEvent, GatherOptions, GatherReport};
pub use hunt::{find_livetype, is_livetype, known_livetype_path};
