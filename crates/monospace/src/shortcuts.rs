//! The table of the shortcuts the application answers to.
//!
//! **The framework does not route them.** The menu shows the shortcut of an entry on the right of
//! it, and `rat-menu` says of that text that _"Hotkey handling is not included in this crate"_:
//! the shortcut is drawn by the widget and answered to by the application. The two are two
//! places that can disagree, and joining them is not this slice's work — it is #180's, where the
//! first command that needs a name arrives. What answers here is the application alone: the bar
//! is asked first, and `callbacks` asks this table only about an event the bar said nothing
//! about.
//!
//! **The framework's own keys are not here.** The navigation character that opens a drop-down,
//! the arrows that move it, `Enter` that takes an entry and `Esc` that closes it belong to the
//! menu widget, which answers to them without asking anybody. What is here is what the
//! application answers to on its own.

use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::vocabulary::{Action, Answer};

/// One shortcut: the key that carries it, and what it means.
struct Shortcut {
    /// The key.
    key: KeyCode,

    /// The modifiers that go with it, and no others.
    modifiers: KeyModifiers,

    /// What the key means.
    action: Action,
}

/// The shortcuts the application answers to.
const SHORTCUTS: [Shortcut; 1] = [Shortcut {
    key: KeyCode::Char('q'),
    modifiers: KeyModifiers::CONTROL,
    action: Action::Quit,
}];

/// What the event means to the application's own keys.
///
/// **A shortcut is the combination it shows and not another one.** `Ctrl+Q` is the way out and
/// `Q` is not, and `Ctrl+Shift+Q` is not either.
pub(crate) fn handle(event: &Event) -> Answer {
    let Event::Key(key) = event else {
        return Answer::Nothing;
    };

    if key.kind != KeyEventKind::Press {
        return Answer::Nothing;
    }

    SHORTCUTS
        .iter()
        .find(|shortcut| shortcut.key == key.code && shortcut.modifiers == key.modifiers)
        .map_or(Answer::Nothing, |shortcut| Answer::Action(shortcut.action))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_crossterm::crossterm::event::{
        KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };

    fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    #[test]
    fn the_way_out_is_the_shortcut_the_menu_shows() {
        let event = key(KeyCode::Char('q'), KeyModifiers::CONTROL);

        assert_eq!(handle(&event), Answer::Action(Action::Quit));
    }

    #[test]
    fn a_shortcut_answers_to_nothing_but_a_press() {
        for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
            let event = Event::Key(KeyEvent::new_with_kind(
                KeyCode::Char('q'),
                KeyModifiers::CONTROL,
                kind,
            ));

            assert_eq!(handle(&event), Answer::Nothing, "{kind:?} answered");
        }
    }

    #[test]
    fn a_shortcut_is_the_combination_it_shows_and_not_another_one() {
        for event in [
            key(KeyCode::Char('q'), KeyModifiers::NONE),
            key(
                KeyCode::Char('q'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ),
            key(KeyCode::Char('x'), KeyModifiers::CONTROL),
        ] {
            assert_eq!(handle(&event), Answer::Nothing, "{event:?} answered");
        }
    }

    #[test]
    fn a_key_that_is_not_a_shortcut_is_not_an_action() {
        let event = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });

        assert_eq!(handle(&event), Answer::Nothing);
    }
}
