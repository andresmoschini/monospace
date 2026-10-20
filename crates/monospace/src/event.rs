//! How an event of the terminal becomes one of the application's own.
//!
//! **The forced exit is translated here, before anything else sees it.** The terminal reports
//! `Ctrl+C` as a key like any other, and a key is something the chrome is asked about: the
//! drop-down may be open, and an open drop-down holds the keyboard. A forced exit that reached
//! the chrome as a key would be answered by the chrome, and the point of it is that it is
//! answered by nothing. So the translation happens where the event enters the application, and
//! what comes out is an action of its own rather than an input.
//!
//! The event type is the one `ratatui-crossterm` carries, and it is named through that crate
//! rather than through a `crossterm` of our own: the poll of the framework reads it, and its
//! bound is on that crate's type, so naming it this way satisfies the bound by construction
//! rather than by two versions happening to agree.

use rat_salsa::event::RenderedEvent;
use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::vocabulary::{Action, AppEvent};

impl From<Event> for AppEvent {
    fn from(event: Event) -> Self {
        if let Event::Key(key) = &event
            && is_forced_exit(key)
        {
            return Self::Action(Action::ForceQuit);
        }

        match event {
            Event::Resize(_, _) => Self::Resized,

            Event::Key(_) | Event::Mouse(_) | Event::Paste(_) => Self::Input(event),

            _ => Self::Ignored,
        }
    }
}

/// The loop reporting on itself. The only event with no foreign payload: there is nothing to
/// translate, so the arm is a single variant.
impl From<RenderedEvent> for AppEvent {
    fn from(_: RenderedEvent) -> Self {
        Self::Rendered
    }
}

/// Whether the key is the forced exit: `Ctrl+C`, and nothing else.
///
/// **Only a press answers, and only that combination.** The terminal reports the release and the
/// repeat of a key as well, and a shortcut that answered to those would end the application
/// twice over; and `Ctrl+Shift+C` is a key of its own, not the forced exit.
fn is_forced_exit(key: &KeyEvent) -> bool {
    key.kind == KeyEventKind::Press
        && key.code == KeyCode::Char('c')
        && key.modifiers == KeyModifiers::CONTROL
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vocabulary::AppEvent;
    use ratatui_crossterm::crossterm::event::{
        KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };

    fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    fn click() -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 4,
            row: 2,
            modifiers: KeyModifiers::NONE,
        })
    }

    #[test]
    fn a_resize_is_the_applications_own_and_no_widgets() {
        assert_eq!(AppEvent::from(Event::Resize(80, 24)), AppEvent::Resized);
    }

    #[test]
    fn the_forced_exit_is_translated_before_anything_else_sees_it() {
        let event = key(KeyCode::Char('c'), KeyModifiers::CONTROL);

        assert_eq!(
            AppEvent::from(event),
            AppEvent::Action(Action::ForceQuit),
            "the forced exit arrived as an input"
        );
    }

    #[test]
    fn only_a_press_is_the_forced_exit() {
        for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
            let event = Event::Key(KeyEvent::new_with_kind(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
                kind,
            ));

            assert!(
                matches!(AppEvent::from(event), AppEvent::Input(_)),
                "{kind:?} was the forced exit"
            );
        }
    }

    #[test]
    fn a_c_that_is_not_forced_is_an_input() {
        for event in [
            key(KeyCode::Char('c'), KeyModifiers::NONE),
            key(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ),
        ] {
            assert!(
                matches!(AppEvent::from(event), AppEvent::Input(_)),
                "a c that is not the forced exit was not an input"
            );
        }
    }

    #[test]
    fn what_a_widget_could_want_travels_untouched() {
        for event in [
            key(KeyCode::Char('a'), KeyModifiers::NONE),
            key(KeyCode::Char('q'), KeyModifiers::CONTROL),
            key(KeyCode::Left, KeyModifiers::NONE),
            click(),
            Event::Paste("pasted".to_string()),
        ] {
            assert_eq!(
                AppEvent::from(event.clone()),
                AppEvent::Input(event.clone()),
                "not carried as it arrived: {event:?}"
            );
        }
    }

    #[test]
    fn what_neither_wants_is_dropped_here() {
        for event in [Event::FocusGained, Event::FocusLost] {
            let shown = format!("{event:?}");

            assert_eq!(AppEvent::from(event), AppEvent::Ignored, "{shown}");
        }
    }

    #[test]
    fn the_loop_reporting_on_itself_is_its_own_event() {
        assert_eq!(AppEvent::from(RenderedEvent), AppEvent::Rendered);
    }
}
