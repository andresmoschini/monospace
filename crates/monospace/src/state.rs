//! What the user is working on, and the chrome's own state.
//!
//! **A widget's state is here rather than in the global because it is the application's**: it is
//! built with the state and survives between frames like the rest of it. The global holds what
//! belongs to the loop, not to the application.
//!
//! **The session is read and never changed.** Every frame draws it into the window the interior
//! gives; no slice of this one performs a command on it, and #181 is where one will.

use monospace_core::{Buffer, GlyphCatalog, Offset, Pos, Size};
use monospace_editing::Session;
use rat_widget::menu::MenubarState;
use rat_widget::statusline::StatusLineState;

use crate::status;

/// The drawing the last frame produced, and the session it was drawn from.
pub(crate) struct DiagramState {
    /// The scroll origin: the position in the plane the interior's top-left corner reaches.
    /// `(0, 0)` until #176 moves it.
    origin: Pos,

    /// The window the last frame drew into, kept so that a click is answered from the picture
    /// the person saw rather than from a drawing made when the click arrives (D4). Empty until
    /// the first frame is drawn.
    buffer: Buffer,

    /// **Read, never changed**: this slice performs no command on the session.
    session: Session,
}

impl DiagramState {
    /// A state over `session` that has not drawn a frame yet.
    #[must_use]
    pub(crate) fn new(session: Session) -> Self {
        let origin = Pos { x: 0, y: 0 };

        Self {
            origin,
            buffer: Buffer::new(
                origin,
                Size {
                    width: 0,
                    height: 0,
                },
            ),
            session,
        }
    }

    /// The glyph the last drawing put at `at`, or `None` where it put none: a position outside the
    /// window, a cell no shape wrote, or a cell this catalog does not answer.
    pub(crate) fn glyph<'a>(&'a self, at: Offset, catalog: &'a GlyphCatalog) -> Option<&'a str> {
        self.position(at)
            .and_then(|position| self.drawing().cell(position))
            .and_then(|cell| cell.glyph_str(catalog))
    }

    /// The drawing of the last frame, which answers a click; empty before the first one.
    ///
    /// **`Buffer::owner` is asked with the click's own offset**, counted from the interior's
    /// corner: the window's origin is the scroll origin, so nothing is added to the column and the
    /// row the terminal reports.
    pub(crate) fn drawing(&self) -> &Buffer {
        &self.buffer
    }

    /// The position `at` reaches, counted from the window's corner, or `None` where the origin
    /// and the offset cannot be added.
    fn position(&self, at: Offset) -> Option<Pos> {
        Some(Pos {
            x: self.origin.x.checked_add_unsigned(at.x)?,
            y: self.origin.y.checked_add_unsigned(at.y)?,
        })
    }

    /// Draws the session's diagram into the window `size` large at the scroll origin, replacing
    /// whatever the frame before drew.
    pub(crate) fn redraw(&mut self, size: Size) {
        let mut buffer = Buffer::new(self.origin, size);
        self.session.diagram().draw(&mut buffer);
        self.buffer = buffer;
    }
}

/// Everything that survives between frames.
pub(crate) struct AppState {
    pub diagram: DiagramState,

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
    pub(crate) fn new(diagram_session: Session) -> Self {
        Self {
            diagram: DiagramState::new(diagram_session),
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
