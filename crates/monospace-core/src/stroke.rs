//! A stroke's name.

/// The name of a stroke. A stroke has no attributes of its own: it exists only because a cell or
/// a glyph rule mentions it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stroke(pub String);

impl From<&str> for Stroke {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}
