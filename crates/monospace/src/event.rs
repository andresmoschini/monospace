//! Where events come from, and what one becomes.
//!
//! # Design notes
//!
//! **An event becomes an action and the action is the only thing that changes the state.** A key
//! does not reach the `Chrome` and a click does not either: both are turned into one
//! [`Action`] here, and the `match` in [`crate::app::App::apply`] is the whole of what the
//! application can do. The alternative — a handler per key and a handler per control, each
//! reaching into the state — is a design in which "what does this key do" has as many answers as
//! there are places to put one.
//!
//! **The focus holder is offered every event first, and only what nothing took falls through to
//! the application's own keys.** That is the order `cursive` documents — _"If the menubar is
//! active, it will be handled the event. The view tree will be handled the event. If ignored,
//! `global_callbacks` will be checked for this event"_ — and it is what makes an open pop-up hold
//! the keyboard: while it is open, `Esc` means "cancel this" and nothing reaches past it.
//!
//! **A pointer moving produces no action at all.** A control under the mouse is not a state (D6),
//! so `Moved` is not offered to anything and the frame that follows is the frame that was already
//! on screen.

use std::io;

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::layout::Position;

use crate::chrome::{Action, Chrome, Focus, Move};

/// Where the loop asks for the next event.
///
/// **A trait so that a test can hand the loop a list and a panic**, which is the only way rules 2
/// and 3 can be observed: a loop that reads from a real terminal cannot be walked into a panic on
/// purpose, and the claim being tested is exactly what happens when it is.
pub(crate) trait EventSource {
    /// The next event, blocking until there is one.
    ///
    /// # Errors
    ///
    /// Whatever the terminal reported.
    fn next(&mut self) -> io::Result<Event>;
}

/// The real event source.
pub(crate) struct CrosstermEvents;

impl EventSource for CrosstermEvents {
    fn next(&mut self) -> io::Result<Event> {
        event::read()
    }
}

/// What `event` becomes, or `None` where it names no action.
///
/// **The focus holder is asked first**, and the application's own keys are asked only with what it
/// did not take — see the design notes. **A key's release is not an event to act on**, which is why
/// `Kind` is checked before anything else: `ratatui`'s own guidance for `crossterm` on Windows is
/// that a terminal reports `Release` as well as `Press`, and treating both as a press is what
/// makes a control answer twice.
pub(crate) fn action_of(chrome: &Chrome, event: &Event) -> Option<Action> {
    if let Some(action) = focused_action(chrome, event) {
        return Some(action);
    }
    own_key_action(event)
}

/// What the control holding the keyboard takes from `event`.
fn focused_action(chrome: &Chrome, event: &Event) -> Option<Action> {
    match (chrome.focus(), event) {
        (Focus::Region, Event::Key(key)) => key_action(key, REGION_KEYS),
        (Focus::Bar(_), Event::Key(key)) => key_action(key, BAR_KEYS),
        (Focus::DropDown, Event::Key(key)) => key_action(key, DROP_DOWN_KEYS),
        (Focus::PopUp, Event::Key(key)) => key_action(key, POPUP_KEYS),
        (_, Event::Mouse(mouse)) => mouse_action(chrome, *mouse),
        _ => None,
    }
}

/// The keys the region takes while the keyboard is in it.
///
/// **Empty, and that is the whole of what the region can be asked to do.** A key that names no
/// action falls through to the application's own, so nothing here means the way out and the menu
/// still work from the resting state, which is where the application is most of the time.
///
/// **The first key the region answers is
/// [#175](https://github.com/andresmoschini/monospace/issues/175)'s to write**, by giving the
/// region something to do with a key. The table exists rather than the match arm being a bare
/// `None` so that adding that one is a row here and nothing else.
const REGION_KEYS: &[(KeyCode, Action)] = &[];

/// The keys the application answers wherever the keyboard is.
///
/// **`Ctrl+Q` and `Ctrl+C` reach the same action**, and `Ctrl+C` is here because raw mode turns
/// `ISIG` off: the signal the shell would have raised never happens, and the key arrives here like
/// any other. A bare `q` is deliberately not among them — a key with no action behind it changes
/// nothing and reports nothing.
///
/// **A released key names nothing here either**, for the same reason it names nothing in a table:
/// `ratatui`'s own guidance for `crossterm` on Windows is that a terminal reports `Release` beside
/// `Press`, and a check that a key's table performs and this one does not is a key released once
/// behaving like a key pressed once.
fn own_key_action(event: &Event) -> Option<Action> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('q' | 'c')) => Some(Action::Quit),
            (_, KeyCode::F(9)) => Some(Action::OpenDropDown),
            _ => None,
        },
        _ => None,
    }
}

