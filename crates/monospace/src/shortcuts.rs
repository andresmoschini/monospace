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

use crate::vocabulary::{Action, Reply};

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

    /// What the key answers.
    ///
    /// **No row is a screen change**: a key answers with a meaning or with an exit, and the frame
    /// being owed again is not something a person asks for. The table holds no such row; the type
    /// cannot say so.
    reply: Reply,

    /// How far it reaches into the order the handlers are asked in.
    reach: Reach,
}

/// The shortcuts the application answers to.
///
/// **The rows differ in one column, and that column is the whole of the precedence.** The forced
/// exit is answered before anything, and the other four after everything: they are the same
/// mechanism with a different scope, rather than one mechanism per row and an `if` somewhere else.
/// The undo and the redo are `Unclaimed` because they are not the chrome's to swallow: which keys
/// the chrome keeps is the focus rule #189 owns, and neither of the two is one of the bar's own
/// (D3 of #181).
///
/// **A slice rather than an array of a counted length**: the count of an array is a number beside
/// the rows it counts, and a row added without it is an error the compiler reports about the wrong
/// thing.
const SHORTCUTS: &[Shortcut] = &[
    // **The shifted character, not `q` with `SHIFT` set**, as the redo row below explains: a
    // terminal that reports the combination at all reports the character the shift produces with
    // `Control` and `Shift` set, and one that folds the shift reports `Ctrl+Q`, the way out, rather
    // than a dead key.
    Shortcut {
        key: KeyCode::Char('Q'),
        modifiers: KeyModifiers::CONTROL.union(KeyModifiers::SHIFT),
        reply: Reply::force_quit(),
        reach: Reach::Always,
    },
    Shortcut {
        key: KeyCode::Char('n'),
        modifiers: KeyModifiers::CONTROL,
        reply: Reply::Act(Action::New),
        reach: Reach::Unclaimed,
    },
    Shortcut {
        key: KeyCode::Char('o'),
        modifiers: KeyModifiers::CONTROL,
        reply: Reply::Act(Action::Open),
        reach: Reach::Unclaimed,
    },
    Shortcut {
        key: KeyCode::Char('s'),
        modifiers: KeyModifiers::CONTROL,
        reply: Reply::Act(Action::Save),
        reach: Reach::Unclaimed,
    },
    Shortcut {
        key: KeyCode::Char('q'),
        modifiers: KeyModifiers::CONTROL,
        reply: Reply::quit(),
        reach: Reach::Unclaimed,
    },
    Shortcut {
        key: KeyCode::Char('z'),
        modifiers: KeyModifiers::CONTROL,
        reply: Reply::Act(Action::Undo),
        reach: Reach::Unclaimed,
    },
    // **The shifted character, not `z` with `SHIFT` set.** A terminal that reports the
    // combination at all reports it as the character the shift produces with `Control` and `Shift`
    // set; and one that folds the shift into the control character reports the undo instead, which
    // is the cost D1 of #181 names. The two rows are told apart by the character.
    Shortcut {
        key: KeyCode::Char('Z'),
        modifiers: KeyModifiers::CONTROL.union(KeyModifiers::SHIFT),
        reply: Reply::Act(Action::Redo),
        reach: Reach::Unclaimed,
    },
];

/// What the event means to a shortcut answered before any handler.
///
/// This is the one `callbacks` asks first, before the chrome. A resize never reaches it: that is
/// answered before anything is routed.
pub(crate) fn handle_always(event: &Event) -> Reply {
    handle(event, Reach::Always)
}

/// What the event means to a shortcut nobody has claimed.
///
/// This is the one `callbacks` asks last, after the chrome.
pub(crate) fn handle_unclaimed(event: &Event) -> Reply {
    handle(event, Reach::Unclaimed)
}

/// What the event means to a shortcut of this reach.
///
/// **A shortcut is the combination it shows and not another one**, and it answers to that
/// combination at its own reach: `Ctrl+Q` is the way out, `Q` is neither, and `Ctrl+Shift+Q` is
/// the forced exit — the three are told apart by the character. A release and a repeat are not a
/// press, so none of them answers.
fn handle(event: &Event, reach: Reach) -> Reply {
    let Event::Key(key) = event else {
        return Reply::Ignored;
    };

    if key.kind != KeyEventKind::Press {
        return Reply::Ignored;
    }

    SHORTCUTS
        .iter()
        .find(|shortcut| {
            shortcut.reach == reach
                && shortcut.key == key.code
                && shortcut.modifiers == key.modifiers
        })
        .map_or(Reply::Ignored, |shortcut| shortcut.reply.clone())
}

