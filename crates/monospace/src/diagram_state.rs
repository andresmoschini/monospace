//! The drawing the last frame produced, and the session it was drawn from.
//!
//! **The session is read on every frame and changed only by an action.** Every frame draws it into
//! the window the interior gives; the region's contextual actions change it, all carried out in
//! `crate::callbacks`: **Delete** performs one removal, **Move** and **Resize** displace and
//! resize the selected figure, **`SetStroke`** gives it another stroke, and the Edit menu's **Undo**
//! and **Redo** walk the history those commands recorded. The application keeps no history of its
//! own: the session is the history.
//!
//! **The selection follows the diagram.** A selection names a figure the session holds, or nothing
//! is selected: [`DiagramState::perform`] reconciles the two after every command, so a command that
//! takes the selected figure away drops the identity without having to name the selection.
//!
//! **The view survives a frame and the offset does not** (D1 of #176): each axis's scroll offset is
//! derived from the origin and the content, so a content change that moves the drawing's first cell
//! does not carry the view with it.

use monospace_core::{
    Buffer, CellReader, Direction, Footprint, FootprintSink, GlyphCatalog, Offset, Orientation,
    OwnerReader, Pos, ShapeId, Size, Stroke, SymbolStyleName, Terminal, Window,
};
use monospace_diagram::{Endpoint, Shape};
use monospace_editing::{Command, Session};
use rat_widget::event::ScrollOutcome;
use rat_widget::scrolled::{ScrollAreaState, ScrollState};
use ratatui_core::layout::{Position, Rect};

use crate::chrome::Target;
use crate::vocabulary::Reply;

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
    /// follows the shape rather than a rectangle left where it was. A walk over the history clears
    /// the selection (#297).
    ///
    /// **An identity names a figure the diagram holds, or nothing is selected**: `perform`
    /// reconciles the selection with the diagram after every command, so one that takes the figure
    /// away drops it without naming the selection.
    selected: Option<ShapeId>,

    /// The active sub-mode, or `None` if in normal mode.
    sub_mode: Option<SubMode>,

    /// **The session is read on every frame and changed by the region's actions and the menu's two
    /// walks**: Delete performs its one command, Move, Resize and `SetStroke` change the selected
    /// figure, Undo and Redo give a step back, and every other slice reads it.
    session: Session,
}

pub(crate) use monospace_diagram::End;

/// An inline sub-mode of the diagram region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SubMode {
    /// Choosing a stroke for the selected figure.
    StrokeSelect,
    /// Moving the selected figure through the layer order.
    OrderMode,
    /// Inspecting and editing a single connector endpoint.
    Endpoint(End),
    /// Choosing a terminal style for the active connector endpoint.
    TerminalSelect(End),
    /// Choosing a leaving direction for the active detached connector endpoint.
    LeavingSelect(End),
}

