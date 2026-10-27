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

use monospace_core::{
    Buffer, CellReader, Footprint, FootprintSink, GlyphCatalog, Offset, OwnerReader, Pos, ShapeId,
    Size, Window,
};
use monospace_editing::{Command, Session};
use rat_widget::event::ScrollOutcome;
use rat_widget::menu::MenubarState;
use rat_widget::msgdialog::MsgDialogState;
use rat_widget::scrolled::{ScrollAreaState, ScrollState};
use rat_widget::statusline::StatusLineState;
use ratatui_core::layout::{Position, Rect};

use crate::status;

/// The drawing the last frame produced, and the session it was drawn from.
pub(crate) struct DiagramState {
    /// The scroll origin: the position in the plane the interior's top-left corner reaches.
    ///
    /// **What survives a frame** (D1): each axis's scroll offset is derived from it and the content
    /// rather than kept beside it, so a content change that moves the drawing's first cell does not
    /// carry the view with it — a person's place is a place in the plane. It starts at `(0, 0)`,
    /// which is the first screen, and every frame puts it at the nearest value the content allows.
    origin: Pos,

    /// The window the last frame drew into, kept so that a click is answered from the picture
    /// the person saw rather than from a drawing made when the click arrives (D4). Empty until
    /// the first frame is drawn.
    buffer: Buffer,

    /// The footprint the last frame's drawing left, measured on its way into `buffer`.
    ///
    /// **Measured while the stamps pass rather than asked of the cells afterwards**: a stamp the
    /// window discarded and a stamp the composition found already decided both count, and the
    /// buffer cannot answer for either. `None` until the first frame is drawn.
    footprint: Option<Footprint>,

    /// The vertical bar's own state: where its thumb stands, and how far the content lets it travel.
    ///
    /// **Kept beside the origin and derived from it** (D1): `redraw` writes the page, the reach and
    /// the offset of every frame, so a content change moves the thumb with the drawing rather than
    /// leaving it where the frame before it stood. The state is the helper's because the bar that
    /// draws it and the press it answers are the helper's too.
    vscroll: ScrollState,

