//! Whether an overlay is up, and what it holds while it is.
//!
//! **An overlay is a value in one of two states**: down, where it leaves the keyboard and the
//! screen to the layer below it, or up, holding the state of the widget it draws. The widget's
//! state exists only while the overlay is up, so "down with a stale buffer" and "down naming a
//! box" are not values of this type.
//!
//! **Every overlay is held this way** — the message dialog, the open dialog, the save dialog and
//! the text editor — so [`crate::state`] has one shape for all four instead of a flag in each, and
//! a handler that closes an overlay drops what it held by construction.
//!
//! **It is not an `Option`**: an overlay is down, not missing, and the type says so with the
//! screen's own words. `show` and `close` also name the transition, where a `Some` and a `None`
//! would leave the handler to spell out what putting a value there means.

/// An overlay's state: down, or up over the state of the widget inside it.
#[derive(Debug)]
pub(crate) enum Overlay<S> {
    /// Nothing is up: the layer below holds the keyboard and the screen.
    Down,

    /// The overlay is up and holds the state of the widget it draws.
    Up(S),
}

impl<S> Overlay<S> {
    /// The overlay that is up over `state`.
    #[must_use]
    pub(crate) fn up(state: S) -> Self {
        Self::Up(state)
    }

    /// Whether the overlay is up, for the tests that say which one is.
    #[cfg(test)]
    pub(crate) fn is_up(&self) -> bool {
        self.get().is_some()
    }

    /// The state inside, while the overlay is up, for the tests that read it.
    #[cfg(test)]
    pub(crate) fn get(&self) -> Option<&S> {
        match self {
            Self::Down => None,
            Self::Up(state) => Some(state),
        }
    }

    /// The state inside, for the handler that answers or paints it while it is up.
    pub(crate) fn get_mut(&mut self) -> Option<&mut S> {
        match self {
            Self::Down => None,
            Self::Up(state) => Some(state),
        }
    }

    /// Puts the overlay up over `state`, replacing whatever it held.
    ///
    /// **Opening an overlay is replacing it**, as the message dialog already did: two overlays
    /// never stack, and the newest one is the one a person is answering.
    pub(crate) fn show(&mut self, state: S) {
        *self = Self::up(state);
    }

    /// Brings the overlay down and drops what it held.
    pub(crate) fn close(&mut self) {
        *self = Self::Down;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_overlay_starts_down() {
        let overlay: Overlay<u8> = Overlay::Down;

        assert!(!overlay.is_up(), "the overlay started up");
        assert_eq!(overlay.get(), None);
    }

    #[test]
    fn show_puts_it_up_over_the_state() {
        let mut overlay = Overlay::Down;
        overlay.show(7);

        assert!(overlay.is_up(), "the overlay stayed down");
        assert_eq!(overlay.get(), Some(&7));
    }

    #[test]
    fn show_replaces_what_it_held() {
        let mut overlay = Overlay::up(7);
        overlay.show(9);

        assert_eq!(
            overlay.get(),
            Some(&9),
            "the show did not replace the state"
        );
    }

    #[test]
    fn close_drops_what_it_held() {
        let mut overlay = Overlay::up(7);
        overlay.close();

        assert!(!overlay.is_up(), "the overlay stayed up");
        assert_eq!(overlay.get(), None);
    }

    #[test]
    fn the_state_is_only_there_while_it_is_up() {
        let mut overlay = Overlay::up(7);

        assert!(overlay.get_mut().is_some(), "the state is not reachable");

        overlay.close();

        assert!(
            overlay.get_mut().is_none(),
            "a closed overlay handed its state out"
        );
    }
}
