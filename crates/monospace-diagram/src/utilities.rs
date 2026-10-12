//! The two rewrites of a persistent sequence that `rpds::Vector` does not have, and this crate
//! needs twice each.
//!
//! **`rpds::Vector` can be written at one place and dropped at the back.** It offers a setter per
//! place, `push_back` and `drop_last`, and nothing else: there is no `swap` to exchange two places
//! and no `retain` to drop the elements matching a test. Spelling the rewrites out here, once,
//! rather than at each of the two call sites each, is what keeps a caller reading as the single
//! operation it is — *move this one place along*, *leave nothing standing under this name*.
//!
//! Both take the sequence by reference and hand a new one back, so **the sequence a caller holds
//! is never the one being written.** That is the whole of what a value is, and the reason these
//! are free functions over a `Vector` rather than methods on [`Diagram`](crate::Diagram): the
//! order a diagram holds is a field behind a borrow, not something these can reach.

use rpds::Vector;

/// The sequence with the places `a` and `b` exchanged, or the sequence unchanged when either place
/// does not stand.
///
/// **The two places are written rather than exchanged in one step**, because `rpds`' `Vector` offers
/// a setter per place and no swap: two `set` calls, each sharing everything outside the place it
/// writes, in place of a `swap` on a copy nobody was holding. Everything the swap would have
/// carried along — the length, the other places — is shared rather than copied either way.
#[must_use]
pub(crate) fn swapped<T: Clone>(sequence: &Vector<T>, a: usize, b: usize) -> Vector<T> {
    let (Some(at_a), Some(at_b)) = (sequence.get(a).cloned(), sequence.get(b).cloned()) else {
        return sequence.clone();
    };

    sequence
        .set(a, at_b)
        .and_then(|sequence| sequence.set(b, at_a))
        .unwrap_or_else(|| sequence.clone())
}

/// The sequence without any of the elements equal to `value`, or the sequence unchanged when none
/// is.
///
/// **Every place `value` stood in goes, not the first one.** A caller here holds a sequence of
/// identities and an identity may stand in it more than once, so one element standing twice is the
/// ordinary case rather than an odd one; dropping a single place would leave the other behind,
/// standing in the sequence and naming nothing.
///
/// **The sequence is built again, because `rpds`' `Vector` keeps only the places it has and has no
/// way of dropping one it does not want.** Nothing is copied while it is written: `collect`
/// appends to a sequence nothing else points at, so each element lands where it belongs without a
/// node being cloned, and the sequence handed in is never the one written to.
#[must_use]
pub(crate) fn without<T: Clone + PartialEq>(sequence: &Vector<T>, value: &T) -> Vector<T> {
    sequence
        .iter()
        .filter(|held| *held != value)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use rpds::Vector;

    use super::{swapped, without};

    /// The sequence `[1, 2, 3, 4]`, built by the pushes a caller builds it with.
    fn sequence_of(numbers: &[u32]) -> Vector<u32> {
        numbers.iter().fold(Vector::new(), |sequence, number| {
            sequence.push_back(*number)
        })
    }

    /// Every element of `sequence`, front to back, so two sequences can be compared by value.
    fn elements(sequence: &Vector<u32>) -> Vec<u32> {
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

    /// **Every place `value` stood in goes, including a second one.**
    ///
    /// The repetition is the claim: a sequence that held `2` once would pass a rewrite that dropped
    /// the first place it found, and only the second one standing tells the two apart. What is
    /// left keeps its order, which is what the drawing reads.
    #[test]
    fn without_drops_every_place_the_value_stood_in() {
        let sequence = sequence_of(&[1, 2, 3, 2, 4]);

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
            without(&Vector::new(), &1).is_empty(),
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
