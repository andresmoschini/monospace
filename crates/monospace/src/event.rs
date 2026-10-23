//! What the loop is told, and the two translations into it.
//!
//! **The application's event type is the union of the two sources the loop polls, and nothing
//! else**: the terminal's event, carried whole, and the loop's own report that a frame was
//! painted. It is a transport and not a taxonomy — what an event *means* is decided in
//! `callbacks`, where every handler is asked in turn, and a type that classified here would be a
//! second place where the same rule lives.
//!
//! The event type is the one `ratatui-crossterm` carries, and it is named through that crate
//! rather than through a `crossterm` of our own: the poll of the framework reads it, and its
//! bound is on that crate's type, so naming it this way satisfies the bound by construction
//! rather than by two versions happening to agree.

use rat_salsa::event::RenderedEvent;
use ratatui_crossterm::crossterm::event::Event;

/// What the loop hands the application.
#[derive(Debug, PartialEq)]
pub(crate) enum AppEvent {
    /// Something the terminal said: a key, a click, a paste, a resize.
    Terminal(Event),

    /// A frame was painted.
    Rendered(RenderedEvent),
}

impl From<Event> for AppEvent {
    fn from(event: Event) -> Self {
        Self::Terminal(event)
    }
}

impl From<RenderedEvent> for AppEvent {
    fn from(event: RenderedEvent) -> Self {
        Self::Rendered(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_crossterm::crossterm::event::{
        KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };

    /// **Nothing the terminal says is classified here.** The forced exit, the resize and the
    /// events no widget wants all arrive as the same thing; each is decided where it is acted
    /// on, never on the way in.
    #[test]
    fn everything_the_terminal_says_travels_whole() {
        for event in [
            Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Event::Resize(80, 24),
            Event::Paste("pasted".to_string()),
            Event::FocusGained,
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 4,
                row: 2,
                modifiers: KeyModifiers::NONE,
            }),
        ] {
            assert_eq!(
                AppEvent::from(event.clone()),
                AppEvent::Terminal(event.clone()),
                "not carried as it arrived: {event:?}"
            );
        }
    }

    #[test]
    fn the_loops_report_carries_its_own() {
        assert_eq!(
            AppEvent::from(RenderedEvent),
            AppEvent::Rendered(RenderedEvent)
        );
    }
}