/// The keys the bar takes while the keyboard is on it.
///
/// **`Enter` activates rather than naming a control**, because which control it activates is
/// whatever holds focus, and repeating the focus inside the key table would be a second answer to
/// "what does Enter do here" — one that goes stale the day the focus moves.
const BAR_KEYS: &[(KeyCode, Action)] = &[
    (KeyCode::Enter, Action::Activate),
    (KeyCode::Right, Action::FocusAlongBar(Move::Down)),
    (KeyCode::Tab, Action::FocusAlongBar(Move::Down)),
    (KeyCode::Left, Action::FocusAlongBar(Move::Up)),
    (KeyCode::BackTab, Action::FocusAlongBar(Move::Up)),
    (KeyCode::Esc, Action::FocusTheRegion),
];

/// The keys the drop-down takes while it holds the keyboard.
///
/// **`Esc` is here and not among the application's own keys**, because while the drop-down is open
/// the keyboard is inside it, and a key that reached past it would close the drop-down and open
/// something else in the same press.
const DROP_DOWN_KEYS: &[(KeyCode, Action)] = &[
    (KeyCode::Up, Action::MoveInDropDown(Move::Up)),
    (KeyCode::Down, Action::MoveInDropDown(Move::Down)),
    (KeyCode::Enter, Action::Activate),
    (KeyCode::Esc, Action::CloseDropDown),
];

/// The keys the pop-up takes while it holds the keyboard.
const POPUP_KEYS: &[(KeyCode, Action)] = &[
    (KeyCode::Enter, Action::Activate),
    (KeyCode::Esc, Action::HideAbout),
];

/// The action `key` names in `keys`, or `None` where it names none.
///
/// **A released key names nothing**, and that is what the first line is for — see
/// [`own_key_action`], which asks the same question of the application's own keys.
fn key_action(key: &KeyEvent, keys: &[(KeyCode, Action)]) -> Option<Action> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    keys.iter()
        .find(|(code, _)| *code == key.code)
        .map(|(_, action)| *action)
}

