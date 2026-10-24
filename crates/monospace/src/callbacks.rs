//! The four signatures the framework dictates, and what the application does when each is called.
//!
//! **They are free functions because the signatures are fixed** by the parameters of `run_tui`.
//!
//! **This is where an event becomes an action.** A terminal event is answered here: a resize
//! repaints, and everything else is routed to the first handler that wants it — the shortcuts that
//! reach everything, the chrome, the shortcuts nobody has claimed — whose answer is carried out:
//! nothing, a repaint, or an action. An entry of the menu and the key that means the same thing
//! therefore arrive at one place, and that place is here.
//!
//! **The order is the whole of the policy, and it is written once.** Nothing here names a key:
//! which keys the application answers to, and whether a key has to be answered before the chrome,
//! is `shortcuts`' table. This module only says when each layer is asked.

use rat_salsa::Control;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_crossterm::crossterm::event::Event;

use crate::context::AppGlobal;
use crate::event::AppEvent;
use crate::frame;
use crate::menu;
use crate::shortcuts;
use crate::state::AppState;
use crate::vocabulary::{Action, Answer, AppError};

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
/// **The order is the whole of the policy, and it is one line per layer.** The shortcuts that
/// reach everything are asked first: a `Ctrl+C` that reached the chrome as a key would be answered
/// by the chrome, and the point of it is that it is answered by nothing. Then the bar, so that a
/// key the widget uses is the widget's; then the shortcuts nobody has claimed; and an event all
/// three leave alone is nothing.
///
/// **The state comes whole and the handler is handed the part it owns.** The bar is given `&mut
/// state.menu` and nothing else, so what a widget may change about the application is what the
/// widget's own signature makes possible: the selection and the drop-down, and never the session.
fn route(event: &Event, state: &mut AppState) -> Answer {
    match shortcuts::handle_always(event) {
        Answer::Nothing => {}
        answer => return answer,
    }

    match menu::handle(event, &mut state.menu) {
        // The bar took no interest in the event, so the shortcuts nobody has claimed may.
        Answer::Nothing => {}
        answer => return answer,
    }

    shortcuts::handle_unclaimed(event)
}

/// What an answer does: nothing, a repaint, or an action that is carried out here.
///
/// **An action that reaches the state is carried out here, and nowhere else.** Neither of the two
/// does today — both end the loop — and the state is taken anyway: the first action that edits the
/// diagram, or opens the dialog of #189, finds it here already, and no caller changes with it.
fn apply(answer: Answer, _state: &mut AppState) -> Control<AppEvent> {
    match answer {
        Answer::Nothing => Control::Continue,
        Answer::Repaint => Control::Changed,

        // The way out and the forced exit both end the loop. **They are two meanings and not two
        // code paths**: the way out grows a pop-up to confirm and the forced exit does not, which
        // is #189's, and until then they end the same way.
        Answer::Action(Action::Quit | Action::ForceQuit) => Control::Quit,
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
    use ratatui_core::layout::Rect;
    use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

    /// The width of the screen these tests draw.
    const WIDTH: u16 = 40;

    /// The height of the screen these tests draw.
    const HEIGHT: u16 = 10;

    /// The state of an application that starts with nothing in it.
    fn a_state() -> AppState {
        AppState::new()
    }

    /// The configuration the application starts with.
    fn a_global() -> AppGlobal {
        AppGlobal::new(Config::default())
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
