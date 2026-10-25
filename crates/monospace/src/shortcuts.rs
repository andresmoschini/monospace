//! The table of the shortcuts the application answers to, and how far each one reaches.
//!
//! **The framework does not route them.** The menu shows the shortcut of an entry on the right of
//! it, and `rat-menu` says of that text that _"Hotkey handling is not included in this crate"_:
//! the shortcut is drawn by the widget and answered to by the application. The two are two
//! places that can disagree, and joining them is not this slice's work — it is #180's, where the
//! first command that needs a name arrives.
//!
//! **The framework's own keys are not here.** The navigation character that opens a drop-down,
//! the arrows that move it, `Enter` that takes an entry and `Esc` that closes it belong to the
//! menu widget, which answers to them without asking anybody. What is here is what the
//! application answers to on its own.
//!
//! **A shortcut that must not be consumed says so in the table, and not in an `if` somewhere
//! else.** The forced exit is the case: it has to be answered before the chrome, because a
//! drop-down may be open and an open drop-down holds the keyboard, and a forced exit that reached
//! it as a key would be answered by the widget. Which of the two reaches a shortcut has is a
//! column of the table, so the difference between the way out and the forced exit is a row the
//! reader sees, and the test that the promise holds walks the table rather than naming the key.

use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::vocabulary::Action;

/// How far a shortcut reaches into the order the handlers are asked in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reach {
    /// **Nobody may consume it.** Answered before any handler, whatever the state is: not even an
    /// open drop-down, which holds the keyboard, is asked first.
    Always,

    /// Only if no handler wanted it. Answered after the last of them, so a key the chrome uses is
    /// the chrome's.
    Unclaimed,
}

/// One shortcut: the key that carries it, what it means, and how far it reaches.
struct Shortcut {
    /// The key.
    key: KeyCode,

    /// The modifiers that go with it, and no others.
    modifiers: KeyModifiers,

    /// What the key means.
    ///
    /// **No row is a screen change**: a key means something a person asks for, and the frame being
    /// owed again is not something to ask for. The table holds no such row; the type cannot say
    /// so.
    action: Action,

    /// How far it reaches into the order the handlers are asked in.
    reach: Reach,
}

/// The shortcuts the application answers to.
///
/// **Three rows that differ in one column, and that column is the whole of the precedence.** The
/// forced exit is answered before anything, and the way out and the deletion after everything: the
/// three are the same mechanism with a different scope, rather than two a row and one an `if`. The
/// deletion is `Unclaimed` because it is not the chrome's to swallow: which keys the chrome keeps is
/// the focus rule #189 owns, and `Delete` is not one of the bar's own (D3).
const SHORTCUTS: [Shortcut; 3] = [
    Shortcut {
        key: KeyCode::Char('c'),
        modifiers: KeyModifiers::CONTROL,
        action: Action::ForceQuit,
        reach: Reach::Always,
    },
    Shortcut {
        key: KeyCode::Char('q'),
        modifiers: KeyModifiers::CONTROL,
        action: Action::Quit,
        reach: Reach::Unclaimed,
    },
    Shortcut {
        key: KeyCode::Delete,
        modifiers: KeyModifiers::NONE,
        action: Action::Delete,
        reach: Reach::Unclaimed,
    },
];

/// What the event means to a shortcut answered before any handler.
///
/// This is the one `callbacks` asks first, before the chrome. A resize never reaches it: that is
/// answered before anything is routed.
pub(crate) fn handle_always(event: &Event) -> Option<Action> {
    handle(event, Reach::Always)
}

/// What the event means to a shortcut nobody has claimed.
///
/// This is the one `callbacks` asks last, after the chrome.
pub(crate) fn handle_unclaimed(event: &Event) -> Option<Action> {
    handle(event, Reach::Unclaimed)
}

/// What the event means to a shortcut of this reach.
///
/// **A shortcut is the combination it shows and not another one**, and it answers to that
/// combination at its own reach: `Ctrl+Q` is the way out and `Q` is not, and `Ctrl+Shift+Q` is
/// not either. A release and a repeat are not a press, so neither answers.
fn handle(event: &Event, reach: Reach) -> Option<Action> {
    let Event::Key(key) = event else {
        return None;
    };

    if key.kind != KeyEventKind::Press {
        return None;
    }

    SHORTCUTS
        .iter()
        .find(|shortcut| {
            shortcut.reach == reach
                && shortcut.key == key.code
                && shortcut.modifiers == key.modifiers
        })
        .map(|shortcut| shortcut.action)
}

