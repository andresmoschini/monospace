//! The application's own state, and what a left press lands on.
//!
//! **A widget's state is here rather than in the global because it is the application's**: it is
//! built with the state and survives between frames like the rest of it. The global holds what
//! belongs to the loop, not to the application.
//!
//! **The drawing and its session have their own module** ([`crate::diagram_state`]): what the last
//! frame produced and the session it was drawn from are read on every frame and changed only by an
//! action, and nothing else here touches them.

use monospace_editing::Session;
use rat_widget::msgdialog::MsgDialogState;
use rat_widget::statusline::StatusLineState;
use ratatui_core::layout::Position;

use crate::chrome::{Chrome, Target};
use crate::diagram_state::DiagramState;
use crate::status;

/// Everything that survives between frames.
pub(crate) struct AppState {
    pub diagram: DiagramState,

    /// The bar and the keyboard, which move together: the region's leaf at startup (D1 of #189).
    pub chrome: Chrome,

    /// The status line's own state: what each of its sections says.
    pub status: StatusLineState,

    /// The About dialog's own state: its message, its `Ok`, and whether it is up (D5 of #189).
    ///
    /// **The dialog runs its own focus** while it is up, and the application's leaf is left where
    /// it was, so closing the dialog gives the keyboard back unchanged.
    pub about: MsgDialogState,
}

impl AppState {
    /// The state of an application that starts over `session`.
    ///
    /// **The session is built by the caller and named here**, because a constructor that took a
    /// diagram would name the model below the editing layer and make that crate an edge of this
    /// one. The caller parses the description, hands the diagram to [`Session::new`], and this
    /// takes what comes out.
    #[must_use]
    pub(crate) fn new(diagram_session: Session) -> Self {
        Self {
            diagram: DiagramState::new(diagram_session),
            chrome: Chrome::new(),
            status: status::state(),
            about: MsgDialogState::default(),
        }
    }

    /// What a left press at `at` lands on.
    ///
    /// **The drop-down is the bar's** (rule 1 of #189): a press on an open entry gives the keyboard
    /// to the bar rather than to the region behind it, so the pop-up is asked about before the
    /// region's rectangle. Everything else is the region's own answer, taken by
    /// [`DiagramState::target_at`].
    pub(crate) fn target_at(&self, at: Position) -> Target {
        if self.chrome.menu().area.contains(at)
            || (self.chrome.popup_active() && self.chrome.menu().popup.area.contains(at))
        {
            Target::Bar
        } else {
            self.diagram.target_at(at)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bar_and_the_status_start_inert() {
        let state = AppState::new(empty_session());

        assert!(
            !state.chrome.popup_active(),
            "a drop-down is open on the first frame"
        );
        assert_eq!(
            state.status.status.len(),
            3,
            "the status bar does not have one section per row of its table"
        );
    }

    /// A session over an empty diagram. **The diagram's type is not named here** either: the
    /// session takes what `Default` gives it, and the application never names the model below
    /// the editing layer.
    fn empty_session() -> Session {
        #[allow(
            clippy::default_trait_access,
            reason = "the diagram's type is not named here: the application depends on the editing layer and not on the model below it"
        )]
        Session::new(Default::default())
    }
}
