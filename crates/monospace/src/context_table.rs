//! The single source of truth for contextual keybindings and bar labels.
//!
//! **One definition for offer and acceptance** (Representation Rule, #288): the contextual
//! status bar displays the actions offered by the current context, and the region dispatches
//! exactly the keys the table offers.
//!
//! **Two tables, one mechanism**: `SELECTION` is the context of a selected figure and
//! `STROKE_SELECT` the stroke sub-mode; `context_blocks` chooses between them from the state, and
//! a block's condition says whether it is active. `format_command_line` renders what
//! `dispatch_key` answers, so a key cannot be offered without being accepted; mouse pick is
//! offered alongside Tab selection when shapes exist.

use monospace_core::{Orientation, Terminal};
use monospace_diagram::{Delta, Direction, EndpointAt, Shape};
use monospace_editing::Command;
use ratatui_crossterm::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::diagram_state::{DiagramState, End, SubMode};
use crate::vocabulary::{Action, Reply};

/// Condition required for a contextual block to be active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Condition {
    /// Always active in this context.
    Always,
    /// Active only if the selected figure is a box.
    Box,
    /// Active only if the selected figure is a horizontal line.
    HorizontalLine,
    /// Active only if the selected figure is a vertical line.
    VerticalLine,
    /// Active only if the selected figure is a box or a line.
    BoxOrLine,
    /// Active only if the selected figure is a connector.
    Connector,
    /// Active only if the active endpoint is detached.
    EndpointDetached,
    /// Active only if the active endpoint is attached.
    EndpointAttached,
    /// Active only if the active endpoint is attached and either offset or out is non-zero.
    EndpointReset,
    /// Active only if the diagram holds at least one figure.
    HasShapes,
}

/// A pattern of key combinations matched by contextual actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyPattern {
    /// Any of the four bare arrow keys (Up, Down, Left, Right).
    Arrows,
    /// Shift plus any of the four arrow keys.
    ShiftArrows,
    /// Shift plus horizontal arrow keys (Left, Right).
    ShiftHorizontalArrows,
    /// Shift plus vertical arrow keys (Up, Down).
    ShiftVerticalArrows,
    /// A single key code with no modifiers.
    Code(KeyCode),
    /// `Shift+Tab` (`KeyCode::BackTab` with `NONE` or `SHIFT`, or `KeyCode::Tab` with `SHIFT`).
    BackTab,
    /// A single bare character key.
    Char(char),
    /// A single character key with no modifiers or Shift.
    CharOrShift(char),
}

impl KeyPattern {
    /// The bracketed key symbol or symbols representing this pattern in the contextual bar.
    #[must_use]
    pub(crate) fn notation(self) -> String {
        match self {
            Self::Arrows => "↑↓←→".to_string(),
            Self::ShiftArrows => "Shift+↑↓←→".to_string(),
            Self::ShiftHorizontalArrows => "Shift+←→".to_string(),
            Self::ShiftVerticalArrows => "Shift+↑↓".to_string(),
            Self::Code(code) => format!("[{}]", key_name(code)),
            Self::BackTab => "[Shift+Tab]".to_string(),
            Self::Char(c) | Self::CharOrShift(c) => format!("[{c}]"),
        }
    }

