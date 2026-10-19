//! What the user is working on, and the widgets' state.
//!
//! The widgets' state is here and not in the global because it has to be built from nothing when
//! the focus moves, and the global does not.

use monospace_diagram::Diagram;
use monospace_editing::Session;

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
}

impl AppState {
    #[must_use]
    pub fn new(initial_diagram: Diagram) -> Self {
        Self {
            session: Session::new(initial_diagram),
            counter: 0,
        }
    }
}
