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
use rat_widget::event::Outcome;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_crossterm::crossterm::event::{Event, MouseButton, MouseEventKind};

use crate::about;
use crate::context::AppGlobal;
use crate::diagram_widget;
use crate::event::AppEvent;
use crate::frame;
use crate::menu;
use crate::shortcuts;
use crate::state::{AppState, Focus};
use crate::vocabulary::{Action, AppError};

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

        AppEvent::Terminal(event) => apply(route(event, state), state),
    })
}

/// Which handler answers the event, and what it answers.
///
/// **The order is the whole of the policy, and the chain is the order.** The shortcuts that reach
/// everything are asked first: a `Ctrl+C` that reached the chrome as a key would be answered by the
/// chrome, and the point of it is that it is answered by nothing. **Then the dialog, while it is
/// up**: it is the overlay, it takes every key and every control, and an event it leaves alone
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
/// (D1): [`address`] gives the keyboard to the leaf a press lands in, and says whether it moved,
/// because the frame that follows is owed even where nothing answers.
///
/// **The state comes whole and the handler is handed the part it owns.** The bar is given `&mut
/// state.menu` and the leaf the keyboard is on, and the region `&mut state.diagram`, so what a
/// handler may change about the application is what its own signature makes possible: the selection,
/// the drop-down and which leaf holds the keyboard, and never the session.
fn route(event: &Event, state: &mut AppState) -> Option<Action> {
    if let Some(action) = shortcuts::handle_always(event) {
        return Some(action);
    }

    // **The dialog is the overlay** (D6): while it is up, it is the only thing asked. A closing
    // event paints; anything else it consumed reaches nothing. `Ctrl+C` was answered above, which
    // is what keeps the forced exit out of the dialog's hands.
    if state.about.active() {
        return match about::handle(event, &mut state.about) {
            Outcome::Changed => Some(Action::ScreenChanged),
            _ => None,
        };
    }

    let moved = address(event, state);

    menu::handle(event, &mut state.menu, &mut state.focus)
        .or_else(|| diagram_widget::handle(event, &mut state.diagram, state.focus))
        .or_else(|| shortcuts::handle_unclaimed(event))
        .or_else(|| moved.then_some(Action::ScreenChanged))
}

/// **A press is addressed before it is answered** (D1 of #189): the leaf it lands in is who the
/// event is for, so the keyboard moves to it before the bar and the region are asked — a click on a
/// label opens nothing if the bar is told it does not hold the keyboard, and a click in the region
/// closes the drop-down on the same press.
///
/// **The keyboard moves where the event is addressed** (D1 of #189), and this is the press's
/// half of it: a click is addressed by its position, and the bar's own three keys are addressed by
/// [`crate::menu::handle`]. It happens before the handlers are asked because the label a press opens
/// and the drop-down a press closes are changes they make while they are asked, so moving the
/// keyboard afterwards would answer the wrong leaf. No action names a leaf.
///
/// **A press on the status line, and one on a rectangle no frame drew into, lands in neither leaf**
/// and moves nothing. **The answer is whether the keyboard moved**, because that is a frame owed
/// even where no handler answers it: the label that was reversed is not any more.
fn address(event: &Event, state: &mut AppState) -> bool {
    let Event::Mouse(mouse) = event else {
        return false;
    };

    if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
        return false;
    }

    state.set_focus_at(Position::new(mouse.column, mouse.row))
}

