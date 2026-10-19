//! What the user is working on, and the widgets' state.
//!
//! A widget's state is here rather than in the global because it is the application's: it is
//! built with the state and survives between frames like the rest of it. The global holds what
//! belongs to the loop, not to the application.

use monospace_diagram::Diagram;
use monospace_editing::Session;
use rat_widget::{menu::MenubarState, msgdialog::MsgDialogState};

/// Everything that survives between frames.
pub(crate) struct AppState {
    #[allow(
        dead_code,
        reason = "rule 8 makes the session part of the state and no slice performs \
        a command on it yet"
    )]
    session: Session,

    /// The demonstration counter, replaced when the state becomes the diagram.
    pub counter: usize,

    pub menu: MenubarState,

    pub about: MsgDialogState,
}

impl AppState {
    #[must_use]
    pub fn new(initial_diagram: Diagram) -> Self {
        Self {
            session: Session::new(initial_diagram),
            counter: 0,
            menu: MenubarState::default(),
            about: MsgDialogState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monospace_diagram::Diagram;

    #[test]
    fn the_bar_and_the_modal_start_inert() {
        let state = AppState::new(Diagram::new());

        assert!(
            !state.menu.popup_active(),
            "a drop-down is open on the first frame"
        );
    }
}
