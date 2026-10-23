//! The four signatures the framework dictates, and what the application does when each is called.
//!
//! **They are free functions because the signatures are fixed** by the parameters of `run_tui`.
//!
//! **This is where an event becomes an action.** The chrome and the application's own keys are
//! handlers: each is handed the event and answers with what it was worth — nothing, a repaint, or
//! an action — and this module asks them in one fixed order and turns the answer into what the
//! loop does next. An entry of the menu and the key that means the same thing therefore arrive at
//! one place, and that place is here.

use rat_salsa::Control;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

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

        AppEvent::Terminal(event) => terminal(event, state),
    })
}

/// What the application does about something the terminal said.
///
/// **The order is the rule.** The forced exit is asked first, because `Ctrl+C` that reached the
/// chrome as a key would be answered by the chrome, and the point of it is that it is answered by
/// nothing. A resize comes next, because no widget wants one and a screen that is not painted
/// after it keeps the layout it had. The bar is asked before the application's own shortcuts, so
/// that a key the widget uses is the widget's.
fn terminal(event: &Event, state: &mut AppState) -> Control<AppEvent> {
    if is_forced_exit(event) {
        return apply(Action::ForceQuit);
    }

    if matches!(event, Event::Resize(_, _)) {
        return Control::Changed;
    }

    match menu::handle(event, &mut state.menu) {
        // The bar took no interest in the event, so the application's own keys may.
        Answer::Nothing => control(shortcuts::handle(event)),
        answer => control(answer),
    }
}

/// What an answer does to the loop.
fn control(answer: Answer) -> Control<AppEvent> {
    match answer {
        Answer::Nothing => Control::Continue,
        Answer::Repaint => Control::Changed,
        Answer::Action(action) => apply(action),
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

/// Whether the event is the forced exit: `Ctrl+C`, and nothing else.
///
/// **Only a press answers, and only that combination.** The terminal reports the release and the
/// repeat of a key as well, and a shortcut that answered to those would end the application
/// twice over; and `Ctrl+Shift+C` is a key of its own, not the forced exit.
fn is_forced_exit(event: &Event) -> bool {
    let Event::Key(key) = event else {
        return false;
    };

    key.kind == KeyEventKind::Press
        && key.code == KeyCode::Char('c')
        && key.modifiers == KeyModifiers::CONTROL
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

    /// **The combination and the press are the whole of it.** A release, a repeat, or a `c` with
    /// another modifier is a key like any other, and a forced exit that answered to those would
    /// end the application twice over.
    #[test]
    fn only_the_forced_exit_is_the_forced_exit() {
        for event in [
            Event::Key(KeyEvent::new_with_kind(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
                KeyEventKind::Release,
            )),
            Event::Key(KeyEvent::new_with_kind(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
                KeyEventKind::Repeat,
            )),
            key(KeyCode::Char('c')),
            key_with(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ),
        ] {
            let mut state = a_drawn_state();
            let outcome = send(&event, &mut state);

            assert!(
                matches!(outcome, Control::Continue),
                "{event:?} ended the application: {outcome:?}"
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
