//! The rewrite of a persistent sequence that `rpds::VectorSync` does not have, and this crate
//! needs.
//!
//! **`rpds::VectorSync` can be written at one place and dropped at the back.** It offers a setter
//! per place, `push_back` and `drop_last`, and nothing else: there is no `swap` to exchange two
//! places, no `retain` to drop the elements matching a test, and no way of saying *keep the first
//! n*. Spelling this one out here rather than at the place that uses it is what keeps that place
//! reading as the operation it is — *what was ahead of the position is discarded* — rather than as
//! a walk whose meaning has to be recovered before it can be read.
//!
//! It takes the sequence by reference and hands a new one back, so **the sequence a session holds
//! is never the one being written.** That is the whole of what a value is, and the reason this is a
//! free function over a `VectorSync` rather than a method on [`Session`](crate::Session): the steps
//! are a field behind a borrow, and this cannot reach them.
//!
//! **There is one of these and one caller today, which is the whole argument for the module and
//! against it at once.** It is here because the same file in `monospace-diagram` holds the same kind
//! of rewrite, so a reader who has met one has met the shape and knows what to expect of the
//! documentation. It is here alone because nothing else in this crate needs it — and a helper with
//! one caller wrapped in a module is what a crate shared between the two of them would be for,
//! which is a decision about the workspace rather than about this file.

use rpds::VectorSync;

/// The sequence without anything from `len` on, or the sequence unchanged when it holds nothing
/// past it.
///
/// **The walk is over the same elements a truncation would have released**, one at a time, because
/// there is nothing else to walk them with. The places behind `len` are not copied while it runs:
/// they are the nodes the answer already shares with the sequence it was given.
///
/// **Nothing here needs a bound on `T`.** An element is never read, moved or written, and
/// `VectorSync<T>` is `Clone` for every `T` — which is what makes this usable over a type that is
/// not itself cloneable, where the two rewrites in `monospace-diagram`'s `utilities.rs` need one
/// because they exchange and filter elements rather than only dropping places. **A `Vec` truncated
/// in one step is the cheaper spelling and cannot be the answer**: it would be a copy of every step,
/// and a step is a whole diagram, so the discard of one place would cost the length of the history.
#[must_use]
pub(crate) fn kept_first<T>(steps: &VectorSync<T>, len: usize) -> VectorSync<T> {
    let mut kept = steps.clone();
    while kept.len() > len {
        kept.drop_last_mut();
    }
    kept
}

#[cfg(test)]
mod tests {
    use rpds::VectorSync;

    use super::kept_first;

    /// The sequence `[1, 2, 3, 4]`, built by the pushes a caller builds it with.
    ///
    /// **`push_back_mut` and not `push_back`**: the second hands a new sequence back rather than
    /// writing into the one it was given, which is the whole of what a persistent sequence is, and a
    /// build that ignored what it returned would stand up an empty one.
    fn sequence_of(numbers: &[u32]) -> VectorSync<u32> {
        let mut sequence: VectorSync<u32> = VectorSync::new_sync();
        for number in numbers {
            sequence.push_back_mut(*number);
        }
        sequence
    }

    /// Every element of `sequence`, front to back, so two sequences can be compared by value.
    fn elements(sequence: &VectorSync<u32>) -> Vec<u32> {
        sequence.iter().copied().collect()
    }

    /// **What was ahead of the position is discarded and what was behind it is kept, in order.**
    ///
    /// The middle rather than either end, because dropping a tail is a truncation and dropping
    /// nothing at all is the case beside it: what separates the two is a position partway along.
    #[test]
    fn kept_first_drops_the_places_behind_the_position_and_keeps_the_rest() {
        let sequence = sequence_of(&[1, 2, 3, 4]);

        let kept = kept_first(&sequence, 2);

        assert_eq!(elements(&kept), vec![1, 2]);
    }

    /// **Asking for the length there already is returns the sequence itself.** This is the edge the
    /// doc promises an answer for and the one a walk gets wrong quietly: the loop's condition is
    /// false from the start, so a rewrite written as `drop_last` on the first place rather than
    /// after the comparison would answer this wrongly and this is the only assertion that says so.
    #[test]
    fn kept_first_asks_for_the_whole_sequence_and_gets_the_whole_sequence() {
        let sequence = sequence_of(&[1, 2, 3]);

        assert_eq!(
            elements(&kept_first(&sequence, 3)),
            vec![1, 2, 3],
            "a position already at the end leaves every place standing"
        );
        assert_eq!(
            elements(&kept_first(&sequence, 9)),
            vec![1, 2, 3],
            "and a position past the end is not an error and does not drop anything more"
        );
    }

    /// **An empty sequence is an answer rather than a special case**, and a length of zero keeps
    /// nothing rather than everything.
    #[test]
    fn kept_first_on_an_empty_sequence_is_an_empty_sequence() {
        let nothing: VectorSync<u32> = VectorSync::new_sync();

        assert!(
            kept_first(&nothing, 0).is_empty(),
            "a sequence holding nothing has nothing to keep"
        );
        assert!(
            kept_first(&nothing, 4).is_empty(),
            "and a position past it is not an error"
        );

        let one = sequence_of(&[7]);
        assert!(
            kept_first(&one, 0).is_empty(),
            "a length of zero keeps nothing rather than everything"
        );
    }
}
