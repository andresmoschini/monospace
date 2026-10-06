//! Q1/Q2: the application's own state and action enum.
//!
//! ARCHITECTURAL PROOF: this module imports NEITHER `ratatui` NOR `tuirealm`.
//! grep for `ratatui\|tuirealm` in this file returns nothing.

#[derive(Debug, Default, PartialEq)]
pub struct App {
    /// the counter the UI displays
    pub counter: i64,
    /// the "log" of every Action that was applied, in order
    pub applied: Vec<Action>,
    /// set when a drop-down is open
    pub menu_open: bool,
    /// set when the modal dialog is open
    pub modal_open: bool,
}

/// The ONLY thing that is allowed to change `App`.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Increment,
    Decrement,
    Reset,
    ToggleMenu,
    ToggleModal,
    Quit,
}

impl App {
    /// The single write path. Every mutation of `App` goes through here.
    pub fn apply(&mut self, a: Action) {
        match &a {
            Action::Increment => self.counter += 1,
            Action::Decrement => self.counter -= 1,
            Action::Reset => self.counter = 0,
            Action::ToggleMenu => self.menu_open = !self.menu_open,
            Action::ToggleModal => self.modal_open = !self.modal_open,
            Action::Quit => {}
        }
        self.applied.push(a);
    }
}

/// Map a raw key code to an Action. Also framework-free.
pub fn action_for_key(code: char) -> Option<Action> {
    Some(match code {
        '+' | '=' => Action::Increment,
        '-' => Action::Decrement,
        'r' => Action::Reset,
        'm' => Action::ToggleMenu,
        'd' => Action::ToggleModal,
        'q' => Action::Quit,
        _ => return None,
    })
}
