//! What the user is working on, and the chrome's own state.
//!
//! **A widget's state is here rather than in the global because it is the application's**: it is
//! built with the state and survives between frames like the rest of it. The global holds what
//! belongs to the loop, not to the application.
//!
//! **The session is held and not read.** No slice of this one performs a command on a session, so
//! there is nothing to ask it yet; what it is here for is that the state of the application is
//! the session and the chrome's own, and a state that leaves out the session is a state that
//! will have to grow one.

use monospace_editing::Session;
use rat_widget::menu::MenubarState;
use rat_widget::statusline::StatusLineState;

use crate::status;

/// Everything that survives between frames.
pub(crate) struct AppState {
    /// The session: the diagram being edited, and the steps behind and ahead of it.
    #[allow(
        dead_code,
        reason = "rule 6 makes the session part of the state and no slice performs \
        a command on it yet"
    )]
    session: Session,

    /// The menu bar's own state: which label is selected and whether the drop-down is open.
    pub menu: MenubarState,

    /// The status line's own state: what each of its sections says.
    pub status: StatusLineState,
}

impl AppState {
    /// The state of an application that starts with nothing in it.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            // The session is over an empty diagram. **The diagram's type is not named here**,
            // because the application depends on the editing layer and not on the model below
            // it: the default of the type the session asks for is the empty diagram, and naming
            // it would be a second edge in the workspace graph.
            #[allow(
                clippy::default_trait_access,
                reason = "the diagram's type is not named here: the application depends on \
                the editing layer and not on the model below it"
            )]
            session: Session::new(Default::default()),
            menu: MenubarState::default(),
            status: status::state(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bar_and_the_status_start_inert() {
        let state = AppState::new();

        assert!(
            !state.menu.popup_active(),
            "a drop-down is open on the first frame"
        );
        assert_eq!(state.status.status.len(), 2, "the status bar says nothing");
    }
}