    /// Whether `event` matches this pattern.
    #[must_use]
    pub(crate) fn matches(self, event: &KeyEvent) -> bool {
        match self {
            Self::Arrows => {
                event.modifiers == KeyModifiers::NONE
                    && matches!(
                        event.code,
                        KeyCode::Right | KeyCode::Left | KeyCode::Up | KeyCode::Down
                    )
            }
            Self::ShiftArrows => {
                event.modifiers == KeyModifiers::SHIFT
                    && matches!(
                        event.code,
                        KeyCode::Right | KeyCode::Left | KeyCode::Up | KeyCode::Down
                    )
            }
            Self::ShiftHorizontalArrows => {
                event.modifiers == KeyModifiers::SHIFT
                    && matches!(event.code, KeyCode::Left | KeyCode::Right)
            }
            Self::ShiftVerticalArrows => {
                event.modifiers == KeyModifiers::SHIFT
                    && matches!(event.code, KeyCode::Up | KeyCode::Down)
            }
            Self::Code(code) => event.modifiers == KeyModifiers::NONE && event.code == code,
            Self::BackTab => {
                (matches!(event.modifiers, KeyModifiers::NONE | KeyModifiers::SHIFT)
                    && event.code == KeyCode::BackTab)
                    || (event.modifiers == KeyModifiers::SHIFT && event.code == KeyCode::Tab)
            }
            Self::Char(c) => {
                event.modifiers == KeyModifiers::NONE && event.code == KeyCode::Char(c)
            }
            Self::CharOrShift(c) => {
                (event.modifiers == KeyModifiers::NONE || event.modifiers == KeyModifiers::SHIFT)
                    && event.code == KeyCode::Char(c)
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn samples(self) -> Vec<(KeyCode, KeyModifiers)> {
        match self {
            Self::Arrows => vec![
                (KeyCode::Right, KeyModifiers::NONE),
                (KeyCode::Left, KeyModifiers::NONE),
                (KeyCode::Up, KeyModifiers::NONE),
                (KeyCode::Down, KeyModifiers::NONE),
            ],
            Self::ShiftArrows => vec![
                (KeyCode::Right, KeyModifiers::SHIFT),
                (KeyCode::Left, KeyModifiers::SHIFT),
                (KeyCode::Up, KeyModifiers::SHIFT),
                (KeyCode::Down, KeyModifiers::SHIFT),
            ],
            Self::ShiftHorizontalArrows => vec![
                (KeyCode::Left, KeyModifiers::SHIFT),
                (KeyCode::Right, KeyModifiers::SHIFT),
            ],
            Self::ShiftVerticalArrows => vec![
                (KeyCode::Up, KeyModifiers::SHIFT),
                (KeyCode::Down, KeyModifiers::SHIFT),
            ],
            Self::Code(code) => vec![(code, KeyModifiers::NONE)],
            Self::BackTab => vec![
                (KeyCode::BackTab, KeyModifiers::SHIFT),
                (KeyCode::BackTab, KeyModifiers::NONE),
                (KeyCode::Tab, KeyModifiers::SHIFT),
            ],
            Self::Char(c) => vec![(KeyCode::Char(c), KeyModifiers::NONE)],
            Self::CharOrShift(c) => vec![
                (KeyCode::Char(c), KeyModifiers::NONE),
                (KeyCode::Char(c), KeyModifiers::SHIFT),
            ],
        }
    }
}

fn key_name(code: KeyCode) -> &'static str {
    match code {
        KeyCode::Tab => "Tab",
        KeyCode::Delete => "Del",
        KeyCode::Esc => "Esc",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::PageUp => "PgUp",
        KeyCode::PageDown => "PgDn",
        _ => "?",
    }
}

/// What invokes a contextual action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Trigger {
    /// A keyboard event matched by `pattern`.
    Key(KeyPattern),
    /// A left press inside the region, answered by the region's click handler.
    Click,
}

impl Trigger {
    /// The bracketed notation representing this trigger in the contextual bar.
    #[must_use]
    pub(crate) fn notation(self) -> String {
        match self {
            Self::Click => "[Click]".to_string(),
            Self::Key(pattern) => pattern.notation(),
        }
    }

    #[cfg(test)]
    pub(crate) fn samples(self) -> Vec<(KeyCode, KeyModifiers)> {
        match self {
            Self::Key(pattern) => pattern.samples(),
            Self::Click => Vec::new(),
        }
    }
}

/// One of the five discrete terminal options offered by the inline selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerminalChoice {
    Arm,
    Solid,
    Outline,
    Ascii,
    Diamond,
}

impl TerminalChoice {
    pub(crate) fn to_terminal(self) -> Terminal {
        match self {
            Self::Arm => Terminal::Arm,
            Self::Solid => Terminal::Symbol {
                style: monospace_core::SymbolStyleName::from("solid"),
            },
            Self::Outline => Terminal::Symbol {
                style: monospace_core::SymbolStyleName::from("outline"),
            },
            Self::Ascii => Terminal::Symbol {
                style: monospace_core::SymbolStyleName::from("ascii"),
            },
            Self::Diamond => Terminal::Glyph {
                glyph: monospace_core::Glyph::new("◆").expect("valid glyph"),
            },
        }
    }
}