    /// The horizontal bar's own state, derived from the origin as [`Self::vscroll`] is.
    hscroll: ScrollState,

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
            buffer: Buffer::new(Window {
                origin,
                size: Size {
                    width: 0,
                    height: 0,
                },
            }),
            footprint: None,
            vscroll: ScrollState::new(),
            hscroll: ScrollState::new(),
            interior: Rect::new(0, 0, 0, 0),
            region: Rect::new(0, 0, 0, 0),
            selected: None,
            session,
        }
    }

    /// The glyph the last drawing put at `at`, or `None` where it put none: a position outside the
    /// window, a cell no shape wrote, or a cell this catalog does not answer.
    ///
    /// **The click's offset is the window's to translate**: `CellReader::cell` takes a position,
    /// and the window turns the column and the row the terminal reports into one.
    pub(crate) fn glyph<'a>(&'a self, at: Offset, catalog: &'a GlyphCatalog) -> Option<&'a str> {
        self.buffer
            .window()
            .position(at)
            .and_then(|position| self.buffer.cell(position))
            .and_then(|cell| cell.glyph_str(catalog))
    }

    /// The footprint the last frame's drawing reaches, or `None` before the first [`redraw`].
    ///
    /// **The drawing's and not the window's**: the sink measures every stamp before the buffer
    /// consults its window, so a diagram larger than the interior reaches past it and one drawn at
    /// a scroll origin away from `(0, 0)` is measured where it stands. `min` is the top-left corner
    /// in the plane, which is what bounds the origin; the click paths above stay in offsets and are
    /// unaffected.
    ///
    /// **The last frame's, measured when that frame was drawn**: it is re-measured by [`redraw`],
    /// and it changes when the session does rather than when the screen does. Its reader is the
    /// status bar, which is why it is read once the region is drawn and not while the state is
    /// built.
    ///
    /// [`redraw`]: Self::redraw
    pub(crate) fn footprint(&self) -> Option<Footprint> {
        self.footprint
    }

    /// The scroll origin the last frame drew at.
    ///
    /// **This read exists for the tests**: no frame asks where the origin is, because the frame
    /// draws at it and the click counts its offset from the window's corner rather than from the
    /// plane.
    #[cfg(test)]
    pub(crate) fn origin(&self) -> Pos {
        self.origin
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
    /// **The answer is `OwnerReader::owner`'s and its `None` is passed on whole**: the window may
    /// not hold the offset, no shape may have written the cell, or no shape may have decided it.
    ///
    /// **The window is asked first and the record second**: `OwnerReader::owner` takes a position,
    /// and the click arrives as an offset from the interior's corner, so `Window::position` is what
    /// turns one into the other. The window's origin is the scroll origin, so nothing is added to
    /// the column and the row the terminal reports.
    pub(crate) fn owner(&self, at: Offset) -> Option<ShapeId> {
        self.buffer
            .window()
            .position(at)
            .and_then(|position| self.buffer.owner(position))
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
        self.perform(Command::Remove { id });
        self.selected = None;
    }

    /// Displaces the shape `id` by `(dx, dy)` cells and keeps the selection.
    ///
    /// **The movement is a replacement like any other** (§9 of the diagram model): one
    /// `Command::Move` is performed, and the diagram below decides what a displacement does to a
    /// reference hanging from the figure and to the gap of one the figure holds. The application
    /// names no `Delta`: the pair is converted where the command is written, so this crate does not
    /// reach below the editing layer for one value.
    ///
    /// **The identity survives**, unlike the removal's: the figure is still there, one cell along,
    /// so the next frame inverts the cells it now owns.
    pub(crate) fn move_shape(&mut self, id: ShapeId, dx: i16, dy: i16) {
        self.perform(Command::Move {
            id,
            by: (dx, dy).into(),
        });
    }

    /// Performs `command` on the session, recording its one step.
    ///
    /// **The session is the only thing performed on, and the application keeps no copy of it.**
    /// Every command a slice adds goes through here, so the layer below stays the one that decides
    /// what a step is.
    fn perform(&mut self, command: Command) {
        self.session = self.session.perform(command);
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
    /// arrives as a column and a row of the screen, and the offset the window turns into the
    /// position `OwnerReader::owner` asks about is counted from the corner of the window the person
    /// saw — the scroll origin belongs to the plane rather than to the screen, and adding it here
    /// would ask about another cell (rule 1).
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

    /// The two bars' own state over `area`, for the widget that draws them and the one that reads a
    /// press on them.
    ///
    /// **The region is the area, because the bars sit on its border** (D4): the helper lays each of
    /// them on the block's own row or column, between the corners, so what it answers a press
    /// against is the rectangle the region drew rather than the window.
    pub(crate) fn scroll_area(&mut self, area: Rect) -> ScrollAreaState<'_> {
        ScrollAreaState::new()
            .area(area)
            .v_scroll(&mut self.vscroll)
            .h_scroll(&mut self.hscroll)
    }

    /// The same two states, for the widget that draws the bars into the border's run.
    ///
    /// **The area each bar is drawn into is the area a press is answered against**: the helper writes
    /// it as it draws, so a press is read against the bar the last frame put on the screen.
    pub(crate) fn scroll_states(&mut self) -> (&mut ScrollState, &mut ScrollState) {
        (&mut self.hscroll, &mut self.vscroll)
    }

    /// Applies what the bars answered, and says whether the view moved (D3).
    ///
    /// **The answer is the helper's, step and all**: a wheel turn is the delta it names and a press
    /// on the bar is the offset it names, so the step is a tenth of the window rather than a rule of
    /// this application's.
    ///
    /// **The origin is derived from the offset and clamped** (D1): `set_offset` puts the value it is
    /// given there without limiting it, and a drag that leaves the bar's track names an offset past
    /// the content's span, so the offset names an origin and the range decides where that lands.
    ///
    /// **A frame is owed only when the origin moved**: a turn the content refuses, and a press where
    /// the view already stands, changed nothing.
    pub(crate) fn scrolled(&mut self, answer: ScrollOutcome) -> bool {
        let changed = match answer {
            ScrollOutcome::Up(n) => self.vscroll.scroll_up(n),
            ScrollOutcome::Down(n) => self.vscroll.scroll_down(n),
            ScrollOutcome::Left(n) => self.hscroll.scroll_left(n),
            ScrollOutcome::Right(n) => self.hscroll.scroll_right(n),
            ScrollOutcome::VPos(at) => self.vscroll.set_offset(at),
            ScrollOutcome::HPos(at) => self.hscroll.set_offset(at),
            // **`Changed` is the helper's own repaint signal and the application does not take it**:
            // it carries no offset, and what this application owes a frame for is an origin that
            // moved (D3), so there is nothing here to apply. Neither `ScrollAreaState` nor
            // `ScrollState` answers it today — a step, an offset or nothing at all is all they say —
            // and the assertion is what makes a helper that starts saying it a failing test rather
            // than a frame lost in silence.
            ScrollOutcome::Changed => {
                debug_assert!(false, "the bars answered with a repaint and no offset");
                false
            }
            ScrollOutcome::Unchanged | ScrollOutcome::Continue => false,
        };

        if !changed {
            return false;
        }

        let (across, down) = axes(self.measure(), self.interior);
        let origin = Pos {
            x: across.at(self.hscroll.offset()),
            y: down.at(self.vscroll.offset()),
        };
        let moved = origin != self.origin;
        self.origin = origin;

        moved
    }

    /// Draws the session's diagram into `interior` — the window, as the screen holds it — at the
    /// scroll origin, replacing whatever the frame before drew.
    ///
    /// **The window is kept whole rather than as a size**, because the two questions later frames
    /// ask of it are both about the whole: how large it is, and where its corner is on the screen.
    ///
    /// **The origin is put where the content allows before anything is drawn** (D1): the measurement
    /// is [taken first](Self::measure) because the window a frame draws through depends on the
    /// drawing rather than the other way round, and a content change that no longer holds the origin
    /// is clamped by the frame that follows it rather than by the scroll that follows that.
    ///
    /// **The region is kept beside it** (rule 2 of #189): a press on the border gives the region
    /// the keyboard without landing in the window, so the outer rectangle is a fact of the frame
    /// the region answered and not one the interior can be asked for.
    pub(crate) fn redraw(&mut self, region: Rect, interior: Rect) {
        let (across, down) = axes(self.measure(), interior);
        self.origin = Pos {
            x: across.clamp(self.origin.x),
            y: down.clamp(self.origin.y),
        };
        across.place(&mut self.hscroll, self.origin.x);
        down.place(&mut self.vscroll, self.origin.y);

        let mut buffer = Buffer::new(Window {
            origin: self.origin,
            size: Size {
                width: interior.width,
                height: interior.height,
            },
        });
        let footprint = {
            let mut sink = FootprintSink::new(&mut buffer);
            self.session.diagram().draw(&mut sink);
            sink.footprint()
        };
        self.buffer = buffer;
        self.footprint = footprint;
        self.interior = interior;
        self.region = region;
    }

    /// How far the session's diagram reaches, measured on a drawing that keeps nothing (D2).
    ///
    /// **The window is empty, so no cell of the traversal survives it**: what the answer holds is
    /// the stamps themselves, which is what lets the origin be chosen before the frame is drawn.
    /// What it costs is a second traversal of the shapes per frame, and the first one writes
    /// nothing.
    fn measure(&self) -> Option<Footprint> {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 0,
                height: 0,
            },
        });
        let mut sink = FootprintSink::new(&mut buffer);
        self.session.diagram().draw(&mut sink);

        sink.footprint()
    }
}

