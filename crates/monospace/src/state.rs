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
use crate::open::OpenDialogState;
use crate::status;

/// Everything that survives between frames.
pub(crate) struct AppState {
    diagram: DiagramState,

    /// The bar and the keyboard, which move together: the region's leaf at startup (D1 of #189).
    chrome: Chrome,

    /// The status line's own state: what each of its sections says.
    status: StatusLineState,

    /// The message dialog's own state: its message, its `Ok`, and whether it is up (D5 of #189).
    ///
    /// **One overlay, whatever it says**: About and every other message open the same dialog, so
    /// a second message replaces the first rather than stacking over it.
    ///
    /// **The dialog runs its own focus** while it is up, and the application's leaf is left where
    /// it was, so closing the dialog gives the keyboard back unchanged.
    dialog: MsgDialogState,

    /// The open dialog's own state: the path being typed, its frame, and whether it is up.
    ///
    /// **The dialog runs its own focus** while it is up, as the message dialog does, so closing it
    /// gives the keyboard back unchanged. It is the second overlay and the last thing painted.
    open: OpenDialogState,
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
            dialog: MsgDialogState::default(),
            open: OpenDialogState::default(),
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

    /// The drawing, and the session it was drawn from.
    pub(crate) fn diagram(&self) -> &DiagramState {
        &self.diagram
    }

    /// The drawing, for the region's handler and the actions it carries out.
    pub(crate) fn diagram_mut(&mut self) -> &mut DiagramState {
        &mut self.diagram
    }

    /// The bar and the keyboard, which move together.
    pub(crate) fn chrome(&self) -> &Chrome {
        &self.chrome
    }

    /// The chrome, for the handler that moves its two parts together.
    pub(crate) fn chrome_mut(&mut self) -> &mut Chrome {
        &mut self.chrome
    }

    /// The chrome and the drawing together, for the chain that asks both.
    ///
    /// **The pair is one borrow** because the region is asked with the leaf the bar left behind,
    /// and taking the two fields separately in one expression borrows the state twice.
    pub(crate) fn chrome_and_diagram(&mut self) -> (&mut Chrome, &mut DiagramState) {
        (&mut self.chrome, &mut self.diagram)
    }

    /// The status line's own state, for the tests that say what it holds.
    ///
    /// **The application only ever paints it**, so the crate's own door is [`Self::status_mut`]
    /// and this one is the test's.
    #[cfg(test)]
    pub(crate) fn status(&self) -> &StatusLineState {
        &self.status
    }

    /// The status line, for the frame that paints it.
    pub(crate) fn status_mut(&mut self) -> &mut StatusLineState {
        &mut self.status
    }

    /// The message dialog's own state, for the tests that say whether it is up.
    ///
    /// **The application opens, answers and paints the dialog**, and all three need it mutable, so
    /// this door is the test's for the same reason as [`Self::status`].
    #[cfg(test)]
    pub(crate) fn dialog(&self) -> &MsgDialogState {
        &self.dialog
    }

    /// The dialog, for the frame that paints it, the handler that answers it and the actions that
    /// open it.
    pub(crate) fn dialog_mut(&mut self) -> &mut MsgDialogState {
        &mut self.dialog
    }

    /// The open dialog's own state, for the tests that say whether it is up.
    #[cfg(test)]
    pub(crate) fn open(&self) -> &OpenDialogState {
        &self.open
    }

    /// The open dialog, for the frame that paints it, the handler that answers it and the actions
    /// that open it.
    pub(crate) fn open_mut(&mut self) -> &mut OpenDialogState {
        &mut self.open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bar_and_the_status_start_inert() {
        let state = AppState::new(empty_session());

        assert!(
            !state.chrome().popup_active(),
            "a drop-down is open on the first frame"
        );
        assert_eq!(
            state.status().status.len(),
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