/// What a contextual keypress means.
#[derive(Debug)]
pub(crate) enum ContextMeaning {
    /// Dispatches a direct application action.
    Act(Action),
    /// Moves the selected figure by the arrow direction.
    Move,
    /// Moves the active endpoint along face or canvas coordinates.
    MoveEndpoint,
    /// Adjusts the active endpoint's outward standoff.
    AdjustEndpointOut(i16),
    /// Resets the active endpoint's offset and standoff to zero.
    ResetEndpoint,
    /// Detaches the active attached endpoint into an open canvas position.
    DetachEndpoint,
    /// Resizes the selected figure by the arrow direction.
    Resize,
    /// Inverts the terminals of the selected connector.
    InvertTerminals,
    /// Removes the selected figure.
    Remove,
    /// Moves the selected figure one place toward the front of the order.
    Forward,
    /// Moves the selected figure one place toward the back of the order.
    Backward,
    /// Moves the selected figure to the front of the order.
    ToFront,
    /// Moves the selected figure to the back of the order.
    ToBack,
    /// Sets the stroke of the selected figure to the catalog name.
    SetStroke(&'static str),
    /// Sets the terminal style of the active connector endpoint.
    SetEndpointTerminal(TerminalChoice),
    /// Sets the departure direction of the active detached connector endpoint.
    SetEndpointLeaving(Direction),
    /// Enters an inline sub-mode.
    EnterSubMode(SubMode),
    /// Leaves an inline sub-mode back to normal selection.
    LeaveSubMode,
    /// Deselects the active figure.
    ///
    /// **Always owes a frame**: the action is offered only while a figure is selected, so the
    /// table reaches it with something to drop.
    Deselect,
    /// Selects the next figure in drawing order (`Diagram::shapes()`, back-to-front).
    SelectNext,
    /// Selects the previous figure in drawing order (`Diagram::shapes()`, front-to-back).
    SelectPrevious,
    /// Creates a 5×5 box at viewport center and selects it.
    CreateBox,
    /// Creates a vertical line of length 5 at viewport center and selects it.
    CreateLine,
    /// Creates a detached connector at viewport center and selects it.
    CreateConnector,
    /// Answered by the region's click, never by a key.
    Pick,
}

/// One contextual action offered by a block.
#[derive(Debug)]
pub(crate) struct ContextAction {
    pub(crate) trigger: Trigger,
    /// The words after the notation, or `""` when the notation is the whole label.
    pub(crate) label: &'static str,
    pub(crate) meaning: ContextMeaning,
}

impl ContextAction {
    pub(crate) fn text(&self) -> String {
        let notation = self.trigger.notation();
        if self.label.is_empty() {
            notation
        } else {
            format!("{notation} {}", self.label)
        }
    }
}

/// A group of related contextual actions sharing a category prefix and condition.
pub(crate) struct ContextBlock {
    pub(crate) category: Option<&'static str>,
    pub(crate) condition: Condition,
    pub(crate) actions: &'static [ContextAction],
}

static IDLE: &[ContextBlock] = &[
    ContextBlock {
        category: Some("CREATE"),
        condition: Condition::Always,
        actions: &[
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('b')),
                label: "Box",
                meaning: ContextMeaning::CreateBox,
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('l')),
                label: "Line",
                meaning: ContextMeaning::CreateLine,
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('c')),
                label: "Connector",
                meaning: ContextMeaning::CreateConnector,
            },
        ],
    },
    ContextBlock {
        category: Some("SELECT"),
        condition: Condition::HasShapes,
        actions: &[
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Code(KeyCode::Tab)),
                label: "Next",
                meaning: ContextMeaning::SelectNext,
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::BackTab),
                label: "Prev",
                meaning: ContextMeaning::SelectPrevious,
            },
            ContextAction {
                trigger: Trigger::Click,
                label: "Pick",
                meaning: ContextMeaning::Pick,
            },
        ],
    },
];