/// One axis of the content the view may show: the first and last cell it reaches in the plane, and
/// the window it is shown through (D1).
///
/// **The content is the drawing's footprint union the first screen** — the window at `(0, 0)`,
/// which is where the view opens. So the content reaches left of or above the origin only where the
/// drawing does, and its last cell is never before the first screen's: a drawing away from `(0, 0)`
/// still opens on the first screen, and one that stamped nothing leaves the content as that screen,
/// which does not scroll.
#[derive(Debug, Clone, Copy)]
struct Axis {
    /// The content's first cell in the plane.
    first: i32,

    /// The content's last cell in the plane.
    last: i32,

    /// The interior's size along this axis.
    window: i32,
}

impl Axis {
    /// The content `footprint` and the first screen make on one axis, `reach` being the footprint's
    /// first and last cell there and `None` while nothing has been drawn.
    ///
    /// **A window of no size is no screen, and so no content**: the first screen's last cell is the
    /// window's own minus one, which is before `(0, 0)` when there is no window, and that is what
    /// leaves the span at zero rather than at the one cell the corner alone would make.
    fn of(reach: Option<(i16, i16)>, window: u16) -> Self {
        let window = i32::from(window);
        let (first, last) = match reach {
            Some((first, last)) => (i32::from(first), i32::from(last)),
            None => (0, window - 1),
        };

        Self {
            first: first.min(0),
            last: last.max(window - 1),
            window,
        }
    }