/// The events that trigger the shortcuts of `reach`.
///
/// **They exist for the tests that the promise holds** — that a shortcut of a reach answers where
/// the order says it does — and they are compiled for tests alone: a test that named the keys
/// would stop covering the next shortcut added to the table.
#[cfg(test)]
fn events_at(reach: Reach) -> Vec<Event> {
    use ratatui_crossterm::crossterm::event::KeyEvent;

    SHORTCUTS
        .iter()
        .filter(|shortcut| shortcut.reach == reach)
        .map(|shortcut| Event::Key(KeyEvent::new(shortcut.key, shortcut.modifiers)))
        .collect()
}

/// The events that trigger the shortcuts answered before any handler.
#[cfg(test)]
pub(crate) fn events_answered_first() -> Vec<Event> {
    events_at(Reach::Always)
}

/// The events that trigger the shortcuts nobody has claimed.
#[cfg(test)]
pub(crate) fn events_unclaimed() -> Vec<Event> {
    events_at(Reach::Unclaimed)
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
    fn handle_at(event: &Event, reach: Reach) -> Reply {
        match reach {
            Reach::Always => handle_always(event),
            Reach::Unclaimed => handle_unclaimed(event),
        }
    }

    #[test]
    fn the_way_out_is_the_shortcut_the_menu_shows() {
        let event = key(KeyCode::Char('q'), KeyModifiers::CONTROL);

        assert_eq!(handle_unclaimed(&event), Reply::quit());
    }

    /// **The two ways out differ in their reach, and that is the decision.** The forced exit is
    /// answered before anything and the way out after everything, so neither answers at the other
    /// one's reach.
    #[test]
    fn the_forced_exit_is_answered_first_and_the_way_out_last() {
        let forced_exit = key(
            KeyCode::Char('Q'),
            KeyModifiers::CONTROL.union(KeyModifiers::SHIFT),
        );
        let way_out = key(KeyCode::Char('q'), KeyModifiers::CONTROL);

        assert_eq!(handle_always(&forced_exit), Reply::force_quit());
        assert_eq!(handle_unclaimed(&way_out), Reply::quit());
        assert_eq!(
            handle_unclaimed(&forced_exit),
            Reply::Ignored,
            "the forced exit answered after the chrome"
        );
        assert_eq!(
            handle_always(&way_out),
            Reply::Ignored,
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
                    Reply::Ignored,
                    "{kind:?} answered {:?}",
                    shortcut.reply
                );
            }
        }
    }

    /// **A shortcut is the combination it shows and not another one.** Every combination a terminal
    /// can report other than the row's own is handed to its reach, and none of them answers.
    ///
    /// **The alternatives are walked rather than named**: the walk has to contain the empty set,
    /// and a row with two modifiers has to be handed the ones it does not have — a hand-written
    /// list of single modifiers would try neither.
    #[test]
    fn a_shortcut_is_the_combination_it_shows_and_not_another_one() {
        for shortcut in SHORTCUTS {
            for modifiers in combinations_but(shortcut.modifiers) {
                assert_eq!(
                    handle_at(&key(shortcut.key, modifiers), shortcut.reach),
                    Reply::Ignored,
                    "{:?} with {modifiers:?} answered",
                    shortcut.reply
                );
            }
        }

        let another_key = key(KeyCode::Char('x'), KeyModifiers::CONTROL);
        assert_eq!(handle_unclaimed(&another_key), Reply::Ignored);
    }

    /// Every combination of the modifiers a terminal reports for these keys, other than `own`.
    fn combinations_but(own: KeyModifiers) -> Vec<KeyModifiers> {
        let mut combinations = Vec::new();

        for shift in [false, true] {
            for control in [false, true] {
                for alt in [false, true] {
                    let mut modifiers = KeyModifiers::NONE;
                    modifiers.set(KeyModifiers::SHIFT, shift);
                    modifiers.set(KeyModifiers::CONTROL, control);
                    modifiers.set(KeyModifiers::ALT, alt);

                    if modifiers != own {
                        combinations.push(modifiers);
                    }
                }
            }
        }

        combinations
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
            assert_eq!(handle_at(&event, reach), Reply::Ignored);
        }
    }
}