static SELECTION: &[ContextBlock] = &[
    ContextBlock {
        category: Some("MOVE"),
        condition: Condition::BoxOrLine,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Arrows),
            label: "",
            meaning: ContextMeaning::Move,
        }],
    },
    ContextBlock {
        category: Some("RESIZE"),
        condition: Condition::Box,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::ShiftArrows),
            label: "",
            meaning: ContextMeaning::Resize,
        }],
    },
    ContextBlock {
        category: Some("RESIZE"),
        condition: Condition::HorizontalLine,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::ShiftHorizontalArrows),
            label: "",
            meaning: ContextMeaning::Resize,
        }],
    },
    ContextBlock {
        category: Some("RESIZE"),
        condition: Condition::VerticalLine,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::ShiftVerticalArrows),
            label: "",
            meaning: ContextMeaning::Resize,
        }],
    },
    ContextBlock {
        category: Some("TEXT"),
        condition: Condition::Box,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('t')),
            label: "Edit",
            meaning: ContextMeaning::Act(Action::EditText),
        }],
    },
    ContextBlock {
        category: Some("ENDPOINTS"),
        condition: Condition::Connector,
        actions: &[
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('1')),
                label: "From",
                meaning: ContextMeaning::EnterSubMode(SubMode::Endpoint(End::From)),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('2')),
                label: "To",
                meaning: ContextMeaning::EnterSubMode(SubMode::Endpoint(End::To)),
            },
        ],
    },
    ContextBlock {
        category: Some("INVERT"),
        condition: Condition::Connector,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('i')),
            label: "",
            meaning: ContextMeaning::InvertTerminals,
        }],
    },
    ContextBlock {
        category: Some("STROKE"),
        condition: Condition::Always,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('s')),
            label: "",
            meaning: ContextMeaning::EnterSubMode(SubMode::StrokeSelect),
        }],
    },
    ContextBlock {
        category: Some("ORDER"),
        condition: Condition::Always,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('z')),
            label: "",
            meaning: ContextMeaning::EnterSubMode(SubMode::OrderMode),
        }],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Code(KeyCode::Tab)),
                label: "Next",
                meaning: ContextMeaning::SelectNext,
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::BackTab),
                label: "Prev",
                meaning: ContextMeaning::SelectPrevious,
            },
        ],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Code(KeyCode::Delete)),
            label: "Delete",
            meaning: ContextMeaning::Remove,
        }],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Code(KeyCode::Esc)),
            label: "Deselect",
            meaning: ContextMeaning::Deselect,
        }],
    },
];

static STROKE_SELECT: &[ContextBlock] = &[
    ContextBlock {
        category: Some("STROKE"),
        condition: Condition::Always,
        actions: &[
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('1')),
                label: "light ┌─┐",
                meaning: ContextMeaning::SetStroke("light"),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('2')),
                label: "double ╔═╗",
                meaning: ContextMeaning::SetStroke("double"),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('3')),
                label: "light-round ╭─╮",
                meaning: ContextMeaning::SetStroke("light-round"),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('4')),
                label: "heavy ┏━┓",
                meaning: ContextMeaning::SetStroke("heavy"),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('5')),
                label: "ascii +-+",
                meaning: ContextMeaning::SetStroke("ascii"),
            },
        ],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Code(KeyCode::Esc)),
            label: "Cancel",
            meaning: ContextMeaning::LeaveSubMode,
        }],
    },
];

static TERMINAL_SELECT: &[ContextBlock] = &[
    ContextBlock {
        category: Some("TERMINAL"),
        condition: Condition::Always,
        actions: &[
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('1')),
                label: "arm ─",
                meaning: ContextMeaning::SetEndpointTerminal(TerminalChoice::Arm),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('2')),
                label: "solid ▶",
                meaning: ContextMeaning::SetEndpointTerminal(TerminalChoice::Solid),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('3')),
                label: "outline ▷",
                meaning: ContextMeaning::SetEndpointTerminal(TerminalChoice::Outline),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('4')),
                label: "ascii >",
                meaning: ContextMeaning::SetEndpointTerminal(TerminalChoice::Ascii),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('5')),
                label: "diamond ◆",
                meaning: ContextMeaning::SetEndpointTerminal(TerminalChoice::Diamond),
            },
        ],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Code(KeyCode::Esc)),
            label: "Cancel",
            meaning: ContextMeaning::LeaveSubMode,
        }],
    },
];

static LEAVING_SELECT: &[ContextBlock] = &[
    ContextBlock {
        category: Some("LEAVING"),
        condition: Condition::Always,
        actions: &[
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('1')),
                label: "↑ North",
                meaning: ContextMeaning::SetEndpointLeaving(Direction::Up),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('2')),
                label: "→ East",
                meaning: ContextMeaning::SetEndpointLeaving(Direction::Right),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('3')),
                label: "↓ South",
                meaning: ContextMeaning::SetEndpointLeaving(Direction::Down),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('4')),
                label: "← West",
                meaning: ContextMeaning::SetEndpointLeaving(Direction::Left),
            },
        ],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Code(KeyCode::Esc)),
            label: "Cancel",
            meaning: ContextMeaning::LeaveSubMode,
        }],
    },
];

static ORDER_MODE: &[ContextBlock] = &[
    ContextBlock {
        category: Some("ORDER"),
        condition: Condition::Always,
        actions: &[
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Code(KeyCode::Home)),
                label: "Front",
                meaning: ContextMeaning::ToFront,
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Code(KeyCode::End)),
                label: "Back",
                meaning: ContextMeaning::ToBack,
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Code(KeyCode::PageUp)),
                label: "Forward",
                meaning: ContextMeaning::Forward,
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Code(KeyCode::PageDown)),
                label: "Backward",
                meaning: ContextMeaning::Backward,
            },
        ],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Code(KeyCode::Esc)),
            label: "Done",
            meaning: ContextMeaning::LeaveSubMode,
        }],
    },
];