    /// The first and last origin D1 allows, both included.
    ///
    /// **Computed in `i32`**, because `last - window + 1` leaves an `i16` at the plane's own corner,
    /// and the `max` is what keeps a content smaller than the window still opening at `(0, 0)`.
    fn origins(&self) -> (i32, i32) {
        (self.first, (self.last - self.window + 1).max(0))
    }

    /// `origin` put at the nearest value of that range, which is what every frame does with it.
    fn clamp(&self, origin: i16) -> i16 {
        let (first, last) = self.origins();
        let at = i32::from(origin).clamp(first, last);

        i16::try_from(at).expect("a cell of the content fits the plane it was measured in")
    }

    /// How far past the window the content reaches, which is how far the view can travel.
    fn span(&self) -> usize {
        usize::try_from((self.last - self.first + 1 - self.window).max(0))
            .expect("a count of cells is a length")
    }

    /// Where `origin` stands, counted from the content's first cell.
    fn offset(&self, origin: i16) -> usize {
        usize::try_from(i32::from(origin) - self.first)
            .expect("the origin lies in the content, so it is never before it")
    }

    /// Writes `origin` and the content into a bar's own state, which is what the bar draws and what
    /// a press on it is answered against.
    fn place(&self, state: &mut ScrollState, origin: i16) {
        state.set_page_len(usize::try_from(self.window).expect("a window's size is a length"));
        state.set_max_offset(self.span());
        state.set_offset(self.offset(origin));
    }

    /// The origin the bar's `offset` names, put at the nearest value the range allows.
    ///
    /// **The bar's answer is not clamped before it arrives**: the helper maps the track's cells over
    /// the content's reach — `pos = row - base - 1` over `span = length - 2` — so a press on the
    /// track's last cell names the span itself, which is the furthest offset there is, and only a
    /// drag that carries the pointer past the track names more: nothing stops it at the area the
    /// drag began in. `set_offset` writes what it is given without limiting it, so the offset names
    /// an origin and the origin's range is what says where a view of nothing can still stand.
    fn at(&self, offset: usize) -> i16 {
        let at = self
            .first
            .saturating_add(i32::try_from(offset).unwrap_or(i32::MAX));
        let (first, last) = self.origins();
        let at = at.clamp(first, last);

        i16::try_from(at).expect("a cell of the content fits the plane it was measured in")
    }
}