/// What an action does: paint the frame again, change the session, end the loop, or nothing at
/// all.
///
/// **An action is carried out here, and nowhere else.** The delete key performs the one command
/// this slice issues, the two walks give a step back, and `About` opens the dialog; all of them are
/// carried out beside the two that end the loop so that a handler never does anything but say what
/// the event meant. **Every meaning that is carried out also leaves the keyboard on the region**
/// (rule 13 of #189), and that is said once, at the end, rather than in each arm.
///
/// **No action names a leaf** (rule 5): the keyboard moves where the event is addressed — a press in
/// `route`, the bar's three keys in `menu::handle` — so the only thing the rule at the end exempts
/// is the screen changing, which carries nothing out.
///
/// **The action carries the meaning and not the identity** (D2): the selected shape is read here,
/// from the state the click left, rather than minted into the action by the region. A press with
/// nothing selected performs no command and records no step, and the frame is owed all the same:
/// carrying the action out repaints, which is what makes the Edit menu's entry close on the screen
/// with nothing selected.
///
/// The way out and the forced exit both end the loop. **They are two meanings and not two code
/// paths**: the way out grows a pop-up to confirm and the forced exit does not, which is #189's,
/// and until then they end the same way.
fn apply(action: Option<Action>, state: &mut AppState) -> Control<AppEvent> {
    let Some(action) = action else {
        return Control::Continue;
    };

    match action {
        // **The two that end the loop**, and the only two that are not carried out.
        Action::Quit | Action::ForceQuit => return Control::Quit,

        // **The frame is owed, but no action is carried out.** `route` has already addressed any
        // press, and this action preserves that focus and the menu state instead of applying the
        // cleanup below.
        Action::ScreenChanged => return Control::Changed,

        // **The walks are the session's** (D2 of #181): each gives back one step and keeps no copy
        // of where the walk stands. At either end the session answers with itself, and the frame is
        // owed all the same, because the action was carried out.
        Action::Undo => state.diagram.undo(),

        Action::Redo => state.diagram.redo(),

        // **The one action that performs a command.** D2: the meaning arrives from the table and
        // the identity is asked of the state, so the removal is one place and the key that means it
        // is another. **The frame is owed either way**: an action that was carried out repaints, so
        // taking the Edit menu's entry closes the drop-down on the screen even when there was no
        // figure to remove.
        Action::Delete => {
            if let Some(selected) = state.diagram.selected() {
                state.diagram.remove(selected);
            }
        }

        // **The movement is the same shape of action as the deletion** (rule 4 of #179): the delta
        // arrives from the region's handler and the identity is read here. **But a move with
        // nothing selected is answered before the cleanup**, because nothing was carried out: no
        // command, no step and no frame owed.
        Action::Move(dx, dy) => {
            let Some(selected) = state.diagram.selected() else {
                return Control::Continue;
            };

            state.diagram.move_shape(selected, dx, dy);
        }

        // **The one action that opens an overlay.** The dialog is the chrome's own, so the
        // session is untouched: no command is performed and no step is recorded (D5 of #189).
        Action::About => about::open(&mut state.about),
    }

    // **A meaning that is carried out leaves the keyboard on the region** (rule 13 of #189): what a
    // person took from the bar is done, and what is left to do is in the drawing. The drop-down the
    // bar was showing closes with it, because a menu nobody holds the keyboard on is not open.
    // **Only actions carried out reach this cleanup**: `ScreenChanged` returns above with the
    // focus and menu state that routing established. Keeping the cleanup here means every other
    // carried-out action closes the drop-down and leaves the keyboard on the region.
    state.menu.set_popup_active(false);
    state.focus = Focus::Region;
    Control::Changed
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
    use super::*;
    use crate::config::Config;
    use crate::status;
    use crate::vocabulary::AppError;
    use monospace_core::Offset;
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
        #[allow(
            clippy::default_trait_access,
            reason = "the diagram's type is not named here: the application depends on \
            the editing layer and not on the model below it"
        )]
        let session = monospace_editing::Session::new(Default::default());

        AppState::new(session)
    }

    /// The configuration the application starts with.
    fn a_global() -> AppGlobal {
        AppGlobal::new(Config::default(), monospace_glyph_sets::all())
    }

    /// A description whose only shape is a light 4×3 box at `(2, 1)` of the window.
    ///
    /// The window is the region's interior, which on a screen of the size these tests draw is
    /// `38×6` at `(1, 2)` of the screen: the box's top-left is the screen's `(3, 3)`.
    const A_BOX: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                      "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
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
                        "leaving": "right", "terminal": { "kind": "arm" } },
              "to":   { "at": { "kind": "reference", "shape": 2, "anchor": "left" },
                        "leaving": "left", "terminal": { "kind": "arm" } },
              "stroke": "light" }
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
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
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
            state.focus,
            Focus::Bar,
            "Alt+{label} did not give the bar the keyboard"
        );
        assert!(
            state.menu.popup_active(),
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
        assert!(state.about.active(), "taking About opened no dialog");
        assert_eq!(
            state.focus,
            Focus::Region,
            "taking About left the keyboard on the bar"
        );
        assert!(
            !state.menu.popup_active(),
            "taking About left the drop-down open"
        );
    }

    /// **The chrome's own and no session** (rule 8): About opens an overlay and records no step,
    /// so the drawing and its history are exactly what they were.
    #[test]
    fn the_about_action_changes_no_session() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram.selected();
        assert!(
            selected.is_some(),
            "the box was not selected, so the test proves nothing"
        );

        open_the_about_dialog(&mut state);

        assert_eq!(
            state.diagram.selected(),
            selected,
            "the dialog changed the selection"
        );
        assert!(
            !state.diagram.can_undo(),
            "opening the dialog recorded a step"
        );
        assert!(
            !state.diagram.can_redo(),
            "opening the dialog left a step ahead"
        );
        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_some(),
            "opening the dialog removed the box"
        );
    }

    /// **The closing event reaches nothing else** (rule 10): `Esc` closes the dialog, leaves the
    /// keyboard where the dialog was given it — the region, rule 13 — opens no drop-down, and
    /// changes neither the selection nor the history.
    #[test]
    fn escape_closes_the_about_dialog_and_the_event_reaches_nothing_else() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram.selected();
        open_the_about_dialog(&mut state);
        assert!(state.about.active(), "the dialog is not open");
        let focus = state.focus;

        let outcome = send(&key(KeyCode::Esc), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.about.active(), "escape did not close the dialog");
        assert_eq!(state.focus, focus, "the closing event moved the keyboard");
        assert!(
            !state.menu.popup_active(),
            "the closing event opened a drop-down"
        );
        assert_eq!(
            state.diagram.selected(),
            selected,
            "the closing event changed the selection"
        );
        assert!(
            !state.diagram.can_undo(),
            "the closing event recorded a step"
        );
    }

    /// **The overlay takes the controls behind it** (rule 10): `Ctrl+Q` is `Unclaimed`, so while
    /// the dialog is up the dialog answers it and nothing happens, and a click on the bar behind
    /// it moves no keyboard and opens no menu. `Ctrl+C` is answered before the dialog, and
    /// [`control_c_ends_the_application_whatever_the_state_is`] is where that is proven.
    #[test]
    fn control_q_does_nothing_while_the_dialog_is_up() {
        let mut state = a_drawn_state_over(A_BOX);
        open_the_about_dialog(&mut state);
        let (column, row) = frame::cell_of(&draw(&mut state), "File");
        let focus = state.focus;

        let outcome = send(
            &key_with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(
            matches!(outcome, Control::Continue),
            "the way out was answered through the dialog: {outcome:?}"
        );
        assert!(state.about.active(), "Ctrl+Q closed the dialog");

        // The bar is behind the overlay even where the dialog does not cover it.
        let _ = send(&click(column, row), &mut state);

        assert_eq!(
            state.focus, focus,
            "the click on the bar moved the keyboard"
        );
        assert!(
            !state.menu.popup_active(),
            "the click behind the dialog opened a drop-down"
        );
        assert!(
            state.about.active(),
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
        open_the_drop_down(&mut state, 'f');
        let outcome = send(&key(KeyCode::Enter), &mut state);

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
        open_the_drop_down(&mut state, 'f');
        let buffer = draw(&mut state);
        let (column, row) = frame::cell_of(&buffer, "Quit");
        let by_click = send(
            &Event::Mouse(ratatui_crossterm::crossterm::event::MouseEvent {
                kind: ratatui_crossterm::crossterm::event::MouseEventKind::Down(
                    ratatui_crossterm::crossterm::event::MouseButton::Left,
                ),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }),
            &mut state,
        );

        assert_eq!(
            by_click, by_shortcut,
            "the click and the shortcut are two code paths"
        );
    }

    #[test]
    fn control_c_ends_the_application_whatever_the_state_is() {
        // The drop-down is open: the chrome holds the keyboard, and the forced exit is not the
        // chrome's to answer.
        let mut state = a_drawn_state();
        open_the_drop_down(&mut state, 'f');

        let outcome = send(
            &key_with(KeyCode::Char('c'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(matches!(outcome, Control::Quit), "{outcome:?}");

        // The About dialog is up: the overlay takes every control but this one, which is
        // answered before it (D6 of #189).
        let mut state = a_drawn_state();
        open_the_about_dialog(&mut state);
        assert!(state.about.active(), "the dialog is not open");

        let outcome = send(
            &key_with(KeyCode::Char('c'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(
            matches!(outcome, Control::Quit),
            "the dialog swallowed the forced exit: {outcome:?}"
        );
    }

    /// **Every shortcut of the first reach, and not the one that happens to be in the table
    /// today.** The promise is that nothing may consume these keys, and a test that named `Ctrl+C`
    /// would keep passing the day a second one is added and the chrome swallows it.
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
            !state.menu.popup_active(),
            "a key with no action changed the state"
        );
    }

    /// **A click asks the drawing rather than a drawing of its own**, and the identity it finds is
    /// what the state holds afterwards.
    #[test]
    fn a_left_click_selects_the_shape_that_owns_the_cell() {
        let mut state = a_drawn_state_over(A_BOX);
        let owner = state.diagram.owner(Offset { x: 2, y: 1 });
        assert!(owner.is_some(), "the box is not in the drawing");

        // The screen's `(3, 3)` is the window's `(2, 1)`: the region's interior starts one row
        // and one column in from the region's corner.
        let outcome = send(&click(3, 3), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram.selected(),
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
            state.diagram.selected().is_some(),
            "the box was not selected"
        );

        // The window's own corner: the interior, and no shape wrote it.
        let outcome = send(&click(1, 2), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram.selected(),
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
            state.focus,
            Focus::Region,
            "the bar started with the keyboard"
        );
    }

    /// **A left press inside the region's rectangle gives it the keyboard, and inside the interior
    /// it selects as well.** The bar held it first, so the press is what moves it.
    #[test]
    fn a_press_inside_the_region_moves_the_keyboard_and_selects() {
        let mut state = a_drawn_state_over(A_BOX);
        state.focus = Focus::Bar;
        let _ = draw(&mut state);

        let outcome = send(&click(3, 3), &mut state);

        assert_eq!(
            state.focus,
            Focus::Region,
            "the press left the keyboard on the bar"
        );
        assert!(
            state.diagram.selected().is_some(),
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
        let selected = state.diagram.selected();
        assert!(
            selected.is_some(),
            "the box was not selected, so the test proves nothing"
        );

        for (column, row, what) in [
            (3, 1, "the top border"),
            (0, 3, "the left border"),
            (WIDTH - 1, 3, "the right border"),
            (3, HEIGHT - 2, "the bottom border"),
        ] {
            state.focus = Focus::Bar;
            let _ = draw(&mut state);

            let outcome = send(&click(column, row), &mut state);

            assert_eq!(
                state.focus,
                Focus::Region,
                "{what} did not give the region the keyboard"
            );
            assert_eq!(
                state.diagram.selected(),
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
        let selected = state.diagram.selected();
        state.focus = Focus::Bar;
        let _ = draw(&mut state);

        let outcome = send(&click(3, HEIGHT - 1), &mut state);

        assert_eq!(state.focus, Focus::Bar, "the status line took the keyboard");
        assert_eq!(
            state.diagram.selected(),
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
            state.focus,
            Focus::Bar,
            "the label did not give the bar the keyboard"
        );
        assert!(
            state.menu.popup_active(),
            "the label did not open its drop-down"
        );
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
    }

    /// **A press in the region is not swallowed by an open drop-down** (D6): it moves the keyboard
    /// to the region, closes the drop-down on the same press, and selects inside the interior.
    #[test]
    fn a_press_in_the_region_closes_the_drop_down_and_selects() {
        let mut state = a_drawn_state_over(A_BOX_AND_WHAT_HANGS_FROM_IT);
        open_the_drop_down(&mut state, 'f');
        assert!(state.menu.popup_active(), "the drop-down did not open");

        // The right box's top-left is outside the File drop-down, so the press is the region's.
        let outcome = send(&click(17, 3), &mut state);

        assert_eq!(
            state.focus,
            Focus::Region,
            "the press left the keyboard on the bar; the drop-down is {:?}",
            state.menu.popup.area
        );
        assert!(
            !state.menu.popup_active(),
            "the drop-down survived the keyboard leaving it"
        );
        assert!(
            state.diagram.selected().is_some(),
            "the press inside the interior selected nothing"
        );
        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
    }

    /// **The arrows are not the bar's while the region holds the keyboard** (rule 5): they open no
    /// drop-down and move no label, so #179 and #176 can bind them from here.
    #[test]
    fn the_arrows_are_not_the_bars_while_the_region_holds_the_keyboard() {
        let mut state = a_drawn_state();
        assert_eq!(
            state.focus,
            Focus::Region,
            "the bar started with the keyboard"
        );

        for arrow in [KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down] {
            let selected = state.menu.bar.selected;
            let outcome = send(&key(arrow), &mut state);

            assert!(
                matches!(outcome, Control::Continue),
                "{arrow:?}: {outcome:?}"
            );
            assert!(!state.menu.popup_active(), "{arrow:?} opened a drop-down");
            assert_eq!(
                state.menu.bar.selected, selected,
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
            state.diagram.selected().is_some(),
            "the box was not selected"
        );
        let before = draw(&mut state);

        let outcome = send(&key(KeyCode::Right), &mut state);
        let after = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(state.diagram.can_undo(), "the move recorded no step");
        assert_eq!(
            state.diagram.owner(Offset { x: 3, y: 1 }),
            state.diagram.selected(),
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
                state.diagram.selected().is_some(),
                "{arrow:?}: the box was not selected"
            );

            let outcome = send(&key(arrow), &mut state);
            let _ = draw(&mut state);

            assert!(
                matches!(outcome, Control::Changed),
                "{arrow:?}: {outcome:?}"
            );
            assert_eq!(
                state.diagram.owner(at),
                state.diagram.selected(),
                "{arrow:?} did not move the box along its own axis"
            );
            assert!(state.diagram.can_undo(), "{arrow:?} recorded no step");
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
            state.diagram.owner(Offset { x: 4, y: 1 }).is_some(),
            "the two presses did not move the box twice"
        );
        assert_eq!(
            state.diagram.owner(Offset { x: 3, y: 1 }),
            None,
            "the second press moved nothing"
        );

        let _ = send(&undo_key(), &mut state);
        let _ = draw(&mut state);

        assert!(
            state.diagram.owner(Offset { x: 3, y: 1 }).is_some(),
            "one undo did not give back one press"
        );

        let _ = send(&undo_key(), &mut state);
        let _ = draw(&mut state);

        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_some(),
            "the second undo did not give back the first press"
        );
        assert!(
            !state.diagram.can_undo(),
            "two presses were more than two steps"
        );
    }

    /// **The selection survives the move** (rule 3): the next frame inverts the cells the figure
    /// now owns rather than the ones it left.
    #[test]
    fn a_move_keeps_the_selection() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram.selected();
        assert!(selected.is_some(), "the box was not selected");

        let _ = send(&key(KeyCode::Right), &mut state);
        let buffer = draw(&mut state);

        assert_eq!(
            state.diagram.selected(),
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

    /// **A move with nothing selected performs no command** (rule 5): no step is recorded and the
    /// answer is `Continue` — nothing was carried out, so no frame is owed.
    #[test]
    fn a_move_with_nothing_selected_performs_no_command() {
        let mut state = a_drawn_state_over(A_BOX);

        let outcome = send(&key(KeyCode::Right), &mut state);

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert!(
            !state.diagram.can_undo(),
            "a move with nothing selected recorded a step"
        );
        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_some(),
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
            state.diagram.selected().is_some(),
            "the box was not selected"
        );
        open_the_about_dialog(&mut state);

        let _ = send(&key(KeyCode::Right), &mut state);

        assert!(
            state.about.active(),
            "the arrow closed the dialog it should not have reached"
        );
        assert!(
            !state.diagram.can_undo(),
            "a key the dialog swallowed recorded a step"
        );
        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_some(),
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
            state.diagram.selected().is_some(),
            "the box was not selected"
        );
        state.focus = Focus::Bar;
        state.menu.set_popup_active(true);

        let outcome = apply(Some(Action::Move(1, 0)), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.focus,
            Focus::Region,
            "the move left the keyboard on the bar"
        );
        assert!(!state.menu.popup_active(), "the move left a drop-down open");
        assert!(state.diagram.can_undo(), "the move recorded no step");
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

        assert!(state.diagram.can_undo(), "the presses recorded no step");

        // The region's own border is a box too, so the window is what is asked: no offset of the
        // drawing answers a glyph, and the copy therefore writes none.
        let catalog = monospace_glyph_sets::all();
        for y in 0..6 {
            for x in 0..38 {
                assert!(
                    state.diagram.glyph(Offset { x, y }, &catalog).is_none(),
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
            assert_eq!(state.focus, Focus::Region, "the test starts on the bar");

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
            state.focus,
            Focus::Region,
            "a combination that names no label moved the keyboard"
        );
        assert!(!state.menu.popup_active(), "a drop-down opened");
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
        assert!(!state.menu.popup_active(), "escape left the drop-down open");
        assert_eq!(state.focus, Focus::Bar, "the first escape left the bar");
        let (column, row) = frame::cell_of(&frame, "File");
        assert!(
            reversed(&frame, column, row),
            "the label stopped looking focused with the bar still holding the keyboard"
        );

        let outcome = send(&key(KeyCode::Esc), &mut state);
        let frame = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.focus,
            Focus::Region,
            "the second escape did not give the keyboard back"
        );
        assert!(
            !reversed(&frame, column, row),
            "a label is reversed with the region holding the keyboard"
        );
    }

    /// **`Esc` is not the region's**: with the keyboard there the key changes nothing, so the bar's
    /// way out is a press that acts on the chrome and never on the drawing.
    #[test]
    fn escape_in_the_region_changes_nothing() {
        let mut state = a_drawn_state();
        assert_eq!(
            state.focus,
            Focus::Region,
            "the region does not start with the keyboard"
        );

        let outcome = send(&key(KeyCode::Esc), &mut state);

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert_eq!(state.focus, Focus::Region, "escape moved the keyboard");
        assert!(!state.menu.popup_active(), "escape opened a drop-down");
    }

    /// **`F10` asks for the bar where a bare `Alt` cannot** (D4, amended): it moves the keyboard and
    /// opens nothing, so the bar looks focused while the region stays the picture under it.
    #[test]
    fn f10_gives_the_bar_the_keyboard_without_opening_a_drop_down() {
        let mut state = a_drawn_state();
        assert_eq!(state.focus, Focus::Region, "the test starts on the bar");

        let outcome = send(&key(KeyCode::F(10)), &mut state);
        let frame = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.focus,
            Focus::Bar,
            "F10 did not give the bar the keyboard"
        );
        assert!(!state.menu.popup_active(), "F10 opened a drop-down");

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
        open_the_drop_down(&mut state, 'f');
        let buffer = draw(&mut state);
        let (column, row) = frame::cell_of(&buffer, "Quit");

        let outcome = send(&click(column, row), &mut state);

        assert!(matches!(outcome, Control::Quit), "{outcome:?}");
        assert_eq!(
            state.diagram.selected(),
            None,
            "the click the drop-down took reached the diagram"
        );
    }

    /// **The delete key is the first action that changes the session.** The selected shape is gone
    /// from the drawing the next frame makes, and the frame is owed because of it.
    #[test]
    fn the_delete_key_removes_the_selected_shape() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram.selected().is_some(),
            "the box was not selected"
        );

        let outcome = send(&key(KeyCode::Delete), &mut state);
        let buffer = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_none(),
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
            state.diagram.selected(),
            None,
            "the identity survived the removal"
        );
        assert!(
            !reversed(&buffer, 3, 3),
            "the frame after the removal inverts a cell"
        );
    }

    /// **A press with nothing selected is not a command.** It removes nothing and records no step,
    /// and the frame is painted again because the action was carried out: that is the repaint an
    /// entry taken with nothing selected needs, and the key costs the same one.
    #[test]
    fn the_delete_key_with_nothing_selected_removes_nothing() {
        let mut state = a_drawn_state_over(A_BOX);

        let outcome = send(&key(KeyCode::Delete), &mut state);
        let _ = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.diagram.can_undo(),
            "a press with nothing selected recorded a step"
        );
        assert_eq!(
            state.diagram.selected(),
            None,
            "a press with nothing selected left a selection"
        );
        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_some(),
            "the box was removed without a selection"
        );
    }

    /// **The Edit menu's entry is the delete key's action.** The menu carries no selection, so the
    /// entry is offered and taken with a figure selected, and it removes it.
    #[test]
    fn the_edit_menus_delete_entry_removes_the_selected_shape() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram.selected().is_some(),
            "the box was not selected"
        );

        let outcome = take_the_entry(&mut state, 'e', "Delete");

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.menu.popup_active(), "the entry left the menu open");
        assert_eq!(
            state.diagram.selected(),
            None,
            "the entry kept the identity it removed"
        );

        let _ = draw(&mut state);
        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_none(),
            "the entry did not remove the selected shape"
        );
    }

    /// **The entry is offered with nothing selected**, not only when a figure is. Taking it removes
    /// nothing and records no step, and the frame is still painted: the action was carried out, and
    /// the drop-down the person saw was closed by the entry.
    #[test]
    fn the_edit_menus_delete_entry_is_offered_with_nothing_selected() {
        let mut state = a_drawn_state_over(A_BOX);

        let outcome = take_the_entry(&mut state, 'e', "Delete");

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            !state.menu.popup_active(),
            "the entry stayed open with nothing to remove"
        );
        assert!(
            !state.diagram.can_undo(),
            "the entry recorded a step with nothing selected"
        );

        let _ = draw(&mut state);
        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_some(),
            "a figure was removed with nothing selected"
        );
    }

    /// **`Delete` is `Unclaimed`, so the drop-down does not keep it** (D3): the figure behind the
    /// open menu is removed, which is what proves the key reached the session. **The keyboard then
    /// goes back to the region** (rule 13), so the drop-down the bar was showing closes with it —
    /// the removal is taken and the person is left looking at the drawing.
    #[test]
    fn the_delete_key_with_the_drop_down_open_removes_and_takes_the_keyboard_back() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        open_the_drop_down(&mut state, 'f');

        let outcome = send(&key(KeyCode::Delete), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram.selected(),
            None,
            "the drop-down kept the selection"
        );
        assert_eq!(
            state.focus,
            Focus::Region,
            "the removal left the keyboard on the bar"
        );
        assert!(
            !state.menu.popup_active(),
            "a drop-down is open with the keyboard on the region"
        );

        let _ = draw(&mut state);
        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_none(),
            "the drop-down kept the delete key from the diagram"
        );
    }

    /// **The Edit menu's entry leaves the keyboard on the region** (rule 13): the shape the person
    /// acted on is gone, and what is left to do is in the drawing.
    #[test]
    fn taking_the_delete_entry_gives_the_keyboard_back_to_the_region() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        assert!(
            state.diagram.selected().is_some(),
            "the box was not selected, so the test proves nothing"
        );

        let outcome = take_the_entry(&mut state, 'e', "Delete");

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.focus,
            Focus::Region,
            "the entry left the keyboard on the bar"
        );
        assert!(
            !state.menu.popup_active(),
            "the entry left a drop-down open"
        );
    }

    /// **The table answers a meaning, and not an identity** (D2): `Action::Delete` has no field to
    /// carry one, and the shape is read from the state by `apply` rather than minted by a handler.
    #[test]
    fn the_delete_shortcut_says_only_that_it_deletes() {
        assert_eq!(
            shortcuts::handle_unclaimed(&key(KeyCode::Delete)),
            Some(Action::Delete),
            "the delete key is not a row of the table, or says something else"
        );
        assert_eq!(
            shortcuts::handle_always(&key(KeyCode::Delete)),
            None,
            "the delete key reaches before the chrome"
        );
    }

    /// **An entry with nothing to give back changes nothing and paints** (rule 11): the drop-down
    /// closes, the drawing is the one it already showed, and no step is recorded either way.
    #[test]
    fn taking_an_entry_with_nothing_to_give_back_changes_nothing() {
        let mut state = a_drawn_state_over(A_BOX);
        let before = draw(&mut state);

        let outcome = take_the_entry(&mut state, 'e', "Undo");

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.menu.popup_active(), "the entry left the menu open");
        assert!(!state.diagram.can_undo(), "the entry recorded a step");
        assert!(!state.diagram.can_redo(), "the entry left a step ahead");

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
        assert!(!state.menu.popup_active(), "the entry left the menu open");
        assert!(!state.diagram.can_undo(), "the redo entry recorded a step");
        assert!(
            !state.diagram.can_redo(),
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
            state.diagram.owner(Offset { x: 2, y: 1 }).is_none(),
            "the box was not removed, so the test proves nothing"
        );

        let outcome = send(&undo_key(), &mut state);
        let buffer = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_some(),
            "the undo did not give the box back"
        );
        assert_eq!(
            buffer[(3, 3)].symbol(),
            "┌",
            "the frame after the undo does not draw the box"
        );
        assert!(!state.diagram.can_undo(), "the undo left a step behind");
        assert!(state.diagram.can_redo(), "the undo left nothing to redo");
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
            state.diagram.owner(Offset { x: 2, y: 1 }).is_some(),
            "the undo did not give the box back, so the test proves nothing"
        );

        let outcome = send(&redo_key(), &mut state);
        let buffer = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_none(),
            "the redo did not take the box out again"
        );
        assert_eq!(
            buffer[(3, 3)].symbol(),
            " ",
            "the frame after the redo still draws the box"
        );
        assert!(state.diagram.can_undo(), "the redo left nothing to undo");
        assert!(!state.diagram.can_redo(), "the redo left a step ahead");
    }

    /// **At the oldest position `Ctrl+Z` walks nothing and paints** (rules 3 and 8): the session is
    /// unchanged, no step is recorded, and the frame is owed because the action was carried out.
    #[test]
    fn the_undo_key_with_nothing_behind_changes_nothing() {
        let mut state = a_drawn_state_over(A_BOX);

        let outcome = send(&undo_key(), &mut state);
        let buffer = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(!state.diagram.can_undo(), "the press recorded a step");
        assert!(!state.diagram.can_redo(), "the press left a step ahead");
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
        assert!(!state.diagram.can_undo(), "the press recorded a step");
        assert!(!state.diagram.can_redo(), "the press left a step ahead");
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
            state.diagram.owner(Offset { x: 2, y: 1 }).is_none(),
            "the box was not removed, so the test proves nothing"
        );

        open_the_drop_down(&mut state, 'f');
        let outcome = send(&undo_key(), &mut state);
        let _ = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_some(),
            "the drop-down kept the undo key from the history"
        );
        assert_eq!(
            state.focus,
            Focus::Region,
            "the undo left the keyboard on the bar"
        );
        assert!(
            !state.menu.popup_active(),
            "a drop-down is open with the keyboard on the region"
        );

        open_the_drop_down(&mut state, 'f');
        let outcome = send(&redo_key(), &mut state);
        let _ = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_none(),
            "the drop-down kept the redo key from the history"
        );
        assert_eq!(
            state.focus,
            Focus::Region,
            "the redo left the keyboard on the bar"
        );
        assert!(
            !state.menu.popup_active(),
            "a drop-down is open with the keyboard on the region"
        );
    }

    /// **The table answers a meaning and not a count of steps** (rule 6): `Action::Undo` has no
    /// field, and the session is asked to walk where the action is carried out.
    #[test]
    fn the_undo_shortcut_says_only_that_it_undoes() {
        assert_eq!(
            shortcuts::handle_unclaimed(&undo_key()),
            Some(Action::Undo),
            "the undo key is not a row of the table, or says something else"
        );
        assert_eq!(
            shortcuts::handle_always(&undo_key()),
            None,
            "the undo key reaches before the chrome"
        );
    }

    /// **The redo key says a meaning too**, and reaches after the chrome as the undo does.
    #[test]
    fn the_redo_shortcut_says_only_that_it_redoes() {
        assert_eq!(
            shortcuts::handle_unclaimed(&redo_key()),
            Some(Action::Redo),
            "the redo key is not a row of the table, or says something else"
        );
        assert_eq!(
            shortcuts::handle_always(&redo_key()),
            None,
            "the redo key reaches before the chrome"
        );
    }

    /// **A walk does not touch the selection** (rule 7). The box is selected again after the undo,
    /// the redo takes the drawing out from under it, and the identity survives: no cell of the
    /// restored drawing records it, so the next frame inverts nothing.
    #[test]
    fn the_undo_and_redo_keys_leave_the_selection_alone() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Delete), &mut state);
        let _ = send(&undo_key(), &mut state);
        let _ = draw(&mut state);

        assert_eq!(
            state.diagram.selected(),
            None,
            "the undo restored a selection"
        );

        let _ = send(&click(3, 3), &mut state);
        let selected = state.diagram.selected();
        assert!(selected.is_some(), "the box was not selected again");

        let outcome = send(&redo_key(), &mut state);
        let buffer = draw(&mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram.selected(),
            selected,
            "the redo changed the selection"
        );
        assert!(
            !reversed(&buffer, 3, 3),
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

        for line in 0..HEIGHT {
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
            state.diagram.selected().is_some(),
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
}