static ENDPOINT_FROM: &[ContextBlock] = &[
    ContextBlock {
        category: Some("MOVE"),
        condition: Condition::EndpointDetached,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Arrows),
            label: "",
            meaning: ContextMeaning::MoveEndpoint,
        }],
    },
    ContextBlock {
        category: Some("LEAVING"),
        condition: Condition::EndpointDetached,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('d')),
            label: "",
            meaning: ContextMeaning::EnterSubMode(SubMode::LeavingSelect(End::From)),
        }],
    },
    ContextBlock {
        category: Some("OFFSET"),
        condition: Condition::EndpointAttached,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Arrows),
            label: "",
            meaning: ContextMeaning::MoveEndpoint,
        }],
    },
    ContextBlock {
        category: Some("OUT"),
        condition: Condition::EndpointAttached,
        actions: &[
            ContextAction {
                trigger: Trigger::Key(KeyPattern::CharOrShift('+')),
                label: "",
                meaning: ContextMeaning::AdjustEndpointOut(1),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('-')),
                label: "",
                meaning: ContextMeaning::AdjustEndpointOut(-1),
            },
        ],
    },
    ContextBlock {
        category: None,
        condition: Condition::EndpointReset,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('r')),
            label: "Reset",
            meaning: ContextMeaning::ResetEndpoint,
        }],
    },
    ContextBlock {
        category: None,
        condition: Condition::EndpointAttached,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('0')),
            label: "Detach",
            meaning: ContextMeaning::DetachEndpoint,
        }],
    },
    ContextBlock {
        category: Some("TERMINAL"),
        condition: Condition::Always,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('t')),
            label: "",
            meaning: ContextMeaning::EnterSubMode(SubMode::TerminalSelect(End::From)),
        }],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('2')),
            label: "To",
            meaning: ContextMeaning::EnterSubMode(SubMode::Endpoint(End::To)),
        }],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Code(KeyCode::Esc)),
            label: "Done",
            meaning: ContextMeaning::LeaveSubMode,
        }],
    },
];

static ENDPOINT_TO: &[ContextBlock] = &[
    ContextBlock {
        category: Some("MOVE"),
        condition: Condition::EndpointDetached,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Arrows),
            label: "",
            meaning: ContextMeaning::MoveEndpoint,
        }],
    },
    ContextBlock {
        category: Some("LEAVING"),
        condition: Condition::EndpointDetached,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('d')),
            label: "",
            meaning: ContextMeaning::EnterSubMode(SubMode::LeavingSelect(End::To)),
        }],
    },
    ContextBlock {
        category: Some("OFFSET"),
        condition: Condition::EndpointAttached,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Arrows),
            label: "",
            meaning: ContextMeaning::MoveEndpoint,
        }],
    },
    ContextBlock {
        category: Some("OUT"),
        condition: Condition::EndpointAttached,
        actions: &[
            ContextAction {
                trigger: Trigger::Key(KeyPattern::CharOrShift('+')),
                label: "",
                meaning: ContextMeaning::AdjustEndpointOut(1),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('-')),
                label: "",
                meaning: ContextMeaning::AdjustEndpointOut(-1),
            },
        ],
    },
    ContextBlock {
        category: None,
        condition: Condition::EndpointReset,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('r')),
            label: "Reset",
            meaning: ContextMeaning::ResetEndpoint,
        }],
    },
    ContextBlock {
        category: None,
        condition: Condition::EndpointAttached,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('0')),
            label: "Detach",
            meaning: ContextMeaning::DetachEndpoint,
        }],
    },
    ContextBlock {
        category: Some("TERMINAL"),
        condition: Condition::Always,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('t')),
            label: "",
            meaning: ContextMeaning::EnterSubMode(SubMode::TerminalSelect(End::To)),
        }],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('1')),
            label: "From",
            meaning: ContextMeaning::EnterSubMode(SubMode::Endpoint(End::From)),
        }],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Code(KeyCode::Esc)),
            label: "Done",
            meaning: ContextMeaning::LeaveSubMode,
        }],
    },
];

