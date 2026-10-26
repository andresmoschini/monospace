//! What the user is working on, and the chrome's own state.
//!
//! **A widget's state is here rather than in the global because it is the application's**: it is
//! built with the state and survives between frames like the rest of it. The global holds what
//! belongs to the loop, not to the application.
//!
//! **The session is read on every frame and changed only by an action.** Every frame draws it into
//! the window the interior gives; the Edit menu's three actions change it, all carried out in
//! `crate::callbacks`: **Delete** performs one removal, and **Undo** and **Redo** walk the history
//! the removal recorded. The application keeps no history of its own: the session is the history.

use monospace_core::{Buffer, GlyphCatalog, Offset, Pos, ShapeId, Size};
use monospace_editing::{Command, Session};
use rat_widget::menu::MenubarState;
use rat_widget::statusline::StatusLineState;
use ratatui_core::layout::{Position, Rect};

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

    /// The interior the last frame drew the diagram into, as the screen holds it. A click arrives
    /// as a column and a row of the screen, and this is what says whether it landed inside the
    /// window and what its offset from the window's corner is. Empty until the first frame is
    /// drawn, and empty for a screen too short to hold a region.
    interior: Rect,

    /// The region's own rectangle, border included, as the last frame drew it. **A press on the
    /// border is the region's without being the window's** (rule 2 of #189), so the outer
    /// rectangle is kept beside the interior rather than derived from it.
    region: Rect,

    /// The shape a click selected, or `None` while no cell of the last frame is inverted.
    ///
    /// **An identity rather than the cells it inverted** (D2): what is inverted is recomputed from
    /// the drawing of each frame, so a resize gives the window another size and the selection
    /// follows the shape rather than a rectangle left where it was. A selection no cell of the
    /// drawing records is kept and inverts nothing (rule 7).
    selected: Option<ShapeId>,

    /// **The session is read on every frame and changed by the Edit menu's three actions**: Delete
    /// performs its one command, Undo and Redo give a step back, and every other slice reads it.
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
            interior: Rect::new(0, 0, 0, 0),
            region: Rect::new(0, 0, 0, 0),
            selected: None,
            session,
        }
    }

    /// The glyph the last drawing put at `at`, or `None` where it put none: a position outside the
    /// window, a cell no shape wrote, or a cell this catalog does not answer.
    pub(crate) fn glyph<'a>(&'a self, at: Offset, catalog: &'a GlyphCatalog) -> Option<&'a str> {
        self.position(at)
            .and_then(|position| self.buffer.cell(position))
            .and_then(|cell| cell.glyph_str(catalog))
    }

    /// The shape a click selected, or `None` while no shape is selected.
    ///
    /// **The identity and not the cells it covers**: a resize, a scroll or a command that moves
    /// the shape changes what is inverted without changing this (D2).
    pub(crate) fn selected(&self) -> Option<ShapeId> {
        self.selected
    }

    /// Makes `selection` the shape a click selected, and says whether the screen has to be painted
    /// again.
    ///
    /// **A click that selects what is already selected changed nothing**, and neither did one on a
    /// cell nobody owns with no selection to drop: the answer is `false` and the frame is not owed.
    pub(crate) fn select(&mut self, selection: Option<ShapeId>) -> bool {
        if self.selected == selection {
            return false;
        }

        self.selected = selection;
        true
    }

    /// The shape the drawing of the last frame records at `at`, or `None` where nobody holds the
    /// cell.
    ///
    /// **The answer is `Buffer::owner`'s and its `None` is passed on whole**: the window may not
    /// hold the offset, no shape may have written the cell, or no shape may have decided it.
    ///
    /// **`Buffer::owner` is asked with the click's own offset**, counted from the interior's
    /// corner: the window's origin is the scroll origin, so nothing is added to the column and the
    /// row the terminal reports.
    pub(crate) fn owner(&self, at: Offset) -> Option<ShapeId> {
        self.buffer.owner(at)
    }

    /// Takes the shape `id` out of the session, and drops the selection.
    ///
    /// **The removal is the session's and not the diagram's** (rule 7): one `Command::Remove` is
    /// performed, which records the one step an undo gives back. The application names no `Diagram`
    /// and no `Shape`, and what hung from the removed shape is frozen by `Diagram::remove`, below
    /// this layer.
    ///
    /// **The identity is dropped with the command** (rule 2). A selection no cell of the next
    /// drawing records would invert nothing on its own, but the shape is gone, so the state stops
    /// naming it. **The undo does not bring the identity back** (D2 of #181): what comes back is
    /// the drawing, and a person who wants the figure selected clicks it again.
    pub(crate) fn remove(&mut self, id: ShapeId) {
        self.session = self.session.perform(Command::Remove { id });
        self.selected = None;
    }

    /// Gives back the step behind the session, if there is one.
    ///
    /// **The walk is the session's and the application keeps no copy of it** (§3 of the application
    /// model): one `Session::undo` moves the position, and the next frame is drawn from what the
    /// session holds then. At the oldest position nothing comes back, and the caller still owes the
    /// frame because its action was carried out.
    ///
    /// **The selection is not restored**: the removal dropped the identity, and the drawing that
    /// comes back arrives with nothing selected.
    pub(crate) fn undo(&mut self) {
        self.session = self.session.undo();
    }

    /// Gives back the step ahead of the session, if there is one.
    ///
    /// **The walk is the session's, here too**: one `Session::redo` moves the position, and the
    /// application keeps no copy of where the walk stands. At the newest position nothing comes
    /// back, and the caller still owes the frame.
    ///
    /// **The selection is not touched**: a redo takes the drawing ahead, and the identity the state
    /// holds survives the walk even where no cell of the restored drawing records it.
    pub(crate) fn redo(&mut self) {
        self.session = self.session.redo();
    }

    /// Whether a step is behind the session.
    ///
    /// **This read exists for the tests**: no frame asks it, because every entry is drawn alike at
    /// every position of the history (D2 of #181). The tests ask it to tell a walk from a press
    /// that performed none.
    #[cfg(test)]
    pub(crate) fn can_undo(&self) -> bool {
        self.session.can_undo()
    }

    /// Whether a step is ahead of the session.
    ///
    /// **This read exists for the tests**, for the same reason [`Self::can_undo`] does.
    #[cfg(test)]
    pub(crate) fn can_redo(&self) -> bool {
        self.session.can_redo()
    }

    /// Whether the shape that owns `at` of the last frame's drawing is the selected shape.
    ///
    /// **This is where the inversion is decided, and it is decided from the record** (D1): a
    /// position nobody owns is never the selection, and an identity no cell records inverts
    /// nothing because no cell asks about it (rule 7).
    pub(crate) fn is_cell_selected(&self, at: Offset) -> bool {
        matches!(
            (self.selected(), self.owner(at)),
            (Some(selected), Some(owner)) if selected == owner
        )
    }

    /// The offset into the last frame's drawing that the screen position `at` reaches, or `None`
    /// where the position is outside the interior.
    ///
    /// **The interior's corner is taken off, and the window's origin takes no part.** A click
    /// arrives as a column and a row of the screen, and the offset `Buffer::owner` asks for is
    /// counted from the corner of the window the person saw — the scroll origin belongs to the
    /// plane rather than to the screen, and adding it here would ask about another cell (rule 1).
    pub(crate) fn offset_of(&self, at: Position) -> Option<Offset> {
        if !self.interior.contains(at) {
            return None;
        }

        Some(Offset {
            x: at.x - self.interior.x,
            y: at.y - self.interior.y,
        })
    }

    /// The region's own rectangle, border included, as the last frame drew it.
    ///
    /// **The outer rectangle and not the window** (rule 2 of #189): a press on the border gives the
    /// region the keyboard without landing in the drawing, and a press outside the rectangle
    /// belongs to another leaf or to none. Empty until the first frame is drawn.
    pub(crate) fn region(&self) -> Rect {
        self.region
    }

    /// The position `at` reaches, counted from the window's corner, or `None` where the origin
    /// and the offset cannot be added.
    fn position(&self, at: Offset) -> Option<Pos> {
        Some(Pos {
            x: self.origin.x.checked_add_unsigned(at.x)?,
            y: self.origin.y.checked_add_unsigned(at.y)?,
        })
    }

    /// Draws the session's diagram into `interior` — the window, as the screen holds it — at the
    /// scroll origin, replacing whatever the frame before drew.
    ///
    /// **The window is kept whole rather than as a size**, because the two questions later frames
    /// ask of it are both about the whole: how large it is, and where its corner is on the screen.
    ///
    /// **The region is kept beside it** (rule 2 of #189): a press on the border gives the region
    /// the keyboard without landing in the window, so the outer rectangle is a fact of the frame
    /// the region answered and not one the interior can be asked for.
    pub(crate) fn redraw(&mut self, region: Rect, interior: Rect) {
        let mut buffer = Buffer::new(
            self.origin,
            Size {
                width: interior.width,
                height: interior.height,
            },
        );
        self.session.diagram().draw(&mut buffer);
        self.buffer = buffer;
        self.interior = interior;
        self.region = region;
    }
}

