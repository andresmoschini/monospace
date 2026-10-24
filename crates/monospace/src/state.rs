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
    /// The state of an application that starts over `session`.
    ///
    /// **The session is built by the caller and named here**, because a constructor that took a
    /// diagram would name the model below the editing layer and make that crate an edge of this
    /// one. The caller parses the description, hands the diagram to [`Session::new`], and this
    /// takes what comes out.
    #[must_use]
    pub(crate) fn new(session: Session) -> Self {
        Self {
            session,
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
        let state = AppState::new(empty_session());

        assert!(
            !state.menu.popup_active(),
            "a drop-down is open on the first frame"
        );
        assert_eq!(state.status.status.len(), 2, "the status bar says nothing");
    }

    /// A session over an empty diagram. **The diagram's type is not named here** either: the
    /// session takes what `Default` gives it, and the application never names the model below
    /// the editing layer.
    fn empty_session() -> Session {
        #[allow(
            clippy::default_trait_access,
            reason = "the diagram's type is not named here: the application depends on \
            the editing layer and not on the model below it"
        )]
        Session::new(Default::default())
    }
}
