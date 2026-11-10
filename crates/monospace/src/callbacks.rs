//! The four signatures the framework dictates, and what the application does when each is called.
//!
//! **They are free functions because the signatures are fixed** by the parameters of `run_tui`.
//!
//! **This is where an event becomes an action.** A terminal event is answered here: a resize
//! repaints, and everything else is routed to the first handler that claims it — the shortcuts
//! that reach everything, the chrome, the diagram, and the shortcuts nobody has claimed — and the
//! action it names is carried out: the screen changed, the way out, or the forced exit. An entry of
//! the menu and the key that means the same thing therefore arrive at one place, and that place is
//! here.
//!
//! **The order is the whole of the policy, and it is written once.** Nothing here names a key:
//! which keys the application answers to, and whether a key has to be answered before the chrome,
//! is `shortcuts`' table. This module only says when each layer is asked.

use rat_salsa::Control;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_crossterm::crossterm::event::{Event, MouseButton, MouseEventKind};

use crate::about;
use crate::chrome::Target;
use crate::context::AppGlobal;
use crate::diagram_widget;
use crate::dialog;
use crate::error;
use crate::event::AppEvent;
use crate::export;
use crate::export::ExportDialogState;
use crate::frame;
use crate::menu;
use crate::open;
use crate::open::OpenDialogState;
use crate::save;
use crate::save::SaveDialogState;
use crate::shortcuts;
use crate::state::AppState;
use crate::text;
use crate::text::TextEditorState;
use crate::vocabulary::{Action, AppError, Reply};
use monospace_diagram::Shape;
#[cfg(test)]
use monospace_editing::Command;

/// The terminal is the framework's: it takes the screen and gives it back, and there is nothing
/// of ours to set up.
#[allow(clippy::unnecessary_wraps, reason = "the signature is the framework's")]
pub(crate) fn init(_state: &mut AppState, _global: &mut AppGlobal) -> Result<(), AppError> {
    Ok(())
}

/// Paints one frame.
#[allow(clippy::unnecessary_wraps, reason = "the signature is the framework's")]
pub(crate) fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut AppState,
    global: &mut AppGlobal,
) -> Result<(), AppError> {
    frame::draw(area, buf, state, global);
    Ok(())
}

/// What the application does about an event.
#[allow(clippy::unnecessary_wraps, reason = "the signature is the framework's")]
pub(crate) fn event(
    event: &AppEvent,
    state: &mut AppState,
    _global: &mut AppGlobal,
) -> Result<Control<AppEvent>, AppError> {
    Ok(match event {
        // The loop reporting on itself: the frame it painted is the whole of the answer.
        AppEvent::Rendered(_) => Control::Continue,

        // **A resize is answered here and not by the handlers.** It is not a question about the
        // event — nobody wants a screen size — it is a fact about the frame: the layout it was
        // painted with is the one it no longer has, so it is painted again.
        AppEvent::Terminal(Event::Resize(_, _)) => Control::Changed,

        AppEvent::Terminal(event) => match route(event, state) {
            // **`Ignored` and `Consumed` are the same answer here**: neither owes a frame.
            Reply::Ignored | Reply::Consumed => Control::Continue,

            // **The frame is owed, but no action is carried out.** `route` has already addressed
            // any press, and this preserves that focus and the menu state instead of applying the
            // cleanup.
            Reply::Repaint => Control::Changed,

            // **The one that ends the loop**, and the only one that is not carried out: nothing
            // about the state changes, so there is no meaning to apply and no frame to owe.
            // **Whether it is forced stops here**: the loop ends either way.
            Reply::Quit { .. } => Control::Quit,

            // **A meaning is carried out, and carrying one out is the same three lines every
            // time.** What owes the frame is the meaning having been said, not the state having
            // changed, which is why the cleanup and the paint are here rather than in `apply`.
            Reply::Act(action) => {
                apply(&action, state);
                cleanup(state);
                Control::Changed
            }
        },
    })
}

/// Which handler answers the event, and what it answers.
///
/// **The order is the whole of the policy, and the chain is the order.** The shortcuts that reach
/// everything are asked first: a `Ctrl+Shift+Q` that reached the chrome as a key would be answered
/// by the chrome, and the point of it is that it is answered by nothing. **Then the dialog, while it
/// is up**: it is the overlay, it takes every key and every control, and an event it leaves alone
/// reaches nothing else. Then the bar, so that a key the widget uses is the widget's; then the
/// region, so that a click a drop-down covers is the drop-down's and the one it leaves is the
/// diagram's; then the shortcuts nobody has claimed. The first action wins, and an event all of
/// them leave alone is answered by nobody.
///
/// **The bar's keys that move the keyboard are read by the bar** (D4 and rule 5 of #189): `Alt` plus
/// a label's navigation character brings it with the press, `F10` asks for it, and the `Esc` that
/// has nothing left to close gives it back. **A leaf is not a meaning**, so these three keys move
/// the keyboard where they are read: the bar is handed the focus along with its own state, and no
/// action names a leaf.
///
/// **The one thing that is settled before a handler is asked is where the event is addressed**
/// (D1): a press moves the keyboard to the leaf it lands in, and it does so first because the label
/// a press opens and the drop-down a press closes are changes the handlers make while they are
/// asked — moving the keyboard afterwards would answer the wrong leaf. **A press that moved it owes
/// a frame even where nobody answers**: the label that was reversed is not any more.
///
/// **The state comes whole and the handler is handed the part it owns.** The chrome is given
/// `state.chrome_mut()`, the bar and the keyboard together, and the region `state.diagram_mut()`,
/// so what a handler may change about the application is what its own signature makes possible: the
/// selection, the drop-down and which leaf holds the keyboard, and never the session.
///
/// **A handler that consumed the event stops the chain**, and that is the difference the protocol
/// carries: `Ignored` is "not mine, ask the next one", `Consumed` is "mine, and nothing changed",
/// and only the first lets the region see a press the drop-down already trapped.
fn route(event: &Event, state: &mut AppState) -> Reply {
    // **The two reaches that answer before the press is addressed come first, and in this order.**
    // A `Ctrl+Shift+Q` that reached the chrome as a key would be answered by the chrome, and the
    // point of it is that it is answered by nothing; the overlays sit above the application, so
    // while one is up an event it leaves alone reaches nothing else. **`Ignored` is the overlay
    // being down**: the handler hands it back without asking the widget, so routing is one call and
    // not a question about whether the overlay is up followed by a question about the event.
    //
    // **Neither moves the keyboard**, which is why addressing sits below and not here: a
    // click on the bar behind an overlay moves nothing, and the press is addressed only once nobody
    // above has answered it.
    let before = shortcuts::handle_always(event)
        .or_else(|| text::handle(event, state.text_mut()))
        .or_else(|| dialog::handle(event, state.dialog_mut()))
        .or_else(|| open::handle(event, state.open_mut()))
        .or_else(|| save::handle(event, state.save_mut()))
        .or_else(|| export::handle(event, state.export_mut()));

    if before != Reply::Ignored {
        return before;
    }

    // **Where the event is addressed is read once**: the keyboard moves to the leaf the press
    // lands in, and the region is handed the same target rather than reading the press again.
    let target = target_of(event, state);
    let moved = state.chrome_mut().move_to(target);

    let answer = menu::handle(event, state.chrome_mut())
        .or_else(|| {
            // **The pair is taken together** so that the region is asked with the leaf the bar
            // left behind, which is the one the event was routed with.
            let (chrome, diagram) = state.chrome_and_diagram();
            diagram_widget::handle(event, diagram, chrome.leaf(), target)
        })
        .or_else(|| shortcuts::handle_unclaimed(event));

    // **The move is added last, and it is not part of the chain**: it is not an answer to the
    // event, it is a change the screen already made and that no handler was told about.
    if moved { answer.and_repaint() } else { answer }
}

/// Where the event is addressed, or [`Target::Nothing`] where it is not a press.
///
/// **Only a left press is addressed**: a release, a drag, a turn of the wheel and a key belong to
/// the handler that reads them, and the keyboard does not move for them.
fn target_of(event: &Event, state: &AppState) -> Target {
    let Event::Mouse(mouse) = event else {
        return Target::Nothing;
    };

    if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
        return Target::Nothing;
    }

    state.target_at(Position::new(mouse.column, mouse.row))
}

/// Carries a meaning out into the state, and does nothing else.
///
/// **Applying and deciding are two jobs, and the framework's boundary is where they split.**
/// `apply` changes what the application knows; whether the loop continues, paints or ends is
/// `event`'s, because that is the framework's own vocabulary and nothing else here speaks it.
///
/// **An action need not change anything.** `Delete` with nothing selected removes nothing, and
/// `Undo` with nothing behind walks nothing; every action is applied all the same. **What leaves
/// the keyboard on the region is the carrying out and not the action** (rule 13 of #189), so it is
/// said once in [`event`], beside the paint.
fn apply(action: &Action, state: &mut AppState) {
    match action {
        // **The walks are the session's** (D2 of #181): each gives back one step and keeps no copy
        // of where the walk stands. At either end the session answers with itself.
        Action::Undo => state.diagram_mut().undo(),

        Action::Redo => state.diagram_mut().redo(),

        Action::Command(command) => {
            state.diagram_mut().perform(command.clone());
        }

        // **The editor is opened over the selected box and records no step.** The identity and the text
        // are read together, so the title and the buffer cannot name different figures.
        Action::EditText => {
            let opened = state
                .diagram()
                .selected_entry()
                .and_then(|(id, shape)| match shape {
                    Shape::Box { text, .. } => Some((id, text.clone())),
                    _ => None,
                });

            if let Some((id, text)) = opened {
                state
                    .text_mut()
                    .show(TextEditorState::new(id, text.as_ref()));
            }
        }

        // **The two actions that open the overlay.** The dialog is the chrome's own, so the
        // session is untouched: no command is performed and no step is recorded (D5 of #189).
        Action::About => state.dialog_mut().show(about::open()),

        // **The same overlay, over a failure's text**: it replaces whatever the dialog held.
        Action::ShowError(message) => state.dialog_mut().show(error::open(message)),

        // **The open dialog is the chrome's own too**, and opening it changes no session.
        Action::Open => state.open_mut().show(OpenDialogState::new()),

        // **The path is read and parsed before anything is replaced** (D3 and D4 of #260): a
        // file that cannot be read or parsed leaves the current session untouched, and reports
        // the failure in the message dialog through ShowError. A success starts a fresh session.
        Action::OpenFile(path) => match crate::load(path) {
            Ok(session) => *state = AppState::new(session),
            Err(error) => apply(&Action::ShowError(error.to_string()), state),
        },

        // **A fresh blank session is started immediately** (D3 of #273), discarding current work,
        // selection, and undo history without confirmation.
        Action::New => *state = AppState::new(crate::empty_session()),

        // **A fresh demonstration session is started immediately** (D3 of #273), discarding current
        // work, selection, and undo history without confirmation.
        Action::OpenDemo => match crate::demo() {
            Ok(session) => *state = AppState::new(session),
            Err(error) => apply(&Action::ShowError(error.to_string()), state),
        },

        // **The save dialog is the chrome's own too**, and opening it changes no session.
        Action::Save => state.save_mut().show(SaveDialogState::new()),

        // **The session's diagram is serialized and written to path** (D2 and D3 of #270): on
        // success the file is written and the current session is untouched; on any error it
        // leaves the session untouched and reports through ShowError.
        Action::SaveFile(path) => {
            if let Err(error) = crate::save(path, state.diagram().session()) {
                apply(&Action::ShowError(error.to_string()), state);
            }
        }

        // **The export dialog is the chrome's own too**, and opening it changes no session.
        Action::ExportText => state.export_mut().show(ExportDialogState::new()),

        // **The session's diagram is rendered over its footprint and written to path** (D3, D4
        // and D5 of #352): on success the file is written and the current session is untouched;
        // on any error it leaves the session untouched and reports through ShowError.
        Action::ExportTextFile(path) => {
            if let Err(error) = crate::export_text(path, state.diagram().session()) {
                apply(&Action::ShowError(error.to_string()), state);
            }
        }
    }
}

/// **A meaning that is carried out leaves the keyboard on the region** (rule 13 of #189): what a
/// person took from the bar is done, and what is left to do is in the drawing. The drop-down the
/// bar was showing closes with it, because a menu nobody holds the keyboard on is not open.
fn cleanup(state: &mut AppState) {
    state.chrome_mut().close();
}

