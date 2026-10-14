//! Turning terminal events into the messages the loop acts on.
//!
//! # Design notes
//!
//! **`parse` is an associated function of `Message` and not a free one**, the shape `char::
//! from_u32` and `Path::new` have: a function that produces the type belongs to the type, so the
//! day a second one lands both share an `impl` instead of two module-level names. It is **not**
//! `impl From<&Event> for Message`, because `From` is infallible and `parse` is partial — `None`
//! is a real answer, not an error. And there is no state to stand in for, so a test needs nothing
//! but the event it calls `Message::parse` with.
//!
//! **It returns `Option<Message>` rather than a message with an ignored variant.** Today one event
//! does anything, and a variant meaning "nothing" would carry no information while costing a match
//! arm in every reader. `None` is the honest answer for an event the loop does not care about yet,
//! and the day a second message lands, `None` stays the arm for the events still unhandled — the
//! pipeline is not redesigned around a placeholder.
//!
//! **The tests live inline rather than in `tests/`**, because this crate is a binary: an
//! integration test cannot name `Message::parse` at all. The rule that Ctrl+Q quits is a behavior
//! rule, and one with no test named against it is unfinished.

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

/// What the loop learned from one event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Message {
    /// Ctrl+Q was pressed; the loop leaves.
    Quit,
    /// The terminal was resized; the next frame repaints at the new size.
    Resize,
}

impl Message {
    /// Read one event as a message, or `None` when the loop has nothing to do with it.
    pub(crate) fn parse(event: &Event) -> Option<Message> {
        if let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
            && key.code == KeyCode::Char('q')
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            Some(Message::Quit)
        } else if matches!(event, Event::Resize(..)) {
            Some(Message::Resize)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
    };

    use super::Message;

    #[test]
    fn ctrl_q_press_is_quit() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL));

        assert_eq!(Message::parse(&event), Some(Message::Quit));
    }

    #[test]
    fn q_without_control_is_nothing() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));

        assert_eq!(Message::parse(&event), None);
    }

    #[test]
    fn ctrl_q_release_is_nothing() {
        let event = Event::Key(KeyEvent {
            kind: KeyEventKind::Release,
            ..KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL)
        });

        assert_eq!(Message::parse(&event), None);
    }

    #[test]
    fn a_resize_is_a_message() {
        let event = Event::Resize(80, 24);

        assert_eq!(Message::parse(&event), Some(Message::Resize));
    }

    #[test]
    fn other_events_are_nothing() {
        let enter = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        let mouse_moved = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });

        assert_eq!(Message::parse(&enter), None);
        assert_eq!(Message::parse(&mouse_moved), None);
    }
}
