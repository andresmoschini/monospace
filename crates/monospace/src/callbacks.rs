//! The four signatures the framework dictates, and what the application does when each is called.
//!
//! **They are free functions because the signatures are fixed** by the parameters of `run_tui`.
//! An entry the chrome takes does not decide for itself: it becomes an event, and the loop asks
//! about it again, so that an entry and the key that means the same thing are decided in one
//! place.

use rat_salsa::Control;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;

use crate::context::AppGlobal;
use crate::frame;
use crate::menu;
use crate::state::AppState;
use crate::vocabulary::{Action, AppError, AppEvent};

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
    match event {
        // An action is the only thing that changes the state.
        AppEvent::Action(action) => Ok(apply(*action)),

        // A resize repaints although nobody asked: the framework does not watch the terminal.
        AppEvent::Resized => Ok(Control::Changed),

        AppEvent::Rendered | AppEvent::Ignored => Ok(Control::Continue),

        // The chrome is asked about the event, and it may answer with an action.
        AppEvent::Input(raw) => Ok(menu::handle(raw, &mut state.menu)),
    }
}

/// What an action does to the loop.
fn apply(action: Action) -> Control<AppEvent> {
    match action {
        // The way out and the forced exit both end the loop. **They are two meanings and not two
        // code paths**: the way out grows a pop-up to confirm and the forced exit does not, which
        // is #189's, and until then they end the same way.
        Action::Quit | Action::ForceQuit => Control::Quit,
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
    use crate::vocabulary::{Action, AppError};
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

    fn key(code: KeyCode) -> AppEvent {
        AppEvent::Input(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)))
    }

    fn key_with(code: KeyCode, modifiers: KeyModifiers) -> AppEvent {
        AppEvent::Input(Event::Key(KeyEvent::new(code, modifiers)))
    }

    /// Hands an event to the application.
    fn send(app_event: &AppEvent, state: &mut AppState) -> Result<Control<AppEvent>, AppError> {
        event(app_event, state, &mut a_global())
    }

    /// Finishes one event the way the framework does: an event control is a question asked
    /// again, and the loop is called with the event it carries. This is the whole of the claim
    /// that an entry of the menu and the key that means the same thing arrive at one place.
    fn drive(app_event: &AppEvent, state: &mut AppState) -> Control<AppEvent> {
        let mut control = send(app_event, state).expect("the event reached the application");
        loop {
            match control {
                Control::Event(next) => {
                    control = send(&next, state).expect("the event reached the application");
                }
                answer => return answer,
            }
        }
    }

    #[test]
    fn the_way_out_is_reached_by_the_shortcut_and_by_the_menus_entry() {
        // The shortcut.
        let mut state = a_drawn_state();
        let outcome = drive(
            &key_with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(matches!(outcome, Control::Quit), "{outcome:?}");

        // The entry of the menu.
        let mut state = a_drawn_state();
        let _ = send(&key(KeyCode::Char('f')), &mut state);
        let _ = draw(&mut state);
        let outcome = drive(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Quit), "{outcome:?}");
    }

    #[test]
    fn a_click_on_the_entry_and_its_shortcut_apply_the_same_action() {
        // The shortcut.
        let mut state = a_drawn_state();
        let by_shortcut = drive(
            &key_with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut state,
        );

        // The click on the entry of the menu.
        let mut state = a_drawn_state();
        let _ = send(&key(KeyCode::Char('f')), &mut state);
        let buffer = draw(&mut state);
        let (column, row) = frame::cell_of(&buffer, "Quit");
        let by_click = drive(
            &AppEvent::Input(Event::Mouse(
                ratatui_crossterm::crossterm::event::MouseEvent {
                    kind: ratatui_crossterm::crossterm::event::MouseEventKind::Down(
                        ratatui_crossterm::crossterm::event::MouseButton::Left,
                    ),
                    column,
                    row,
                    modifiers: KeyModifiers::NONE,
                },
            )),
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

        let forced = AppEvent::from(Event::Key(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        )));
        let outcome = drive(&forced, &mut state);

        assert!(matches!(outcome, Control::Quit), "{outcome:?}");
    }

    #[test]
    fn a_key_reaches_the_state_only_as_an_action() {
        // **The chrome's own keys are the framework's**: the navigation character opens a
        // drop-down, and that is the widget's own state rather than the application's. What
        // this holds is the application's own half — a key the table does not name is not an
        // action, and an action is the only thing the application's handling produces.
        let mut state = a_drawn_state();

        let outcome = send(&key(KeyCode::Char('z')), &mut state).expect("the key arrived");

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert!(
            !state.menu.popup_active(),
            "a key with no action changed the state"
        );

        let outcome = send(
            &key_with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut state,
        )
        .expect("the key arrived");

        assert!(
            matches!(outcome, Control::Event(AppEvent::Action(Action::Quit))),
            "the key did not become an action: {outcome:?}"
        );
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