/// The events that trigger the shortcuts answered before any handler.
///
/// **It exists for the test that the promise holds** — that a shortcut answered before the chrome
/// answers with a drop-down open — and it is compiled for tests alone: a test that named the keys
/// would stop covering the next shortcut added to the table.
#[cfg(test)]
pub(crate) fn events_answered_first() -> Vec<Event> {
    use ratatui_crossterm::crossterm::event::KeyEvent;

    SHORTCUTS
        .iter()
        .filter(|shortcut| shortcut.reach == Reach::Always)
        .map(|shortcut| Event::Key(KeyEvent::new(shortcut.key, shortcut.modifiers)))
        .collect()
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

    /// Hands the event to the entry point of one reach, the way `callbacks` does.
    fn handle_at(event: &Event, reach: Reach) -> Option<Action> {
        match reach {
            Reach::Always => handle_always(event),
            Reach::Unclaimed => handle_unclaimed(event),
        }
    }

    #[test]
    fn the_way_out_is_the_shortcut_the_menu_shows() {
        let event = key(KeyCode::Char('q'), KeyModifiers::CONTROL);

        assert_eq!(handle_unclaimed(&event), Some(Action::Quit));
    }

    /// **The two exits differ in their reach, and that is the decision.** The forced exit is
    /// answered before anything and the way out after everything, so neither answers at the other
    /// one's reach.
    #[test]
    fn the_forced_exit_is_answered_first_and_the_way_out_last() {
        let forced_exit = key(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let way_out = key(KeyCode::Char('q'), KeyModifiers::CONTROL);

        assert_eq!(handle_always(&forced_exit), Some(Action::ForceQuit));
        assert_eq!(handle_unclaimed(&way_out), Some(Action::Quit));
        assert_eq!(
            handle_unclaimed(&forced_exit),
            None,
            "the forced exit answered after the chrome"
        );
        assert_eq!(
            handle_always(&way_out),
            None,
            "the way out answered before the chrome"
        );
    }

    #[test]
    fn a_shortcut_answers_to_nothing_but_a_press() {
        for shortcut in SHORTCUTS {
            for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
                let event = Event::Key(KeyEvent::new_with_kind(
                    shortcut.key,
                    shortcut.modifiers,
                    kind,
                ));

                assert_eq!(
                    handle_at(&event, shortcut.reach),
                    None,
                    "{kind:?} answered {:?}",
                    shortcut.action
                );
            }
        }
    }

    /// **A shortcut is the combination it shows and not another one.** Every combination a terminal
    /// can report other than the row's own is handed to its reach, and none of them answers.
    ///
    /// **The alternatives are counted rather than assumed to be `Shift`**: `Delete` is the first row
    /// whose own combination is no modifier at all, so a test that only added `Shift` would hand the
    /// row's own key to the table and call it another one.
    #[test]
    fn a_shortcut_is_the_combination_it_shows_and_not_another_one() {
        for shortcut in SHORTCUTS {
            for modifiers in [
                KeyModifiers::NONE,
                KeyModifiers::SHIFT,
                KeyModifiers::CONTROL,
                KeyModifiers::ALT,
            ] {
                if modifiers == shortcut.modifiers {
                    continue;
                }

                assert_eq!(
                    handle_at(&key(shortcut.key, modifiers), shortcut.reach),
                    None,
                    "{:?} with {modifiers:?} answered",
                    shortcut.action
                );
            }
        }

        let another_key = key(KeyCode::Char('x'), KeyModifiers::CONTROL);
        assert_eq!(handle_unclaimed(&another_key), None);
    }

    #[test]
    fn a_key_that_is_not_a_shortcut_is_not_an_action() {
        let event = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });

        for reach in [Reach::Always, Reach::Unclaimed] {
            assert_eq!(handle_at(&event, reach), None);
        }
    }
}
