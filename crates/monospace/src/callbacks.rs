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
use ratatui_core::layout::Rect;
use ratatui_crossterm::crossterm::event::Event;

use crate::context::AppGlobal;
use crate::diagram_widget;
use crate::event::AppEvent;
use crate::frame;
use crate::menu;
use crate::shortcuts;
use crate::state::AppState;
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
/// chrome, and the point of it is that it is answered by nothing. Then the bar, so that a key the
/// widget uses is the widget's; then the region, so that a click a drop-down covers is the
/// drop-down's and the one it leaves is the diagram's; then the shortcuts nobody has claimed. The
/// first action wins, and an event all of them leave alone is answered by nobody.
///
/// **The state comes whole and the handler is handed the part it owns.** The bar is given `&mut
/// state.menu` and the region `&mut state.diagram`, so what a handler may change about the
/// application is what its own signature makes possible: the selection and the drop-down, and never
/// the session.
fn route(event: &Event, state: &mut AppState) -> Option<Action> {
    shortcuts::handle_always(event)
        .or_else(|| menu::handle(event, &mut state.menu))
        .or_else(|| diagram_widget::handle(event, &mut state.diagram))
        .or_else(|| shortcuts::handle_unclaimed(event))
}

/// What an action does: paint the frame again, remove a shape, end the loop, or nothing at all.
///
/// **An action is carried out here, and nowhere else.** The delete key is the first that changes
/// the session, and it is carried out beside the two that end the loop so that a handler never does
/// anything but say what the event meant.
///
/// **The action carries the meaning and not the identity** (D2): the selected shape is read here,
/// from the state the click left, rather than minted into the action by the region. A press with
/// nothing selected performs no command and records no step, so the frame is not owed.
///
/// The way out and the forced exit both end the loop. **They are two meanings and not two code
/// paths**: the way out grows a pop-up to confirm and the forced exit does not, which is #189's,
/// and until then they end the same way.
fn apply(action: Option<Action>, state: &mut AppState) -> Control<AppEvent> {
    match action {
        None => Control::Continue,

        // The screen changed, and the loop paints the frame it is owed.
        Some(Action::ScreenChanged) => Control::Changed,

        // **The one action that changes the session.** D2: the meaning arrives from the table and
        // the identity is asked of the state, so the removal is one place and the key that means it
        // is another.
        Some(Action::Delete) => match state.diagram.selected() {
            Some(selected) => {
                state.diagram.remove(selected);
                Control::Changed
            }
            None => Control::Continue,
        },

        Some(Action::Quit | Action::ForceQuit) => Control::Quit,
    }
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
        let _ = send(&key(KeyCode::Char('f')), &mut state);
        let _ = draw(&mut state);
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
        let _ = send(&key(KeyCode::Char('f')), &mut state);
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
        let _ = send(&key(KeyCode::Char('f')), &mut state);
        assert!(state.menu.popup_active(), "the drop-down did not open");

        let outcome = send(
            &key_with(KeyCode::Char('c'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(matches!(outcome, Control::Quit), "{outcome:?}");
    }

    /// **Every shortcut of the first reach, and not the one that happens to be in the table
    /// today.** The promise is that nothing may consume these keys, and a test that named `Ctrl+C`
    /// would keep passing the day a second one is added and the chrome swallows it.
    #[test]
    fn every_shortcut_answered_first_is_answered_with_the_drop_down_open() {
        for event in shortcuts::events_answered_first() {
            let mut state = a_drawn_state();
            let _ = send(&key(KeyCode::Char('f')), &mut state);
            assert!(state.menu.popup_active(), "the drop-down did not open");

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

    /// **A press outside the interior is nobody's**, and the frame is not owed: the bar, the
    /// region's border, the status bar and a rectangle no frame drew into are all outside the
    /// window the drawing answered from.
    #[test]
    fn a_click_outside_the_interior_changes_nothing() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);

        for (column, row, what) in [
            (WIDTH - 1, 0, "the bar"),
            (3, 1, "the region's top border"),
            (0, 3, "the region's left border"),
            (3, HEIGHT - 1, "the status bar"),
            (WIDTH - 1, 3, "the region's right border"),
        ] {
            let outcome = send(&click(column, row), &mut state);

            assert!(
                matches!(outcome, Control::Continue),
                "{what} was the region's: {outcome:?}"
            );
        }

        assert!(
            state.diagram.selected().is_some(),
            "a press outside the interior dropped the selection"
        );
    }

    /// **The drop-down is asked before the region**, so a click it answers never reaches the
    /// diagram — and a click on its `Quit` entry is the way out rather than a selection.
    #[test]
    fn a_click_the_drop_down_takes_never_reaches_the_diagram() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&key(KeyCode::Char('f')), &mut state);
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

    /// **A press with nothing selected is not a command.** It removes nothing, records no step and
    /// owes no frame; the drawing still holds what it held.
    #[test]
    fn the_delete_key_with_nothing_selected_changes_nothing() {
        let mut state = a_drawn_state_over(A_BOX);

        let outcome = send(&key(KeyCode::Delete), &mut state);
        let _ = draw(&mut state);

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
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

    /// **`Delete` is `Unclaimed`, so the drop-down does not keep it** (D3): the figure behind the
    /// open menu is removed and the menu's own state — including whether it is open — is untouched.
    #[test]
    fn the_delete_key_is_answered_with_the_drop_down_open() {
        let mut state = a_drawn_state_over(A_BOX);
        let _ = send(&click(3, 3), &mut state);
        let _ = send(&key(KeyCode::Char('f')), &mut state);
        assert!(state.menu.popup_active(), "the drop-down did not open");

        let outcome = send(&key(KeyCode::Delete), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(
            state.diagram.selected(),
            None,
            "the drop-down kept the selection"
        );
        assert!(
            state.menu.popup_active(),
            "the deletion closed the drop-down"
        );

        let _ = draw(&mut state);
        assert!(
            state.diagram.owner(Offset { x: 2, y: 1 }).is_none(),
            "the drop-down kept the delete key from the diagram"
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
