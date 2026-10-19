//! The four signatures `rat_salsa` dictates, and what the application does when each is called.
//!
//! The signatures are fixed by `run_tui`'s parameters, which is why they are free functions. What
//! is inside them is the application's behavior: a `+` increments, `Control+q` quits, a resize
//! repaints.

use rat_salsa::Control;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::state::AppState;
use crate::vocabulary::{AppError, AppEvent};
use crate::{context::AppGlobal, paint};

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn init(_state: &mut AppState, _global: &mut AppGlobal) -> Result<(), AppError> {
    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut AppState,
    global: &mut AppGlobal,
) -> Result<(), AppError> {
    paint::draw(area, buf, state, global);
    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
// `Input` is the arm that grows into the widget layer, so it must not be merged into the two that
// do nothing.
#[allow(clippy::match_same_arms, reason = "see the arm it is attached to")]
pub(crate) fn event(
    event: &AppEvent,
    state: &mut AppState,
    _global: &mut AppGlobal,
) -> Result<Control<AppEvent>, AppError> {
    match event {
        AppEvent::Quit => Ok(Control::Quit),

        AppEvent::Increment => {
            state.counter += 1;
            Ok(Control::Changed)
        }

        // Repaints although nobody asked. `rat_salsa` does not watch the terminal for resizes.
        AppEvent::Resized => Ok(Control::Changed),

        AppEvent::Rendered | AppEvent::Ignored => Ok(Control::Continue),

        // No widget layer yet: nothing consumes a raw event, so it changes nothing.
        AppEvent::Input(_) => Ok(Control::Continue),
    }
}

pub(crate) fn error(
    error: AppError,
    _state: &mut AppState,
    _global: &mut AppGlobal,
) -> Result<Control<AppEvent>, AppError> {
    Err(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use monospace_diagram::Diagram;
    use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

    fn a_state() -> AppState {
        AppState::new(Diagram::new())
    }

    fn a_global() -> AppGlobal {
        AppGlobal::new(Config::default())
    }

    fn a_raw_event() -> AppEvent {
        AppEvent::Input(Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)))
    }

    fn send(app_event: &AppEvent, state: &mut AppState) -> Result<Control<AppEvent>, AppError> {
        event(app_event, state, &mut a_global())
    }

    #[test]
    fn quit_is_the_way_out() {
        let outcome = send(&AppEvent::Quit, &mut a_state());

        assert!(matches!(outcome, Ok(Control::Quit)));
    }

    #[test]
    fn increment_changes_the_state_and_asks_for_a_repaint() {
        let mut state = a_state();
        let outcome = send(&AppEvent::Increment, &mut state);

        assert!(matches!(outcome, Ok(Control::Changed)));
        assert_eq!(state.counter, 1);
    }

    #[test]
    fn a_resize_repaints_although_nobody_asked() {
        let mut state = a_state();
        let outcome = send(&AppEvent::Resized, &mut state);

        assert!(matches!(outcome, Ok(Control::Changed)));
        assert_eq!(state.counter, 0, "a resize is not a command");
    }

    #[test]
    fn everything_else_is_a_no_op() {
        for app_event in [AppEvent::Rendered, AppEvent::Ignored, a_raw_event()] {
            let mut state = a_state();
            let outcome = send(&app_event, &mut state);

            assert!(matches!(outcome, Ok(Control::Continue)), "{app_event:?}");
            assert_eq!(state.counter, 0, "{app_event:?} changed the state");
        }
    }

    #[test]
    fn the_loop_set_the_terminal_up_itself_so_there_is_nothing_to_init() {
        assert!(init(&mut a_state(), &mut a_global()).is_ok());
    }

    #[test]
    fn an_error_is_passed_on_rather_than_swallowed() {
        let failure: AppError = "no terminal".into();
        let outcome = error(failure, &mut a_state(), &mut a_global());

        assert!(outcome.is_err());
    }
}