/// The delta along screen axes named by the four arrow keys, or `None` for any other key code.
///
/// Right is one cell along the x axis, Down along the y one, and the two negatives are the other
/// directions.
pub(crate) fn arrow_direction(code: KeyCode) -> Option<(i16, i16)> {
    match code {
        KeyCode::Right => Some((1, 0)),
        KeyCode::Left => Some((-1, 0)),
        KeyCode::Up => Some((0, -1)),
        KeyCode::Down => Some((0, 1)),
        _ => None,
    }
}

/// The contextual blocks active for `state`.
pub(crate) fn context_blocks(
    state: &DiagramState,
) -> impl Iterator<Item = &'static ContextBlock> + use<> {
    let sub_mode = state.sub_mode();
    let selected_shape = state.selected_shape();
    let selected = selected_shape.is_some();
    let is_box = selected_shape.is_some_and(|s| matches!(s, Shape::Box { .. }));
    let is_horizontal_line = selected_shape.is_some_and(|s| {
        matches!(
            s,
            Shape::Line {
                orientation: Orientation::Horizontal,
                ..
            }
        )
    });
    let is_vertical_line = selected_shape.is_some_and(|s| {
        matches!(
            s,
            Shape::Line {
                orientation: Orientation::Vertical,
                ..
            }
        )
    });
    let is_connector = selected_shape.is_some_and(|s| matches!(s, Shape::Connector { .. }));
    let is_box_or_line =
        selected_shape.is_some_and(|s| matches!(s, Shape::Box { .. } | Shape::Line { .. }));
    let has_shapes = state.session().diagram().shape_count() > 0;

    let (is_endpoint_detached, is_endpoint_attached, is_endpoint_reset) =
        match (sub_mode, selected_shape) {
            (Some(SubMode::Endpoint(end)), Some(Shape::Connector { from, to, .. })) => {
                let active = match end {
                    End::From => from,
                    End::To => to,
                };
                match active.at {
                    EndpointAt::Absolute { .. } => (true, false, false),
                    EndpointAt::Reference(ref r) => {
                        (false, true, r.offset != Delta::ZERO || r.out != 0)
                    }
                }
            }
            _ => (false, false, false),
        };

    let slice: &'static [ContextBlock] = match sub_mode {
        Some(SubMode::StrokeSelect) if selected => STROKE_SELECT,
        Some(SubMode::OrderMode) if selected => ORDER_MODE,
        Some(SubMode::Endpoint(End::From)) if selected && is_connector => ENDPOINT_FROM,
        Some(SubMode::Endpoint(End::To)) if selected && is_connector => ENDPOINT_TO,
        Some(SubMode::TerminalSelect(_)) if selected && is_connector => TERMINAL_SELECT,
        Some(SubMode::LeavingSelect(_)) if selected && is_connector => LEAVING_SELECT,
        _ if selected => SELECTION,
        _ => IDLE,
    };

    slice.iter().filter(move |block| match block.condition {
        Condition::Always => true,
        Condition::Box => is_box,
        Condition::HorizontalLine => is_horizontal_line,
        Condition::VerticalLine => is_vertical_line,
        Condition::BoxOrLine => is_box_or_line,
        Condition::Connector => is_connector,
        Condition::EndpointDetached => is_endpoint_detached,
        Condition::EndpointAttached => is_endpoint_attached,
        Condition::EndpointReset => is_endpoint_reset,
        Condition::HasShapes => has_shapes,
    })
}

/// Formats the contextual command row (row 1) from the diagram state.
pub(crate) fn format_command_line(state: &DiagramState) -> String {
    let mut result = String::new();
    for (i, block) in context_blocks(state).enumerate() {
        if i > 0 {
            result.push_str(" · ");
        }
        if let Some(category) = block.category {
            result.push_str(category);
            result.push_str(": ");
        }
        for (j, action) in block.actions.iter().enumerate() {
            if j > 0 {
                result.push(' ');
            }
            result.push_str(&action.text());
        }
    }
    result
}

