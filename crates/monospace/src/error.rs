//! The error dialog: what the application tells a person when something they asked for failed.
//!
//! **What it says and nothing else**, as `crate::about` is: the pop-up is the application's message
//! dialog, [`crate::dialog`], and this module fills it with a title and the text of the failure.
//! **The text is the caller's**: whoever failed knows what to say, so it arrives whole and is
//! wrapped here to the dialog's inner width (`MsgDialog` scrolls a long line horizontally instead
//! of wrapping it).
//!
//! **A failure here is not an [`AppError`](crate::vocabulary::AppError)**: that one ends the loop,
//! and this one is told to the person and the session carries on.

use rat_widget::msgdialog::MsgDialogState;
use ratatui_core::text::Line;

use crate::dialog::TEXT_WIDTH;

/// The title every error is shown under.
const TITLE: &str = "Error";

/// The dialog over `message`, wrapped to [`TEXT_WIDTH`].
///
/// **Opening it replaces whatever it held**, About included: there is one overlay, and the newest
/// message is the one a person needs to read.
pub(crate) fn open(message: &str) -> MsgDialogState {
    MsgDialogState::new_active(TITLE, wrap(message, TEXT_WIDTH))
}

/// Wraps `text` on word boundaries so that no line exceeds `max_width` columns unless a single word
/// is wider than that on its own.
fn wrap(text: &str, max_width: usize) -> String {
    let mut wrapped = Vec::new();

    for paragraph in text.split('\n') {
        let mut current = String::new();
        let mut current_width = 0;

        for word in paragraph.split_whitespace() {
            let word_width = Line::from(word).width();

            if current.is_empty() {
                current.push_str(word);
                current_width = word_width;
            } else if current_width + 1 + word_width <= max_width {
                current.push(' ');
                current.push_str(word);
                current_width += 1 + word_width;
            } else {
                wrapped.push(current);
                current = word.to_string();
                current_width = word_width;
            }
        }

        wrapped.push(current);
    }

    wrapped.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::testing::{REGION, picture, theme};
    use crate::overlay::Overlay;
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;

    /// The dialog after `open`, as text.
    fn opened_over(dialog: &mut Overlay<MsgDialogState>) -> String {
        let mut buffer = Buffer::empty(REGION);
        crate::dialog::draw(REGION, &mut buffer, dialog, &theme());
        picture(&buffer)
    }

    /// **The title is the application's and the text is the caller's**, and the way out is there.
    #[test]
    fn the_error_dialog_carries_the_title_and_the_message() {
        let mut dialog = Overlay::up(open("The file could not be read"));
        let frame = opened_over(&mut dialog);

        assert!(dialog.is_up(), "opening the error left the dialog down");
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
        let mut dialog = Overlay::up(open("first failure"));
        dialog.show(open("second failure"));
        let frame = opened_over(&mut dialog);

        assert!(
            frame.contains("second failure"),
            "the second message is not shown: {frame:?}"
        );
        assert!(
            !frame.contains("first failure"),
            "the first message is still shown: {frame:?}"
        );
    }

    /// **A sentence longer than the dialog's inner width is wrapped across lines**, so every word
    /// is shown without a horizontal scroll bar.
    #[test]
    fn a_long_error_is_wrapped_across_lines_without_horizontal_scrolling() {
        let tall = Rect::new(0, 0, 40, 20);
        let mut dialog = Overlay::up(open("This is a demonstration error, raised by Ctrl+E."));

        let mut buffer = Buffer::empty(tall);
        crate::dialog::draw(tall, &mut buffer, &mut dialog, &theme());
        let frame = picture(&buffer);

        assert!(
            frame.contains("This is a demonstration error,"),
            "the first wrapped line is missing: {frame:?}"
        );
        assert!(
            frame.contains("raised by Ctrl+E."),
            "the second wrapped line is missing: {frame:?}"
        );
        assert!(
            !frame.contains('◄') && !frame.contains('►'),
            "the dialog drew a horizontal scroll bar: {frame:?}"
        );
    }
}
