//! A shape's identity: the ordinal a figure is drawn under and a buffer records beside the cells it
//! decided. See _Stamping_ in [`docs/model.md`](../../../docs/model.md).
//!
//! # Design notes
//!
//! **The type is an ordinal and holds no rule about a diagram.** Who issues an identity, whether two
//! may share one and what may be done with it are all questions about a diagram rather than about a
//! number, and they stay with the diagram — see _Identity_ in
//! [`docs/diagram-model.md`](../../../docs/diagram-model.md). What is left here is the one thing
//! the core has to hold: a number it can compare and write down.
//!
//! **The one rule it does hold is about the value rather than about any diagram.** Zero is not an
//! identity, so the ordinal is a [`NonZeroU32`] and [`ShapeId::new`] is the way in. That is a
//! narrower claim than the issuing and uniqueness questions above, and the distinction is what puts
//! it here: no diagram ever issued a `0`, and a diagram seeded at `1` says so.
//!
//! **Four bytes that are `Copy` rather than a string.** A buffer records one identity per stamped
//! position, and an ordinal is compared, copied and stored without an allocation; a borrowed
//! `&ShapeId` per position would put a lifetime in front of every caller to save nothing.
//!
//! **A module of its own rather than a line in `geometry`.** `Pos`, `Size`, `Direction` and
//! `Orientation` are places and amounts on the plane, and an identity is neither — putting the two
//! in one file would give the identity a neighbor it has nothing to say to. [`crate::stroke`] is
//! the precedent: one type, one module.

use std::fmt;
use std::num::NonZeroU32;

/// A shape's identity: the ordinal a figure is drawn under, and what a buffer records beside each
/// cell that figure decided.
///
/// Built from its ordinal by [`ShapeId::new`] and written back out as the bare number, `1` and `2`
/// and so on. It is `Copy`, and it carries one rule of its own: zero is not an identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShapeId(NonZeroU32);

impl ShapeId {
    /// Builds an identity from its ordinal — `1`, `2`, and so on.
    ///
    /// The [`NonZeroU32`] is the rule that `0` is not an identity, so a caller cannot ask for one:
    /// what a plain number would have to be checked for is instead a type a zero cannot have.
    #[must_use]
    pub fn new(ordinal: NonZeroU32) -> Self {
        Self(ordinal)
    }
}

impl fmt::Display for ShapeId {
    /// Writes the ordinal and nothing else — `1`, `2`, and so on.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::ShapeId;

    /// An identity of the ordinal `ordinal`, which every test below builds one with.
    fn id(ordinal: u32) -> ShapeId {
        ShapeId::new(NonZeroU32::new(ordinal).expect("no test asks for zero"))
    }

    /// `0` is not an identity, and there is no way to ask for one.
    ///
    /// **The type half is the absence itself**: `ShapeId::new` takes a [`NonZeroU32`] and there is
    /// no other constructor, so the value this test shows `0` cannot produce is the value the whole
    /// crate has no way to hold. Asserting what `NonZeroU32::new(0)` answers is therefore not a
    /// claim about the standard library — it is the observation that the door is closed, and it is
    /// the only thing about a rule this absolute that a test can observe at all.
    ///
    /// **The lowest ordinal there is writes `1`,** which is what the second half says: the first
    /// identity any diagram issues is the one after zero rather than zero itself.
    ///
    /// The reader is the other half of the name, and it lives in the crate that reads a file:
    /// `a_next_id_of_zero_is_refused_by_name_on_stderr_and_fails` and
    /// `an_identity_written_as_a_string_is_refused_by_name_on_stderr_and_fails` in
    /// `crates/monospace-cli/tests/cli.rs`, where a description is something that could have asked
    /// for a `0` and is refused.
    #[test]
    fn an_identity_cannot_be_zero_and_nothing_in_the_reader_or_the_type_allows_it_to_be_asked_for()
    {
        assert!(
            NonZeroU32::new(0).is_none(),
            "the ordinal an identity is built from cannot be zero, which is what keeps `0` \
             unrepresentable rather than merely unused"
        );
        assert_eq!(
            id(1).to_string(),
            "1",
            "the lowest ordinal there is writes itself, so a diagram issues `1` first"
        );
    }

    /// `Display` writes the ordinal alone.
    ///
    /// **The absence of the `#` is the claim**, and a table of rendered strings is what shows it:
    /// each of these is the whole of what a reader of the message sees, and none of them carries a
    /// character the type does not hold. The largest ordinal is there because it is where a `u32`
    /// stops being a short string, and the second because a multi-digit ordinal is where a
    /// hand-written `#` in a caption would have been most tempting.
    #[test]
    fn an_identity_writes_its_ordinal_and_nothing_else() {
        for (ordinal, written) in [
            (1, "1"),
            (2, "2"),
            (7, "7"),
            (10, "10"),
            (u32::MAX, "4294967295"),
        ] {
            assert_eq!(
                id(ordinal).to_string(),
                written,
                "an identity writes its ordinal and nothing else"
            );
        }
    }
}
