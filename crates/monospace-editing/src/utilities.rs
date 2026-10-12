//! The rewrite of a persistent sequence that `rpds::Vector` does not have, and this crate needs.
//!
//! **`rpds::Vector` can be written at one place and dropped at the back.** It offers a setter per
//! place, `push_back` and `drop_last`, and nothing else: there is no `swap` to exchange two places,
//! no `retain` to drop the elements matching a test, and no way of saying *keep the first n*.
//! Spelling this one out here rather than at the place that uses it is what keeps that place reading
//! as the operation it is — *what was ahead of the position is discarded* — rather than as a walk
//! whose meaning has to be recovered before it can be read.
//!
//! It takes the sequence by reference and hands a new one back, so **the sequence a session holds is
//! never the one being written.** That is the whole of what a value is, and the reason this is a
//! free function over a `Vector` rather than a method on [`Session`](crate::Session): the steps are
//! a field behind a borrow, and this cannot reach them.
//!
//! **There is one of these and one caller today, which is the whole argument for the module and
//! against it at once.** It is here because the same file in `monospace-diagram` holds the same kind
//! of rewrite, so a reader who has met one has met the shape and knows what to expect of the
//! documentation. It is here alone because nothing else in this crate needs it — and a helper with
//! one caller wrapped in a module is what a crate shared between the two of them would be for,
//! which is a decision about the workspace rather than about this file.

use rpds::Vector;

/// The sequence without anything from `len` on, or the sequence unchanged when it holds nothing
/// past it.
///
/// **The walk is over the same elements a truncation would have released**, one at a time, because
/// there is nothing else to walk them with. The places behind `len` are not copied while it runs:
/// they are the nodes the answer already shares with the sequence it was given.
///
/// **Nothing here needs a bound on `T`.** An element is never read, moved or written, and
/// `Vector<T>` is `Clone` for every `T` — which is what makes this usable over a type that is not
/// itself cloneable, where the two rewrites in `monospace-diagram`'s `utilities.rs` need one because
/// they exchange and filter elements rather than only dropping places.
#[must_use]
pub(crate) fn kept_first<T>(steps: &Vector<T>, len: usize) -> Vector<T> {
    let mut kept = steps.clone();
    while kept.len() > len {
        kept.drop_last_mut();
    }
    kept
}