impl SubMode {
    /// The parent sub-mode to return to when leaving this sub-mode, or `None` if leaving returns to
    /// normal selection.
    #[must_use]
    pub(crate) fn parent(self) -> Option<Self> {
        match self {
            Self::TerminalSelect(end) | Self::LeavingSelect(end) => Some(Self::Endpoint(end)),
            Self::StrokeSelect | Self::OrderMode | Self::Endpoint(_) => None,
        }
    }
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
            sub_mode: None,
            session,
        }
    }

    /// The session this diagram state holds.
    pub(crate) fn session(&self) -> &Session {
        &self.session
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
    /// **The status line reads it on every frame** to report the viewport position in the idle
    /// context, and the tests verify where scrolling moves it.
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

    /// The selected identity and the shape it names, or `None` if nothing is selected or the
    /// selected id is not in the diagram.
    ///
    /// **One answer rather than two reads**: a caller that needs the identity and the shape asks
    /// once, so the two cannot name different figures.
    pub(crate) fn selected_entry(&self) -> Option<(ShapeId, &Shape)> {
        let id = self.selected?;
        self.session.diagram().get(id).map(|shape| (id, shape))
    }

    /// The shape currently selected, or `None` if nothing is selected or the selected id is not in
    /// the diagram.
    pub(crate) fn selected_shape(&self) -> Option<&Shape> {
        self.selected_entry().map(|(_, shape)| shape)
    }

    /// The active sub-mode, or `None` if in normal mode.
    pub(crate) fn sub_mode(&self) -> Option<SubMode> {
        self.sub_mode
    }

    /// Sets the active sub-mode.
    pub(crate) fn set_sub_mode(&mut self, sub_mode: Option<SubMode>) {
        self.sub_mode = sub_mode;
    }

    /// Makes `selection` the shape a click selected, and says whether the screen has to be painted
    /// again.
    ///
    /// **A click that selects what is already selected changed nothing**, and neither did one on a
    /// cell nobody owns with no selection to drop: the answer is `false` and the frame is not owed.
    ///
    /// **Changing or dropping the selection ends any active sub-mode**: the sub-mode belongs to the
    /// figure that was selected, so another selection or no selection starts fresh.
    pub(crate) fn select(&mut self, selection: Option<ShapeId>) -> bool {
        if self.selected == selection {
            return false;
        }

        self.selected = selection;
        self.sub_mode = None;
        true
    }

    /// Selects the next figure in drawing order (`Diagram::shapes()`, back-to-front), wrapping from
    /// the front-most figure back to the first. When nothing is selected, selects the first
    /// (back-most) figure. Returns `true` if the selection changed.
    pub(crate) fn select_next(&mut self) -> bool {
        let ids: Vec<ShapeId> = self.session.diagram().shapes().map(|(id, _)| id).collect();
        if ids.is_empty() {
            return false;
        }
        let next = match self
            .selected
            .and_then(|current| ids.iter().position(|&id| id == current))
        {
            Some(index) => ids[(index + 1) % ids.len()],
            None => ids[0],
        };
        self.select(Some(next))
    }

    /// Selects the previous figure in drawing order (`Diagram::shapes()`, front-to-back), wrapping
    /// from the first (back-most) figure to the last (front-most). When nothing is selected,
    /// selects the last (front-most) figure. Returns `true` if the selection changed.
    pub(crate) fn select_previous(&mut self) -> bool {
        let ids: Vec<ShapeId> = self.session.diagram().shapes().map(|(id, _)| id).collect();
        if ids.is_empty() {
            return false;
        }
        let prev = match self
            .selected
            .and_then(|current| ids.iter().position(|&id| id == current))
        {
            Some(index) => ids[(index + ids.len() - 1) % ids.len()],
            None => *ids.last().expect("non-empty diagram has a last shape"),
        };
        self.select(Some(prev))
    }

    /// The canvas position at the center of the viewport window.
    ///
    /// Computed from the current scroll origin and half the buffer window dimensions.
    pub(crate) fn viewport_center(&self) -> Pos {
        let window = self.buffer.window();
        Pos {
            x: self.origin.x + window.size.width.cast_signed() / 2,
            y: self.origin.y + window.size.height.cast_signed() / 2,
        }
    }

    /// Adds `shape` to the session, selects the newly assigned identity, and returns it.
    pub(crate) fn add_shape(&mut self, shape: Shape) -> ShapeId {
        let (session, id) = self.session.add(shape);
        self.session = session;
        self.select(Some(id));
        id
    }

    /// Creates a 5×5 box with `"light"` stroke at the viewport center and selects it.
    pub(crate) fn create_box(&mut self) -> ShapeId {
        let center = self.viewport_center();
        self.add_shape(Shape::Box {
            at: center,
            size: Size {
                width: 5,
                height: 5,
            },
            stroke: Stroke::from("light"),
            fill: None,
            text: None,
        })
    }

    /// Creates a vertical line of length 5 with `"light"` stroke at the viewport center and selects
    /// it.
    pub(crate) fn create_line(&mut self) -> ShapeId {
        let center = self.viewport_center();
        self.add_shape(Shape::Line {
            at: center,
            len: 5,
            orientation: Orientation::Vertical,
            stroke: Stroke::from("light"),
        })
    }

    /// Creates a detached connector with `"light"` stroke at the viewport center and selects it.
    pub(crate) fn create_connector(&mut self) -> ShapeId {
        let center = self.viewport_center();
        self.add_shape(Shape::Connector {
            from: Endpoint::absolute(
                center,
                Direction::Right,
                Terminal::Symbol {
                    style: SymbolStyleName::from("outline"),
                },
            ),
            to: Endpoint::absolute(
                Pos {
                    x: center.x + 5,
                    y: center.y + 5,
                },
                Direction::Left,
                Terminal::Symbol {
                    style: SymbolStyleName::from("solid"),
                },
            ),
            stroke: Stroke::from("light"),
        })
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

    /// Performs `command` on the session, recording a step when it changes the diagram.
    ///
    /// **A command that changes nothing records no step** (#285): the session is handed back as it
    /// was, and the action is still carried out, which is why the caller still owes the frame.
    ///
    /// **The session is the only thing performed on, and the application keeps no copy of it.**
    /// Every command a slice adds goes through here, so the layer below stays the one that decides
    /// what a step is.
    ///
    /// **Callers hand it a command they have already written.** The state does not name a wrapped
    /// meaning for each one: the client builds the `Command` and this is the one place the session
    /// is changed by it.
    ///
    /// **A selection that named a figure the command took away stops being the selection.** The
    /// question is the diagram's — is the figure still there? — and not the command's name, so
    /// every command that removes a figure drops it without having to say so.
    pub(crate) fn perform(&mut self, command: Command) {
        self.session = self.session.perform(command);
        self.reconcile_selection();
    }

    /// Drops the selection when the diagram no longer holds the figure it names.
    ///
    /// **The selection is a reference and a command may take its referent away without naming the
    /// selection.** Asking the diagram whether the figure is still there is what makes the rule
    /// hold for every command that removes a figure, not only the one whose name says remove.
    ///
    /// **A command that leaves the figure in place changes nothing here**: Move, Resize, `SetStroke`
    /// and `SetText` rewrite the figure under the same identity, and the selection survives them as
    /// it did.
    fn reconcile_selection(&mut self) {
        let Some(selected) = self.selected else {
            return;
        };

        if self.session.diagram().get(selected).is_none() {
            self.select(None);
        }
    }

    /// Gives back the step behind the session, if there is one.
    ///
    /// **The walk is the session's and the application keeps no copy of it** (§3 of the application
    /// model): one `Session::undo` moves the position, and the next frame is drawn from what the
    /// session holds then. At the oldest position nothing comes back, and the caller still owes the
    /// frame because its action was carried out.
    ///
    /// **The walk clears the selection** (#297): walking history returns the application to `IDLE`
    /// and makes the restored drawing a clean starting point. Any active sub-mode drops with it.
    pub(crate) fn undo(&mut self) {
        if self.session.can_undo() {
            self.session = self.session.undo();
            self.select(None);
        }
    }

    /// Gives back the step ahead of the session, if there is one.
    ///
    /// **The walk is the session's, here too**: one `Session::redo` moves the position, and the
    /// application keeps no copy of where the walk stands. At the newest position nothing comes
    /// back, and the caller still owes the frame.
    ///
    /// **The walk clears the selection, here too** (#297): the restored drawing starts with
    /// nothing selected and any active sub-mode dropped.
    pub(crate) fn redo(&mut self) {
        if self.session.can_redo() {
            self.session = self.session.redo();
            self.select(None);
        }
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

    /// What a press at `at` lands on, as far as the region is concerned.
    ///
    /// **The interior is asked about before the border**: the window is inside the region's own
    /// rectangle, so the cell comes first and the frame drawn around it is the border — the
    /// keyboard's without being a cell of the drawing (rule 2 of #189).
    pub(crate) fn target_at(&self, at: Position) -> Target {
        if let Some(offset) = self.offset_of(at) {
            Target::Interior(offset)
        } else if self.region.contains(at) {
            Target::Border
        } else {
            Target::Nothing
        }
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
    /// the view already stands, changed nothing — but the bars took the event, so the answer is
    /// [`Reply::Consumed`] and the region's click does not read the same press again.
    pub(crate) fn scrolled(&mut self, answer: ScrollOutcome) -> Reply {
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
            ScrollOutcome::Unchanged => return Reply::Consumed,
            ScrollOutcome::Continue => return Reply::Ignored,
        };

        if !changed {
            return Reply::Consumed;
        }

        let (across, down) = axes(self.measure(), self.interior);
        let origin = Pos {
            x: across.at(self.hscroll.offset()),
            y: down.at(self.vscroll.offset()),
        };
        let moved = origin != self.origin;
        self.origin = origin;

        if moved {
            Reply::Repaint
        } else {
            Reply::Consumed
        }
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

    /// The diagram state over the diagram `json` describes.
    fn a_diagram_over(json: &str) -> DiagramState {
        let diagram =
            monospace_description::parse(json).expect("the test's description is readable");

        DiagramState::new(Session::new(diagram))
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

    /// **A state that has not drawn a frame has measured nothing**, and the first `redraw` is what
    /// measures the drawing: the box of `A_BOX` reaches `(5, 3)` from `(2, 1)`.
    #[test]
    fn a_state_that_has_not_drawn_a_frame_has_no_footprint() {
        let diagram =
            monospace_description::parse(A_BOX).expect("the test's description is readable");
        let mut state = DiagramState::new(Session::new(diagram));

        assert_eq!(
            state.footprint(),
            None,
            "a state that has drawn no frame reached somewhere"
        );

        state.redraw(Rect::new(0, 0, 20, 10), Rect::new(0, 0, 20, 10));

        assert_eq!(
            state.footprint(),
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
        let mut state = DiagramState::new(Session::new(diagram));

        state.redraw(Rect::new(0, 0, 3, 2), Rect::new(0, 0, 1, 1));

        assert_eq!(
            state.footprint(),
            Some(Footprint {
                min: Pos { x: 2, y: 1 },
                max: Pos { x: 5, y: 3 },
            }),
            "the interior decided how far the drawing reaches"
        );

        state.redraw(Rect::new(0, 0, 20, 10), Rect::new(0, 0, 20, 10));

        assert_eq!(
            state.footprint(),
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
        let mut state = a_diagram_over(THE_EXAMPLE);

        state.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);

        assert_eq!(
            state.origin(),
            Pos { x: 0, y: 0 },
            "the first frame did not open on the first screen"
        );
        assert_eq!(
            at(&state, 0, 0),
            "┌",
            "the drawing's own corner is not in the window"
        );
    }

    /// **A drawing inside the first screen does not scroll** (rule 1). The content is the union of
    /// the drawing and the window at `(0, 0)`, so a box that fits leaves neither axis any room to
    /// travel and the frame opens where the drawing is.
    #[test]
    fn a_drawing_inside_the_first_screen_does_not_scroll() {
        let mut state = a_diagram_over(A_BOX);

        state.redraw(Rect::new(0, 0, 20, 10), Rect::new(1, 1, 18, 6));

        assert_eq!(
            state.origin(),
            Pos { x: 0, y: 0 },
            "a drawing inside the first screen opened somewhere else"
        );
        assert_eq!(
            state.hscroll.max_offset(),
            0,
            "the content is wider than the window it fits"
        );
        assert_eq!(
            state.vscroll.max_offset(),
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
        let mut state = a_diagram_over(THE_EXAMPLE);
        state.origin = Pos { x: 7, y: 2 };

        state.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);

        assert_eq!(
            state.origin(),
            Pos { x: 7, y: 2 },
            "the frame moved an origin the content holds"
        );
        assert_eq!(
            window_of(&state).join("\n"),
            "   ╚╝\n     \n──   ",
            "the window does not hold what the drawing reaches at that origin"
        );
    }

    /// **A frame at the end of the content owes no frame** (rule 2). An origin the content does not
    /// hold is put at the nearest value it does by the frame that draws it, and the frame after it
    /// draws the same picture: the clamp is not a change the next frame has to be owed for.
    #[test]
    fn a_frame_at_the_end_of_the_content_owes_no_frame() {
        let mut state = a_diagram_over(THE_EXAMPLE);
        state.origin = Pos { x: 20, y: 20 };

        state.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);
        let clamped = window_of(&state);

        assert_eq!(
            state.origin(),
            Pos { x: 7, y: 2 },
            "the frame did not put the origin at the furthest the content allows"
        );

        state.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);

        assert_eq!(
            state.origin(),
            Pos { x: 7, y: 2 },
            "the frame that followed moved the view"
        );
        assert_eq!(
            window_of(&state),
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
        let mut state = a_diagram_over(THE_EXAMPLE);
        let finding = ShapeId::new(NonZeroU16::new(2).expect("2 is not zero"));
        let line = ShapeId::new(NonZeroU16::new(3).expect("3 is not zero"));
        state.origin = Pos { x: 7, y: 2 };
        state.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);

        state.perform(Command::Move {
            id: finding,
            by: (2, 0).into(),
        });
        state.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);

        assert_eq!(
            state.origin(),
            Pos { x: 7, y: 2 },
            "a change to the drawing carried the view with it"
        );

        state.perform(Command::Remove { id: finding });
        state.perform(Command::Remove { id: line });
        state.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);

        assert_eq!(
            state.origin(),
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
        let mut state = DiagramState::new(Session::new(diagram));
        let shape = ShapeId::new(NonZeroU16::new(1).expect("1 is not zero"));

        assert!(
            !state.session.can_undo(),
            "a fresh session has a step behind it"
        );

        state.perform(Command::Remove { id: shape });

        assert!(state.session.can_undo(), "the removal recorded no step");
        assert!(
            !state.session.can_redo(),
            "the removal recorded more than one step"
        );
        assert!(
            state.session.diagram().get(shape).is_none(),
            "the session still holds the removed shape"
        );
        assert_eq!(
            state.selected(),
            None,
            "the removal kept the identity it took out"
        );
    }

    /// **A command that removes the selected figure drops the selection**, and the sub-mode that
    /// belonged to it goes with the identity: the figure the selection named is no longer in the
    /// diagram, so there is nothing left to select.
    #[test]
    fn a_command_that_removes_the_selected_figure_drops_the_selection() {
        let selected = ShapeId::new(NonZeroU16::new(1).expect("1 is not zero"));

        for sub_mode in [
            SubMode::StrokeSelect,
            SubMode::OrderMode,
            SubMode::Endpoint(End::From),
            SubMode::Endpoint(End::To),
            SubMode::TerminalSelect(End::From),
            SubMode::TerminalSelect(End::To),
            SubMode::LeavingSelect(End::From),
            SubMode::LeavingSelect(End::To),
        ] {
            let mut state = a_diagram_over(THE_EXAMPLE);
            state.select(Some(selected));
            state.set_sub_mode(Some(sub_mode));
            state.perform(Command::Remove { id: selected });

            assert_eq!(
                state.selected(),
                None,
                "the selection named a figure the diagram lost"
            );
            assert_eq!(
                state.sub_mode(),
                None,
                "the sub-mode survived the figure it belonged to"
            );
        }
    }

    #[test]
    fn sub_mode_parent_returns_expected_parent() {
        assert_eq!(
            SubMode::TerminalSelect(End::From).parent(),
            Some(SubMode::Endpoint(End::From))
        );
        assert_eq!(
            SubMode::TerminalSelect(End::To).parent(),
            Some(SubMode::Endpoint(End::To))
        );
        assert_eq!(
            SubMode::LeavingSelect(End::From).parent(),
            Some(SubMode::Endpoint(End::From))
        );
        assert_eq!(
            SubMode::LeavingSelect(End::To).parent(),
            Some(SubMode::Endpoint(End::To))
        );
        assert_eq!(SubMode::StrokeSelect.parent(), None);
        assert_eq!(SubMode::OrderMode.parent(), None);
        assert_eq!(SubMode::Endpoint(End::From).parent(), None);
        assert_eq!(SubMode::Endpoint(End::To).parent(), None);
    }

    /// **A command that removes another figure keeps the selection.** The rule asks whether the
    /// diagram still holds the figure the selection names, so a removal that takes another one away
    /// changes nothing here. This is what distinguishes the state's rule from a cleanup tied to the
    /// command's name: `Delete` could never produce this case.
    #[test]
    fn a_command_that_removes_another_figure_keeps_the_selection() {
        let mut state = a_diagram_over(THE_EXAMPLE);
        let selected = ShapeId::new(NonZeroU16::new(1).expect("1 is not zero"));
        let other = ShapeId::new(NonZeroU16::new(2).expect("2 is not zero"));

        state.select(Some(selected));
        state.perform(Command::Remove { id: other });

        assert!(
            state.session.diagram().get(other).is_none(),
            "the session still holds the figure the command removed"
        );
        assert_eq!(
            state.selected(),
            Some(selected),
            "the selection named a figure the diagram still holds"
        );
    }

    /// **The movement is the session's too, and it records one step.** The figure stays under its
    /// identity and there is a step behind the position, which is what `Undo` gives back.
    #[test]
    fn a_move_leaves_the_session_one_step_further_on() {
        let diagram =
            monospace_description::parse(A_BOX).expect("the test's description is readable");
        let mut state = DiagramState::new(Session::new(diagram));
        let shape = ShapeId::new(NonZeroU16::new(1).expect("1 is not zero"));

        state.perform(Command::Move {
            id: shape,
            by: (1, 0).into(),
        });

        assert!(state.session.can_undo(), "the move recorded no step");
        assert!(
            !state.session.can_redo(),
            "the move recorded more than one step"
        );
        assert!(
            state.session.diagram().get(shape).is_some(),
            "the session lost the shape it moved"
        );
    }

    /// **The resize is the session's too, and it records one step.**
    #[test]
    fn a_resize_leaves_the_session_one_step_further_on() {
        let diagram =
            monospace_description::parse(A_BOX).expect("the test's description is readable");
        let mut state = DiagramState::new(Session::new(diagram));
        let shape = ShapeId::new(NonZeroU16::new(1).expect("1 is not zero"));

        state.perform(Command::Resize {
            id: shape,
            by: (1, 0).into(),
        });

        assert!(state.session.can_undo(), "the resize recorded no step");
        assert!(
            !state.session.can_redo(),
            "the resize recorded more than one step"
        );
        let Some(monospace_diagram::Shape::Box { size, .. }) = state.session.diagram().get(shape)
        else {
            panic!("expected box");
        };
        assert_eq!(size.width, 5);
    }

    /// **The walk stops at both ends of the history.** An undo at the oldest position and a redo at
    /// the newest give nothing back, and the step between them is the one the removal recorded.
    #[test]
    fn undo_and_redo_stop_at_the_ends_of_the_history() {
        let diagram =
            monospace_description::parse(A_BOX).expect("the test's description is readable");
        let mut state = DiagramState::new(Session::new(diagram));
        let shape = ShapeId::new(NonZeroU16::new(1).expect("1 is not zero"));

        state.undo();

        assert!(
            state.session.diagram().get(shape).is_some(),
            "the undo at the oldest position took the box out"
        );

        state.perform(Command::Remove { id: shape });

        assert!(
            state.session.diagram().get(shape).is_none(),
            "the removal kept the box"
        );
        assert!(!state.can_redo(), "the removal left a step ahead");

        state.undo();

        assert!(
            state.session.diagram().get(shape).is_some(),
            "the undo did not give the box back"
        );
        assert!(!state.can_undo(), "one step back left another behind it");
        assert!(state.can_redo(), "the undo left nothing to redo");

        state.redo();

        assert!(
            state.session.diagram().get(shape).is_none(),
            "the redo did not take the box out again"
        );
        assert!(
            !state.can_redo(),
            "one step forward left another ahead of it"
        );
        assert!(state.can_undo(), "the redo left nothing to undo");

        state.redo();

        assert!(
            state.session.diagram().get(shape).is_none(),
            "the redo at the newest position took the box out of nothing"
        );
        assert!(
            !state.can_redo(),
            "the redo at the newest position recorded a step"
        );
    }

    #[test]
    fn the_session_is_exposed() {
        let diagram =
            monospace_description::parse(A_BOX).expect("the test's description is readable");
        let state = DiagramState::new(Session::new(diagram));
        assert!(!state.session().can_undo());
    }

    /// **A walk clears the selection and any active sub-mode** (#297).
    #[test]
    fn undo_and_redo_clear_the_selection_and_sub_mode() {
        let diagram =
            monospace_description::parse(A_BOX).expect("the test's description is readable");
        let mut state = DiagramState::new(Session::new(diagram));
        let shape = ShapeId::new(NonZeroU16::new(1).expect("1 is not zero"));

        state.perform(Command::Move {
            id: shape,
            by: (1, 0).into(),
        });
        state.select(Some(shape));
        state.set_sub_mode(Some(SubMode::StrokeSelect));

        state.undo();

        assert_eq!(state.selected(), None, "the undo kept the selection");
        assert_eq!(state.sub_mode(), None, "the undo kept the sub-mode");

        state.select(Some(shape));
        state.set_sub_mode(Some(SubMode::StrokeSelect));

        state.redo();

        assert_eq!(state.selected(), None, "the redo kept the selection");
        assert_eq!(state.sub_mode(), None, "the redo kept the sub-mode");

        // When can_undo or can_redo is false, no action is performed and selection is preserved.
        state.select(Some(shape));
        state.set_sub_mode(Some(SubMode::StrokeSelect));

        assert!(!state.can_redo());
        state.redo();
        assert_eq!(
            state.selected(),
            Some(shape),
            "ineffective redo cleared the selection"
        );
        assert_eq!(
            state.sub_mode(),
            Some(SubMode::StrokeSelect),
            "ineffective redo cleared the sub-mode"
        );

        state.undo();
        assert_eq!(state.selected(), None);

        state.select(Some(shape));
        state.set_sub_mode(Some(SubMode::StrokeSelect));

        assert!(!state.can_undo());
        state.undo();
        assert_eq!(
            state.selected(),
            Some(shape),
            "ineffective undo cleared the selection"
        );
        assert_eq!(
            state.sub_mode(),
            Some(SubMode::StrokeSelect),
            "ineffective undo cleared the sub-mode"
        );
    }

    #[test]
    fn select_next_and_select_previous_cycle_in_drawing_order_and_wrap() {
        let mut empty = DiagramState::new(Session::new(monospace_diagram::Diagram::new()));
        assert!(!empty.select_next());
        assert!(!empty.select_previous());
        assert_eq!(empty.selected(), None);

        let single_diagram =
            monospace_description::parse(A_BOX).expect("the test's description is readable");
        let mut single = DiagramState::new(Session::new(single_diagram));
        let id1 = ShapeId::new(NonZeroU16::new(1).expect("1 is not zero"));
        assert!(single.select_next());
        assert_eq!(single.selected(), Some(id1));
        assert!(!single.select_next());
        assert_eq!(single.selected(), Some(id1));
        assert!(!single.select_previous());
        assert_eq!(single.selected(), Some(id1));

        let three_shapes = r#"{
            "next_id": 4,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 }, "stroke": "light" },
                { "kind": "box", "id": 2, "at": { "x": 6, "y": 0 }, "size": { "width": 4, "height": 3 }, "stroke": "light" },
                { "kind": "box", "id": 3, "at": { "x": 12, "y": 0 }, "size": { "width": 4, "height": 3 }, "stroke": "light" }
            ]
        }"#;
        let diagram = monospace_description::parse(three_shapes).expect("valid description");
        let mut state = DiagramState::new(Session::new(diagram));
        let id2 = ShapeId::new(NonZeroU16::new(2).expect("2 is not zero"));
        let id3 = ShapeId::new(NonZeroU16::new(3).expect("3 is not zero"));

        assert!(state.select_next());
        assert_eq!(state.selected(), Some(id1));
        assert!(state.select_next());
        assert_eq!(state.selected(), Some(id2));
        assert!(state.select_next());
        assert_eq!(state.selected(), Some(id3));
        assert!(state.select_next());
        assert_eq!(state.selected(), Some(id1));

        assert!(state.select_previous());
        assert_eq!(state.selected(), Some(id3));
        assert!(state.select_previous());
        assert_eq!(state.selected(), Some(id2));
        assert!(state.select_previous());
        assert_eq!(state.selected(), Some(id1));

        state.select(None);
        assert!(state.select_previous());
        assert_eq!(state.selected(), Some(id3));
    }

    #[test]
    fn create_box_creates_default_5x5_box_at_viewport_center_and_selects_it() {
        let mut state = DiagramState::new(Session::new(monospace_diagram::Diagram::new()));
        state.redraw(Rect::new(0, 0, 20, 10), Rect::new(0, 0, 20, 10));
        assert_eq!(state.viewport_center(), Pos { x: 10, y: 5 });

        let box_id = state.create_box();
        assert_eq!(state.selected(), Some(box_id));
        assert_eq!(state.sub_mode(), None);

        let shape = state.selected_shape().expect("box is in diagram");
        assert_eq!(
            shape,
            &Shape::Box {
                at: Pos { x: 10, y: 5 },
                size: Size {
                    width: 5,
                    height: 5
                },
                stroke: Stroke::from("light"),
                fill: None,
                text: None,
            }
        );

        // Undo removes the box
        state.session = state.session.undo();
        assert_eq!(state.session.diagram().shape_count(), 0);
    }

    #[test]
    fn create_line_creates_vertical_line_on_scrolled_viewport_and_selects_it() {
        let mut state = a_diagram_over(THE_EXAMPLE);
        // Scrolled viewport where origin != (0, 0)
        state.origin = Pos { x: 6, y: 2 };
        state.redraw(EXAMPLE_REGION, EXAMPLE_INTERIOR);
        let center = state.viewport_center();
        assert_eq!(
            center,
            Pos {
                x: state.origin().x + EXAMPLE_INTERIOR.width.cast_signed() / 2,
                y: state.origin().y + EXAMPLE_INTERIOR.height.cast_signed() / 2,
            }
        );

        let prior_count = state.session.diagram().shape_count();
        let line_id = state.create_line();
        assert_eq!(state.selected(), Some(line_id));
        assert_eq!(state.sub_mode(), None);
        assert_eq!(state.session.diagram().shape_count(), prior_count + 1);

        let shape = state.selected_shape().expect("line is in diagram");
        assert_eq!(
            shape,
            &Shape::Line {
                at: center,
                len: 5,
                orientation: Orientation::Vertical,
                stroke: Stroke::from("light"),
            }
        );

        // Undo removes the line
        state.session = state.session.undo();
        assert_eq!(state.session.diagram().shape_count(), prior_count);
    }

    #[test]
    fn create_connector_creates_detached_connector_with_terminals_and_selects_it() {
        use monospace_diagram::EndpointAt;

        let mut state = DiagramState::new(Session::new(monospace_diagram::Diagram::new()));
        state.redraw(Rect::new(0, 0, 16, 8), Rect::new(0, 0, 16, 8));
        let center = state.viewport_center();
        assert_eq!(center, Pos { x: 8, y: 4 });

        let conn_id = state.create_connector();
        assert_eq!(state.selected(), Some(conn_id));
        assert_eq!(state.sub_mode(), None);

        let shape = state.selected_shape().expect("connector is in diagram");
        let Shape::Connector { from, to, stroke } = shape else {
            panic!("expected connector shape");
        };
        assert_eq!(
            from.at,
            EndpointAt::Absolute {
                at: Pos { x: 8, y: 4 },
                leaving: Direction::Right,
            }
        );
        assert_eq!(
            from.terminal,
            Terminal::Symbol {
                style: SymbolStyleName::from("outline"),
            }
        );
        assert_eq!(
            to.at,
            EndpointAt::Absolute {
                at: Pos { x: 13, y: 9 },
                leaving: Direction::Left,
            }
        );
        assert_eq!(
            to.terminal,
            Terminal::Symbol {
                style: SymbolStyleName::from("solid"),
            }
        );
        assert_eq!(*stroke, Stroke::from("light"));

        // Undo restores prior diagram
        state.session = state.session.undo();
        assert_eq!(state.session.diagram().shape_count(), 0);
    }
}