/// Which leaf of the chrome holds the keyboard.
///
/// **The bar and the region are the two leaves** (rule 1 of #189): the status line is not one, and
/// a control inside a leaf — a label, an entry, the dialog's button — never holds the keyboard on
/// its own. The dialog is an overlay and runs its own focus while it is up; the application's leaf
/// stays where it was, which is what makes closing the dialog give the keyboard back unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    /// The menu bar, and the drop-down it opens: the pop-up is the bar's.
    Bar,

    /// The diagram region, border included.
    Region,
}

/// Everything that survives between frames.
pub(crate) struct AppState {
    pub diagram: DiagramState,

    /// The menu bar's own state: which label is selected and whether the drop-down is open.
    pub menu: MenubarState,

    /// The status line's own state: what each of its sections says.
    pub status: StatusLineState,

    /// Which leaf holds the keyboard: the region at startup (D1 of #189).
    pub focus: Focus,
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
            focus: Focus::Region,
        }
    }

    /// The leaf a left press at `at` lands in, or `None` where it lands in neither.
    ///
    /// **The drop-down is the bar's** (rule 1 of #189): a press on an open entry gives the keyboard
    /// to the bar rather than to the region behind it, so the pop-up is asked about before the
    /// region's rectangle. A press on the status line lands in neither leaf and moves nothing.
    pub(crate) fn leaf_at(&self, at: Position) -> Option<Focus> {
        if self.menu.area.contains(at)
            || (self.menu.popup_active() && self.menu.popup.area.contains(at))
        {
            Some(Focus::Bar)
        } else if self.diagram.region().contains(at) {
            Some(Focus::Region)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU16;

    /// A description whose only shape is a light 4×3 box at `(2, 1)`.
    const A_BOX: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                      "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
    }"#;

    #[test]
    fn the_bar_and_the_status_start_inert() {
        let state = AppState::new(empty_session());

        assert!(
            !state.menu.popup_active(),
            "a drop-down is open on the first frame"
        );
        assert_eq!(state.status.status.len(), 2, "the status bar says nothing");
    }

    /// **The removal is the session's, and it records the step #181 gives back.** One command is
    /// one step: there is something behind the position and nothing ahead of it.
    #[test]
    fn a_removal_leaves_the_session_one_step_further_on() {
        let diagram =
            monospace_description::parse(A_BOX).expect("the test's description is readable");
        let mut state = AppState::new(Session::new(diagram));
        let shape = ShapeId::new(NonZeroU16::new(1).expect("1 is not zero"));

        assert!(
            !state.diagram.session.can_undo(),
            "a fresh session has a step behind it"
        );

        state.diagram.remove(shape);

        assert!(
            state.diagram.session.can_undo(),
            "the removal recorded no step"
        );
        assert!(
            !state.diagram.session.can_redo(),
            "the removal recorded more than one step"
        );
        assert!(
            state.diagram.session.diagram().get(shape).is_none(),
            "the session still holds the removed shape"
        );
        assert_eq!(
            state.diagram.selected(),
            None,
            "the removal kept the identity it took out"
        );
    }

    /// **The walk stops at both ends of the history.** An undo at the oldest position and a redo at
    /// the newest give nothing back, and the step between them is the one the removal recorded.
    #[test]
    fn undo_and_redo_stop_at_the_ends_of_the_history() {
        let diagram =
            monospace_description::parse(A_BOX).expect("the test's description is readable");
        let mut state = AppState::new(Session::new(diagram));
        let shape = ShapeId::new(NonZeroU16::new(1).expect("1 is not zero"));

        state.diagram.undo();

        assert!(
            state.diagram.session.diagram().get(shape).is_some(),
            "the undo at the oldest position took the box out"
        );

        state.diagram.remove(shape);

        assert!(
            state.diagram.session.diagram().get(shape).is_none(),
            "the removal kept the box"
        );
        assert!(!state.diagram.can_redo(), "the removal left a step ahead");

        state.diagram.undo();

        assert!(
            state.diagram.session.diagram().get(shape).is_some(),
            "the undo did not give the box back"
        );
        assert!(
            !state.diagram.can_undo(),
            "one step back left another behind it"
        );
        assert!(state.diagram.can_redo(), "the undo left nothing to redo");

        state.diagram.redo();

        assert!(
            state.diagram.session.diagram().get(shape).is_none(),
            "the redo did not take the box out again"
        );
        assert!(
            !state.diagram.can_redo(),
            "one step forward left another ahead of it"
        );
        assert!(state.diagram.can_undo(), "the redo left nothing to undo");

        state.diagram.redo();

        assert!(
            state.diagram.session.diagram().get(shape).is_none(),
            "the redo at the newest position took the box out of nothing"
        );
        assert!(
            !state.diagram.can_redo(),
            "the redo at the newest position recorded a step"
        );
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
