//! The error dialog: what the application tells a person when something they asked for failed.
//!
//! **What it says and nothing else**, as `crate::about` is: the pop-up is the application's message
//! dialog, [`crate::dialog`], and this module fills it with a title and the text of the failure.
//! **The text is the caller's**: whoever failed knows what to say, so it arrives whole.
//!
//! **A failure here is not an [`AppError`](crate::vocabulary::AppError)**: that one ends the loop,
//! and this one is told to the person and the session carries on.

use rat_widget::msgdialog::MsgDialogState;

/// The title every error is shown under.
const TITLE: &str = "Error";

/// Opens the dialog over `message`.
///
/// **Opening it replaces whatever it held**, About included: there is one overlay, and the newest
/// message is the one a person needs to read.
pub(crate) fn open(state: &mut MsgDialogState, message: &str) {
    state.clear();
    state.title(TITLE);
    state.append(message);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::testing::{REGION, picture, theme};
    use ratatui_core::buffer::Buffer;

    /// The dialog after `open`, as text.
    fn opened_over(state: &mut MsgDialogState, message: &str) -> String {
        open(state, message);

        let mut buffer = Buffer::empty(REGION);
        crate::dialog::draw(REGION, &mut buffer, state, &theme());
        picture(&buffer)
    }

    /// **The title is the application's and the text is the caller's**, and the way out is there.
    #[test]
    fn the_error_dialog_carries_the_title_and_the_message() {
        let mut state = MsgDialogState::default();
        let frame = opened_over(&mut state, "The file could not be read");

        assert!(state.active(), "opening the error left the dialog down");
        assert!(frame.contains(TITLE), "the dialog has no title: {frame:?}");
        assert!(
            frame.contains("The file could not be read"),
            "the dialog does not carry the message: {frame:?}"
        );
        assert!(frame.contains("Ok"), "the dialog has no way out: {frame:?}");
    }

    /// **A second message replaces the first** rather than being appended under it.
    #[test]
    fn a_second_error_replaces_the_first() {
        let mut state = MsgDialogState::default();
        let _ = opened_over(&mut state, "first failure");
        let frame = opened_over(&mut state, "second failure");

        assert!(
            frame.contains("second failure"),
            "the second message is not shown: {frame:?}"
        );
        assert!(
            !frame.contains("first failure"),
            "the first message is still shown: {frame:?}"
        );
    }
}
