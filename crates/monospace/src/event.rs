//! The two ways an `AppEvent` comes into being, together so that finding one leads to the other.
//!
//! The crossterm event does not arrive through a crossterm dependency of this crate:
//! `poll::PollCrossterm` reads it, and its bound is on `ratatui_crossterm::crossterm::event::Event`.
//! `ratatui::crossterm` is that crate, so naming it this way satisfies the bound by construction
//! rather than by two versions happening to agree.
//!
//! The classification asks what a widget could want and what belongs to the application alone.
//! The resize has a meaning of its own because no widget wants it, and `rat_salsa` does not watch
//! for resizes, so dropping it would leave the screen on the previous layout. The keys the
//! application reserves are not decided here: the menu owns the shortcuts, and the widgets are
//! asked first, so a dialog that is open keeps them.

use rat_salsa::event::RenderedEvent;
use ratatui::crossterm::event::Event;

use crate::vocabulary::AppEvent;

impl From<Event> for AppEvent {
    fn from(event: Event) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vocabulary::AppEvent;
    use ratatui::crossterm::event::{
        KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
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
    fn what_a_widget_could_want_travels_untouched() {
        for event in [
            key(KeyCode::Char('a'), KeyModifiers::NONE),
            key(KeyCode::Char('q'), KeyModifiers::CONTROL),
            key(KeyCode::Char('+'), KeyModifiers::NONE),
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
