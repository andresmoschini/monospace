//! What the application calls its actions and its failures.
//!
//! **Two words, and they are not the same thing.** An *action* is what the application decides to
//! do about an event; a *failure* is what the application reports when it cannot carry on. The two
//! are kept apart because the rule the whole application rests on is that an event becomes a reply
//! and the action the reply carries is the only thing that changes the session — and a failure ends
//! the loop instead of becoming one.
//!
//! **An action need not change anything.** `Action::Delete` takes a shape out where one is
//! selected and owes a paint either way; `Action::Move` displaces what is selected and owes a
//! paint whether or not there was one; `Action::Undo` and `Action::Redo` walk the session's
//! history and owe a paint whether or not there is a step to give back; and `Action::About` opens
//! the chrome's dialog. **The two exits are not actions**: nothing about the state changes, so
//! there is nothing to carry out, and they are [`Reply::Quit`] and [`Reply::ForceQuit`].
//!
//! **A handler says an action rather than doing one**, so that an entry of the menu and the key
//! that means the same thing are decided in one place. A handler answers with a [`Reply`], which
//! keeps "not mine" apart from "mine, and nothing changed": only [`Reply::Ignored`] lets the next
//! handler be asked.
//!
//! The event itself is not here: it is what the loop polls, and it lives in `crate::event`
//! beside the two translations into it.

/// What the application reports when it cannot carry on.
pub(crate) type AppError = Box<dyn std::error::Error>;

/// What the application does about an event.
///
/// **Every action is a meaning the state must show.** `Delete` takes a shape out, `Undo` and
/// `Redo` give a step back, `Move` displaces what is selected, and `About` opens the chrome's
/// dialog. **Ending the application is not one of them**: it changes no state, so there is nothing
/// to carry out, and it is an answer of the [`Reply`]'s own instead.
///
/// **A meaning that is carried out leaves the keyboard on the region** (rule 13 of #189), and the
/// drop-down the bar was showing closes with it. **So a meaning does not say where the keyboard is
/// left** — `callbacks::apply` says it once — which is why this is a table of meanings and not a
/// table of leaves.
///
/// **No action names a leaf** (D1 of #189): the keyboard moves where the event is addressed — a
/// press by its position, the bar's own three keys inside [`crate::menu::handle`] — so there is no
/// action for reaching the bar and none for leaving it.
///
/// **A change of the screen is not a meaning of a person's, and undo will never hold one.** It is
/// what [`Reply::Repaint`] says instead; see _The action model_ in
/// [`docs/application-model.md`](../../../docs/application-model.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    /// The step behind the session is given back, and the frame is owed either way.
    ///
    /// **A press with nothing behind walks nothing**: the session answers with itself, and the
    /// action still paints, which is the silence at the oldest position (D2 of #181).
    Undo,

    /// The step ahead of the session is given back, and the frame is owed either way.
    ///
    /// **A press with nothing ahead walks nothing**, and the action still paints.
    Redo,

    /// The selected shape is taken out of the session, and the frame is owed either way.
    ///
    /// **The meaning and not the identity.** The shape is read from the state where the action is
    /// carried out, so the key is one row of the shortcut table rather than a second way of
    /// naming what a click selected. **A deletion with nothing selected removes nothing and still
    /// paints**: carrying an action out owes the frame, which is what closes the menu's entry on
    /// the screen when there was no figure to take.
    Delete,

    /// The selected shape is displaced by `(dx, dy)` cells, and the frame is owed either way.
    ///
    /// **The meaning and not the identity**, as [`Self::Delete`] is: the shape is read from the
    /// state where the action is carried out, so the four arrows are one action carrying a delta
    /// rather than four actions naming a figure each. **A move with nothing selected moves nothing
    /// and paints all the same**: what owes the frame is the action having been carried out, not
    /// the drawing having changed.
    Move(i16, i16),

    /// The About dialog is opened, and the frame is owed.
    ///
    /// **The chrome's own and no session** (D5 of #189): it opens an overlay and records no step,
    /// so the drawing and its history are untouched.
    About,
}

