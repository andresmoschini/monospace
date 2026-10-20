//! The four signatures `rat_salsa` dictates, and what the application does when each is called.
//!
//! The signatures are fixed by `run_tui`'s parameters, which is why they are free functions. An
//! entry the chrome takes does not decide for itself: it becomes an `AppEvent` and the loop
//! re-enters `event` with it, so an entry and the key that means the same thing are decided in one
//! match.

use rat_salsa::Control;
use rat_widget::event::flow;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::Rect;

use crate::state::AppState;
use crate::vocabulary::{AppError, AppEvent};
use crate::{about_dialog, context::AppGlobal, menu, paint};

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

        AppEvent::About => {
            about_dialog::open(&mut state.about);
            Ok(Control::Changed)
        }

        // Repaints although nobody asked. `rat_salsa` does not watch the terminal for resizes.
        AppEvent::Resized => Ok(Control::Changed),

        AppEvent::Rendered | AppEvent::Ignored => Ok(Control::Continue),

        AppEvent::Input(raw) => Ok(widgets(raw, state)),
    }
}

/// Where a key or a click goes, in the order the widgets are asked.
///
/// Dispatch runs the reverse of drawing, and the dialog is drawn last of all, so it is asked first
/// and keeps the keyboard to itself while it is open. The drop-down is over the body, so the menu
/// is asked before whatever is behind it.
fn widgets(raw: &Event, state: &mut AppState) -> Control<AppEvent> {
    flow!(about_dialog::handle(raw, &mut state.about));
    flow!(menu::handle(raw, &mut state.menu));
    Control::Continue
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
    use ratatui::buffer::Buffer;
    use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::layout::Rect;

    const WIDTH: u16 = 60;
    const HEIGHT: u16 = 14;

    fn a_state() -> AppState {
        AppState::new(Diagram::new())
    }

    fn a_global() -> AppGlobal {
        AppGlobal::new(Config::default())
    }

    /// A key no widget of the chrome uses.
    fn a_raw_event() -> AppEvent {
        AppEvent::Input(Event::Key(KeyEvent::new(
            KeyCode::Char('z'),
            KeyModifiers::NONE,
        )))
    }

    fn send(app_event: &AppEvent, state: &mut AppState) -> Result<Control<AppEvent>, AppError> {
        event(app_event, state, &mut a_global())
    }

    fn key(code: KeyCode) -> AppEvent {
        AppEvent::Input(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)))
    }

    fn draw(state: &mut AppState) -> Buffer {
        let area = Rect::new(0, 0, WIDTH, HEIGHT);
        let mut buffer = Buffer::empty(area);
        render(area, &mut buffer, state, &mut a_global()).expect("a frame");
        buffer
    }

    /// The bar only knows its labels after a frame has drawn them.
    fn a_drawn_state() -> AppState {
        let mut state = a_state();
        draw(&mut state);
        state
    }

    fn press(code: KeyCode, state: &mut AppState) {
        let outcome = send(&key(code), state).expect("the key reached the application");
        assert_eq!(outcome, Control::Changed, "{code:?} changed nothing");
    }

    /// Opens a label's drop-down by its letter, leaving the first entry selected.
    fn open(label: char, state: &mut AppState) {
        press(KeyCode::Char(label), state);
        draw(state);
    }

    /// Finishes one event the way `rat_salsa` does: an `Event` control is a question asked again,
    /// and the loop calls `event` with the event it carries. This is the whole of the claim that a
    /// menu entry and a key arrive at one place.
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
            let mut state = a_drawn_state();
            let outcome = send(&app_event, &mut state);

            assert!(matches!(outcome, Ok(Control::Continue)), "{app_event:?}");
            assert_eq!(state.counter, 0, "{app_event:?} changed the state");
            assert!(
                !state.menu.popup_active(),
                "{app_event:?} opened a drop-down"
            );
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

    #[test]
    fn an_entry_of_the_bar_leaves_where_the_key_leaves() {
        let mut state = a_drawn_state();
        open('f', &mut state);

        let outcome = drive(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Quit), "{outcome:?}");
    }

    #[test]
    fn the_increment_entry_counts_the_way_the_key_does() {
        let mut state = a_drawn_state();
        open('e', &mut state);

        let outcome = drive(&key(KeyCode::Enter), &mut state);

        assert!(matches!(outcome, Control::Changed), "{outcome:?}");
        assert_eq!(state.counter, 1, "the entry did not count");
    }

    #[test]
    fn an_open_dialog_keeps_the_menu_from_the_event() {
        let mut state = a_drawn_state();
        about_dialog::open(&mut state.about);

        let outcome =
            send(&key(KeyCode::Char('f')), &mut state).expect("the key reached the application");

        assert!(matches!(outcome, Control::Unchanged), "{outcome:?}");
        assert!(
            !state.menu.popup_active(),
            "the menu opened a drop-down behind the dialog"
        );
    }
}
