//! The About dialog: the first pop-up, and the first control that is not a menu entry.
//!
//! **What it says and nothing else.** The pop-up is the application's message dialog,
//! [`crate::dialog`], which owns its size, its colors, its focus and its answers; this module fills
//! it with the application's own name and version (D5 of #189).

use rat_widget::msgdialog::MsgDialogState;

use crate::status;

/// Opens the dialog over the application's own name and version.
///
/// **The message is built from `status`**, where the name and the version already live: the status
/// line and the dialog cannot disagree about what the application is. Opening it a second time
/// replaces the message rather than appending to it.
pub(crate) fn open(state: &mut MsgDialogState) {
    state.clear();
    state.title(format!("About {}", status::NAME));
    state.append(&format!("{} {}", status::NAME, status::VERSION));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::testing::{REGION, picture, theme};
    use ratatui_core::buffer::Buffer;

    /// **The title and the body come from the application's own name and version** (D5), so the
    /// dialog and the status line cannot say two different things.
    #[test]
    fn the_about_dialog_carries_the_name_and_the_version() {
        let mut state = MsgDialogState::default();
        open(&mut state);

        let mut buffer = Buffer::empty(REGION);
        crate::dialog::draw(REGION, &mut buffer, &mut state, &theme());
        let frame = picture(&buffer);

        assert!(
            frame.contains(&format!("About {}", status::NAME)),
            "the dialog has no title: {frame:?}"
        );
        assert!(
            frame.contains(&format!("{} {}", status::NAME, status::VERSION)),
            "the dialog does not carry the name and the version: {frame:?}"
        );
        assert!(frame.contains("Ok"), "the dialog has no way out: {frame:?}");
    }
}
