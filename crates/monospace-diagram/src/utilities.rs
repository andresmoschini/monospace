//! The two rewrites of a persistent sequence that `rpds::VectorSync` does not have, and this crate
//! needs twice each.
//!
//! **`rpds::VectorSync` can be written at one place and dropped at the back.** It offers a setter per
//! place, `push_back` and `drop_last`, and nothing else: there is no `swap` to exchange two places
//! and no way of dropping the one place a caller names. Spelling the rewrites out here, once,
//! rather than at each of the two call sites each, is what keeps a caller reading as the single
//! operation it is — *move this one place along*, *leave nothing standing under this name*.
//!
//! Both take the sequence by reference and hand a new one back, so **the sequence a caller holds
//! is never the one being written.** That is the whole of what a value is, and the reason these
//! are free functions over a `VectorSync` rather than methods on [`Diagram`](crate::Diagram): the
//! order a diagram holds is a field behind a borrow, not something these can reach.

use rpds::VectorSync;

/// The sequence with the places `a` and `b` exchanged, or the sequence unchanged when either place
/// does not stand.
///
/// **The two places are written rather than exchanged in one step**, because `rpds`' sequence offers
/// a setter per place and no swap: two `set` calls, each sharing everything outside the place it
/// writes, in place of a `swap` on a copy nobody was holding. Everything the swap would have
/// carried along — the length, the other places — is shared rather than copied either way.
#[must_use]
pub(crate) fn swapped<T: Clone>(sequence: &VectorSync<T>, a: usize, b: usize) -> VectorSync<T> {
    let (Some(at_a), Some(at_b)) = (sequence.get(a).cloned(), sequence.get(b).cloned()) else {
        return sequence.clone();
    };

    sequence
        .set(a, at_b)
        .and_then(|sequence| sequence.set(b, at_a))
        .unwrap_or_else(|| sequence.clone())
}

/// The sequence without the element equal to `value`, or the sequence unchanged when it holds no
/// such element.
///
/// **The sequence is built again, because `rpds`' sequence keeps only the places it has and has no
/// way of dropping one it does not want.** Nothing is copied while it is written: `collect` appends
/// to a sequence nothing else points at, so each element lands where it belongs without a node being
/// cloned, and the sequence handed in is never the one written to.
///
/// **One place goes rather than every place it stood in, because one element standing twice is not
/// a case this crate's order has.** §3 _Identity_ says an identity names one figure and one place
/// in the order, and `remove` is the only method that writes the order apart from two additions
/// that put a name in exactly one place. The earlier wording here said the opposite — that an
/// identity may stand in the order more than once and dropping one place would leave the other
/// standing and naming nothing — which was true of a diagram that held two figures under one name
/// and is not true of one that does not.
#[must_use]
pub(crate) fn without<T: Clone + PartialEq>(sequence: &VectorSync<T>, value: &T) -> VectorSync<T> {
    sequence
        .iter()
        .filter(|held| *held != value)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use rpds::VectorSync;

    use super::{swapped, without};

    /// The sequence `[1, 2, 3, 4]`, built by the pushes a caller builds it with.
    fn sequence_of(numbers: &[u16]) -> VectorSync<u16> {
        numbers
            .iter()
            .fold(VectorSync::new_sync(), |sequence, number| {
                sequence.push_back(*number)
            })
    }

    /// Every element of `sequence`, front to back, so two sequences can be compared by value.
    fn elements(sequence: &VectorSync<u16>) -> Vec<u16> {
        sequence.iter().copied().collect()
    }

    /// The two places are exchanged and **every other place stands where it stood.**
    ///
    /// The pair `(0, 3)` rather than two that touch, because exchanging two places that stand
    /// next to each other is the case where a wrong implementation and a right one are hardest to
    /// tell apart — and an exchange of the first and last places also shows that neither end is
    /// special.
    #[test]
    fn swapping_two_places_exchanges_them_and_leaves_every_other_place_alone() {
        let sequence = sequence_of(&[1, 2, 3, 4]);

        let swapped = swapped(&sequence, 0, 3);

        assert_eq!(elements(&swapped), vec![4, 2, 3, 1]);
    }

    /// **The sequence handed in is the sequence it was, and not the one written.** The claim is the
    /// one a persistent structure exists to make, and it is the one a swap written in place would
    /// break while still passing every other test here.
    ///
    /// Both results are compared, because a rewrite that corrupted its own input would fail on the
    /// second half of this alone.
    #[test]
    fn swapping_does_not_disturb_the_sequence_it_was_given() {
        let sequence = sequence_of(&[1, 2, 3, 4]);

        let swapped = swapped(&sequence, 0, 3);

        assert_eq!(
            elements(&sequence),
            vec![1, 2, 3, 4],
            "the sequence it was given"
        );
        assert_eq!(
            elements(&swapped),
            vec![4, 2, 3, 1],
            "the sequence it handed back"
        );
    }

    /// A place that does not stand leaves the sequence as it was, and the two answers are the same
    /// whether **one** place is missing or **both** are.
    ///
    /// Both are checked because a rewrite that reads one place and stops would answer the first
    /// correctly and the second wrongly, and the second is the one a caller hits when the sequence
    /// is shorter than it thought.
    #[test]
    fn swapping_places_that_do_not_stand_changes_nothing() {
        let sequence = sequence_of(&[1, 2, 3]);

        assert_eq!(
            elements(&swapped(&sequence, 1, 7)),
            vec![1, 2, 3],
            "one place missing"
        );
        assert_eq!(
            elements(&swapped(&sequence, 7, 8)),
            vec![1, 2, 3],
            "both missing"
        );
    }

    /// **The place `value` stood in goes and every other place keeps its order**, which is what the
    /// drawing reads.
    ///
    /// The middle of the sequence rather than an end, because an element that was dropped from an
    /// end is a truncation and would pass a rewrite that kept the wrong places. The last place is
    /// left out on purpose: a sequence of identities is a set in an order, so what is here is one
    /// identity standing in it, not one standing twice.
    #[test]
    fn without_drops_the_place_the_value_stood_in() {
        let sequence = sequence_of(&[1, 2, 3, 4]);

        let left = without(&sequence, &2);

        assert_eq!(elements(&left), vec![1, 3, 4]);
    }

    /// A value the sequence never held leaves it exactly as it was, and **an empty sequence is an
    /// answer rather than a special case.**
    #[test]
    fn without_a_value_the_sequence_never_held_changes_nothing() {
        let sequence = sequence_of(&[1, 2, 3]);

        assert_eq!(elements(&without(&sequence, &9)), vec![1, 2, 3]);
        assert!(
            without(&VectorSync::new_sync(), &1).is_empty(),
            "a sequence holding nothing has nothing to drop"
        );
    }

    /// Dropping the only value there is leaves a sequence holding nothing, which is a sequence and
    /// not an absence of one.
    #[test]
    fn without_the_only_element_leaves_an_empty_sequence_rather_than_nothing() {
        let sequence = sequence_of(&[1]);

        let left = without(&sequence, &1);

        assert!(left.is_empty());
        assert_eq!(left.len(), 0);
    }
}