/// What a mouse event becomes, for the control whose rectangle the pointer is inside.
///
/// **A control is found by the rectangle the draw pass stored**, so a resize moves the control and
/// the click with it. A pointer that moved produces nothing, so nothing on screen follows it.
fn mouse_action(chrome: &Chrome, mouse: MouseEvent) -> Option<Action> {
    let at = Position {
        x: mouse.column,
        y: mouse.row,
    };
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => chrome.control_at(at).map(Action::Press),
        MouseEventKind::Up(MouseButton::Left) => Some(Action::Release(chrome.control_at(at))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
        MouseEventKind,
    };
    use ratatui::layout::Position;

    use super::{action_of, own_key_action};
    use crate::chrome::{Action, Chrome, ControlId, Move};

    /// A key pressed, which is the only kind of key an event loop acts on.
    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    /// The same key with a modifier held.
    fn chord(modifiers: KeyModifiers, code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    /// A mouse button at a position.
    fn mouse(kind: MouseEventKind, x: u16, y: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        })
    }

    fn at(x: u16, y: u16) -> Position {
        Position { x, y }
    }

    /// A chrome with every control drawn at `area`, so that a click has somewhere to land.
    fn chrome_with_controls_at(x: u16, y: u16) -> Chrome {
        let mut chrome = Chrome::default();
        for control in crate::chrome::Geometry::controls() {
            chrome
                .geometry_mut_for_test()
                .set(control, ratatui::layout::Rect::new(x, y, 6, 1));
        }
        chrome
    }

    #[test]
    fn a_bare_q_names_no_action() {
        assert_eq!(own_key_action(&key(KeyCode::Char('q'))), None);
    }

    #[test]
    fn control_q_and_control_c_both_reach_the_way_out() {
        assert_eq!(
            own_key_action(&chord(KeyModifiers::CONTROL, KeyCode::Char('q'))),
            Some(Action::Quit)
        );
        assert_eq!(
            own_key_action(&chord(KeyModifiers::CONTROL, KeyCode::Char('c'))),
            Some(Action::Quit),
            "Ctrl+C is a key here because raw mode has turned ISIG off"
        );
    }

    #[test]
    fn a_released_key_names_no_action() {
        let released = Event::Key(KeyEvent {
            kind: KeyEventKind::Release,
            ..KeyEvent::new(KeyCode::F(9), KeyModifiers::NONE)
        });
        assert_eq!(own_key_action(&released), None);
    }

    #[test]
    fn a_key_goes_to_whatever_holds_focus_before_the_applications_own_keys() {
        let mut chrome = Chrome::default();

        // The region is where the keyboard rests, and it takes nothing, so everything falls
        // through — including `Enter`, which would activate a control if there were one under it.
        assert_eq!(action_of(&chrome, &key(KeyCode::Enter)), None);
        assert_eq!(action_of(&chrome, &key(KeyCode::Esc)), None);
        assert_eq!(
            action_of(&chrome, &key(KeyCode::F(9))),
            Some(Action::OpenDropDown)
        );

        // With the drop-down open, `Esc` is the drop-down's and nothing else gets it.
        chrome.apply(Action::OpenDropDown);
        assert_eq!(
            action_of(&chrome, &key(KeyCode::Esc)),
            Some(Action::CloseDropDown)
        );
        assert_eq!(
            action_of(&chrome, &key(KeyCode::Down)),
            Some(Action::MoveInDropDown(Move::Down))
        );

        // With the keyboard on the bar, `Enter` reaches the control and `Esc` goes back down the
        // stack to the region.
        chrome.apply(Action::CloseDropDown);
        assert_eq!(
            action_of(&chrome, &key(KeyCode::Enter)),
            Some(Action::Activate),
            "the key the focus holder takes did not reach it"
        );
        assert_eq!(
            action_of(&chrome, &key(KeyCode::Esc)),
            Some(Action::FocusTheRegion)
        );
    }

    /// **What "an open pop-up holds the keyboard" means here is which of two keys goes where, not
    /// that the application goes deaf.** Rule 14 says the focus holder is offered every event first
    /// and only what nothing took falls through to the application's own keys, which is the order
    /// `cursive` documents — so `Ctrl+Q` still reaches the way out from under a pop-up, and what a
    /// pop-up takes is the keys that belonged to whatever held focus before it opened. **The keys
    /// that would move a diagram are the keys this is about, and this slice has none**; #175 is
    /// what gives the region its own, and they are routed through exactly this order.
    #[test]
    fn the_diagrams_keys_do_nothing_while_a_pop_up_is_open() {
        let mut chrome = Chrome::default();

        // The keyboard rests in the region, which is where a diagram goes and takes no keys yet.
        // **This is the state rule 14 is about**: the pop-up has to take the keyboard from here.
        assert_eq!(chrome.focus(), crate::chrome::Focus::Region);
        assert_eq!(action_of(&chrome, &key(KeyCode::Enter)), None);
        assert_eq!(action_of(&chrome, &key(KeyCode::Esc)), None);

        chrome.apply(Action::ShowAbout);
        assert_eq!(chrome.focus(), crate::chrome::Focus::PopUp);
        assert!(
            action_of(&chrome, &key(KeyCode::Enter)).is_some_and(|a| a == Action::Activate),
            "the pop-up took the keyboard and the key that acts on it did not reach it"
        );
        assert_eq!(
            action_of(&chrome, &key(KeyCode::Esc)),
            Some(Action::HideAbout),
            "the pop-up did not take the key that cancels it"
        );
    }

    #[test]
    fn a_key_that_names_no_action_changes_nothing_and_reports_nothing() {
        let chrome = Chrome::default();
        assert_eq!(action_of(&chrome, &key(KeyCode::Char('z'))), None);
        assert!(chrome.running());
        assert!(!chrome.drop_down_is_open());
        assert_eq!(chrome.focus(), crate::chrome::Focus::Region);
    }

    #[test]
    fn a_click_is_tested_against_the_rectangle_the_control_was_drawn_into() {
        let chrome = chrome_with_controls_at(4, 1);

        assert_eq!(
            action_of(
                &chrome,
                &mouse(MouseEventKind::Down(MouseButton::Left), 5, 1)
            ),
            Some(Action::Press(ControlId::File))
        );
        assert_eq!(
            action_of(&chrome, &mouse(MouseEventKind::Up(MouseButton::Left), 5, 1)),
            Some(Action::Release(Some(ControlId::File)))
        );
        assert_eq!(
            action_of(
                &chrome,
                &mouse(MouseEventKind::Down(MouseButton::Left), 30, 8)
            ),
            None,
            "a click outside every rectangle named a control"
        );
    }

    #[test]
    fn moving_the_pointer_over_a_control_changes_nothing_on_screen() {
        let chrome = chrome_with_controls_at(4, 1);
        assert_eq!(
            action_of(&chrome, &mouse(MouseEventKind::Moved, 5, 1)),
            None
        );
        assert_eq!(
            action_of(&chrome, &mouse(MouseEventKind::Moved, 6, 1)),
            None
        );
        let _ = at(5, 1);
    }

    #[test]
    fn a_click_that_names_no_action_changes_nothing_and_reports_nothing() {
        let chrome = chrome_with_controls_at(4, 1);
        assert_eq!(
            action_of(
                &chrome,
                &mouse(MouseEventKind::Up(MouseButton::Left), 30, 8)
            ),
            Some(Action::Release(None))
        );
        assert!(chrome.running());
    }
}