fn dispatch_endpoint_meaning(
    meaning: &ContextMeaning,
    state: &DiagramState,
    key: &KeyEvent,
) -> Reply {
    let Some(SubMode::Endpoint(end)) = state.sub_mode() else {
        return Reply::Ignored;
    };
    let id = state
        .selected()
        .expect("endpoint block matched without selection");

    match *meaning {
        ContextMeaning::MoveEndpoint => {
            let (dx, dy) = arrow_direction(key.code).expect("arrow pattern matched arrow key");
            Reply::Act(Action::Command(Command::MoveEndpoint {
                id,
                end,
                by: (dx, dy).into(),
                out: 0,
            }))
        }
        ContextMeaning::AdjustEndpointOut(out) => {
            Reply::Act(Action::Command(Command::MoveEndpoint {
                id,
                end,
                by: Delta::ZERO,
                out,
            }))
        }
        ContextMeaning::ResetEndpoint => {
            Reply::Act(Action::Command(Command::ResetEndpoint { id, end }))
        }
        ContextMeaning::DetachEndpoint => {
            Reply::Act(Action::Command(Command::DetachEndpoint { id, end }))
        }
        _ => Reply::Ignored,
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "executes each contextual meaning in one match"
)]
fn execute_meaning(meaning: &ContextMeaning, state: &mut DiagramState, key: &KeyEvent) -> Reply {
    match *meaning {
        ContextMeaning::Act(ref act) => Reply::Act(act.clone()),
        ContextMeaning::Move => {
            let (dx, dy) = arrow_direction(key.code).expect("arrow pattern matched arrow key");
            let id = state
                .selected()
                .expect("selection block matched without selection");
            Reply::Act(Action::Command(Command::Move {
                id,
                by: (dx, dy).into(),
            }))
        }
        ContextMeaning::Resize => {
            let (dx, dy) = arrow_direction(key.code).expect("arrow pattern matched arrow key");
            let id = state
                .selected()
                .expect("selection block matched without selection");
            Reply::Act(Action::Command(Command::Resize {
                id,
                by: (dx, dy).into(),
            }))
        }
        ContextMeaning::InvertTerminals => {
            let id = state
                .selected()
                .expect("connector block matched without selection");
            Reply::Act(Action::Command(Command::InvertTerminals { id }))
        }
        ContextMeaning::Remove => {
            let id = state
                .selected()
                .expect("selection block matched without selection");
            Reply::Act(Action::Command(Command::Remove { id }))
        }
        ContextMeaning::Forward => {
            let id = state
                .selected()
                .expect("order block matched without selection");
            Reply::Act(Action::Command(Command::Forward { id }))
        }
        ContextMeaning::Backward => {
            let id = state
                .selected()
                .expect("order block matched without selection");
            Reply::Act(Action::Command(Command::Backward { id }))
        }
        ContextMeaning::ToFront => {
            let id = state
                .selected()
                .expect("order block matched without selection");
            Reply::Act(Action::Command(Command::ToFront { id }))
        }
        ContextMeaning::ToBack => {
            let id = state
                .selected()
                .expect("order block matched without selection");
            Reply::Act(Action::Command(Command::ToBack { id }))
        }
        ContextMeaning::SetStroke(name) => {
            state.set_sub_mode(None);
            let id = state
                .selected()
                .expect("stroke select block matched without selection");
            Reply::Act(Action::Command(Command::SetStroke {
                id,
                stroke: monospace_diagram::Stroke::from(name),
            }))
        }
        ContextMeaning::EnterSubMode(sub_mode) => {
            state.set_sub_mode(Some(sub_mode));
            Reply::Repaint
        }
        ContextMeaning::LeaveSubMode => {
            let next_sub_mode = match state.sub_mode() {
                Some(SubMode::TerminalSelect(end) | SubMode::LeavingSelect(end)) => {
                    Some(SubMode::Endpoint(end))
                }
                _ => None,
            };
            state.set_sub_mode(next_sub_mode);
            Reply::Repaint
        }
        ContextMeaning::SetEndpointTerminal(choice) => {
            let Some(SubMode::TerminalSelect(end)) = state.sub_mode() else {
                return Reply::Ignored;
            };
            let id = state
                .selected()
                .expect("terminal select block matched without selection");
            state.set_sub_mode(Some(SubMode::Endpoint(end)));
            Reply::Act(Action::Command(Command::SetEndpointTerminal {
                id,
                end,
                terminal: choice.to_terminal(),
            }))
        }
        ContextMeaning::SetEndpointLeaving(leaving) => {
            let Some(SubMode::LeavingSelect(end)) = state.sub_mode() else {
                return Reply::Ignored;
            };
            let id = state
                .selected()
                .expect("leaving select block matched without selection");
            state.set_sub_mode(Some(SubMode::Endpoint(end)));
            Reply::Act(Action::Command(Command::SetEndpointLeaving {
                id,
                end,
                leaving,
            }))
        }
        ContextMeaning::Deselect => {
            state.select(None);
            Reply::Repaint
        }
        ContextMeaning::SelectNext => {
            if state.select_next() {
                Reply::Repaint
            } else {
                Reply::Consumed
            }
        }
        ContextMeaning::SelectPrevious => {
            if state.select_previous() {
                Reply::Repaint
            } else {
                Reply::Consumed
            }
        }
        ContextMeaning::CreateBox => {
            state.create_box();
            Reply::Repaint
        }
        ContextMeaning::CreateLine => {
            state.create_line();
            Reply::Repaint
        }
        ContextMeaning::CreateConnector => {
            state.create_connector();
            Reply::Repaint
        }
        ContextMeaning::Pick => Reply::Ignored,
        ContextMeaning::MoveEndpoint
        | ContextMeaning::AdjustEndpointOut(_)
        | ContextMeaning::ResetEndpoint
        | ContextMeaning::DetachEndpoint => dispatch_endpoint_meaning(meaning, state, key),
    }
}