/// What the application does about a failure.
///
/// **The error is reported once, and the loop does not spin on it.** The framework hands the
/// error to this handler and takes back a control; answering with the error again is the
/// framework re-queueing the same failure for ever. So the handler ends the loop and keeps the
/// error, and the error is written out with the terminal back.
#[allow(clippy::unnecessary_wraps, reason = "the signature is the framework's")]
pub(crate) fn error(
    error: AppError,
    _state: &mut AppState,
    global: &mut AppGlobal,
) -> Result<Control<AppEvent>, AppError> {
    global.set_failure(error);
    Ok(Control::Quit)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;

    use super::*;
    use crate::chrome::Focus;
    use crate::config::Config;
    use crate::status;
    use crate::vocabulary::AppError;
    use monospace_core::{Offset, Pos};
    use monospace_diagram::{ShapeId, Text};
    use ratatui_core::layout::Rect;
    use ratatui_core::style::Modifier;
    use ratatui_crossterm::crossterm::event::{
        Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };

    /// The width of the screen these tests draw.
    const WIDTH: u16 = 40;

    /// The height of the screen these tests draw.
    const HEIGHT: u16 = 10;

    /// The state of an application that starts with nothing in it.
    fn a_state() -> AppState {
        AppState::new(crate::empty_session())
    }

    /// The configuration the application starts with.
    fn a_global() -> AppGlobal {
        AppGlobal::new(Config::default(), monospace_glyph_sets::all())
    }

    /// A description whose only shape is a light 4×3 box at `(2, 1)` of the window.
    ///
    /// The window is the region's interior, which on a screen of the size these tests draw is
    /// `38×5` at `(1, 2)` of the screen: the box's top-left is the screen's `(3, 3)`.
    const A_BOX: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                      "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
    }"#;

    /// A description whose only shape is a light 4×3 box at `(2, 1)` holding `hi`.
    const A_BOX_WITH_TEXT: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                      "size": { "width": 4, "height": 3 }, "stroke": "light",
                      "text": "hi" } ]
    }"#;

    /// A description whose only shape is a light connector between `(2, 1)` and `(10, 1)`.
    ///
    /// **`callbacks` does not have one yet**; it is the fixture `diagram_widget` already uses, added
    /// here because `t_is_ignored_off_a_box` needs a connector to select.
    const A_CONNECTOR: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "connector", "id": 1,
                      "from": { "at": { "kind": "point", "x": 2, "y": 1 },
                                "leaving": "right", "terminal": { "kind": "arm" } },
                      "to": { "at": { "kind": "point", "x": 10, "y": 1 },
                              "leaving": "left", "terminal": { "kind": "arm" } },
                      "stroke": "light" } ]
    }"#;

    /// A description whose only shape is a light `2×2` box at `(2, 1)`: the smallest a box shrinks to,
    /// so shrinking it again changes nothing.
    const A_BOX_AT_ITS_SMALLEST: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                      "size": { "width": 2, "height": 2 }, "stroke": "light" } ]
    }"#;

    /// A description whose only shape is a light horizontal line of length 6 at `(2, 1)`.
    const A_HORIZONTAL_LINE: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "line", "id": 1, "at": { "x": 2, "y": 1 },
                      "len": 6, "orientation": "horizontal", "stroke": "light" } ]
    }"#;

    /// A description whose only shape is a light vertical line of length 3 at `(2, 1)`.
    const A_VERTICAL_LINE: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "line", "id": 1, "at": { "x": 2, "y": 1 },
                      "len": 3, "orientation": "vertical", "stroke": "light" } ]
    }"#;

    /// A description whose only shape is a horizontal line of length 1 at `(2, 1)`.
    const A_LINE_AT_ITS_MINIMUM: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "line", "id": 1, "at": { "x": 2, "y": 1 },
                      "len": 1, "orientation": "horizontal", "stroke": "light" } ]
    }"#;

    /// A description whose only shape is a horizontal line of length 0 at `(2, 1)`.
    const A_ZERO_LENGTH_LINE: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "line", "id": 1, "at": { "x": 2, "y": 1 },
                      "len": 0, "orientation": "horizontal", "stroke": "light" } ]
    }"#;

    /// The arrangement §4 and §6 of the diagram model are shown with: a box at the left, a box at
    /// the right, and one connector hung from the right side of the first and reaching the left
    /// side of the second.
    ///
    /// The left box is at `(0, 1)` of the window, so its top-left is the screen's `(1, 3)`; the
    /// right box's is at `(16, 1)`, the screen's `(17, 3)`.
    const A_BOX_AND_WHAT_HANGS_FROM_IT: &str = r#"{
        "next_id": 4,
        "shapes": [
            { "kind": "box", "id": 1, "at": { "x": 0, "y": 1 },
              "size": { "width": 4, "height": 3 }, "stroke": "light" },
            { "kind": "box", "id": 2, "at": { "x": 16, "y": 1 },
              "size": { "width": 4, "height": 3 }, "stroke": "light" },
            { "kind": "connector", "id": 3,
              "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" },
                        "terminal": { "kind": "arm" } },
              "to":   { "at": { "kind": "reference", "shape": 2, "anchor": "left" },
                        "terminal": { "kind": "arm" } },
              "stroke": "light" }
        ]
    }"#;

    /// A description whose only shape is a light `60×14` box at `(0, 0)`: larger than the window on
    /// both axes, so the view has room to travel.
    ///
    /// The window is `38×5`, so the content's span is `22` cells across and `9` down. The helper's
    /// step is a tenth of the window — `3` across, and `1` down because a tenth of five cells is
    /// nothing and the helper answers at least one.
    const A_BOX_TOO_BIG: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                      "size": { "width": 60, "height": 14 }, "stroke": "light" } ]
    }"#;

    /// Three boxes, at `(0, 0)`, `(27, 0)` and `(50, 0)`, the last two `3×3` and the first `4×3`.
    ///
    /// **The third gives the content its width**, so the view has `16` columns to travel, and the
    /// second is what a click after a scroll has to find: three columns across puts its corner at
    /// the window's `(24, 0)`, and the window's `(27, 0)` — what a click that added the origin would
    /// ask about — is a cell no shape wrote.
    const BOXES_APART: &str = r#"{
        "next_id": 4,
        "shapes": [
            { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
              "size": { "width": 4, "height": 3 }, "stroke": "light" },
            { "kind": "box", "id": 2, "at": { "x": 27, "y": 0 },
              "size": { "width": 3, "height": 3 }, "stroke": "light" },
            { "kind": "box", "id": 3, "at": { "x": 50, "y": 0 },
              "size": { "width": 3, "height": 3 }, "stroke": "light" }
        ]
    }"#;

    /// The state of an application over the diagram `json` describes.
    fn a_state_over(json: &str) -> AppState {
        let diagram =
            monospace_description::parse(json).expect("the test's description is readable");

        AppState::new(monospace_editing::Session::new(diagram))
    }

    /// The state of an application over the diagram `json` describes, with a frame painted.
    fn a_drawn_state_over(json: &str) -> AppState {
        let mut state = a_state_over(json);
        let _ = draw(&mut state);
        state
    }

    /// The event the terminal reports for a left press at the screen position `(column, row)`.
    fn click(column: u16, row: u16) -> Event {
        press(MouseButton::Left, column, row)
    }

    /// The event the terminal reports for a press of `button` at the screen position
    /// `(column, row)`.
    fn press(button: MouseButton, column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(button),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// The event the terminal reports for a turn of the wheel at the screen position
    /// `(column, row)`.
    fn wheel(kind: MouseEventKind, column: u16, row: u16) -> Event {
        wheel_with(kind, KeyModifiers::NONE, column, row)
    }

    /// The event the terminal reports for a turn of the wheel with `modifiers` held.
    fn wheel_with(kind: MouseEventKind, modifiers: KeyModifiers, column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers,
        })
    }

    /// The event the terminal reports for the pointer moving with the left button down.
    fn drag(column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// The event the terminal reports for the pointer moving to `(column, row)`.
    fn moved(column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// The state of an application that has been on the screen once.
    ///
    /// **The frame comes first**: the bar knows where its labels are only once it has been
    /// drawn, and a key that arrives before that finds nothing to open.
    fn a_drawn_state() -> AppState {
        let mut state = a_state();
        let _ = draw(&mut state);
        state
    }

    /// Paints a frame and hands back the buffer it was painted into.
    fn draw(state: &mut AppState) -> ratatui_core::buffer::Buffer {
        let area = Rect::new(0, 0, WIDTH, HEIGHT);
        let mut buffer = ratatui_core::buffer::Buffer::empty(area);
        frame::draw(area, &mut buffer, state, &a_global());
        buffer
    }

    /// The event the terminal reports for a key.
    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    /// The event the terminal reports for a key with modifiers.
    fn key_with(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    /// The event the terminal reports for `Ctrl+Z`.
    fn undo_key() -> Event {
        key_with(KeyCode::Char('z'), KeyModifiers::CONTROL)
    }

    /// The event the terminal reports for `Ctrl+Shift+Z`.
    ///
    /// **The shifted character with both modifiers**, which is what a terminal that reports the
    /// combination at all reports: a terminal that folds the shift into the control character
    /// reports `undo_key` instead, and D1 of #181 names what that costs.
    fn redo_key() -> Event {
        key_with(
            KeyCode::Char('Z'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        )
    }

    /// One row of a frame, as text.
    fn row_of(buffer: &Buffer, line: u16) -> String {
        (buffer.area.left()..buffer.area.right())
            .map(|column| buffer[(column, line)].symbol().to_string())
            .collect()
    }

    /// Whether the cell at a frame position is painted in reverse video.
    fn reversed(buffer: &Buffer, column: u16, line: u16) -> bool {
        buffer[(column, line)]
            .style()
            .add_modifier
            .contains(Modifier::REVERSED)
    }

    /// Hands an event to the application, the way the loop does.
    fn send(raw: &Event, state: &mut AppState) -> Control<AppEvent> {
        event(&AppEvent::Terminal(raw.clone()), state, &mut a_global())
            .expect("the event reached the application")
    }

    /// Opens the drop-down of `label` the way the region reaches the bar: `Alt` and the label's
    /// navigation character in one press (D4 of #189).
    fn open_the_drop_down(state: &mut AppState, label: char) {
        let outcome = send(&key_with(KeyCode::Char(label), KeyModifiers::ALT), state);
        let _ = draw(state);

        assert!(
            matches!(outcome, Control::Changed),
            "Alt+{label} did not owe a frame: {outcome:?}"
        );
        assert_eq!(
            state.chrome().leaf(),
            Focus::Bar,
            "Alt+{label} did not give the bar the keyboard"
        );
        assert!(
            state.chrome().popup_active(),
            "Alt+{label} did not open the drop-down"
        );
    }

    /// Takes the entry of `menu` whose text is `text`, by clicking it on the frame.
    ///
    /// **The entry is found by its name**: a menu that gains entries above one moves where it is
    /// but not what it says, and a test that pressed `Enter` would take whatever happens to be
    /// first. **The pointer moves onto it before the button goes down**, because the widget
    /// activates the entry it has selected and a press alone selects nothing.
    fn take_the_entry(state: &mut AppState, menu: char, text: &str) -> Control<AppEvent> {
        open_the_drop_down(state, menu);
        let buffer = draw(state);
        let (column, row) = frame::cell_of(&buffer, text);
        let _ = send(&moved(column, row), state);

        send(&click(column, row), state)
    }

    /// Opens the About dialog the way a person does: `Help > About` by pointer.
    ///
    /// **It also checks the keyboard goes back to the region** (rule 13): the dialog takes the
    /// whole screen while it is up, so the leaf is already where it will be given back, and every
    /// test that opens the dialog is checking that without saying so again.
    fn open_the_about_dialog(state: &mut AppState) {
        let outcome = take_the_entry(state, 'h', "About");

        assert!(
            matches!(outcome, Control::Changed),
            "taking About owed no frame: {outcome:?}"
        );
        assert!(state.dialog().is_up(), "taking About opened no dialog");
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "taking About left the keyboard on the bar"
        );
        assert!(
            !state.chrome().popup_active(),
            "taking About left the drop-down open"
        );
    }

    /// Opens the open dialog the way a person does: `File > Open` by pointer.
    ///
    /// **It also checks the keyboard goes back to the region** (rule 13): the dialog takes the
    /// whole screen while it is up, so the leaf is already where it will be given back.
    fn open_the_open_dialog(state: &mut AppState) {
        let outcome = take_the_entry(state, 'f', "Open");

        assert!(
            matches!(outcome, Control::Changed),
            "taking Open owed no frame: {outcome:?}"
        );
        assert!(state.open().is_up(), "taking Open opened no dialog");
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "taking Open left the keyboard on the bar"
        );
        assert!(
            !state.chrome().popup_active(),
            "taking Open left the drop-down open"
        );
    }

    /// Opens the save dialog the way a person does: `File > Save` by pointer.
    ///
    /// **It also checks the keyboard goes back to the region** (rule 13): the dialog takes the
    /// whole screen while it is up, so the leaf is already where it will be given back.
    fn open_the_save_dialog(state: &mut AppState) {
        let outcome = take_the_entry(state, 'f', "Save");

        assert!(
            matches!(outcome, Control::Changed),
            "taking Save owed no frame: {outcome:?}"
        );
        assert!(state.save().is_up(), "taking Save opened no dialog");
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "taking Save left the keyboard on the bar"
        );
        assert!(
            !state.chrome().popup_active(),
            "taking Save left the drop-down open"
        );
    }

    /// Opens the export dialog the way a person does: `File > Export as Text` by pointer.
    ///
    /// **It also checks the keyboard goes back to the region** (rule 13): the dialog takes the
    /// whole screen while it is up, so the leaf is already where it will be given back.
    fn open_the_export_dialog(state: &mut AppState) {
        let outcome = take_the_entry(state, 'f', "Export as Text");

        assert!(
            matches!(outcome, Control::Changed),
            "taking Export as Text owed no frame: {outcome:?}"
        );
        assert!(
            state.export().is_up(),
            "taking Export as Text opened no dialog"
        );
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "taking Export as Text left the keyboard on the bar"
        );
        assert!(
            !state.chrome().popup_active(),
            "taking Export as Text left the drop-down open"
        );
    }

    /// Writes `contents` to a fresh file under the system temp directory, named after the calling
    /// test so parallel tests never collide.
    fn write_description(name: &str, contents: &str) -> std::path::PathBuf {
        use std::io::Write as _;

        let path =
            std::env::temp_dir().join(format!("monospace-open-{name}-{}.json", std::process::id()));
        std::fs::File::create(&path)
            .and_then(|mut file| file.write_all(contents.as_bytes()))
            .expect("temp description file should be writable");
        path
    }

    /// **The chrome's own and no session** (rule 8): About opens an overlay and records no step,
    /// so the drawing and its history are exactly what they were.
    #[test]
    fn the_about_action_changes_no_session() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(
            selected.is_some(),
            "the box was not selected, so the test proves nothing"
        );

        open_the_about_dialog(&mut state);

        assert_eq!(
            state.diagram().selected(),
            selected,
            "the dialog changed the selection"
        );
        assert!(
            !state.diagram().can_undo(),
            "opening the dialog recorded a step"
        );
        assert!(
            !state.diagram().can_redo(),
            "opening the dialog left a step ahead"
        );
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_some(),
            "opening the dialog removed the box"
        );
    }

    /// **The error is the same overlay and no session** (rule 8): it opens the dialog over the
    /// text it carries, replaces the About it finds there, and records no step.
    #[test]
    fn showing_an_error_opens_the_dialog_over_its_text_and_changes_no_session() {
        let mut state = a_drawn_state_over(A_BOX);
        open_the_about_dialog(&mut state);

        apply(&Action::ShowError("Something failed".into()), &mut state);

        assert!(state.dialog().is_up(), "the error left the dialog down");

        let mut buffer = Buffer::empty(Rect::new(0, 0, 40, 10));
        crate::dialog::draw(
            Rect::new(0, 1, 40, 8),
            &mut buffer,
            state.dialog_mut(),
            &Config::default().theme,
        );
        let frame = crate::dialog::testing::picture(&buffer);
        assert!(
            frame.contains("Something failed"),
            "the dialog does not carry the error: {frame:?}"
        );
        assert!(
            !frame.contains("About"),
            "the error did not replace the About: {frame:?}"
        );
        assert!(
            !state.diagram().can_undo(),
            "showing the error recorded a step"
        );
    }

    /// **`Ctrl+N` and the File menu's first entry start a fresh session**, and both apply the same
    /// action.
    #[test]
    fn the_new_action_is_reached_by_the_shortcut_and_by_the_menus_entry() {
        // The shortcut.
        let mut state = a_drawn_state_over(A_BOX);
        let outcome = send(
            &key_with(KeyCode::Char('n'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().session().diagram().shapes().count(),
            0,
            "the shortcut did not reset to a blank diagram"
        );

        // The entry of the menu.
        let mut state = a_drawn_state_over(A_BOX);
        let outcome = take_the_entry(&mut state, 'f', "New");

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().session().diagram().shapes().count(),
            0,
            "the menu entry did not reset to a blank diagram"
        );
    }

    /// **`Ctrl+O` and the File menu's second entry open the same dialog**, and neither touches the
    /// session: the shortcut and the entry are one action.
    #[test]
    fn the_open_dialog_is_reached_by_the_shortcut_and_by_the_menus_entry() {
        // The shortcut.
        let mut state = a_drawn_state();
        let outcome = send(
            &key_with(KeyCode::Char('o'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(state.open().is_up(), "the shortcut did not open the dialog");

        // The entry of the menu.
        let mut state = a_drawn_state();
        open_the_open_dialog(&mut state);
        assert!(
            state.open().is_up(),
            "the menu entry did not open the dialog"
        );
    }

    /// **`Ctrl+S` and the File menu's third entry open the save dialog**, and neither touches the
    /// session: the shortcut and the entry are one action.
    #[test]
    fn the_save_dialog_is_reached_by_the_shortcut_and_by_the_menus_entry() {
        // The shortcut.
        let mut state = a_drawn_state();
        let outcome = send(
            &key_with(KeyCode::Char('s'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            state.save().is_up(),
            "the shortcut did not open the save dialog"
        );

        // The entry of the menu.
        let mut state = a_drawn_state();
        open_the_save_dialog(&mut state);
        assert!(
            state.save().is_up(),
            "the menu entry did not open the save dialog"
        );
    }

    /// **The closing event reaches nothing else** (rule 10): `Esc` closes the dialog, leaves the
    /// keyboard where the dialog was given it — the region, rule 13 — opens no drop-down, and
    /// changes neither the selection nor the history.
    #[test]
    fn escape_closes_the_about_dialog_and_the_event_reaches_nothing_else() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        open_the_about_dialog(&mut state);
        assert!(state.dialog().is_up(), "the dialog is not open");
        let focus = state.chrome().leaf();

        let outcome = send(&key(KeyCode::Esc), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.dialog().is_up(), "escape did not close the dialog");
        assert_eq!(
            state.chrome().leaf(),
            focus,
            "the closing event moved the keyboard"
        );
        assert!(
            !state.chrome().popup_active(),
            "the closing event opened a drop-down"
        );
        assert_eq!(
            state.diagram().selected(),
            selected,
            "the closing event changed the selection"
        );
        assert!(
            !state.diagram().can_undo(),
            "the closing event recorded a step"
        );
    }

    /// **The chrome's own and no session**: opening the open dialog records no step and changes
    /// no selection, so the drawing is exactly what it was.
    #[test]
    fn opening_the_open_dialog_changes_no_session() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(
            selected.is_some(),
            "the box was not selected, so the test proves nothing"
        );

        open_the_open_dialog(&mut state);

        assert_eq!(
            state.diagram().selected(),
            selected,
            "the dialog changed the selection"
        );
        assert!(
            !state.diagram().can_undo(),
            "opening the dialog recorded a step"
        );
        assert!(
            !state.diagram().can_redo(),
            "opening the dialog left a step ahead"
        );
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_some(),
            "opening the dialog removed the box"
        );
    }

    /// **The chrome's own and no session**: opening the save dialog records no step and changes
    /// no selection, so the drawing is exactly what it was.
    #[test]
    fn opening_the_save_dialog_changes_no_session() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(
            selected.is_some(),
            "the box was not selected, so the test proves nothing"
        );

        open_the_save_dialog(&mut state);

        assert_eq!(
            state.diagram().selected(),
            selected,
            "the dialog changed the selection"
        );
        assert!(
            !state.diagram().can_undo(),
            "opening the dialog recorded a step"
        );
        assert!(
            !state.diagram().can_redo(),
            "opening the dialog left a step ahead"
        );
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_some(),
            "opening the dialog removed the box"
        );
    }

    /// **The closing event reaches nothing else**: `Esc` closes the open dialog, leaves the
    /// keyboard on the region, opens no drop-down, and changes neither the selection nor history.
    #[test]
    fn escape_closes_the_open_dialog_and_the_event_reaches_nothing_else() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        open_the_open_dialog(&mut state);
        assert!(state.open().is_up(), "the open dialog is not open");
        let focus = state.chrome().leaf();

        let outcome = send(&key(KeyCode::Esc), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.open().is_up(),
            "escape did not close the open dialog"
        );
        assert_eq!(
            state.chrome().leaf(),
            focus,
            "the closing event moved the keyboard"
        );
        assert!(
            !state.chrome().popup_active(),
            "the closing event opened a drop-down"
        );
        assert_eq!(
            state.diagram().selected(),
            selected,
            "the closing event changed the selection"
        );
        assert!(
            !state.diagram().can_undo(),
            "the closing event recorded a step"
        );
    }

    /// **The closing event reaches nothing else**: `Esc` closes the save dialog, leaves the
    /// keyboard on the region, opens no drop-down, and changes neither the selection nor history.
    #[test]
    fn escape_closes_the_save_dialog_and_the_event_reaches_nothing_else() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        open_the_save_dialog(&mut state);
        assert!(state.save().is_up(), "the save dialog is not open");
        let focus = state.chrome().leaf();

        let outcome = send(&key(KeyCode::Esc), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.save().is_up(),
            "escape did not close the save dialog"
        );
        assert_eq!(
            state.chrome().leaf(),
            focus,
            "the closing event moved the keyboard"
        );
        assert!(
            !state.chrome().popup_active(),
            "the closing event opened a drop-down"
        );
        assert_eq!(
            state.diagram().selected(),
            selected,
            "the closing event changed the selection"
        );
        assert!(
            !state.diagram().can_undo(),
            "the closing event recorded a step"
        );
    }

    /// **The overlay takes the controls behind it**: `Ctrl+Q` does nothing while the open dialog
    /// is up, and a click on the bar behind it moves no keyboard and opens no menu.
    #[test]
    fn control_q_does_nothing_while_the_open_dialog_is_up() {
        let mut state = a_drawn_state_over(A_BOX);
        open_the_open_dialog(&mut state);
        let (column, row) = frame::cell_of(&draw(&mut state), "File");
        let focus = state.chrome().leaf();

        let outcome = send(
            &key_with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(
            matches!(outcome, Control::Continue),
            "the way out was answered through the open dialog: {outcome:?}"
        );
        assert!(state.open().is_up(), "Ctrl+Q closed the open dialog");

        let _ = send(&click(column, row), &mut state);

        assert_eq!(
            state.chrome().leaf(),
            focus,
            "the click on the bar moved the keyboard"
        );
        assert!(
            !state.chrome().popup_active(),
            "the click behind the dialog opened a drop-down"
        );
        assert!(
            state.open().is_up(),
            "the click on the bar closed the open dialog"
        );
    }

    /// **The overlay takes the controls behind it**: `Ctrl+Q` does nothing while the save dialog
    /// is up, and a click on the bar behind it moves no keyboard and opens no menu.
    #[test]
    fn control_q_does_nothing_while_the_save_dialog_is_up() {
        let mut state = a_drawn_state_over(A_BOX);
        open_the_save_dialog(&mut state);
        let (column, row) = frame::cell_of(&draw(&mut state), "File");
        let focus = state.chrome().leaf();

        let outcome = send(
            &key_with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(
            matches!(outcome, Control::Continue),
            "the way out was answered through the save dialog: {outcome:?}"
        );
        assert!(state.save().is_up(), "Ctrl+Q closed the save dialog");

        let _ = send(&click(column, row), &mut state);

        assert_eq!(
            state.chrome().leaf(),
            focus,
            "the click on the bar moved the keyboard"
        );
        assert!(
            !state.chrome().popup_active(),
            "the click behind the save dialog opened a drop-down"
        );
        assert!(
            state.save().is_up(),
            "the click on the bar closed the save dialog"
        );
    }

    /// **A file that loads starts a new session**: the diagram is the file's, the selection is
    /// gone, the history is empty, and no confirmation asked before the old one was discarded.
    #[test]
    fn a_file_that_loads_starts_a_new_session() {
        let path = write_description(
            "loaded",
            r#"{ "next_id": 2, "shapes": [ { "kind": "box", "id": 1, "at": { "x": 8, "y": 2 },
                                            "size": { "width": 3, "height": 2 }, "stroke": "light" } ] }"#,
        );

        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the box was not selected, so the test proves nothing"
        );
        let _ = send(&key(KeyCode::Delete), &mut state);
        assert!(
            state.diagram().can_undo(),
            "the removal recorded no step, so the test proves nothing"
        );

        open_the_open_dialog(&mut state);
        state
            .open_mut()
            .get_mut()
            .expect("the open dialog is up")
            .set_path(path.to_str().expect("the temp path is UTF-8"));

        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.open().is_up(), "the load left the dialog up");
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "the load moved the keyboard off the region"
        );
        assert_eq!(
            state.diagram().selected(),
            None,
            "the new session kept the old selection"
        );
        assert!(
            !state.diagram().can_undo(),
            "the new session kept the old history"
        );
        assert!(
            !state.diagram().can_redo(),
            "the new session kept the old history"
        );

        let _ = draw(&mut state);

        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_none(),
            "the old diagram is still there"
        );
        assert!(
            state.diagram().owner(Offset { x: 8, y: 2 }).is_some(),
            "the file's box is not there"
        );
    }

    /// **Taking New resets to an empty diagram immediately** (rule 4 of #273): the current
    /// session, selection, and undo history are discarded without confirmation.
    #[test]
    fn taking_new_discards_current_session_and_starts_blank() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the box was not selected, so the test proves nothing"
        );
        let _ = send(&key(KeyCode::Delete), &mut state);
        assert!(
            state.diagram().can_undo(),
            "the removal recorded no step, so the test proves nothing"
        );

        let outcome = take_the_entry(&mut state, 'f', "New");

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "taking New moved the keyboard off the region"
        );
        assert_eq!(
            state.diagram().selected(),
            None,
            "the new session kept the old selection"
        );
        assert!(
            !state.diagram().can_undo(),
            "the new session kept the old history"
        );
        assert!(
            !state.diagram().can_redo(),
            "the new session kept the old history"
        );

        let _ = draw(&mut state);

        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_none(),
            "the old diagram is still there"
        );
        assert_eq!(
            state.diagram().session().diagram().shapes().count(),
            0,
            "the blank diagram is not empty"
        );
    }

    /// **Taking Open Demo resets to the demonstration diagram immediately** (rule 5 of #273): the
    /// current session, selection, and undo history are discarded without confirmation.
    #[test]
    fn taking_open_demo_discards_current_session_and_loads_demo() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the box was not selected, so the test proves nothing"
        );
        let _ = send(&key(KeyCode::Delete), &mut state);
        assert!(
            state.diagram().can_undo(),
            "the removal recorded no step, so the test proves nothing"
        );

        let outcome = take_the_entry(&mut state, 'f', "Open Demo");

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "taking Open Demo moved the keyboard off the region"
        );
        assert_eq!(
            state.diagram().selected(),
            None,
            "the new session kept the old selection"
        );
        assert!(
            !state.diagram().can_undo(),
            "the new session kept the old history"
        );
        assert!(
            !state.diagram().can_redo(),
            "the new session kept the old history"
        );

        let _ = draw(&mut state);

        assert!(
            state.diagram().session().diagram().shapes().count() > 0,
            "the demonstration has no shapes"
        );
    }

    /// **Taking New or Open Demo leaves the keyboard on the region** (rule 6 of #273), and closes
    /// the drop-down.
    #[test]
    fn taking_new_or_open_demo_leaves_the_keyboard_on_the_region() {
        // Taking New from the menu.
        let mut state = a_drawn_state_over(A_BOX);
        let _ = take_the_entry(&mut state, 'f', "New");
        assert_eq!(state.chrome().leaf(), Focus::Region);
        assert!(!state.chrome().popup_active());

        // Pressing Ctrl+N with drop-down open.
        let mut state = a_drawn_state_over(A_BOX);
        open_the_drop_down(&mut state, 'f');
        assert_eq!(state.chrome().leaf(), Focus::Bar);
        let _ = send(
            &key_with(KeyCode::Char('n'), KeyModifiers::CONTROL),
            &mut state,
        );
        assert_eq!(state.chrome().leaf(), Focus::Region);
        assert!(!state.chrome().popup_active());

        // Taking Open Demo from the menu.
        let mut state = a_drawn_state_over(A_BOX);
        let _ = take_the_entry(&mut state, 'f', "Open Demo");
        assert_eq!(state.chrome().leaf(), Focus::Region);
        assert!(!state.chrome().popup_active());
    }

    /// **A missing file closes the open dialog and opens the error dialog**: the session is
    /// untouched and the failure names the missing file in the message dialog.
    #[test]
    fn a_missing_file_closes_the_open_dialog_and_opens_the_error_dialog() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();

        open_the_open_dialog(&mut state);
        state
            .open_mut()
            .get_mut()
            .expect("the open dialog is up")
            .set_path("missing.json");

        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.open().is_up(), "the failure left the open dialog up");
        assert!(
            state.dialog().is_up(),
            "the failure did not open the error dialog"
        );
        assert_eq!(
            state.diagram().selected(),
            selected,
            "the failure changed the selection"
        );
        assert!(!state.diagram().can_undo(), "the failure recorded a step");
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_some(),
            "the failure removed the box"
        );
    }

    /// **A file that does not parse closes the open dialog and opens the error dialog**: the
    /// session is untouched and the failure names the file in the message dialog.
    #[test]
    fn a_file_that_does_not_parse_closes_the_open_dialog_and_opens_the_error_dialog() {
        let path = write_description("bad", "not json");
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();

        open_the_open_dialog(&mut state);
        state
            .open_mut()
            .get_mut()
            .expect("the open dialog is up")
            .set_path(path.to_str().expect("the temp path is UTF-8"));

        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.open().is_up(), "the failure left the open dialog up");
        assert!(
            state.dialog().is_up(),
            "the failure did not open the error dialog"
        );
        assert_eq!(
            state.diagram().selected(),
            selected,
            "the failure changed the selection"
        );
        assert!(!state.diagram().can_undo(), "the failure recorded a step");
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_some(),
            "the failure removed the box"
        );
    }

    /// **A diagram is saved to a file and overwrites existing content**: saving serializes the
    /// current diagram to the path, creating or replacing the file.
    #[test]
    fn a_diagram_is_saved_to_a_file_and_overwrites_existing_content() {
        let path = std::env::temp_dir().join(format!(
            "monospace-save-overwrite-{}.json",
            std::process::id()
        ));
        let path_str = path.to_str().expect("temp path is UTF-8");
        std::fs::write(&path, "old content").expect("temp file is writable");

        let mut state = a_drawn_state_over(A_BOX);
        open_the_save_dialog(&mut state);
        state
            .save_mut()
            .get_mut()
            .expect("the save dialog is up")
            .set_path(path_str);

        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.save().is_up(), "saving left the save dialog up");
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "saving moved the keyboard off the region"
        );

        let read = std::fs::read_to_string(&path).expect("saved file can be read");
        assert_ne!(
            read, "old content",
            "saving did not overwrite existing content"
        );
        let parsed = monospace_description::parse(&read).expect("saved JSON is valid description");
        assert_eq!(
            parsed.shapes().count(),
            1,
            "saved diagram is missing the box"
        );

        let _ = std::fs::remove_file(path);
    }

    /// **Saving leaves the current session and history intact** (rule 6 of #270): the file is
    /// written, but the current diagram, selection, and undo/redo stacks remain unchanged.
    #[test]
    fn saving_leaves_the_current_session_and_history_intact() {
        let path =
            std::env::temp_dir().join(format!("monospace-save-intact-{}.json", std::process::id()));
        let path_str = path.to_str().expect("temp path is UTF-8");

        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(selected.is_some(), "the box was not selected");

        // Perform a modification to create history
        let _ = send(&key(KeyCode::Right), &mut state);
        assert!(state.diagram().can_undo(), "movement created no undo step");

        open_the_save_dialog(&mut state);
        state
            .save_mut()
            .get_mut()
            .expect("the save dialog is up")
            .set_path(path_str);

        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.save().is_up(), "saving left the save dialog up");
        assert_eq!(
            state.diagram().selected(),
            selected,
            "saving changed the selection"
        );
        assert!(state.diagram().can_undo(), "saving dropped the undo stack");
        assert!(!state.diagram().can_redo(), "saving modified redo stack");

        let _ = std::fs::remove_file(path);
    }

    /// **An empty path closes the save dialog and opens the error dialog** (rule 7 of #270): the
    /// session is untouched and the failure reports `": a file name is needed"`.
    #[test]
    fn an_empty_path_closes_the_save_dialog_and_opens_the_error_dialog() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();

        open_the_save_dialog(&mut state);
        assert_eq!(
            state.save().get().expect("the save dialog is up").path(),
            ""
        );

        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.save().is_up(), "the failure left the save dialog up");
        assert!(
            state.dialog().is_up(),
            "the failure did not open the error dialog"
        );
        assert_eq!(
            state.diagram().selected(),
            selected,
            "the failure changed the selection"
        );
        assert!(!state.diagram().can_undo(), "the failure recorded a step");

        let area = Rect::new(0, 0, WIDTH, 12);
        let mut buffer = Buffer::empty(area);
        frame::draw(area, &mut buffer, &mut state, &a_global());
        let shown: String = (0..12).map(|line| row_of(&buffer, line)).collect();
        assert!(
            shown.contains(": a file name is needed"),
            "the error dialog does not report the empty path message: {shown:?}"
        );
    }

    /// **A path in a missing directory closes the save dialog and opens the error dialog**
    /// (rule 7 of #270): the session is untouched and the failure names the missing directory.
    #[test]
    fn a_path_in_a_missing_directory_closes_the_save_dialog_and_opens_the_error_dialog() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();

        open_the_save_dialog(&mut state);
        let missing_path = "nonexistent_dir_12345/saved.json";
        state
            .save_mut()
            .get_mut()
            .expect("the save dialog is up")
            .set_path(missing_path);

        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.save().is_up(), "the failure left the save dialog up");
        assert!(
            state.dialog().is_up(),
            "the failure did not open the error dialog"
        );
        assert_eq!(
            state.diagram().selected(),
            selected,
            "the failure changed the selection"
        );
        assert!(!state.diagram().can_undo(), "the failure recorded a step");

        let area = Rect::new(0, 0, WIDTH, 12);
        let mut buffer = Buffer::empty(area);
        frame::draw(area, &mut buffer, &mut state, &a_global());
        let shown: String = (0..12).map(|line| row_of(&buffer, line)).collect();
        assert!(
            shown.contains("nonexistent_dir_12345"),
            "the error dialog does not name the missing directory: {shown:?}"
        );
    }

    /// **The chrome's own and no session**: opening the export dialog records no step and changes
    /// no selection, so the drawing is exactly what it was.
    #[test]
    fn opening_the_export_dialog_changes_no_session() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(
            selected.is_some(),
            "the box was not selected, so the test proves nothing"
        );

        open_the_export_dialog(&mut state);

        assert_eq!(
            state.diagram().selected(),
            selected,
            "the dialog changed the selection"
        );
        assert!(
            !state.diagram().can_undo(),
            "opening the dialog recorded a step"
        );
        assert!(
            !state.diagram().can_redo(),
            "opening the dialog left a step ahead"
        );
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_some(),
            "opening the dialog removed the box"
        );
    }

    /// **The closing event reaches nothing else**: `Esc` closes the export dialog, leaves the
    /// keyboard on the region, opens no drop-down, and changes neither the selection nor history.
    #[test]
    fn escape_closes_the_export_dialog_and_the_event_reaches_nothing_else() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        open_the_export_dialog(&mut state);
        assert!(state.export().is_up(), "the export dialog is not open");
        let focus = state.chrome().leaf();

        let outcome = send(&key(KeyCode::Esc), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.export().is_up(),
            "escape did not close the export dialog"
        );
        assert_eq!(
            state.chrome().leaf(),
            focus,
            "the closing event moved the keyboard"
        );
        assert!(
            !state.chrome().popup_active(),
            "the closing event opened a drop-down"
        );
        assert_eq!(
            state.diagram().selected(),
            selected,
            "the closing event changed the selection"
        );
        assert!(
            !state.diagram().can_undo(),
            "the closing event recorded a step"
        );
    }

    /// **The overlay takes the controls behind it**: `Ctrl+Q` does nothing while the export dialog
    /// is up, and a click on the bar behind it moves no keyboard and opens no menu.
    #[test]
    fn control_q_does_nothing_while_the_export_dialog_is_up() {
        let mut state = a_drawn_state_over(A_BOX);
        open_the_export_dialog(&mut state);
        let (column, row) = frame::cell_of(&draw(&mut state), "File");
        let focus = state.chrome().leaf();

        let outcome = send(
            &key_with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(
            matches!(outcome, Control::Continue),
            "the way out was answered through the export dialog: {outcome:?}"
        );
        assert!(state.export().is_up(), "Ctrl+Q closed the export dialog");

        let _ = send(&click(column, row), &mut state);

        assert_eq!(
            state.chrome().leaf(),
            focus,
            "the click on the bar moved the keyboard"
        );
        assert!(
            !state.chrome().popup_active(),
            "the click behind the export dialog opened a drop-down"
        );
        assert!(
            state.export().is_up(),
            "the click on the bar closed the export dialog"
        );
    }

    /// **A diagram is exported as text over its footprint and overwrites existing content** (rule
    /// 6 of #352): exporting renders the tight footprint of the current diagram to the path,
    /// creating or replacing the file.
    #[test]
    fn a_diagram_is_exported_as_text_over_its_footprint_and_overwrites_existing_content() {
        let path = std::env::temp_dir().join(format!(
            "monospace-export-overwrite-{}.txt",
            std::process::id()
        ));
        let path_str = path.to_str().expect("temp path is UTF-8");
        std::fs::write(&path, "old content").expect("temp file is writable");

        let mut state = a_drawn_state_over(A_BOX);
        open_the_export_dialog(&mut state);
        state
            .export_mut()
            .get_mut()
            .expect("the export dialog is up")
            .set_path(path_str);

        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.export().is_up(),
            "exporting left the export dialog up"
        );
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "exporting moved the keyboard off the region"
        );

        let read = std::fs::read_to_string(&path).expect("exported file can be read");
        assert_eq!(
            read, "┌──┐\n│  │\n└──┘\n",
            "exported text did not match the 4x3 box footprint"
        );

        let _ = std::fs::remove_file(path);
    }

    /// **An empty diagram exports an empty file** (rule 6 of #352): when nothing is drawn, the
    /// footprint is `None` and the exported file is `""` (0 bytes).
    #[test]
    fn an_empty_diagram_exports_an_empty_file() {
        let path =
            std::env::temp_dir().join(format!("monospace-export-empty-{}.txt", std::process::id()));
        let path_str = path.to_str().expect("temp path is UTF-8");
        std::fs::write(&path, "old content").expect("temp file is writable");

        let mut state = a_drawn_state_over(
            r#"{
                "next_id": 1,
                "shapes": []
            }"#,
        );
        open_the_export_dialog(&mut state);
        state
            .export_mut()
            .get_mut()
            .expect("the export dialog is up")
            .set_path(path_str);

        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.export().is_up(),
            "exporting left the export dialog up"
        );

        let read = std::fs::read_to_string(&path).expect("exported file can be read");
        assert_eq!(read, "", "empty diagram did not export an empty file");

        let _ = std::fs::remove_file(path);
    }

    /// **Exporting leaves the current session and history intact** (rule 6 of #352): the file is
    /// written, but the current diagram, selection, and undo/redo stacks remain unchanged.
    #[test]
    fn exporting_leaves_the_current_session_and_history_intact() {
        let path = std::env::temp_dir().join(format!(
            "monospace-export-intact-{}.txt",
            std::process::id()
        ));
        let path_str = path.to_str().expect("temp path is UTF-8");

        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(selected.is_some(), "the box was not selected");

        let _ = send(&key(KeyCode::Right), &mut state);
        assert!(state.diagram().can_undo(), "movement created no undo step");

        open_the_export_dialog(&mut state);
        state
            .export_mut()
            .get_mut()
            .expect("the export dialog is up")
            .set_path(path_str);

        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.export().is_up(),
            "exporting left the export dialog up"
        );
        assert_eq!(
            state.diagram().selected(),
            selected,
            "exporting changed the selection"
        );
        assert!(
            state.diagram().can_undo(),
            "exporting dropped the undo stack"
        );
        assert!(!state.diagram().can_redo(), "exporting modified redo stack");

        let _ = std::fs::remove_file(path);
    }

    /// **An empty path closes the export dialog and opens the error dialog** (rule 7 of #352): the
    /// session is untouched and the failure reports `": a file name is needed"`.
    #[test]
    fn an_empty_path_closes_the_export_dialog_and_opens_the_error_dialog() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();

        open_the_export_dialog(&mut state);
        assert_eq!(
            state
                .export()
                .get()
                .expect("the export dialog is up")
                .path(),
            ""
        );

        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.export().is_up(),
            "the failure left the export dialog up"
        );
        assert!(
            state.dialog().is_up(),
            "the failure did not open the error dialog"
        );
        assert_eq!(
            state.diagram().selected(),
            selected,
            "the failure changed the selection"
        );
        assert!(!state.diagram().can_undo(), "the failure recorded a step");

        let area = Rect::new(0, 0, WIDTH, 12);
        let mut buffer = Buffer::empty(area);
        frame::draw(area, &mut buffer, &mut state, &a_global());
        let shown: String = (0..12).map(|line| row_of(&buffer, line)).collect();
        assert!(
            shown.contains(": a file name is needed"),
            "the error dialog does not report the empty path message: {shown:?}"
        );
    }

    /// **A path in a missing directory closes the export dialog and opens the error dialog**
    /// (rule 7 of #352): the session is untouched and the failure names the missing directory.
    #[test]
    fn a_path_in_a_missing_directory_closes_the_export_dialog_and_opens_the_error_dialog() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();

        open_the_export_dialog(&mut state);
        let missing_path = "nonexistent_dir_12345/exported.txt";
        state
            .export_mut()
            .get_mut()
            .expect("the export dialog is up")
            .set_path(missing_path);

        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.export().is_up(),
            "the failure left the export dialog up"
        );
        assert!(
            state.dialog().is_up(),
            "the failure did not open the error dialog"
        );
        assert_eq!(
            state.diagram().selected(),
            selected,
            "the failure changed the selection"
        );
        assert!(!state.diagram().can_undo(), "the failure recorded a step");

        let area = Rect::new(0, 0, WIDTH, 12);
        let mut buffer = Buffer::empty(area);
        frame::draw(area, &mut buffer, &mut state, &a_global());
        let shown: String = (0..12).map(|line| row_of(&buffer, line)).collect();
        assert!(
            shown.contains("nonexistent_dir_12345"),
            "the error dialog does not name the missing directory: {shown:?}"
        );
    }

    /// **The overlay takes the controls behind it** (rule 10): `Ctrl+Q` is `Unclaimed`, so while
    /// the dialog is up the dialog answers it and nothing happens, and a click on the bar behind
    /// it moves no keyboard and opens no menu. `Ctrl+Shift+Q` is answered before the dialog, and
    /// [`the_forced_exit_ends_the_application_whatever_the_state_is`] is where that is proven.
    #[test]
    fn control_q_does_nothing_while_the_dialog_is_up() {
        let mut state = a_drawn_state_over(A_BOX);
        open_the_about_dialog(&mut state);
        let (column, row) = frame::cell_of(&draw(&mut state), "File");
        let focus = state.chrome().leaf();

        let outcome = send(
            &key_with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(
            matches!(outcome, Control::Continue),
            "the way out was answered through the dialog: {outcome:?}"
        );
        assert!(state.dialog().is_up(), "Ctrl+Q closed the dialog");

        // The bar is behind the overlay even where the dialog does not cover it.
        let _ = send(&click(column, row), &mut state);

        assert_eq!(
            state.chrome().leaf(),
            focus,
            "the click on the bar moved the keyboard"
        );
        assert!(
            !state.chrome().popup_active(),
            "the click behind the dialog opened a drop-down"
        );
        assert!(
            state.dialog().is_up(),
            "the click on the bar closed the dialog"
        );
    }

    /// **Closing leaves the region byte-identical** (rule 11): the dialog is drawn over it and
    /// taken away, and every cell it covered is what it was, the selection's inversion included.
    #[test]
    fn the_region_is_byte_identical_after_the_dialog_closes() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let before = draw(&mut state);

        open_the_about_dialog(&mut state);
        let during = draw(&mut state);
        let shown: String = (0..HEIGHT).map(|line| row_of(&during, line)).collect();
        assert!(
            shown.contains(&format!("About {}", status::NAME)),
            "the dialog is not over the region: {shown:?}"
        );

        let outcome = send(&key(KeyCode::Esc), &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");

        let after = draw(&mut state);
        let region = frame::regions(Rect::new(0, 0, WIDTH, HEIGHT)).region;

        for line in region.top()..region.bottom() {
            for column in region.left()..region.right() {
                assert_eq!(
                    before[(column, line)].symbol(),
                    after[(column, line)].symbol(),
                    "({column}, {line}) is not what it was"
                );
                assert_eq!(
                    before[(column, line)].style(),
                    after[(column, line)].style(),
                    "({column}, {line}) is painted differently"
                );
            }
        }
    }

    #[test]
    fn the_way_out_is_reached_by_the_shortcut_and_by_the_menus_entry() {
        // The shortcut.
        let mut state = a_drawn_state();
        let outcome = send(
            &key_with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(matches!(outcome, Control::Quit), "{outcome:?}");

        // The entry of the menu.
        let mut state = a_drawn_state();
        let outcome = take_the_entry(&mut state, 'f', "Quit");

        assert!(matches!(outcome, Control::Quit), "{outcome:?}");
    }

    #[test]
    fn a_click_on_the_entry_and_its_shortcut_apply_the_same_action() {
        // The shortcut.
        let mut state = a_drawn_state();
        let by_shortcut = send(
            &key_with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut state,
        );

        // The click on the entry of the menu.
        let mut state = a_drawn_state();
        let by_click = take_the_entry(&mut state, 'f', "Quit");

        assert_eq!(
            by_click, by_shortcut,
            "the click and the shortcut are two code paths"
        );
    }

    #[test]
    fn the_forced_exit_ends_the_application_whatever_the_state_is() {
        // The drop-down is open: the chrome holds the keyboard, and the forced exit is not the
        // chrome's to answer.
        let mut state = a_drawn_state();
        open_the_drop_down(&mut state, 'f');

        let outcome = send(
            &key_with(
                KeyCode::Char('Q'),
                KeyModifiers::CONTROL.union(KeyModifiers::SHIFT),
            ),
            &mut state,
        );

        assert!(matches!(outcome, Control::Quit), "{outcome:?}");

        // The About dialog is up: the overlay takes every control but this one, which is
        // answered before it (D6 of #189).
        let mut state = a_drawn_state();
        open_the_about_dialog(&mut state);
        assert!(state.dialog().is_up(), "the dialog is not open");

        let outcome = send(
            &key_with(
                KeyCode::Char('Q'),
                KeyModifiers::CONTROL.union(KeyModifiers::SHIFT),
            ),
            &mut state,
        );

        assert!(
            matches!(outcome, Control::Quit),
            "the dialog swallowed the forced exit: {outcome:?}"
        );

        // The save dialog is up: the overlay takes every control but this one.
        let mut state = a_drawn_state();
        open_the_save_dialog(&mut state);
        assert!(state.save().is_up(), "the save dialog is not open");

        let outcome = send(
            &key_with(
                KeyCode::Char('Q'),
                KeyModifiers::CONTROL.union(KeyModifiers::SHIFT),
            ),
            &mut state,
        );

        assert!(
            matches!(outcome, Control::Quit),
            "the save dialog swallowed the forced exit: {outcome:?}"
        );

        // The export dialog is up: the overlay takes every control but this one.
        let mut state = a_drawn_state();
        open_the_export_dialog(&mut state);
        assert!(state.export().is_up(), "the export dialog is not open");

        let outcome = send(
            &key_with(
                KeyCode::Char('Q'),
                KeyModifiers::CONTROL.union(KeyModifiers::SHIFT),
            ),
            &mut state,
        );

        assert!(
            matches!(outcome, Control::Quit),
            "the export dialog swallowed the forced exit: {outcome:?}"
        );

        // The text editor is up: the overlay takes every control but this one.
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Char('t')), &mut state);
        assert!(state.text().is_up(), "the text editor is not open");

        let outcome = send(
            &key_with(
                KeyCode::Char('Q'),
                KeyModifiers::CONTROL.union(KeyModifiers::SHIFT),
            ),
            &mut state,
        );

        assert!(
            matches!(outcome, Control::Quit),
            "the text editor swallowed the forced exit: {outcome:?}"
        );
    }

    /// **Every shortcut of the first reach, and not the one that happens to be in the table
    /// today.** The promise is that nothing may consume these keys, and a test that named
    /// `Ctrl+Shift+Q` would keep passing the day a second one is added and the chrome swallows it.
    #[test]
    fn every_shortcut_answered_first_is_answered_with_the_drop_down_open() {
        for event in shortcuts::events_answered_first() {
            let mut state = a_drawn_state();
            open_the_drop_down(&mut state, 'f');

            let outcome = send(&event, &mut state);

            assert!(
                matches!(outcome, Control::Quit),
                "{event:?} was swallowed by the chrome: {outcome:?}"
            );
        }
    }

    /// **A key the table does not name is not an action**, and an action is the only thing the
    /// application's handling produces. What the chrome does with its own keys is the widget's
    /// state rather than the application's, and this is the application's half of it.
    #[test]
    fn a_key_with_no_action_changes_nothing() {
        let mut state = a_drawn_state();

        let outcome = send(&key(KeyCode::Char('z')), &mut state);

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert!(
            !state.chrome().popup_active(),
            "a key with no action changed the state"
        );
    }

    /// **A click asks the drawing rather than a drawing of its own**, and the identity it finds is
    /// what the state holds afterwards.
    #[test]
    fn a_left_click_selects_the_shape_that_owns_the_cell() {
        let mut state = a_drawn_state_over(A_BOX);
        let owner = state.diagram().owner(Offset { x: 2, y: 1 });
        assert!(owner.is_some(), "the box is not in the drawing");

        // The screen's `(3, 3)` is the window's `(2, 1)`: the region's interior starts one row
        // and one column in from the region's corner.
        let outcome = send(&click(3, 3), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().selected(),
            owner,
            "the click selected another shape"
        );
    }

    /// **A click on a cell nobody owns is the way out of a selection** (D3): the empty space is the
    /// target that always means nothing. A second click there has nothing left to drop.
    #[test]
    fn a_click_on_a_cell_nobody_owns_drops_the_selection() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the box was not selected"
        );

        // The window's own corner: the interior, and no shape wrote it.
        let outcome = send(&click(1, 2), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().selected(),
            None,
            "the selection survived a blank cell"
        );

        let outcome = send(&click(1, 2), &mut state);

        assert!(
            matches!(outcome, Control::Continue),
            "there was nothing to drop and the frame was owed: {outcome:?}"
        );
    }

    /// **The region is where the keyboard starts** (D1 of #189): the diagram is what a person came
    /// to work on, so the bar does not promise a menu before a drawing.
    #[test]
    fn the_diagram_region_holds_the_keyboard_at_startup() {
        let state = a_state();

        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "the bar started with the keyboard"
        );
    }

    /// **A left press inside the region's rectangle gives it the keyboard, and inside the interior
    /// it selects as well.** The bar held it first, so the press is what moves it.
    #[test]
    fn a_press_inside_the_region_moves_the_keyboard_and_selects() {
        let mut state = a_drawn_state_over(A_BOX);
        state.chrome_mut().move_to_bar();
        let _ = draw(&mut state);

        let outcome = send(&click(3, 3), &mut state);

        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "the press left the keyboard on the bar"
        );
        assert!(
            state.diagram().selected().is_some(),
            "the press inside the interior selected nothing"
        );
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
    }

    /// **The border is the region's without being the window's**: a press there gives it the
    /// keyboard and selects nothing, and a selection already made survives.
    #[test]
    fn a_press_on_the_border_moves_the_keyboard_and_selects_nothing() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(
            selected.is_some(),
            "the box was not selected, so the test proves nothing"
        );

        for (column, row, what) in [
            (3, 1, "the top border"),
            (0, 3, "the left border"),
            (WIDTH - 1, 3, "the right border"),
            (3, HEIGHT - 3, "the bottom border"),
        ] {
            state.chrome_mut().move_to_bar();
            let _ = draw(&mut state);

            let outcome = send(&click(column, row), &mut state);

            assert_eq!(
                state.chrome().leaf(),
                Focus::Region,
                "{what} did not give the region the keyboard"
            );
            assert_eq!(
                state.diagram().selected(),
                selected,
                "{what} changed the selection"
            );
            assert!(matches!(outcome, Control::Changed), "{what}: {outcome:?}");
        }
    }

    /// **The status line is not a leaf**: a press there moves the keyboard nowhere and owes no
    /// frame.
    #[test]
    fn a_click_on_the_status_bar_moves_nothing() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        state.chrome_mut().move_to_bar();
        let _ = draw(&mut state);

        let outcome = send(&click(3, HEIGHT - 1), &mut state);

        assert_eq!(
            state.chrome().leaf(),
            Focus::Bar,
            "the status line took the keyboard"
        );
        assert_eq!(
            state.diagram().selected(),
            selected,
            "the status line changed the selection"
        );
        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
    }

    /// **A press on a label gives the bar the keyboard and opens that label's drop-down**, which is
    /// what lets a person work by pointer without reaching for `Alt` first.
    #[test]
    fn a_press_on_a_label_moves_the_keyboard_and_opens_its_drop_down() {
        let mut state = a_drawn_state_over(A_BOX);
        let buffer = draw(&mut state);
        let (column, row) = frame::cell_of(&buffer, "File");

        let outcome = send(&click(column, row), &mut state);

        assert_eq!(
            state.chrome().leaf(),
            Focus::Bar,
            "the label did not give the bar the keyboard"
        );
        assert!(
            state.chrome().popup_active(),
            "the label did not open its drop-down"
        );
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
    }

    /// **A press in the region is not swallowed by an open drop-down** (D6): it moves the keyboard
    /// to the region, closes the drop-down on the same press, and selects inside the interior.
    #[test]
    fn a_press_in_the_region_closes_the_drop_down_and_selects() {
        let mut state = a_drawn_state_over(A_BOX_AND_WHAT_HANGS_FROM_IT);
        open_the_drop_down(&mut state, 'h');
        assert!(state.chrome().popup_active(), "the drop-down did not open");

        // The left box's top-left is outside the Help drop-down, so the press is the region's.
        let outcome = send(&click(1, 3), &mut state);

        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "the press left the keyboard on the bar; the drop-down is {:?}",
            state.chrome().menu().popup.area
        );
        assert!(
            !state.chrome().popup_active(),
            "the drop-down survived the keyboard leaving it"
        );
        assert!(
            state.diagram().selected().is_some(),
            "the press inside the interior selected nothing"
        );
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
    }

    /// **The arrows are not the bar's while the region holds the keyboard** (rule 5): they open no
    /// drop-down and move no label, so #179 and #176 can bind them from here. With nothing selected,
    /// each answers `Continue` because the arrow is ignored by the region.
    #[test]
    fn the_arrows_are_not_the_bars_while_the_region_holds_the_keyboard() {
        let mut state = a_drawn_state();
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "the bar started with the keyboard"
        );

        for arrow in [KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down] {
            let selected = state.chrome().menu().bar.selected;
            let outcome = send(&key(arrow), &mut state);

            assert!(
                matches!(outcome, Control::Continue),
                "{arrow:?}: {outcome:?}"
            );
            assert!(
                !state.chrome().popup_active(),
                "{arrow:?} opened a drop-down"
            );
            assert_eq!(
                state.chrome().menu().bar.selected,
                selected,
                "{arrow:?} moved the bar's label"
            );
        }
    }

    /// **The right arrow displaces the selected figure one cell** (rule 1): one command, one step,
    /// and the next frame draws the box where the move put it.
    #[test]
    fn the_right_arrow_moves_the_selected_shape_one_cell() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the box was not selected"
        );
        let before = draw(&mut state);

        let outcome = send(&key(KeyCode::Right), &mut state);
        let after = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(state.diagram().can_undo(), "the move recorded no step");
        assert_eq!(
            state.diagram().owner(Offset { x: 3, y: 1 }),
            state.diagram().selected(),
            "the drawing does not record the box one cell right"
        );
        assert_eq!(
            before[(3, 3)].symbol(),
            "┌",
            "the test's frame does not start where it says"
        );
        assert_eq!(
            after[(4, 3)].symbol(),
            "┌",
            "the frame does not draw the box one cell right"
        );
        assert_eq!(
            after[(3, 3)].symbol(),
            " ",
            "the frame still draws the box where it stood"
        );
    }

    /// **Each arrow moves along its own axis** (rule 1): Right, Left, Up and Down displace the
    /// figure by the sign their axis names and no other.
    #[test]
    fn each_arrow_moves_along_its_own_axis() {
        for (arrow, at) in [
            (KeyCode::Right, Offset { x: 3, y: 1 }),
            (KeyCode::Left, Offset { x: 1, y: 1 }),
            (KeyCode::Up, Offset { x: 2, y: 0 }),
            (KeyCode::Down, Offset { x: 2, y: 2 }),
        ] {
            let mut state = a_drawn_state_over(A_BOX);
            let _ = send(&click(3, 3), &mut state);
            assert!(
                state.diagram().selected().is_some(),
                "{arrow:?}: the box was not selected"
            );

            let outcome = send(&key(arrow), &mut state);
            let _ = draw(&mut state);

            assert!(
                matches!(outcome, Control::Changed),
                "{arrow:?}: {outcome:?}"
            );
            assert_eq!(
                state.diagram().owner(at),
                state.diagram().selected(),
                "{arrow:?} did not move the box along its own axis"
            );
            assert!(state.diagram().can_undo(), "{arrow:?} recorded no step");
        }
    }

    /// **Each press is its own step** (rule 2): two presses are two commands, and the undo gives
    /// back one cell per press rather than the whole walk.
    #[test]
    fn two_presses_are_two_steps() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);

        let _ = send(&key(KeyCode::Right), &mut state);
        let _ = send(&key(KeyCode::Right), &mut state);
        let _ = draw(&mut state);

        assert!(
            state.diagram().owner(Offset { x: 4, y: 1 }).is_some(),
            "the two presses did not move the box twice"
        );
        assert_eq!(
            state.diagram().owner(Offset { x: 3, y: 1 }),
            None,
            "the second press moved nothing"
        );

        let _ = send(&undo_key(), &mut state);
        let _ = draw(&mut state);

        assert!(
            state.diagram().owner(Offset { x: 3, y: 1 }).is_some(),
            "one undo did not give back one press"
        );

        let _ = send(&undo_key(), &mut state);
        let _ = draw(&mut state);

        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_some(),
            "the second undo did not give back the first press"
        );
        assert!(
            !state.diagram().can_undo(),
            "two presses were more than two steps"
        );
    }

    /// **The selection survives the move** (rule 3): the next frame inverts the cells the figure
    /// now owns rather than the ones it left.
    #[test]
    fn a_move_keeps_the_selection() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(selected.is_some(), "the box was not selected");

        let _ = send(&key(KeyCode::Right), &mut state);
        let buffer = draw(&mut state);

        assert_eq!(
            state.diagram().selected(),
            selected,
            "the move dropped the selection"
        );
        assert!(
            reversed(&buffer, 4, 3),
            "the frame does not invert the cells the box now owns"
        );
        assert!(
            !reversed(&buffer, 3, 3),
            "the frame still inverts the cell the box left"
        );
    }

    /// **The selection survives the resize**: applying `Command::Resize` keeps the selected shape.
    #[test]
    fn an_action_resize_resizes_the_selected_shape_and_keeps_selection() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(selected.is_some(), "the box was not selected");

        apply(
            &Action::Command(Command::Resize {
                id: selected.unwrap(),
                by: (1, 0).into(),
            }),
            &mut state,
        );

        assert_eq!(state.diagram().selected(), selected);
        assert!(state.diagram().can_undo());
        let shape = state.diagram().session().diagram().get(selected.unwrap());
        let Some(monospace_diagram::Shape::Box { size, .. }) = shape else {
            panic!("expected box");
        };
        assert_eq!(size.width, 5);
    }

    /// **Shift plus an arrow resizes the selected box by one cell** (rule 1 of #284).
    #[test]
    fn shift_arrows_resize_the_selected_box_by_one_cell() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(selected.is_some(), "the box was not selected");

        let shift_right = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        let outcome = send(&shift_right, &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");

        let shape = state.diagram().session().diagram().get(selected.unwrap());
        let Some(monospace_diagram::Shape::Box { size, .. }) = shape else {
            panic!("expected box");
        };
        assert_eq!(size.width, 5);
        assert_eq!(size.height, 3);
    }

    /// **A box at its minimum still accepts the resize and records no step** (#285).
    ///
    /// **The key is dispatched and the frame is owed**, which is what #288 settled: the box has a
    /// size, so the region answers. What does not happen is a step: the figure comes back 2×2, so
    /// `Undo` has nothing to give back.
    #[test]
    fn a_box_at_its_minimum_accepts_the_resize_and_records_no_step() {
        let mut state = a_drawn_state_over(A_BOX_AT_ITS_SMALLEST);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the box was not selected"
        );

        let shift_left = Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        let outcome = send(&shift_left, &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.diagram().can_undo(),
            "a resize that changes no size records no step"
        );
    }

    /// **With nothing selected, Shift+arrow performs no command and records no step** (rule 4 of #284).
    #[test]
    fn a_resize_with_nothing_selected_performs_no_command() {
        let mut state = a_drawn_state_over(A_BOX);
        let shift_right = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        let outcome = send(&shift_right, &mut state);

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert!(!state.diagram().can_undo());
    }

    /// **A resize keeps the selection** (rule 5 of #284).
    #[test]
    fn a_resize_keeps_the_selection() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();

        let shift_right = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        let _ = send(&shift_right, &mut state);

        assert_eq!(state.diagram().selected(), selected);
    }

    /// **Two resizes are two steps** (rule 7 of #284): undo gives back one cell per press.
    #[test]
    fn two_resizes_are_two_steps() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();

        let shift_right = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        let _ = send(&shift_right, &mut state);
        let _ = send(&shift_right, &mut state);

        let shape = state.diagram().session().diagram().get(selected.unwrap());
        let Some(monospace_diagram::Shape::Box { size, .. }) = shape else {
            panic!("expected box");
        };
        assert_eq!(size.width, 6);

        state.diagram_mut().undo();
        let shape = state.diagram().session().diagram().get(selected.unwrap());
        let Some(monospace_diagram::Shape::Box { size, .. }) = shape else {
            panic!("expected box");
        };
        assert_eq!(size.width, 5);
    }

    /// **Shift arrows resize a horizontal line along its row; vertical arrows are ignored** (D1 of #301).
    #[test]
    fn shift_arrows_resize_the_selected_horizontal_line_along_its_orientation() {
        let mut state = a_drawn_state_over(A_HORIZONTAL_LINE);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(selected.is_some(), "the line was not selected");

        let shift_up = Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT));
        let shift_down = Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT));
        assert!(matches!(send(&shift_up, &mut state), Control::Continue));
        assert!(matches!(send(&shift_down, &mut state), Control::Continue));
        assert!(!state.diagram().can_undo());

        let shift_right = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        let outcome = send(&shift_right, &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");

        let shape = state.diagram().session().diagram().get(selected.unwrap());
        let Some(monospace_diagram::Shape::Line { len, .. }) = shape else {
            panic!("expected line");
        };
        assert_eq!(*len, 7);
        assert!(state.diagram().can_undo());

        let shift_left = Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        let outcome = send(&shift_left, &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");

        let shape = state.diagram().session().diagram().get(selected.unwrap());
        let Some(monospace_diagram::Shape::Line { len, .. }) = shape else {
            panic!("expected line");
        };
        assert_eq!(*len, 6);
    }

    /// **Shift arrows resize a vertical line along its column; horizontal arrows are ignored** (D1 of #301).
    #[test]
    fn shift_arrows_resize_the_selected_vertical_line_along_its_orientation() {
        let mut state = a_drawn_state_over(A_VERTICAL_LINE);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(selected.is_some(), "the line was not selected");

        let shift_left = Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        let shift_right = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        assert!(matches!(send(&shift_left, &mut state), Control::Continue));
        assert!(matches!(send(&shift_right, &mut state), Control::Continue));
        assert!(!state.diagram().can_undo());

        let shift_down = Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT));
        let outcome = send(&shift_down, &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");

        let shape = state.diagram().session().diagram().get(selected.unwrap());
        let Some(monospace_diagram::Shape::Line { len, .. }) = shape else {
            panic!("expected line");
        };
        assert_eq!(*len, 4);

        let shift_up = Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT));
        let outcome = send(&shift_up, &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");

        let shape = state.diagram().session().diagram().get(selected.unwrap());
        let Some(monospace_diagram::Shape::Line { len, .. }) = shape else {
            panic!("expected line");
        };
        assert_eq!(*len, 3);
    }

    /// **A line at its minimum length accepts the resize and records no step** (rule 3 of #301, #285).
    #[test]
    fn a_line_at_its_minimum_accepts_the_resize_and_records_no_step() {
        let mut state = a_drawn_state_over(A_LINE_AT_ITS_MINIMUM);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the line was not selected"
        );

        let shift_left = Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        let outcome = send(&shift_left, &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.diagram().can_undo(),
            "a resize that changes no length records no step"
        );
    }

    /// **A zero-length line is not shrunk and grows to one in the application** (rule 3 of #301).
    #[test]
    fn a_zero_length_line_is_not_shrunk_and_grows_to_one_in_the_application() {
        let mut state = a_drawn_state_over(A_ZERO_LENGTH_LINE);
        state
            .diagram_mut()
            .select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));

        let shift_left = Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        let outcome = send(&shift_left, &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");

        let shape = state
            .diagram()
            .session()
            .diagram()
            .get(ShapeId::new(NonZeroU16::new(1).unwrap()));
        let Some(monospace_diagram::Shape::Line { len, .. }) = shape else {
            panic!("expected line");
        };
        assert_eq!(*len, 0);
        assert!(!state.diagram().can_undo());

        let shift_right = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        let outcome = send(&shift_right, &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");

        let shape = state
            .diagram()
            .session()
            .diagram()
            .get(ShapeId::new(NonZeroU16::new(1).unwrap()));
        let Some(monospace_diagram::Shape::Line { len, .. }) = shape else {
            panic!("expected line");
        };
        assert_eq!(*len, 1);
        assert!(state.diagram().can_undo());
    }

    /// **Two line resizes are two steps** (rule 5 of #301): undo gives back one cell per press.
    #[test]
    fn two_line_resizes_are_two_steps() {
        let mut state = a_drawn_state_over(A_HORIZONTAL_LINE);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();

        let shift_right = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        let _ = send(&shift_right, &mut state);
        let _ = send(&shift_right, &mut state);

        let shape = state.diagram().session().diagram().get(selected.unwrap());
        let Some(monospace_diagram::Shape::Line { len, .. }) = shape else {
            panic!("expected line");
        };
        assert_eq!(*len, 8);

        state.diagram_mut().undo();
        let shape = state.diagram().session().diagram().get(selected.unwrap());
        let Some(monospace_diagram::Shape::Line { len, .. }) = shape else {
            panic!("expected line");
        };
        assert_eq!(*len, 7);

        state.diagram_mut().undo();
        let shape = state.diagram().session().diagram().get(selected.unwrap());
        let Some(monospace_diagram::Shape::Line { len, .. }) = shape else {
            panic!("expected line");
        };
        assert_eq!(*len, 6);

        state.diagram_mut().redo();
        let shape = state.diagram().session().diagram().get(selected.unwrap());
        let Some(monospace_diagram::Shape::Line { len, .. }) = shape else {
            panic!("expected line");
        };
        assert_eq!(*len, 7);
    }

    /// **A line resize keeps the selection** (rule 5 of #301).
    #[test]
    fn a_line_resize_keeps_the_selection() {
        let mut state = a_drawn_state_over(A_HORIZONTAL_LINE);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();

        let shift_right = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        let _ = send(&shift_right, &mut state);

        assert_eq!(state.diagram().selected(), selected);
    }

    /// **A plain arrow still moves** (rule 8 of #284).
    #[test]
    fn a_plain_arrow_still_moves() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();

        let right = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        let _ = send(&right, &mut state);

        let shape = state.diagram().session().diagram().get(selected.unwrap());
        let Some(monospace_diagram::Shape::Box { at, size, .. }) = shape else {
            panic!("expected box");
        };
        assert_eq!(at.x, 3);
        assert_eq!(size.width, 4);
    }

    /// **No other modifier combination resizes** (rule 8 of #284): Ctrl+Shift is ignored.
    #[test]
    fn no_other_modifier_resizes() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);

        let ctrl_shift_right = Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        ));
        let outcome = send(&ctrl_shift_right, &mut state);
        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert!(!state.diagram().can_undo());
    }

    /// **While the bar holds the keyboard, Shift+arrow resizes nothing** (rule 9 of #284).
    #[test]
    fn shift_arrow_with_the_bar_focused_resizes_nothing() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::F(10)), &mut state);

        let shift_right = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        let _ = send(&shift_right, &mut state);

        assert!(!state.diagram().can_undo());
    }

    /// **A resize carries the connector hanging from the right side** (rule 6 of #284).
    #[test]
    fn a_resize_carries_the_connector_hanging_from_the_right_side() {
        let text = r#"{
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": 2,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right",
                        "offset": { "dx": 2, "dy": 0 } },
                "terminal": { "kind": "arm" } },
      "to":   { "at": { "kind": "point", "x": 15, "y": 2 },
                "leaving": "left", "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ]
}"#;
        let mut state = a_drawn_state_over(text);
        let _ = send(&click(3, 3), &mut state);
        let shift_right = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        let _ = send(&shift_right, &mut state);

        let shape = state
            .diagram()
            .session()
            .diagram()
            .get(state.diagram().selected().expect("shape was selected"));
        let Some(monospace_diagram::Shape::Box { size, .. }) = shape else {
            panic!("expected box");
        };
        assert_eq!(size.width, 5);
    }

    /// **A move with nothing selected moves nothing**: no figure is selected, so no step is
    /// recorded, no figure moves, and the answer is `Continue` because the arrow is ignored.
    #[test]
    fn a_move_with_nothing_selected_moves_nothing() {
        let mut state = a_drawn_state_over(A_BOX);

        let outcome = send(&key(KeyCode::Right), &mut state);

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert!(
            !state.diagram().can_undo(),
            "a move with nothing selected recorded a step"
        );
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_some(),
            "the box moved with nothing selected"
        );
    }

    /// **The dialog is the overlay and takes the arrow too** (rule 6): a move behind it changes
    /// nothing, because the event never reaches the region.
    #[test]
    fn an_arrow_while_the_dialog_is_up_moves_nothing() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the box was not selected"
        );
        open_the_about_dialog(&mut state);

        let _ = send(&key(KeyCode::Right), &mut state);

        assert!(
            state.dialog().is_up(),
            "the arrow closed the dialog it should not have reached"
        );
        assert!(
            !state.diagram().can_undo(),
            "a key the dialog swallowed recorded a step"
        );
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_some(),
            "the box moved behind the dialog"
        );
    }

    /// **A carried-out move leaves the keyboard on the region** (rule 7), closing the drop-down
    /// the bar was showing, as every other carried-out meaning does.
    #[test]
    fn a_move_leaves_the_keyboard_on_the_region() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the box was not selected"
        );
        state.chrome_mut().menu_mut().set_popup_active(true);

        let outcome = send(&key(KeyCode::Right), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "the move left the keyboard on the bar"
        );
        assert!(
            !state.chrome().popup_active(),
            "the move left a drop-down open"
        );
        assert!(state.diagram().can_undo(), "the move recorded no step");
    }

    /// **A frame after a move draws the figure one cell further on** (rule 9): the drawing is the
    /// session's, and the application adds no rule to it.
    #[test]
    fn a_frame_after_a_move_draws_the_shape_one_cell_further_on() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let before = draw(&mut state);

        let _ = send(&key(KeyCode::Right), &mut state);
        let after = draw(&mut state);

        assert_eq!(
            before[(3, 3)].symbol(),
            after[(4, 3)].symbol(),
            "the frame does not draw the figure one cell further on"
        );
        assert_eq!(
            after[(3, 3)].symbol(),
            " ",
            "the frame still draws the figure where it stood"
        );
    }

    /// **A figure moved past the window's edge is not drawn** (D2): its cells fall outside the
    /// window, and the frame draws what the window holds, which is nothing.
    #[test]
    fn a_frame_after_a_move_out_of_the_window_draws_nothing() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);

        for _ in 0..7 {
            let _ = send(&key(KeyCode::Left), &mut state);
        }
        let _ = draw(&mut state);

        assert!(state.diagram().can_undo(), "the presses recorded no step");

        // The region's own border is a box too, so the window is what is asked: no offset of the
        // drawing answers a glyph, and the copy therefore writes none.
        let catalog = monospace_glyph_sets::all();
        for y in 0..6 {
            for x in 0..38 {
                assert!(
                    state.diagram().glyph(Offset { x, y }, &catalog).is_none(),
                    "({x}, {y}) still draws the figure that left the window"
                );
            }
        }
    }

    /// **`Alt` and a label's character gives the bar the keyboard and opens that label's
    /// drop-down in one press** (D4), from the region.
    #[test]
    fn alt_and_a_labels_character_opens_that_labels_drop_down() {
        for (label, text) in [('f', "Quit"), ('e', "Undo"), ('h', "About")] {
            let mut state = a_drawn_state();
            assert_eq!(
                state.chrome().leaf(),
                Focus::Region,
                "the test starts on the bar"
            );

            open_the_drop_down(&mut state, label);

            let frame = draw(&mut state);
            let shown: String = (0..HEIGHT).map(|line| row_of(&frame, line)).collect();

            assert!(
                shown.contains(text),
                "Alt+{label} opened another drop-down: {shown:?}"
            );
        }
    }

    /// **A combination that names no label changes nothing and moves no keyboard** (D4): `Alt+Z`
    /// is not a character of any label, and a bar that does not hold the keyboard answers nothing.
    #[test]
    fn alt_with_a_character_that_names_no_label_changes_nothing() {
        let mut state = a_drawn_state();

        let outcome = send(&key_with(KeyCode::Char('z'), KeyModifiers::ALT), &mut state);

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "a combination that names no label moved the keyboard"
        );
        assert!(!state.chrome().popup_active(), "a drop-down opened");
    }

    /// **`Esc` closes the drop-down, and the next one gives the keyboard back to the region**
    /// (rule 5): the key is the widget's while there is something to close, and the bar's way out
    /// after it. Neither press reaches the diagram.
    #[test]
    fn a_second_escape_gives_the_keyboard_back_to_the_region() {
        let mut state = a_drawn_state();
        open_the_drop_down(&mut state, 'f');

        let outcome = send(&key(KeyCode::Esc), &mut state);
        let frame = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.chrome().popup_active(),
            "escape left the drop-down open"
        );
        assert_eq!(
            state.chrome().leaf(),
            Focus::Bar,
            "the first escape left the bar"
        );
        let (column, row) = frame::cell_of(&frame, "File");
        assert!(
            reversed(&frame, column, row),
            "the label stopped looking focused with the bar still holding the keyboard"
        );

        let outcome = send(&key(KeyCode::Esc), &mut state);
        let frame = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "the second escape did not give the keyboard back"
        );
        assert!(
            !reversed(&frame, column, row),
            "a label is reversed with the region holding the keyboard"
        );
    }

    /// **`Esc` with nothing selected is not the region's**: with the keyboard there the key changes
    /// nothing, so the bar's way out is a press that acts on the chrome and never on the drawing.
    #[test]
    fn escape_in_the_region_changes_nothing() {
        let mut state = a_drawn_state();
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "the region does not start with the keyboard"
        );

        let outcome = send(&key(KeyCode::Esc), &mut state);

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "escape moved the keyboard"
        );
        assert!(!state.chrome().popup_active(), "escape opened a drop-down");
    }

    /// **`F10` asks for the bar where a bare `Alt` cannot** (D4, amended): it moves the keyboard and
    /// opens nothing, so the bar looks focused while the region stays the picture under it.
    #[test]
    fn f10_gives_the_bar_the_keyboard_without_opening_a_drop_down() {
        let mut state = a_drawn_state();
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "the test starts on the bar"
        );

        let outcome = send(&key(KeyCode::F(10)), &mut state);
        let frame = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.chrome().leaf(),
            Focus::Bar,
            "F10 did not give the bar the keyboard"
        );
        assert!(!state.chrome().popup_active(), "F10 opened a drop-down");

        let (column, row) = frame::cell_of(&frame, "File");
        assert!(
            reversed(&frame, column, row),
            "the bar does not look focused"
        );
    }

    /// **The drop-down is asked before the region**, so a click it answers never reaches the
    /// diagram — and a click on its `Quit` entry is the way out rather than a selection.
    #[test]
    fn a_click_the_drop_down_takes_never_reaches_the_diagram() {
        let mut state = a_drawn_state_over(A_BOX);
        let outcome = take_the_entry(&mut state, 'f', "Quit");

        assert!(matches!(outcome, Control::Quit), "{outcome:?}");
        assert_eq!(
            state.diagram().selected(),
            None,
            "the click the drop-down took reached the diagram"
        );
    }

    /// **The drop-down's own rectangle is the drop-down's, border and padding included** (rule 1
    /// of #189): a press between its entries is not a press on the drawing behind it. The cell is
    /// found on the frame rather than named, because where the pop-up hangs is the widget's
    /// arithmetic.
    #[test]
    fn a_click_on_the_drop_down_border_never_reaches_the_diagram() {
        let mut state = a_drawn_state_over(A_BOX);
        open_the_drop_down(&mut state, 'e');

        let popup = state.chrome().menu().popup.area;
        let mut border = None;

        for row in popup.top()..popup.bottom() {
            for column in popup.left()..popup.right() {
                let at = Position::new(column, row);

                if state.chrome().menu().popup.item_at((column, row)).is_some() {
                    continue;
                }

                let Some(offset) = state.diagram().offset_of(at) else {
                    continue;
                };

                if state.diagram().owner(offset).is_some() {
                    border = Some(at);
                    break;
                }
            }

            if border.is_some() {
                break;
            }
        }

        let border = border.unwrap_or_else(|| {
            panic!(
                "the Edit drop-down {popup:?} has no border cell over the box, so the test proves nothing"
            )
        });

        assert_eq!(
            state.diagram().selected(),
            None,
            "the test starts with a selection"
        );

        let outcome = send(&click(border.x, border.y), &mut state);

        assert_eq!(
            state.diagram().selected(),
            None,
            "the click on the drop-down's border reached the drawing behind it at {border:?}"
        );
        assert_eq!(
            state.chrome().leaf(),
            Focus::Bar,
            "the click on the drop-down's border moved the keyboard"
        );
        assert!(
            state.chrome().popup_active(),
            "the click on the drop-down's border closed the drop-down"
        );
        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
    }

    /// **The delete key is the first action that changes the session.** The selected shape is gone
    /// from the drawing the next frame makes, and the frame is owed because of it.
    #[test]
    fn the_delete_key_removes_the_selected_shape() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the box was not selected"
        );

        let outcome = send(&key(KeyCode::Delete), &mut state);
        let buffer = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_none(),
            "the drawing still records the removed box"
        );
        assert_eq!(
            buffer[(3, 3)].symbol(),
            " ",
            "the frame still draws the removed box"
        );
    }

    /// **The identity goes with the command** (rule 2): nothing is selected afterwards, and the
    /// next frame inverts no cell. The cell the click selected is inverted before the removal, so
    /// the answer after it proves the inversion went with the selection.
    #[test]
    fn the_delete_key_drops_the_selection() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            reversed(&draw(&mut state), 3, 3),
            "the click selected nothing, so the test proves nothing"
        );

        let _ = send(&key(KeyCode::Delete), &mut state);
        let buffer = draw(&mut state);

        assert_eq!(
            state.diagram().selected(),
            None,
            "the identity survived the removal"
        );
        assert!(
            !reversed(&buffer, 3, 3),
            "the frame after the removal inverts a cell"
        );
    }

    /// **A press with nothing selected is ignored** (D3 of #292). Deletion is contextual and
    /// offered only when a shape is selected; with nothing selected, the key is ignored and the
    /// loop continues without a step or repaint.
    #[test]
    fn the_delete_key_with_nothing_selected_is_ignored() {
        let mut state = a_drawn_state_over(A_BOX);

        let outcome = send(&key(KeyCode::Delete), &mut state);

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert!(
            !state.diagram().can_undo(),
            "a press with nothing selected recorded a step"
        );
        assert_eq!(
            state.diagram().selected(),
            None,
            "a press with nothing selected left a selection"
        );
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_some(),
            "the box was removed without a selection"
        );
    }

    /// **With the drop-down open, Delete is ignored and the menu stays open** (D3 of #292).
    /// Deletion is no longer a global unclaimed shortcut or an edit menu entry; with focus on the
    /// bar/drop-down, Delete does not remove the figure and the menu remains active.
    #[test]
    fn the_drop_down_keeps_the_delete_key() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        open_the_drop_down(&mut state, 'f');

        let outcome = send(&key(KeyCode::Delete), &mut state);

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert!(
            state.diagram().selected().is_some(),
            "the figure was deleted"
        );
        assert_eq!(state.chrome().leaf(), Focus::Bar, "focus left the bar");
        assert!(state.chrome().popup_active(), "the drop-down was closed");
    }

    /// **Delete on the bar does nothing** (D3 of #292). Even with a figure selected, when the bar
    /// holds the keyboard without a drop-down, Delete is ignored.
    #[test]
    fn the_delete_key_on_the_bar_does_nothing() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::F(10)), &mut state);
        assert_eq!(state.chrome().leaf(), Focus::Bar);

        let outcome = send(&key(KeyCode::Delete), &mut state);

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert!(
            state.diagram().selected().is_some(),
            "the figure was deleted"
        );
    }

    #[test]
    fn click_outside_cancels_sub_mode_and_clears_selection() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Char('s')), &mut state);
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::StrokeSelect)
        );

        let _ = send(&click(1, 2), &mut state);
        assert_eq!(state.diagram().sub_mode(), None);
        assert_eq!(state.diagram().selected(), None);
    }

    #[test]
    fn clicking_another_figure_cancels_sub_mode_and_changes_selection() {
        let mut state = a_drawn_state_over(A_BOX_AND_WHAT_HANGS_FROM_IT);
        let _ = send(&click(3, 3), &mut state);
        let first = state.diagram().selected();
        assert!(first.is_some());
        let _ = send(&key(KeyCode::Char('s')), &mut state);
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::StrokeSelect)
        );

        let _ = send(&click(18, 3), &mut state);
        let second = state.diagram().selected();
        assert!(second.is_some());
        assert_ne!(first, second);
        assert_eq!(state.diagram().sub_mode(), None);
    }

    #[test]
    fn clicking_the_same_figure_in_sub_mode_preserves_sub_mode() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Char('s')), &mut state);
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::StrokeSelect)
        );

        let _ = send(&click(3, 3), &mut state);
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::StrokeSelect)
        );
    }

    #[test]
    fn moving_focus_to_bar_and_returning_preserves_sub_mode() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Char('s')), &mut state);
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::StrokeSelect)
        );

        let _ = send(&key(KeyCode::F(10)), &mut state);
        assert_eq!(state.chrome().leaf(), Focus::Bar);
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::StrokeSelect)
        );

        let _ = send(&key(KeyCode::Esc), &mut state);
        assert_eq!(state.chrome().leaf(), Focus::Region);
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::StrokeSelect)
        );
        assert!(state.diagram().selected().is_some());
    }

    #[test]
    fn opening_dialog_and_closing_preserves_sub_mode() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Char('s')), &mut state);
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::StrokeSelect)
        );

        let _ = send(
            &key_with(KeyCode::Char('s'), KeyModifiers::CONTROL),
            &mut state,
        );
        assert!(state.save().is_up());
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::StrokeSelect)
        );

        let _ = send(&key(KeyCode::Esc), &mut state);
        assert!(!state.save().is_up());
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::StrokeSelect)
        );
        assert!(state.diagram().selected().is_some());
    }

    /// The stroke `shape` carries, whichever kind of figure it is.
    fn stroke_of(shape: &monospace_diagram::Shape) -> monospace_diagram::Stroke {
        match shape {
            monospace_diagram::Shape::Box { stroke, .. }
            | monospace_diagram::Shape::Line { stroke, .. }
            | monospace_diagram::Shape::Connector { stroke, .. } => stroke.clone(),
        }
    }

    /// **A stroke the figure already carries changes nothing** (#285): the command leaves the
    /// diagram as it found it, so the session records no step and an undo has nothing to give
    /// back. The key was offered and carried out all the same, so the sub-mode ends and the
    /// identity stays.
    #[test]
    fn setting_identical_stroke_leaves_diagram_untouched_and_records_no_step() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(selected.is_some(), "the box was not selected");
        let _ = send(&key(KeyCode::Char('s')), &mut state);

        let outcome = send(&key(KeyCode::Char('1')), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().sub_mode(),
            None,
            "the sub-mode survived the stroke it was asked for"
        );
        assert_eq!(
            stroke_of(
                state
                    .diagram()
                    .selected_shape()
                    .expect("the box is still there")
            ),
            monospace_diagram::Stroke::from("light"),
            "the stroke changed"
        );
        assert!(
            !state.diagram().can_undo(),
            "an unchanged stroke recorded a step"
        );
        assert_eq!(
            state.diagram().selected(),
            selected,
            "the identity went with the stroke"
        );
    }

    /// **A stroke the figure does not carry is one step** (#292, behavior 5): the diagram holds
    /// the new stroke, and one undo gives the old one back with nothing behind it.
    #[test]
    fn setting_different_stroke_records_one_step_in_session_history() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(selected.is_some(), "the box was not selected");
        let _ = send(&key(KeyCode::Char('s')), &mut state);

        let outcome = send(&key(KeyCode::Char('2')), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().sub_mode(),
            None,
            "the sub-mode survived the stroke it was asked for"
        );
        assert_eq!(
            stroke_of(
                state
                    .diagram()
                    .selected_shape()
                    .expect("the box is still there")
            ),
            monospace_diagram::Stroke::from("double"),
            "the stroke did not change"
        );
        assert!(state.diagram().can_undo(), "no step was recorded");
        assert_eq!(
            state.diagram().selected(),
            selected,
            "the identity went with the stroke"
        );

        let _ = send(&undo_key(), &mut state);

        assert_eq!(
            state.diagram().selected(),
            None,
            "the undo kept the selection"
        );
        let restored_shape = state
            .diagram()
            .session()
            .diagram()
            .get(selected.unwrap())
            .expect("the box is still in the diagram");
        assert_eq!(
            stroke_of(restored_shape),
            monospace_diagram::Stroke::from("light"),
            "one undo did not give the old stroke back"
        );
        assert!(
            !state.diagram().can_undo(),
            "the stroke recorded more than one step"
        );
    }

    /// **Undo and redo walk the history from inside the sub-mode and clear the selection** (#297):
    /// the global keys fall through the region, the walk clears the selection, and dropping the
    /// selection ends the active sub-mode.
    #[test]
    fn undo_and_redo_in_stroke_select_walk_history() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram().selected();
        assert!(selected.is_some(), "the box was not selected");
        let _ = send(&key(KeyCode::Char('s')), &mut state);
        let _ = send(&key(KeyCode::Char('2')), &mut state);
        let _ = send(&key(KeyCode::Char('s')), &mut state);
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::StrokeSelect),
            "the sub-mode was not entered again"
        );

        let outcome = send(&undo_key(), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        let restored_shape = state
            .diagram()
            .session()
            .diagram()
            .get(selected.unwrap())
            .expect("the box is still in the diagram");
        assert_eq!(
            stroke_of(restored_shape),
            monospace_diagram::Stroke::from("light"),
            "the undo did not give the old stroke back"
        );
        assert_eq!(
            state.diagram().sub_mode(),
            None,
            "the undo kept the sub-mode"
        );
        assert_eq!(
            state.diagram().selected(),
            None,
            "the undo kept the selection"
        );

        let outcome = send(&redo_key(), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        let redone_shape = state
            .diagram()
            .session()
            .diagram()
            .get(selected.unwrap())
            .expect("the box is still in the diagram");
        assert_eq!(
            stroke_of(redone_shape),
            monospace_diagram::Stroke::from("double"),
            "the redo did not give the new stroke back"
        );
        assert_eq!(
            state.diagram().sub_mode(),
            None,
            "the redo restored a sub-mode"
        );
        assert_eq!(
            state.diagram().selected(),
            None,
            "the redo restored a selection"
        );
    }

    /// Two overlapping boxes: id1 at the back, id2 in front. The cell `(0, 1)` of the window belongs
    /// only to id1, so a click there selects the back box without hitting the front one.
    const TWO_OVERLAPPING_BOXES: &str = r#"{
        "next_id": 3,
        "shapes": [
            { "kind": "box", "id": 1, "at": { "x": 0, "y": 1 },
              "size": { "width": 4, "height": 3 }, "stroke": "light", "fill": "░" },
            { "kind": "box", "id": 2, "at": { "x": 2, "y": 2 },
              "size": { "width": 4, "height": 3 }, "stroke": "light", "fill": "▓" }
        ]
    }"#;

    #[test]
    fn z_opens_order_mode_and_escape_leaves_it() {
        let mut state = a_drawn_state_over(TWO_OVERLAPPING_BOXES);
        let _ = send(&click(1, 3), &mut state);
        let id = state.diagram().selected();
        assert!(id.is_some(), "box #1 was not selected");

        let outcome = send(&key(KeyCode::Char('z')), &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::OrderMode),
            "z did not enter order mode"
        );

        let buffer = draw(&mut state);
        assert!(
            row_of(&buffer, HEIGHT - 2).starts_with("LAYER ORDER for Box #1"),
            "status row did not report layer order: {}",
            row_of(&buffer, HEIGHT - 2)
        );
        assert!(
            row_of(&buffer, HEIGHT - 1).contains("ORDER: [Home] Front"),
            "command row did not offer order commands: {}",
            row_of(&buffer, HEIGHT - 1)
        );

        let outcome = send(&key(KeyCode::Esc), &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().sub_mode(),
            None,
            "Esc did not leave order mode"
        );
        assert_eq!(state.diagram().selected(), id, "Esc dropped the selection");
    }

    #[test]
    fn page_up_moves_the_selected_figure_forward_and_records_one_step() {
        let mut state = a_drawn_state_over(TWO_OVERLAPPING_BOXES);
        let _ = send(&click(1, 3), &mut state);
        let id1 = state.diagram().selected().expect("box #1 selected");
        assert_eq!(
            state.diagram().session().diagram().place_of(id1),
            Some(0),
            "box #1 starts at the back"
        );

        let _ = send(&key(KeyCode::Char('z')), &mut state);
        let outcome = send(&key(KeyCode::PageUp), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().session().diagram().place_of(id1),
            Some(1),
            "PageUp did not move box #1 forward"
        );
        assert!(
            state.diagram().session().can_undo(),
            "moving forward recorded no step"
        );
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::OrderMode),
            "PageUp left order mode"
        );
    }

    #[test]
    fn page_down_moves_the_selected_figure_backward() {
        let mut state = a_drawn_state_over(TWO_OVERLAPPING_BOXES);
        let _ = send(&click(5, 4), &mut state);
        let id2 = state.diagram().selected().expect("box #2 selected");
        assert_eq!(
            state.diagram().session().diagram().place_of(id2),
            Some(1),
            "box #2 starts in front"
        );

        let _ = send(&key(KeyCode::Char('z')), &mut state);
        let outcome = send(&key(KeyCode::PageDown), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().session().diagram().place_of(id2),
            Some(0),
            "PageDown did not move box #2 backward"
        );
        assert!(
            state.diagram().session().can_undo(),
            "moving backward recorded no step"
        );
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::OrderMode),
            "PageDown left order mode"
        );
    }

    #[test]
    fn home_moves_the_selected_figure_to_the_front_in_one_step() {
        let mut state = a_drawn_state_over(TWO_OVERLAPPING_BOXES);
        let _ = send(&click(1, 3), &mut state);
        let id1 = state.diagram().selected().expect("box #1 selected");
        assert_eq!(state.diagram().session().diagram().place_of(id1), Some(0));

        let _ = send(&key(KeyCode::Char('z')), &mut state);
        let outcome = send(&key(KeyCode::Home), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().session().diagram().place_of(id1),
            Some(1),
            "Home did not move box #1 to the front"
        );
        assert!(state.diagram().session().can_undo());

        let undo_outcome = send(&undo_key(), &mut state);
        assert!(matches!(undo_outcome, Control::Changed), "{undo_outcome:?}");
        assert_eq!(
            state.diagram().session().diagram().place_of(id1),
            Some(0),
            "undo did not restore box #1 position"
        );
        assert!(
            !state.diagram().session().can_undo(),
            "more than one step recorded"
        );
        assert_eq!(state.diagram().selected(), None);
        assert_eq!(state.diagram().sub_mode(), None);
    }

    #[test]
    fn end_moves_the_selected_figure_to_the_back_in_one_step() {
        let mut state = a_drawn_state_over(TWO_OVERLAPPING_BOXES);
        let _ = send(&click(5, 4), &mut state);
        let id2 = state.diagram().selected().expect("box #2 selected");
        assert_eq!(state.diagram().session().diagram().place_of(id2), Some(1));

        let _ = send(&key(KeyCode::Char('z')), &mut state);
        let outcome = send(&key(KeyCode::End), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().session().diagram().place_of(id2),
            Some(0),
            "End did not move box #2 to the back"
        );
        assert!(state.diagram().session().can_undo());

        let undo_outcome = send(&undo_key(), &mut state);
        assert!(matches!(undo_outcome, Control::Changed), "{undo_outcome:?}");
        assert_eq!(
            state.diagram().session().diagram().place_of(id2),
            Some(1),
            "undo did not restore box #2 position"
        );
        assert!(
            !state.diagram().session().can_undo(),
            "more than one step recorded"
        );
    }

    #[test]
    fn a_move_at_the_limit_records_no_step() {
        // Case 1: id1 at back, PageDown
        let mut state = a_drawn_state_over(TWO_OVERLAPPING_BOXES);
        let _ = send(&click(1, 3), &mut state);
        let _ = send(&key(KeyCode::Char('z')), &mut state);
        let outcome = send(&key(KeyCode::PageDown), &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.diagram().session().can_undo(),
            "PageDown at back recorded a step"
        );
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::OrderMode),
            "PageDown at back left order mode"
        );

        // Case 2: id1 at back, End
        let mut state = a_drawn_state_over(TWO_OVERLAPPING_BOXES);
        let _ = send(&click(1, 3), &mut state);
        let _ = send(&key(KeyCode::Char('z')), &mut state);
        let outcome = send(&key(KeyCode::End), &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.diagram().session().can_undo(),
            "End at back recorded a step"
        );
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::OrderMode),
            "End at back left order mode"
        );

        // Case 3: id2 at front, PageUp
        let mut state = a_drawn_state_over(TWO_OVERLAPPING_BOXES);
        let _ = send(&click(5, 4), &mut state);
        let _ = send(&key(KeyCode::Char('z')), &mut state);
        let outcome = send(&key(KeyCode::PageUp), &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.diagram().session().can_undo(),
            "PageUp at front recorded a step"
        );
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::OrderMode),
            "PageUp at front left order mode"
        );

        // Case 4: id2 at front, Home
        let mut state = a_drawn_state_over(TWO_OVERLAPPING_BOXES);
        let _ = send(&click(5, 4), &mut state);
        let _ = send(&key(KeyCode::Char('z')), &mut state);
        let outcome = send(&key(KeyCode::Home), &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.diagram().session().can_undo(),
            "Home at front recorded a step"
        );
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::OrderMode),
            "Home at front left order mode"
        );
    }

    #[test]
    fn undo_and_redo_leave_order_mode() {
        let mut state = a_drawn_state_over(TWO_OVERLAPPING_BOXES);
        let _ = send(&click(1, 3), &mut state);
        let id1 = state.diagram().selected().expect("box #1 selected");
        let _ = send(&key(KeyCode::Char('z')), &mut state);
        let _ = send(&key(KeyCode::Home), &mut state);
        assert_eq!(state.diagram().session().diagram().place_of(id1), Some(1));

        let undo_outcome = send(&undo_key(), &mut state);
        assert!(matches!(undo_outcome, Control::Changed), "{undo_outcome:?}");
        assert_eq!(state.diagram().selected(), None);
        assert_eq!(state.diagram().sub_mode(), None);
        assert_eq!(
            state.diagram().session().diagram().place_of(id1),
            Some(0),
            "undo did not restore original place"
        );

        let redo_outcome = send(&redo_key(), &mut state);
        assert!(matches!(redo_outcome, Control::Changed), "{redo_outcome:?}");
        assert_eq!(state.diagram().selected(), None);
        assert_eq!(state.diagram().sub_mode(), None);
        assert_eq!(
            state.diagram().session().diagram().place_of(id1),
            Some(1),
            "redo did not restore forward place"
        );
    }

    #[test]
    fn clicking_another_figure_cancels_order_mode() {
        let mut state = a_drawn_state_over(TWO_OVERLAPPING_BOXES);
        let _ = send(&click(1, 3), &mut state);
        let id1 = state.diagram().selected();
        assert!(id1.is_some());
        let _ = send(&key(KeyCode::Char('z')), &mut state);
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::OrderMode)
        );

        let outcome = send(&click(5, 4), &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        let id2 = state.diagram().selected();
        assert!(id2.is_some());
        assert_ne!(id1, id2);
        assert_eq!(state.diagram().sub_mode(), None);
    }

    #[test]
    fn clicking_outside_cancels_order_mode_and_clears_the_selection() {
        let mut state = a_drawn_state_over(TWO_OVERLAPPING_BOXES);
        let _ = send(&click(1, 3), &mut state);
        assert!(state.diagram().selected().is_some());
        let _ = send(&key(KeyCode::Char('z')), &mut state);
        assert_eq!(
            state.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::OrderMode)
        );

        let outcome = send(&click(20, 2), &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(state.diagram().sub_mode(), None);
        assert_eq!(state.diagram().selected(), None);
    }

    /// **Escape drops the active selection** (D1 of #279): pressing Escape while a shape is
    /// selected drops the selection and owes a frame. Pressing Escape when nothing is selected
    /// changes nothing.
    #[test]
    fn escape_in_the_region_drops_the_selection() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the box was not selected"
        );

        let outcome = send(&key(KeyCode::Esc), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().selected(),
            None,
            "Escape did not drop the selection"
        );

        let second = send(&key(KeyCode::Esc), &mut state);

        assert!(matches!(second, Control::Continue), "{second:?}");
    }

    #[test]
    fn t_opens_the_text_editor_on_a_box() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the box was not selected"
        );

        let outcome = send(&key(KeyCode::Char('t')), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(state.text().is_up(), "t opened no editor");
        assert!(
            !state.diagram().can_undo(),
            "opening the editor recorded a step"
        );
    }

    #[test]
    fn t_is_ignored_off_a_box() {
        // **A line**, selected by clicking it. `click(1, 3)` would select nothing here, and on
        // `A_BOX_AND_WHAT_HANGS_FROM_IT` it is inside a box, so neither fixture is usable as-is.
        let mut line = a_drawn_state_over(A_HORIZONTAL_LINE);
        let _ = send(&click(3, 3), &mut line);
        assert!(
            line.diagram().selected().is_some(),
            "the line was not selected"
        );
        let outcome = send(&key(KeyCode::Char('t')), &mut line);
        assert!(matches!(outcome, Control::Continue), "line: {outcome:?}");
        assert!(!line.text().is_up(), "line: t opened an editor");

        // **A connector**, selected directly: its route is the drawing's, and what this test says
        // is that `t` is not accepted for it, not how a press selects one.
        let mut connector = a_drawn_state_over(A_CONNECTOR);
        connector
            .diagram_mut()
            .select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        let outcome = send(&key(KeyCode::Char('t')), &mut connector);
        assert!(
            matches!(outcome, Control::Continue),
            "connector: {outcome:?}"
        );
        assert!(!connector.text().is_up(), "connector: t opened an editor");

        // **Nothing selected.**
        let mut nothing = a_drawn_state_over(A_BOX);
        let outcome = send(&key(KeyCode::Char('t')), &mut nothing);
        assert!(
            matches!(outcome, Control::Continue),
            "nothing selected: {outcome:?}"
        );
        assert!(
            !nothing.text().is_up(),
            "nothing selected: t opened an editor"
        );

        // **A box in the stroke sub-mode**: that bar offers digits and `Esc`, and not `t` (rule 2).
        let mut sub_mode = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut sub_mode);
        let _ = send(&key(KeyCode::Char('s')), &mut sub_mode);
        assert_eq!(
            sub_mode.diagram().sub_mode(),
            Some(crate::diagram_state::SubMode::StrokeSelect)
        );
        let outcome = send(&key(KeyCode::Char('t')), &mut sub_mode);
        assert!(
            matches!(outcome, Control::Continue),
            "sub-mode: {outcome:?}"
        );
        assert!(!sub_mode.text().is_up(), "sub-mode: t opened an editor");
    }

    #[test]
    fn pressing_i_on_selected_connector_inverts_terminals() {
        let fixture = r#"{
            "next_id": 2,
            "shapes": [ { "kind": "connector", "id": 1,
                          "from": { "at": { "kind": "point", "x": 2, "y": 1 },
                                    "leaving": "right", "terminal": { "kind": "arm" } },
                          "to": { "at": { "kind": "point", "x": 10, "y": 1 },
                                  "leaving": "left", "terminal": { "kind": "symbol", "style": "solid" } },
                          "stroke": "light" } ]
        }"#;
        let mut state = a_drawn_state_over(fixture);
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.diagram_mut().select(Some(id));

        let i_key = Event::Key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
        let outcome = send(&i_key, &mut state);
        assert!(matches!(outcome, Control::Changed));

        let shape = state.diagram().session().diagram().get(id);
        let Some(monospace_diagram::Shape::Connector { from, to, .. }) = shape else {
            panic!("expected connector");
        };
        assert_eq!(
            from.terminal,
            monospace_core::Terminal::Symbol {
                style: monospace_core::SymbolStyleName::from("solid")
            }
        );
        assert_eq!(to.terminal, monospace_core::Terminal::Arm);
        assert!(state.diagram().session().can_undo());

        // Pressing Undo restores the original terminals
        let undo_key = Event::Key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        let undo_outcome = send(&undo_key, &mut state);
        assert!(matches!(undo_outcome, Control::Changed));

        let shape_restored = state.diagram().session().diagram().get(id);
        let Some(monospace_diagram::Shape::Connector { from, to, .. }) = shape_restored else {
            panic!("expected connector");
        };
        assert_eq!(from.terminal, monospace_core::Terminal::Arm);
        assert_eq!(
            to.terminal,
            monospace_core::Terminal::Symbol {
                style: monospace_core::SymbolStyleName::from("solid")
            }
        );
    }

    #[test]
    fn pressing_r_on_selected_line_toggles_orientation_and_updates_status() {
        let fixture = r#"{
            "next_id": 2,
            "shapes": [ { "kind": "line", "id": 1, "at": { "x": 1, "y": 1 },
                          "len": 5, "orientation": "vertical", "stroke": "light" } ]
        }"#;
        let mut state = a_drawn_state_over(fixture);
        let id = ShapeId::new(NonZeroU16::new(1).unwrap());
        state.diagram_mut().select(Some(id));

        assert_eq!(
            crate::status::status_line(state.diagram()),
            "Line #1 · Pos: (1, 1) · Length: 5 · Orientation: Vertical · Stroke: light"
        );
        assert!(
            crate::status::command_line(state.diagram()).contains("RESIZE: Shift+↑↓ · ROTATE: [r]")
        );

        let r_key = Event::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));
        let outcome = send(&r_key, &mut state);
        assert!(matches!(outcome, Control::Changed));

        let shape = state.diagram().session().diagram().get(id);
        let Some(monospace_diagram::Shape::Line {
            orientation,
            at,
            len,
            stroke,
        }) = shape
        else {
            panic!("expected line");
        };
        assert_eq!(*orientation, monospace_core::Orientation::Horizontal);
        assert_eq!(*at, monospace_core::Pos { x: 1, y: 1 });
        assert_eq!(*len, 5);
        assert_eq!(*stroke, monospace_diagram::Stroke::from("light"));

        assert_eq!(
            crate::status::status_line(state.diagram()),
            "Line #1 · Pos: (1, 1) · Length: 5 · Orientation: Horizontal · Stroke: light"
        );
        assert!(
            crate::status::command_line(state.diagram()).contains("RESIZE: Shift+←→ · ROTATE: [r]")
        );
        assert!(state.diagram().session().can_undo());

        // Second 'r' toggles back to Vertical
        let outcome2 = send(&r_key, &mut state);
        assert!(matches!(outcome2, Control::Changed));
        assert_eq!(
            crate::status::status_line(state.diagram()),
            "Line #1 · Pos: (1, 1) · Length: 5 · Orientation: Vertical · Stroke: light"
        );

        // Undo restores Horizontal in a single step
        let undo_key = Event::Key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        let undo_outcome = send(&undo_key, &mut state);
        assert!(matches!(undo_outcome, Control::Changed));

        let shape_restored = state.diagram().session().diagram().get(id);
        let Some(monospace_diagram::Shape::Line { orientation, .. }) = shape_restored else {
            panic!("expected line");
        };
        assert_eq!(*orientation, monospace_core::Orientation::Horizontal);
    }

    #[test]
    fn enter_remains_ignored_on_a_box() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);

        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert!(!state.text().is_up(), "Enter opened the editor");
    }

    #[test]
    fn ok_saves_and_escape_discards() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Char('t')), &mut state);

        for character in ['h', 'i'] {
            let _ = send(&key(KeyCode::Char(character)), &mut state);
        }
        let _ = send(&key(KeyCode::Tab), &mut state);
        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.text().is_up(), "Ok did not close the editor");
        assert!(state.diagram().can_undo(), "saving recorded no step");
        assert_eq!(
            state.diagram().selected(),
            Some(ShapeId::new(NonZeroU16::new(1).unwrap())),
            "saving dropped the selection"
        );

        // The second opening starts from the saved text; Escape closes it without a step.
        let _ = send(&key(KeyCode::Char('t')), &mut state);
        let _ = send(&key(KeyCode::Char('x')), &mut state);
        let outcome = send(&key(KeyCode::Esc), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.text().is_up(), "Escape did not close the editor");

        // **`Cancel` with `Enter` discards too** (rule 7): two `Tab`s reach it from the buffer.
        let _ = send(&key(KeyCode::Char('t')), &mut state);
        let _ = send(&key(KeyCode::Char('y')), &mut state);
        let _ = send(&key(KeyCode::Tab), &mut state);
        let _ = send(&key(KeyCode::Tab), &mut state);
        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.text().is_up(),
            "Enter on Cancel did not close the editor"
        );
        let Shape::Box { text: kept, .. } = state
            .diagram()
            .session()
            .diagram()
            .get(ShapeId::new(NonZeroU16::new(1).unwrap()))
            .expect("the box is there")
        else {
            panic!("expected box");
        };
        assert_eq!(
            *kept,
            Some(Text::new("hi").unwrap()),
            "Cancel saved the buffer"
        );
    }

    #[test]
    fn saving_the_same_text_records_no_step() {
        let mut state = a_drawn_state_over(A_BOX_WITH_TEXT);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Char('t')), &mut state);
        let _ = send(&key(KeyCode::Tab), &mut state);
        let outcome = send(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.text().is_up(), "Ok did not close the editor");
        assert!(
            !state.diagram().can_undo(),
            "saving the same text recorded a step"
        );
    }

    #[test]
    fn ctrl_c_copies_while_the_text_editor_is_up() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Char('t')), &mut state);

        for character in ['h', 'i'] {
            let _ = send(&key(KeyCode::Char(character)), &mut state);
        }
        let _ = send(
            &key_with(KeyCode::Char('a'), KeyModifiers::CONTROL),
            &mut state,
        );
        let outcome = send(
            &key_with(KeyCode::Char('c'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert!(state.text().is_up(), "Ctrl+C closed the editor");
        assert_eq!(
            rat_widget::text::clipboard::global_clipboard()
                .get_string()
                .as_deref()
                .ok(),
            Some("hi")
        );
    }

    #[test]
    fn the_editor_consumes_every_key() {
        // Ctrl+Q/S/O/N: nothing behind the overlay moves.
        for code in [
            KeyCode::Char('q'),
            KeyCode::Char('s'),
            KeyCode::Char('o'),
            KeyCode::Char('n'),
        ] {
            let mut state = a_drawn_state_over(A_BOX);
            let _ = send(&click(3, 3), &mut state);
            let _ = send(&key(KeyCode::Char('t')), &mut state);

            let outcome = send(&key_with(code, KeyModifiers::CONTROL), &mut state);

            assert!(
                matches!(outcome, Control::Continue),
                "{code:?}: {outcome:?}"
            );
            assert!(state.text().is_up(), "{code:?}: the editor closed");
            assert!(
                !state.dialog().is_up(),
                "{code:?}: the message dialog opened"
            );
            assert!(!state.open().is_up(), "{code:?}: the open dialog opened");
            assert!(!state.save().is_up(), "{code:?}: the save dialog opened");
            assert!(!state.diagram().can_undo(), "{code:?}: the diagram changed");
        }

        // `t` inside the editor is a character, not the key that opens it.
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Char('t')), &mut state);
        let outcome = send(&key(KeyCode::Char('t')), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(state.text().is_up(), "t inside the editor closed it");
        assert_eq!(
            state.text().get().expect("the text editor is up").buffer(),
            "t"
        );

        // `Tab` moves the focus inside the editor and inserts no tab.
        let outcome = send(&key(KeyCode::Tab), &mut state);
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(state.text().is_up(), "Tab closed the editor");
        assert_eq!(
            state.text().get().expect("the text editor is up").buffer(),
            "t",
            "Tab inserted a tab"
        );

        // A click on the bar reaches nothing: the editor stays up and the keyboard does not move.
        let leaf = state.chrome().leaf();
        let outcome = send(&click(3, 0), &mut state);
        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert!(state.text().is_up(), "a click on the bar closed the editor");
        assert_eq!(state.chrome().leaf(), leaf, "the click moved the keyboard");
    }

    /// **An entry with nothing to give back changes nothing and paints** (rule 11): the drop-down
    /// closes, the drawing is the one it already showed, and no step is recorded either way.
    #[test]
    fn taking_an_entry_with_nothing_to_give_back_changes_nothing() {
        let mut state = a_drawn_state_over(A_BOX);
        let before = draw(&mut state);

        let outcome = take_the_entry(&mut state, 'e', "Undo");

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.chrome().popup_active(),
            "the entry left the menu open"
        );
        assert!(!state.diagram().can_undo(), "the entry recorded a step");
        assert!(!state.diagram().can_redo(), "the entry left a step ahead");

        let after = draw(&mut state);

        for line in 0..HEIGHT {
            assert_eq!(
                row_of(&before, line),
                row_of(&after, line),
                "row {line} is not what it was"
            );
        }

        let outcome = take_the_entry(&mut state, 'e', "Redo");

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.chrome().popup_active(),
            "the entry left the menu open"
        );
        assert!(
            !state.diagram().can_undo(),
            "the redo entry recorded a step"
        );
        assert!(
            !state.diagram().can_redo(),
            "the redo entry left a step ahead"
        );
    }

    /// **`Ctrl+Z` gives the step behind back** (rule 1). The deletion recorded one, and the undo
    /// takes the drawing to the state the step before it left: the box is back on the frame and
    /// there is a step ahead.
    #[test]
    fn the_undo_key_gives_the_step_back() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Delete), &mut state);
        let _ = draw(&mut state);
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_none(),
            "the box was not removed, so the test proves nothing"
        );

        let outcome = send(&undo_key(), &mut state);
        let buffer = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_some(),
            "the undo did not give the box back"
        );
        assert_eq!(
            buffer[(3, 3)].symbol(),
            "┌",
            "the frame after the undo does not draw the box"
        );
        assert!(!state.diagram().can_undo(), "the undo left a step behind");
        assert!(state.diagram().can_redo(), "the undo left nothing to redo");
    }

    /// **`Ctrl+Shift+Z` gives the step ahead back** (rule 2): the undo left one, and the redo takes
    /// the drawing to the state the step left.
    #[test]
    fn the_redo_key_gives_it_forward_again() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Delete), &mut state);
        let _ = send(&undo_key(), &mut state);
        let _ = draw(&mut state);
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_some(),
            "the undo did not give the box back, so the test proves nothing"
        );

        let outcome = send(&redo_key(), &mut state);
        let buffer = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_none(),
            "the redo did not take the box out again"
        );
        assert_eq!(
            buffer[(3, 3)].symbol(),
            " ",
            "the frame after the redo still draws the box"
        );
        assert!(state.diagram().can_undo(), "the redo left nothing to undo");
        assert!(!state.diagram().can_redo(), "the redo left a step ahead");
    }

    /// **At the oldest position `Ctrl+Z` walks nothing and paints** (rules 3 and 8): the session is
    /// unchanged, no step is recorded, and the frame is owed because the action was carried out.
    #[test]
    fn the_undo_key_with_nothing_behind_changes_nothing() {
        let mut state = a_drawn_state_over(A_BOX);

        let outcome = send(&undo_key(), &mut state);
        let buffer = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.diagram().can_undo(), "the press recorded a step");
        assert!(!state.diagram().can_redo(), "the press left a step ahead");
        assert_eq!(
            buffer[(3, 3)].symbol(),
            "┌",
            "the press changed the drawing"
        );
    }

    /// **At the newest position `Ctrl+Shift+Z` walks nothing and paints** (rules 3 and 8).
    #[test]
    fn the_redo_key_with_nothing_ahead_changes_nothing() {
        let mut state = a_drawn_state_over(A_BOX);

        let outcome = send(&redo_key(), &mut state);
        let buffer = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.diagram().can_undo(), "the press recorded a step");
        assert!(!state.diagram().can_redo(), "the press left a step ahead");
        assert_eq!(
            buffer[(3, 3)].symbol(),
            "┌",
            "the press changed the drawing"
        );
    }

    /// **Both keys are `Unclaimed`, so an open drop-down does not keep either** (rule 7): the
    /// history walks behind the menu, the shape comes back and goes again, and **the keyboard then
    /// goes back to the region, which closes the drop-down with it** (rule 13).
    #[test]
    fn the_walks_are_answered_with_the_drop_down_open_and_take_the_keyboard_back() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Delete), &mut state);
        let _ = draw(&mut state);
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_none(),
            "the box was not removed, so the test proves nothing"
        );

        open_the_drop_down(&mut state, 'f');
        let outcome = send(&undo_key(), &mut state);
        let _ = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_some(),
            "the drop-down kept the undo key from the history"
        );
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "the undo left the keyboard on the bar"
        );
        assert!(
            !state.chrome().popup_active(),
            "a drop-down is open with the keyboard on the region"
        );

        open_the_drop_down(&mut state, 'f');
        let outcome = send(&redo_key(), &mut state);
        let _ = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            state.diagram().owner(Offset { x: 2, y: 1 }).is_none(),
            "the drop-down kept the redo key from the history"
        );
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "the redo left the keyboard on the bar"
        );
        assert!(
            !state.chrome().popup_active(),
            "a drop-down is open with the keyboard on the region"
        );
    }

    /// **The table answers a meaning and not a count of steps** (rule 6): `Action::Undo` has no
    /// field, and the session is asked to walk where the action is carried out.
    #[test]
    fn the_undo_shortcut_says_only_that_it_undoes() {
        assert_eq!(
            shortcuts::handle_unclaimed(&undo_key()),
            Reply::Act(Action::Undo),
            "the undo key is not a row of the table, or says something else"
        );
        assert_eq!(
            shortcuts::handle_always(&undo_key()),
            Reply::Ignored,
            "the undo key reaches before the chrome"
        );
    }

    /// **The redo key says a meaning too**, and reaches after the chrome as the undo does.
    #[test]
    fn the_redo_shortcut_says_only_that_it_redoes() {
        assert_eq!(
            shortcuts::handle_unclaimed(&redo_key()),
            Reply::Act(Action::Redo),
            "the redo key is not a row of the table, or says something else"
        );
        assert_eq!(
            shortcuts::handle_always(&redo_key()),
            Reply::Ignored,
            "the redo key reaches before the chrome"
        );
    }

    /// **A walk clears the selection** (#297). The box is selected before the undo; the undo gives
    /// the step back and clears the selection, returning the application to `IDLE`. Selecting the
    /// box again and redoing clears the selection too, leaving the frame with no cell inverted.
    #[test]
    fn the_undo_and_redo_keys_clear_the_selection() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Right), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the box was not selected"
        );

        let outcome = send(&undo_key(), &mut state);
        let buffer = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().selected(),
            None,
            "the undo kept the selection"
        );
        assert!(
            !reversed(&buffer, 3, 3),
            "the frame after the undo inverts a cell"
        );

        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the box was not selected again"
        );

        let outcome = send(&redo_key(), &mut state);
        let buffer = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().selected(),
            None,
            "the redo kept the selection"
        );
        assert!(
            !reversed(&buffer, 4, 3),
            "the frame after the redo inverts a cell"
        );
    }

    /// **The next frame draws the restored diagram into the same window** (rule 13): the undo
    /// changes no region, and every figure is where the step left it.
    #[test]
    fn a_frame_after_an_undo_draws_the_restored_diagram() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let before = draw(&mut state);

        let _ = send(&key(KeyCode::Delete), &mut state);
        let _ = send(&undo_key(), &mut state);
        let after = draw(&mut state);

        let region = frame::regions(Rect::new(0, 0, WIDTH, HEIGHT)).region;
        for line in region.top()..region.bottom() {
            assert_eq!(
                row_of(&before, line),
                row_of(&after, line),
                "row {line} is not where the step left it"
            );
        }
    }

    /// **What hung from the removed shape stays drawn where it stood** (rule 8). The diagram model
    /// freezes the reference at the point it resolved to, and the frame that follows the removal
    /// draws the connector there rather than leaving it out.
    #[test]
    fn a_frame_after_a_removal_draws_the_connector_where_it_stood() {
        let mut state = a_drawn_state_over(A_BOX_AND_WHAT_HANGS_FROM_IT);
        let _ = send(&click(1, 3), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the left box was not selected"
        );

        let _ = send(&key(KeyCode::Delete), &mut state);
        let buffer = draw(&mut state);

        // The left box is gone, and its cells are blank.
        for (column, line) in [(1, 3), (4, 3), (1, 4), (1, 5), (4, 5)] {
            assert_eq!(
                buffer[(column, line)].symbol(),
                " ",
                "({column}, {line}) still holds the removed box"
            );
        }

        // The connector's near end was frozen at the point it resolved to: the window's `(3, 2)`
        // is the screen's `(4, 4)`, and the line reaches the right box's left side at `(17, 4)`.
        assert_eq!(
            row_of(&buffer, 4),
            "│   ─────────────┤  │                  │",
            "the frame does not draw the connector where it stood"
        );
        assert_eq!(
            buffer[(17, 3)].symbol(),
            "┌",
            "the right box went with the connector"
        );
        assert_eq!(
            buffer[(20, 3)].symbol(),
            "┐",
            "the right box is not four cells wide"
        );
    }

    /// **A turn of the wheel over the region moves the view by the helper's step** (rule 3): a
    /// tenth of the window on the axis the wheel reports, and at least one cell where the tenth is
    /// less. The step is the helper's rather than a rule of ours, so the numbers are its
    /// arithmetic: `38/10 = 3` across and `5/10 = 0`, which it answers as one cell down.
    ///
    /// **`Alt` is the wheel's across**: the helper reads a turn with `Alt` held as the horizontal
    /// axis, which is the one gesture the wheel and the bar share.
    #[test]
    fn the_wheel_moves_the_view_by_the_helpers_step() {
        let mut state = a_drawn_state_over(A_BOX_TOO_BIG);

        let outcome = send(&wheel(MouseEventKind::ScrollDown, 5, 4), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 0, y: 1 },
            "a turn down did not move the view by a tenth of the window, at least one cell"
        );

        let outcome = send(
            &wheel_with(MouseEventKind::ScrollDown, KeyModifiers::ALT, 5, 4),
            &mut state,
        );

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 3, y: 1 },
            "Alt and a turn down did not move the view across by a tenth of the window"
        );
    }

    /// **A turn the content refuses changes nothing and owes no frame** (rule 3): at the far end of
    /// the content there is nowhere left to go, so the region does not claim the event.
    #[test]
    fn a_wheel_at_the_end_moves_nothing_and_owes_no_frame() {
        let mut state = a_drawn_state_over(A_BOX_TOO_BIG);

        for _ in 0..8 {
            let _ = send(
                &wheel_with(MouseEventKind::ScrollDown, KeyModifiers::ALT, 5, 4),
                &mut state,
            );
        }
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 22, y: 0 },
            "eight turns did not reach the content's far end, so the test proves nothing"
        );

        let outcome = send(
            &wheel_with(MouseEventKind::ScrollDown, KeyModifiers::ALT, 5, 4),
            &mut state,
        );

        assert!(
            matches!(outcome, Control::Continue),
            "the end of the content owed a frame: {outcome:?}"
        );
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 22, y: 0 },
            "the view moved past the content"
        );
    }

    /// **A press on the bar takes the view to the position it names** (rule 4): the helper maps the
    /// track's cells over the content's reach, so the middle of the track names the middle of the
    /// content.
    ///
    /// The vertical track is the region's last column, rows `2` to `6`, and the content's span is
    /// `9`: the row `5` the press lands on is the fourth of the five cells the track maps, so the view
    /// stands six rows down.
    #[test]
    fn a_press_on_the_bar_jumps_the_view() {
        let mut state = a_drawn_state_over(A_BOX_TOO_BIG);

        let outcome = send(&click(39, 5), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 0, y: 6 },
            "the press did not take the view to the position it named"
        );
    }

    /// **A drag on the bar follows the pointer** (rule 4): the press starts the drag where the thumb
    /// stands, and every position the pointer reaches names the offset under it.
    ///
    /// The track is rows `2` to `6` of the region's last column. The press is at row `3`, which maps
    /// to the content's first row — where the view already stands, so it moves nothing and is only
    /// what starts the drag — and the two drags walk the track to its last cell, which is the end of
    /// the content.
    #[test]
    fn a_drag_on_the_bar_follows_the_pointer() {
        let mut state = a_drawn_state_over(A_BOX_TOO_BIG);

        let outcome = send(&click(39, 3), &mut state);
        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 0, y: 0 },
            "the press where the thumb stands moved the view"
        );

        let outcome = send(&drag(39, 5), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 0, y: 6 },
            "the drag did not follow the pointer down the track"
        );

        let outcome = send(&drag(39, 6), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 0, y: 9 },
            "the drag across the track's last cell did not reach the content's end"
        );
    }

    /// **A drag past the track stops at the content's end** (rule 2): the helper keeps reading a drag
    /// once it began on the bar, so a pointer that leaves the track names an offset past the
    /// content's span — `10` where the span is `9` — and the origin's range is what holds it.
    ///
    /// The track is rows `2` to `6` of the region's last column, so row `7` is the region's bottom
    /// border: one cell past the track, and still a drag the helper answers.
    #[test]
    fn a_drag_past_the_track_is_clamped_to_the_content() {
        let mut state = a_drawn_state_over(A_BOX_TOO_BIG);

        let outcome = send(&click(39, 3), &mut state);
        assert!(
            matches!(outcome, Control::Continue),
            "the press moved the view: {outcome:?}"
        );

        let outcome = send(&drag(39, 7), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 0, y: 9 },
            "the drag past the track did not stop at the content's end"
        );
    }

    /// **A scroll changes no session and records no step** (rule 5): the view is the state's and the
    /// walk is the session's, so a turn of the wheel leaves the history and the drawing alone.
    #[test]
    fn a_scroll_records_no_step() {
        let mut state = a_drawn_state_over(A_BOX_TOO_BIG);
        let reach = state.diagram().footprint();
        assert!(reach.is_some(), "the drawing reached nothing");

        let outcome = send(&wheel(MouseEventKind::ScrollDown, 5, 4), &mut state);

        assert!(
            matches!(outcome, Control::Changed),
            "the view did not move, so the test proves nothing: {outcome:?}"
        );
        assert!(!state.diagram().can_undo(), "the scroll recorded a step");
        assert!(!state.diagram().can_redo(), "the scroll left a step ahead");
        assert_eq!(
            state.diagram().footprint(),
            reach,
            "the scroll changed what the drawing reaches"
        );
    }

    /// **Neither an undo nor a redo restores a view** (rule 5): the moves are the session's and the
    /// view is the state's, so walking the history changes what is drawn without changing where it
    /// is drawn from.
    #[test]
    fn neither_an_undo_nor_a_redo_restores_a_view() {
        let mut state = a_drawn_state_over(A_BOX_TOO_BIG);
        let _ = send(&click(2, 2), &mut state);
        assert!(
            state.diagram().selected().is_some(),
            "the box was not selected"
        );
        let _ = send(&key(KeyCode::Right), &mut state);
        assert!(state.diagram().can_undo(), "the move recorded no step");
        let _ = send(&wheel(MouseEventKind::ScrollDown, 5, 4), &mut state);
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 0, y: 1 },
            "the wheel did not move the view, so the test proves nothing"
        );

        let outcome = send(&undo_key(), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            state.diagram().can_redo(),
            "the undo did not walk the history"
        );
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 0, y: 1 },
            "the undo restored a view"
        );

        let outcome = send(&redo_key(), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            state.diagram().can_undo(),
            "the redo did not walk the history"
        );
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 0, y: 1 },
            "the redo restored a view"
        );
    }

    /// **A scroll keeps the selection** (rule 6): the identity is the drawing's rather than a
    /// cell's, so the frame after the scroll inverts the cells the drawing records for it inside the
    /// new window, and no others.
    #[test]
    fn a_scroll_keeps_the_selection() {
        let mut state = a_drawn_state_over(A_BOX_TOO_BIG);
        let _ = send(&click(2, 2), &mut state);
        let selected = state.diagram().selected();
        assert!(selected.is_some(), "the box was not selected");
        assert!(
            reversed(&draw(&mut state), 2, 2),
            "the selection is not inverted before the scroll"
        );

        let outcome = send(&wheel(MouseEventKind::ScrollDown, 5, 4), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 0, y: 1 },
            "the wheel did not move the view, so the test proves nothing"
        );

        let buffer = draw(&mut state);
        let region = frame::regions(Rect::new(0, 0, WIDTH, HEIGHT)).region;

        assert_eq!(
            state.diagram().selected(),
            selected,
            "the scroll dropped the selection"
        );
        for y in 0..region.height - 2 {
            for x in 0..region.width - 2 {
                let at = Offset { x, y };

                assert_eq!(
                    reversed(&buffer, region.left() + 1 + x, region.top() + 1 + y),
                    state.diagram().is_cell_selected(at),
                    "the cell at {at:?} is not the inversion the drawing records"
                );
            }
        }
    }

    /// **A click after a scroll is answered at the offset from the window's corner** (rule 7): the
    /// origin belongs to the plane and takes no part in the click's arithmetic, so the cell under
    /// the pointer is the cell the person saw.
    ///
    /// `Alt` and a turn of the wheel move the view three columns across, which puts the middle box's
    /// corner at the window's `(24, 0)` — the screen's `(25, 2)`. A click that added the origin to
    /// the offset would ask about the window's `(27, 0)`, which no shape wrote.
    #[test]
    fn a_click_after_a_scroll_selects_the_cell_under_the_pointer() {
        let mut state = a_drawn_state_over(BOXES_APART);

        let outcome = send(
            &wheel_with(MouseEventKind::ScrollDown, KeyModifiers::ALT, 20, 4),
            &mut state,
        );
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 3, y: 0 },
            "the view did not move across, so the test proves nothing"
        );

        // **The frame comes before the click**, as it does in the loop: `Reply::Repaint` repaints,
        // and a click is answered from the picture the person saw.
        let _ = draw(&mut state);
        let outcome = send(&click(25, 2), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        let under = state.diagram().owner(Offset { x: 24, y: 0 });
        assert!(
            under.is_some(),
            "the cell under the pointer is not the middle box's corner"
        );
        assert_eq!(
            state.diagram().selected(),
            under,
            "the click selected another shape"
        );
    }

    /// **A press on the bar is the bar's and not the diagram's** (rule 7): it moves the view and
    /// keeps a selection that a press on a cell of the drawing would have dropped, and it still
    /// gives the region the keyboard, because a press on the border is the region's.
    #[test]
    fn a_press_on_the_bar_gives_the_region_the_keyboard() {
        let mut state = a_drawn_state_over(A_BOX_TOO_BIG);
        let _ = send(&click(2, 2), &mut state);
        let selected = state.diagram().selected();
        assert!(selected.is_some(), "the box was not selected");
        state.chrome_mut().move_to_bar();
        let _ = draw(&mut state);

        let outcome = send(&click(39, 5), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.chrome().leaf(),
            Focus::Region,
            "the press on the bar left the keyboard on the bar"
        );
        assert_eq!(
            state.diagram().selected(),
            selected,
            "the press on the bar changed the selection"
        );
        assert_eq!(
            state.diagram().origin(),
            Pos { x: 0, y: 6 },
            "the press on the bar did not move the view"
        );
    }

    #[test]
    fn a_resize_repaints_although_nobody_asked() {
        let mut state = a_drawn_state();

        let outcome = send(&Event::Resize(80, 24), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
    }

    #[test]
    fn the_error_handler_does_not_answer_with_the_error_again() {
        let mut state = a_state();
        let mut global = a_global();
        let failure: AppError = "no terminal".into();

        let outcome = error(failure, &mut state, &mut global);

        assert!(
            matches!(outcome, Ok(Control::Quit)),
            "the handler answered with the error again: {outcome:?}"
        );
        assert!(
            global.take_failure().is_some(),
            "the error was not kept to be written out"
        );
    }

    #[test]
    fn the_loop_set_the_terminal_up_itself_so_there_is_nothing_to_init() {
        assert!(init(&mut a_state(), &mut a_global()).is_ok());
    }

    #[test]
    fn pressing_d_duplicates_selected_shape_selects_new_id_and_undoes_in_one_step() {
        let fixture = r#"{
            "next_id": 2,
            "shapes": [ { "kind": "box", "id": 1, "at": { "x": 1, "y": 1 },
                          "size": { "width": 6, "height": 3 }, "stroke": "double", "text": "API" } ]
        }"#;
        let mut state = a_drawn_state_over(fixture);
        let id1 = ShapeId::new(NonZeroU16::new(1).unwrap());
        let id2 = ShapeId::new(NonZeroU16::new(2).unwrap());

        // In IDLE, 'd' is ignored.
        let d_key = Event::Key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
        let idle_outcome = send(&d_key, &mut state);
        assert!(matches!(idle_outcome, Control::Continue));
        assert_eq!(state.diagram().session().diagram().shape_count(), 1);

        // Select Box #1.
        state.diagram_mut().select(Some(id1));
        assert_eq!(
            crate::status::status_line(state.diagram()),
            "Box #1 \"API\" · Pos: (1, 1) · Size: 6×3 · Stroke: double · Fill: None"
        );
        assert!(
            crate::status::command_line(state.diagram())
                .ends_with("· [d] Duplicate · [Del] Delete · [Esc] Deselect")
        );

        // Pressing 'd' duplicates Box #1 into Box #2 and selects #2.
        let dup_outcome = send(&d_key, &mut state);
        assert!(matches!(dup_outcome, Control::Changed));
        assert_eq!(state.diagram().session().diagram().shape_count(), 2);
        assert_eq!(state.diagram().selected(), Some(id2));
        assert_eq!(
            crate::status::status_line(state.diagram()),
            "Box #2 \"API\" · Pos: (1, 1) · Size: 6×3 · Stroke: double · Fill: None"
        );

        // Undo (Ctrl+z) removes the duplicate in a single step and reconciles selection.
        let undo_key = Event::Key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        let undo_outcome = send(&undo_key, &mut state);
        assert!(matches!(undo_outcome, Control::Changed));
        assert_eq!(state.diagram().session().diagram().shape_count(), 1);
        assert_eq!(state.diagram().selected(), None);
        assert!(!state.diagram().session().can_undo());

        // Redo (Ctrl+Shift+Z) restores the duplicate in a single step.
        let redo_key = Event::Key(KeyEvent::new(
            KeyCode::Char('Z'),
            KeyModifiers::CONTROL.union(KeyModifiers::SHIFT),
        ));
        let redo_outcome = send(&redo_key, &mut state);
        assert!(matches!(redo_outcome, Control::Changed));
        assert_eq!(state.diagram().session().diagram().shape_count(), 2);
        assert!(state.diagram().session().diagram().get(id2).is_some());
    }
}