/// What a handler says about an event.
///
/// **The six answers are the protocol, and their order is [`Reply::merge`]'s.** `Ignored` lets the
/// next handler be asked; the other five stop the chain. `Consumed` is the event taken with no
/// frame owed, `Repaint` is the event taken with one, `Act` carries a meaning for
/// `callbacks::apply` to carry out, and `Quit` and `ForceQuit` end the loop: nothing about the
/// state changes, so there is nothing to apply and no frame to owe.
///
/// **"Not mine" and "mine, and nothing changed" are different answers**, which is what the
/// protocol this replaced could not say: `None` meant both, so a handler that had consumed an
/// event left the next one to answer it as well. **A change of the screen is not a meaning of a
/// person's**, so it is `Repaint` rather than a variant of [`Action`].
///
/// **It is `Copy`**: an answer is a word or a meaning of two numbers, so it travels by value and
/// [`merge`](Self::merge) and [`or_else`](Self::or_else) compose without borrowing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reply {
    /// Not mine: ask the next handler.
    Ignored,

    /// Mine, and nothing changed: the chain stops and no frame is owed.
    Consumed,

    /// Mine, and the frame is owed.
    Repaint,

    /// Mine, and this action is carried out before the frame is painted.
    Act(Action),

    /// The way out: the application ends and the terminal comes back.
    ///
    /// **It is not an [`Action`]**: nothing about the state changes, so there is nothing to carry
    /// out. **The way out and the forced exit are two answers and not two code paths**: the way out
    /// grows a pop-up to confirm and the forced exit does not, which is #189's, and until then they
    /// end the loop the same way.
    Quit,

    /// The forced exit: the application ends whatever the state is.
    ///
    /// **Answered before any handler**: it is the one request the chrome and the dialog may not
    /// swallow (D6 of #189), so it is `shortcuts`' first reach and [`Reply::merge`]'s highest
    /// place.
    ForceQuit,
}

impl Reply {
    /// Combines this reply with what addressing the event did.
    ///
    /// **The order is `ForceQuit > Quit > Act > Repaint > Consumed > Ignored`**, and the arms are
    /// written from the top of it: each names what it beats, and the last two answers are the only
    /// pair left for the end. **`Ignored` is the only answer that gives way**, and a handler that
    /// took the event stops the chain even where addressing it changed nothing.
    ///
    /// **The two exits beat everything**: a person asking to leave is not stopped by a meaning that
    /// would be carried out or a frame that would be owed. **Addressing a press only ever says
    /// `Ignored` or `Repaint` today**, and the order is written for all six answers so that a change
    /// to what addressing says cannot be lost here in silence.
    pub(crate) fn merge(self, addressed: Reply) -> Reply {
        match (self, addressed) {
            (Reply::Ignored, addressed) => addressed,
            (this, Reply::Ignored) => this,
            (Reply::ForceQuit, _) | (_, Reply::ForceQuit) => Reply::ForceQuit,
            (Reply::Quit, _) | (_, Reply::Quit) => Reply::Quit,
            (Reply::Act(action), _) | (_, Reply::Act(action)) => Reply::Act(action),
            (Reply::Repaint, _) | (_, Reply::Repaint) => Reply::Repaint,
            (Reply::Consumed, Reply::Consumed) => Reply::Consumed,
        }
    }