/// Dispatches a keyboard event in the diagram region using the contextual table.
pub(crate) fn dispatch_key(state: &mut DiagramState, key: &KeyEvent) -> Reply {
    if key.kind != KeyEventKind::Press {
        return Reply::Ignored;
    }

    for block in context_blocks(state) {
        for action in block.actions {
            let Trigger::Key(pattern) = action.trigger else {
                continue;
            };
            if pattern.matches(key) {
                return execute_meaning(&action.meaning, state, key);
            }
        }
    }

    Reply::Ignored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_pattern_notation_covers_all_variants() {
        assert_eq!(KeyPattern::Arrows.notation(), "↑↓←→");
        assert_eq!(KeyPattern::ShiftArrows.notation(), "Shift+↑↓←→");
        assert_eq!(KeyPattern::ShiftHorizontalArrows.notation(), "Shift+←→");
        assert_eq!(KeyPattern::ShiftVerticalArrows.notation(), "Shift+↑↓");
        assert_eq!(KeyPattern::Code(KeyCode::Tab).notation(), "[Tab]");
        assert_eq!(KeyPattern::Code(KeyCode::Delete).notation(), "[Del]");
        assert_eq!(KeyPattern::Code(KeyCode::Esc).notation(), "[Esc]");
        assert_eq!(KeyPattern::Code(KeyCode::Home).notation(), "[Home]");
        assert_eq!(KeyPattern::Code(KeyCode::End).notation(), "[End]");
        assert_eq!(KeyPattern::Code(KeyCode::PageUp).notation(), "[PgUp]");
        assert_eq!(KeyPattern::Code(KeyCode::PageDown).notation(), "[PgDn]");
        assert_eq!(KeyPattern::Code(KeyCode::F(1)).notation(), "[?]");
        assert_eq!(KeyPattern::BackTab.notation(), "[Shift+Tab]");
        assert_eq!(KeyPattern::Char('b').notation(), "[b]");
        assert_eq!(KeyPattern::CharOrShift('+').notation(), "[+]");
    }

    #[test]
    fn trigger_notation_covers_keys_and_click() {
        assert_eq!(Trigger::Click.notation(), "[Click]");
        assert_eq!(Trigger::Key(KeyPattern::Char('x')).notation(), "[x]");
    }

    #[test]
    fn context_action_text_with_empty_and_non_empty_label() {
        let action_with_label = ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('b')),
            label: "Box",
            meaning: ContextMeaning::CreateBox,
        };
        assert_eq!(action_with_label.text(), "[b] Box");

        let action_without_label = ContextAction {
            trigger: Trigger::Key(KeyPattern::Arrows),
            label: "",
            meaning: ContextMeaning::Move,
        };
        assert_eq!(action_without_label.text(), "↑↓←→");

        let click_action = ContextAction {
            trigger: Trigger::Click,
            label: "Pick",
            meaning: ContextMeaning::Pick,
        };
        assert_eq!(click_action.text(), "[Click] Pick");
    }

    #[test]
    fn all_actions_in_all_tables_have_valid_notation() {
        let all_tables: &[&[ContextBlock]] = &[
            IDLE,
            SELECTION,
            STROKE_SELECT,
            TERMINAL_SELECT,
            LEAVING_SELECT,
            ORDER_MODE,
            ENDPOINT_FROM,
            ENDPOINT_TO,
        ];
        for table in all_tables {
            for block in *table {
                for action in block.actions {
                    let notation = action.trigger.notation();
                    assert!(
                        !notation.contains("[?]"),
                        "action in table produced [?]: {notation}"
                    );
                    assert!(
                        !action.text().contains("[?]"),
                        "action text produced [?]: {}",
                        action.text()
                    );
                }
            }
        }
    }
}