/// The two axes of the content, from a footprint and the window it is shown through.
fn axes(footprint: Option<Footprint>, interior: Rect) -> (Axis, Axis) {
    (
        Axis::of(footprint.map(|it| (it.min.x, it.max.x)), interior.width),
        Axis::of(footprint.map(|it| (it.min.y, it.max.y)), interior.height),
    )
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
            menu: MenubarState::default(),
            status: status::state(),
            focus: Focus::Region,
            about: MsgDialogState::default(),
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

    /// Moves the keyboard to the leaf at `at`, returning whether the focus changed.
    ///
    /// **A position in neither leaf leaves the focus alone**: the status line and empty space
    /// are not destinations for the keyboard. The leaf is resolved by [`Self::leaf_at`], so an
    /// open drop-down belongs to the bar even where it overlaps the region.
    pub(crate) fn set_focus_at(&mut self, at: Position) -> bool {
        let Some(leaf) = self.leaf_at(at) else {
            return false;
        };

        let moved = self.focus != leaf;
        self.focus = leaf;
        moved
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

    /// The description #176's examples argue D1 with: a light `4×3` box at `(0, 0)`, a double `2×2`
    /// box at `(10, 1)` and a seven-cell line at `(2, 4)`, so the drawing reaches `(11, 4)`.
    ///
    /// **A window of `5×3` is the interior of a small region**, which is the size the examples use:
    /// the drawing is larger than it on both axes, so the origin has room to move and the far end of
    /// that room is `(7, 2)`.
    const THE_EXAMPLE: &str = r#"{
        "next_id": 4,
        "shapes": [
            { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
              "size": { "width": 4, "height": 3 }, "stroke": "light" },
            { "kind": "box", "id": 2, "at": { "x": 10, "y": 1 },
              "size": { "width": 2, "height": 2 }, "stroke": "double" },
            { "kind": "line", "id": 3, "at": { "x": 2, "y": 4 },
              "len": 7, "orientation": "horizontal", "stroke": "light" }
        ]
    }"#;

    /// The region the example is shown through, and its interior: `5×3` at `(1, 1)` of the screen.
    const EXAMPLE_REGION: Rect = Rect::new(0, 0, 7, 5);
    const EXAMPLE_INTERIOR: Rect = Rect::new(1, 1, 5, 3);

    /// The state of an application over the diagram `json` describes.
    fn a_state_over(json: &str) -> AppState {
        let diagram =
            monospace_description::parse(json).expect("the test's description is readable");

        AppState::new(Session::new(diagram))
    }

    /// The glyph the last frame drew at `(x, y)` of its window, or `" "` where it drew none.
    fn at(state: &DiagramState, x: u16, y: u16) -> String {
        let catalog = monospace_glyph_sets::all();

        state
            .glyph(Offset { x, y }, &catalog)
            .unwrap_or(" ")
            .to_string()
    }

    /// Every glyph of the last frame's window, row by row.
    fn window_of(state: &DiagramState) -> Vec<String> {
        (0..EXAMPLE_INTERIOR.height)
            .map(|y| {
                (0..EXAMPLE_INTERIOR.width)
                    .map(|x| at(state, x, y))
                    .collect()
            })
            .collect()
    }

    #[test]
    fn the_bar_and_the_status_start_inert() {
        let state = AppState::new(empty_session());

        assert!(
            !state.menu.popup_active(),
            "a drop-down is open on the first frame"
        );
        assert_eq!(
            state.status.status.len(),
            3,
            "the status bar does not have one section per row of its table"
        );
    }

    /// **A state that has not drawn a frame has measured nothing**, and the first `redraw` is what
    /// measures the drawing: the box of `A_BOX` reaches `(5, 3)` from `(2, 1)`.
    #[test]
    fn a_state_that_has_not_drawn_a_frame_has_no_footprint() {
        let diagram =
            monospace_description::parse(A_BOX).expect("the test's description is readable");
        let mut state = AppState::new(Session::new(diagram));

        assert_eq!(
            state.diagram.footprint(),
            None,
            "a state that has drawn no frame reached somewhere"
        );

        state
            .diagram
            .redraw(Rect::new(0, 0, 20, 10), Rect::new(0, 0, 20, 10));

        assert_eq!(
            state.diagram.footprint(),
            Some(Footprint {
                min: Pos { x: 2, y: 1 },
                max: Pos { x: 5, y: 3 },
            }),
            "the first frame did not measure the drawing"
        );
    }

    /// The footprint of the last frame is the drawing's and not the interior's: the box of
    /// `A_BOX` stands at `(2, 1)` and is four by three, so it reaches `(5, 3)` whether the interior
    /// holds it or not.
    ///
    /// **The two interiors are the case**: one holds a single cell of the box and the other holds
    /// all of it, and the answer does not move. A record taken from the cells would give `(2, 1)`
    /// in the first, because the one cell the window holds is the box's corner.
    #[test]
    fn the_last_frames_footprint_is_the_diagrams() {
        let diagram =
            monospace_description::parse(A_BOX).expect("the test's description is readable");
        let mut state = AppState::new(Session::new(diagram));

        state
            .diagram
            .redraw(Rect::new(0, 0, 3, 2), Rect::new(0, 0, 1, 1));

        assert_eq!(
            state.diagram.footprint(),
            Some(Footprint {
                min: Pos { x: 2, y: 1 },
                max: Pos { x: 5, y: 3 },
            }),
            "the interior decided how far the drawing reaches"
        );

        state
            .diagram
            .redraw(Rect::new(0, 0, 20, 10), Rect::new(0, 0, 20, 10));

        assert_eq!(
            state.diagram.footprint(),
            Some(Footprint {
                min: Pos { x: 2, y: 1 },
                max: Pos { x: 5, y: 3 },
            }),
            "a larger interior changed the diagram's footprint"
        );
    }

    /// **The view opens at `(0, 0)`, whatever the drawing reaches** (rule 1). The example's content
    /// runs from `0` to `11` across and from `0` to `4` down, so the origin may lie between `0` and
    /// `7` and between `0` and `2` — and the first frame puts it at the near end of both.
    #[test]
    fn a_frame_opens_at_the_origin() {
        let mut state = a_state_over(THE_EXAMPLE);

        state.diagram.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);

        assert_eq!(
            state.diagram.origin(),
            Pos { x: 0, y: 0 },
            "the first frame did not open on the first screen"
        );
        assert_eq!(
            at(&state.diagram, 0, 0),
            "┌",
            "the drawing's own corner is not in the window"
        );
    }

    /// **A drawing inside the first screen does not scroll** (rule 1). The content is the union of
    /// the drawing and the window at `(0, 0)`, so a box that fits leaves neither axis any room to
    /// travel and the frame opens where the drawing is.
    #[test]
    fn a_drawing_inside_the_first_screen_does_not_scroll() {
        let mut state = a_state_over(A_BOX);

        state
            .diagram
            .redraw(Rect::new(0, 0, 20, 10), Rect::new(1, 1, 18, 6));

        assert_eq!(
            state.diagram.origin(),
            Pos { x: 0, y: 0 },
            "a drawing inside the first screen opened somewhere else"
        );
        assert_eq!(
            state.diagram.hscroll.max_offset(),
            0,
            "the content is wider than the window it fits"
        );
        assert_eq!(
            state.diagram.vscroll.max_offset(),
            0,
            "the content is taller than the window it fits"
        );
    }

    /// **A window of no size has nothing to scroll** (D1). The content is the first screen, and a
    /// screen of no cells is no content at all: the region of a terminal too small to hold one draws
    /// no border and answers no click, so a bar claiming a reach of one there would be a bar that
    /// cannot be drawn and cannot be moved.
    #[test]
    fn a_window_of_no_size_has_nothing_to_scroll() {
        assert_eq!(
            Axis::of(None, 0).span(),
            0,
            "a window of no size claims a cell the view can travel"
        );
        assert_eq!(
            Axis::of(None, 18).span(),
            0,
            "a screen with nothing drawn claims a cell to travel"
        );
    }

    /// **The frame draws at the origin the content allows** (rule 2). At `(7, 2)`, the last value
    /// D1 allows for a `5×3` window, the window's right edge is the drawing's and its bottom edge is
    /// the drawing's: the double box's last row and the line's last two cells are all that is left.
    #[test]
    fn a_frame_draws_at_the_origin_the_content_allows() {
        let mut state = a_state_over(THE_EXAMPLE);
        state.diagram.origin = Pos { x: 7, y: 2 };

        state.diagram.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);

        assert_eq!(
            state.diagram.origin(),
            Pos { x: 7, y: 2 },
            "the frame moved an origin the content holds"
        );
        assert_eq!(
            window_of(&state.diagram).join("\n"),
            "   ╚╝\n     \n──   ",
            "the window does not hold what the drawing reaches at that origin"
        );
    }

    /// **A frame at the end of the content owes no frame** (rule 2). An origin the content does not
    /// hold is put at the nearest value it does by the frame that draws it, and the frame after it
    /// draws the same picture: the clamp is not a change the next frame has to be owed for.
    #[test]
    fn a_frame_at_the_end_of_the_content_owes_no_frame() {
        let mut state = a_state_over(THE_EXAMPLE);
        state.diagram.origin = Pos { x: 20, y: 20 };

        state.diagram.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);
        let clamped = window_of(&state.diagram);

        assert_eq!(
            state.diagram.origin(),
            Pos { x: 7, y: 2 },
            "the frame did not put the origin at the furthest the content allows"
        );

        state.diagram.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);

        assert_eq!(
            state.diagram.origin(),
            Pos { x: 7, y: 2 },
            "the frame that followed moved the view"
        );
        assert_eq!(
            window_of(&state.diagram),
            clamped,
            "the frame that followed drew another picture"
        );
    }

    /// **The origin is what survives a frame, and not the offset** (rule 2). The finding box is
    /// moved two cells further out and the content still holds the origin, so the view stays where
    /// the person left it. The figures away from the origin are then taken out, and the origin the
    /// content no longer holds is put at the nearest value it does.
    #[test]
    fn a_content_change_keeps_the_origin() {
        let mut state = a_state_over(THE_EXAMPLE);
        let finding = ShapeId::new(NonZeroU16::new(2).expect("2 is not zero"));
        let line = ShapeId::new(NonZeroU16::new(3).expect("3 is not zero"));
        state.diagram.origin = Pos { x: 7, y: 2 };
        state.diagram.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);

        state.diagram.move_shape(finding, 2, 0);
        state.diagram.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);

        assert_eq!(
            state.diagram.origin(),
            Pos { x: 7, y: 2 },
            "a change to the drawing carried the view with it"
        );

        state.diagram.remove(finding);
        state.diagram.remove(line);
        state.diagram.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);

        assert_eq!(
            state.diagram.origin(),
            Pos { x: 0, y: 0 },
            "an origin the content no longer holds was kept"
        );
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

    /// **The movement is the session's too, and it records one step.** The figure stays under its
    /// identity and there is a step behind the position, which is what `Undo` gives back.
    #[test]
    fn a_move_leaves_the_session_one_step_further_on() {
        let diagram =
            monospace_description::parse(A_BOX).expect("the test's description is readable");
        let mut state = AppState::new(Session::new(diagram));
        let shape = ShapeId::new(NonZeroU16::new(1).expect("1 is not zero"));

        state.diagram.move_shape(shape, 1, 0);

        assert!(
            state.diagram.session.can_undo(),
            "the move recorded no step"
        );
        assert!(
            !state.diagram.session.can_redo(),
            "the move recorded more than one step"
        );
        assert!(
            state.diagram.session.diagram().get(shape).is_some(),
            "the session lost the shape it moved"
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