    /// Asks `next` only where this handler ignored the event.
    pub(crate) fn or_else(self, next: impl FnOnce() -> Reply) -> Reply {
        match self {
            Reply::Ignored => next(),
            this => this,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    /// The six answers, one of each.
    fn every_reply() -> [Reply; 6] {
        [
            Reply::Ignored,
            Reply::Consumed,
            Reply::Repaint,
            Reply::Act(Action::About),
            Reply::Quit,
            Reply::ForceQuit,
        ]
    }

    /// **`Ignored` is the only answer that gives way**, from either side of the merge.
    #[test]
    fn an_ignored_reply_gives_way_to_the_other_side() {
        for reply in every_reply() {
            assert_eq!(
                reply.merge(Reply::Ignored),
                reply,
                "a handler's {reply:?} lost to the press's silence"
            );
            assert_eq!(
                Reply::Ignored.merge(reply),
                reply,
                "the press's silence beat the handler's {reply:?}"
            );
        }
    }

    /// **A frame owed by either side is owed**: a silence and a repaint merge with the repaint,
    /// and two of the same answer stay that answer.
    #[test]
    fn a_frame_owed_by_either_side_is_owed() {
        assert_eq!(Reply::Consumed.merge(Reply::Repaint), Reply::Repaint);
        assert_eq!(Reply::Repaint.merge(Reply::Consumed), Reply::Repaint);
        assert_eq!(Reply::Repaint.merge(Reply::Repaint), Reply::Repaint);
        assert_eq!(Reply::Consumed.merge(Reply::Consumed), Reply::Consumed);
    }

    /// **An action outweighs every answer below it**: `Act > Repaint > Consumed > Ignored`.
    #[test]
    fn an_action_outweighs_every_answer_below_it() {
        let action = Reply::Act(Action::About);

        for reply in [Reply::Ignored, Reply::Consumed, Reply::Repaint] {
            assert_eq!(
                action.merge(reply),
                action,
                "the handler's action lost to {reply:?}"
            );
            assert_eq!(
                reply.merge(action),
                action,
                "the press's action lost to {reply:?}"
            );
        }
    }

    /// **The two exits beat everything below them**, and the forced exit beats the way out:
    /// `ForceQuit > Quit > Act > Repaint > Consumed > Ignored`. A person asking to leave is not
    /// stopped by a meaning that would be carried out or a frame that would be owed.
    #[test]
    fn the_exits_outweigh_every_other_answer() {
        for reply in [
            Reply::Ignored,
            Reply::Consumed,
            Reply::Repaint,
            Reply::Act(Action::About),
        ] {
            assert_eq!(
                Reply::Quit.merge(reply),
                Reply::Quit,
                "{reply:?} outlived the way out"
            );
            assert_eq!(
                reply.merge(Reply::Quit),
                Reply::Quit,
                "{reply:?} outlived the way out"
            );
            assert_eq!(
                Reply::ForceQuit.merge(reply),
                Reply::ForceQuit,
                "{reply:?} outlived the forced exit"
            );
            assert_eq!(
                reply.merge(Reply::ForceQuit),
                Reply::ForceQuit,
                "{reply:?} outlived the forced exit"
            );
        }

        assert_eq!(Reply::Quit.merge(Reply::ForceQuit), Reply::ForceQuit);
        assert_eq!(Reply::ForceQuit.merge(Reply::Quit), Reply::ForceQuit);
    }

    /// **The order does not depend on the side**: every pair merges to the same answer whichever
    /// way it is handed in. The one pair that could differ is two different actions, and they
    /// never meet — only a handler's answer and what addressing said are merged, and addressing
    /// names no action.
    #[test]
    fn the_order_does_not_depend_on_the_side() {
        for left in every_reply() {
            for right in every_reply() {
                assert_eq!(
                    left.merge(right),
                    right.merge(left),
                    "{left:?} and {right:?} merge by side"
                );
            }
        }
    }

    /// **Only `Ignored` asks the next handler**, so an event that was taken is never answered
    /// twice.
    #[test]
    fn only_an_ignored_reply_asks_the_next_handler() {
        for reply in every_reply() {
            let asked = Cell::new(false);
            let next = || {
                asked.set(true);
                Reply::Repaint
            };

            let answer = reply.or_else(next);

            if reply == Reply::Ignored {
                assert!(asked.get(), "an ignored event stopped the chain");
                assert_eq!(answer, Reply::Repaint);
            } else {
                assert!(!asked.get(), "{reply:?} asked the next handler");
                assert_eq!(answer, reply);
            }
        }
    }
}
