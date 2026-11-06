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
use ratatui_core::layout::Position;

use crate::chrome::{Chrome, Target};
use crate::diagram_state::DiagramState;
use crate::open::OpenDialogState;
use crate::overlay::Overlay;
use crate::save::SaveDialogState;
use crate::text::TextEditorState;

/// Everything that survives between frames.
pub(crate) struct AppState {
    diagram: DiagramState,

    /// The bar and the keyboard, which move together: the region's leaf at startup (D1 of #189).
    chrome: Chrome,

    /// The message dialog, while it is up: its message, its `Ok` and its focus (D5 of #189).
    ///
    /// **One overlay, whatever it says**: About and every other message open the same dialog, so
    /// a second message replaces the first rather than stacking over it.
    ///
    /// **The dialog runs its own focus** while it is up, and the application's leaf is left where
    /// it was, so closing the dialog gives the keyboard back unchanged.
    dialog: Overlay<MsgDialogState>,

    /// The open dialog, while it is up: the path being typed and its frame.
    ///
    /// **The dialog runs its own focus** while it is up, as the message dialog does, so closing it
    /// gives the keyboard back unchanged. It is the second overlay and the last thing painted.
    open: Overlay<OpenDialogState>,

    /// The save dialog, while it is up: the path being typed and its frame.
    ///
    /// **The dialog runs its own focus** while it is up, as the message dialog does, so closing it
    /// gives the keyboard back unchanged. It is an overlay at the frame's top layer.
    save: Overlay<SaveDialogState>,

    /// The text editor, while it is up: the box it edits, its buffer and its frame.
    ///
    /// **The dialog runs its own focus** while it is up, as the message, open and save dialogs do,
    /// so closing it gives the keyboard back unchanged. It is an overlay at the frame's top layer.
    text: Overlay<TextEditorState>,
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
            dialog: Overlay::Down,
            open: Overlay::Down,
            save: Overlay::Down,
            text: Overlay::Down,
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

    /// The message dialog, for the tests that say whether it is up.
    ///
    /// **The application opens, answers and paints the dialog**, and all three need it mutable, so
    /// this door is the test's for the same reason as [`Self::status`].
    #[cfg(test)]
    pub(crate) fn dialog(&self) -> &Overlay<MsgDialogState> {
        &self.dialog
    }

    /// The dialog, for the frame that paints it, the handler that answers it and the actions that
    /// open it.
    pub(crate) fn dialog_mut(&mut self) -> &mut Overlay<MsgDialogState> {
        &mut self.dialog
    }

    /// The open dialog, for the tests that say whether it is up.
    #[cfg(test)]
    pub(crate) fn open(&self) -> &Overlay<OpenDialogState> {
        &self.open
    }

    /// The open dialog, for the frame that paints it, the handler that answers it and the actions
    /// that open it.
    pub(crate) fn open_mut(&mut self) -> &mut Overlay<OpenDialogState> {
        &mut self.open
    }

    /// The save dialog, for the tests that say whether it is up.
    #[cfg(test)]
    pub(crate) fn save(&self) -> &Overlay<SaveDialogState> {
        &self.save
    }

    /// The save dialog, for the frame that paints it, the handler that answers it and the actions
    /// that open it.
    pub(crate) fn save_mut(&mut self) -> &mut Overlay<SaveDialogState> {
        &mut self.save
    }

    /// The text editor, for the tests that say whether it is up.
    #[cfg(test)]
    pub(crate) fn text(&self) -> &Overlay<TextEditorState> {
        &self.text
    }

    /// The text editor, for the frame that paints it, the handler that answers it and the actions
    /// that open it.
    pub(crate) fn text_mut(&mut self) -> &mut Overlay<TextEditorState> {
        &mut self.text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bar_starts_inert() {
        let state = AppState::new(empty_session());

        assert!(
            !state.chrome().popup_active(),
            "a drop-down is open on the first frame"
        );
        assert!(
            state.diagram().selected().is_none(),
            "the diagram starts with a selection"
        );
    }

    /// A session over an empty diagram.
    fn empty_session() -> Session {
        crate::empty_session()
    }
}
