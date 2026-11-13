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

use monospace_core::{Orientation, ShapeId, Terminal};
use monospace_diagram::{Anchor, Delta, Direction, EndpointAt, Shape};
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
    /// Active only if the selected figure is a horizontal or vertical line.
    Line,
    /// Active only if the selected figure is a box, a line, or a connector with at least one fixed
    /// endpoint — the figures a bare arrow can move.
    Movable,
    /// Active only if the selected figure is a connector.
    Connector,
    /// Active only if the active endpoint is detached.
    EndpointDetached,
    /// Active only if the active endpoint is attached.
    EndpointAttached,
    /// Active only if the active endpoint is attached and either offset or out is non-zero.
    EndpointReset,
    /// Active only if the active endpoint is detached and the diagram holds at least one
    /// attachable shape (a box or a line).
    AttachableEndpoint,
    /// Active only if the diagram holds at least one figure.
    HasShapes,
    /// Active only if the active endpoint is the given end.
    ActiveEndpointIs(End),
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
    /// Toggles the orientation of the selected line.
    RotateLine,
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
    /// Enters the leaving direction selector for the active endpoint.
    EnterLeavingSelect,
    /// Enters the terminal style selector for the active endpoint.
    EnterTerminalSelect,
    /// Enters the attach picker for the active detached endpoint.
    EnterAttachSelect,
    /// Focuses the next attachable candidate in drawing order.
    FocusNextAttachTarget,
    /// Focuses the previous attachable candidate in drawing order.
    FocusPreviousAttachTarget,
    /// Attaches the active detached endpoint to the focused target's anchor.
    AttachEndpoint(Anchor),
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
    /// Clones the selected figure in place and selects the duplicate.
    Duplicate,
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
#[derive(Debug)]
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
        condition: Condition::Movable,
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
        category: Some("ROTATE"),
        condition: Condition::Line,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('r')),
            label: "",
            meaning: ContextMeaning::RotateLine,
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
            trigger: Trigger::Key(KeyPattern::Char('d')),
            label: "Duplicate",
            meaning: ContextMeaning::Duplicate,
        }],
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

static ENDPOINT: &[ContextBlock] = &[
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
            meaning: ContextMeaning::EnterLeavingSelect,
        }],
    },
    ContextBlock {
        category: Some("ATTACH"),
        condition: Condition::AttachableEndpoint,
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('a')),
            label: "",
            meaning: ContextMeaning::EnterAttachSelect,
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
            meaning: ContextMeaning::EnterTerminalSelect,
        }],
    },
    ContextBlock {
        category: None,
        condition: Condition::ActiveEndpointIs(End::To),
        actions: &[ContextAction {
            trigger: Trigger::Key(KeyPattern::Char('1')),
            label: "From",
            meaning: ContextMeaning::EnterSubMode(SubMode::Endpoint(End::From)),
        }],
    },
    ContextBlock {
        category: None,
        condition: Condition::ActiveEndpointIs(End::From),
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

static ATTACH_SELECT: &[ContextBlock] = &[
    ContextBlock {
        category: Some("TARGET"),
        condition: Condition::Always,
        actions: &[
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Code(KeyCode::Tab)),
                label: "Next",
                meaning: ContextMeaning::FocusNextAttachTarget,
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::BackTab),
                label: "Prev",
                meaning: ContextMeaning::FocusPreviousAttachTarget,
            },
        ],
    },
    ContextBlock {
        category: Some("ANCHOR"),
        condition: Condition::Always,
        actions: &[
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('1')),
                label: "Top",
                meaning: ContextMeaning::AttachEndpoint(Anchor::Top),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('2')),
                label: "Right",
                meaning: ContextMeaning::AttachEndpoint(Anchor::Right),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('3')),
                label: "Bottom",
                meaning: ContextMeaning::AttachEndpoint(Anchor::Bottom),
            },
            ContextAction {
                trigger: Trigger::Key(KeyPattern::Char('4')),
                label: "Left",
                meaning: ContextMeaning::AttachEndpoint(Anchor::Left),
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

#[cfg(test)]
pub(crate) static ALL_TABLES: &[&[ContextBlock]] = &[
    IDLE,
    SELECTION,
    STROKE_SELECT,
    ORDER_MODE,
    ENDPOINT,
    TERMINAL_SELECT,
    LEAVING_SELECT,
    ATTACH_SELECT,
];

#[cfg(test)]
pub(crate) fn all_offered_samples() -> Vec<(KeyCode, KeyModifiers)> {
    ALL_TABLES
        .iter()
        .flat_map(|table| table.iter())
        .flat_map(|block| block.actions.iter())
        .flat_map(|action| action.trigger.samples())
        .collect()
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectionKind {
    Box,
    HorizontalLine,
    VerticalLine,
    Connector,
}

impl SelectionKind {
    fn of(shape: &Shape) -> Self {
        match shape {
            Shape::Box { .. } => Self::Box,
            Shape::Line {
                orientation: Orientation::Horizontal,
                ..
            } => Self::HorizontalLine,
            Shape::Line {
                orientation: Orientation::Vertical,
                ..
            } => Self::VerticalLine,
            Shape::Connector { .. } => Self::Connector,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Context {
    Idle,
    Selection,
    Stroke,
    Order,
    Endpoint(End),
    Terminal(End),
    Leaving(End),
    Attach(End),
    /// A sub-mode and a selection that do not fit. No action is offered.
    Inconsistent,
}

pub(crate) fn context(state: &DiagramState) -> Context {
    let kind = state.selected_shape().map(SelectionKind::of);

    match state.sub_mode() {
        None => match kind {
            None => Context::Idle,
            Some(_) => Context::Selection,
        },
        Some(SubMode::StrokeSelect) if kind.is_some() => Context::Stroke,
        Some(SubMode::OrderMode) if kind.is_some() => Context::Order,
        Some(SubMode::Endpoint(end)) if kind == Some(SelectionKind::Connector) => {
            Context::Endpoint(end)
        }
        Some(SubMode::TerminalSelect(end)) if kind == Some(SelectionKind::Connector) => {
            Context::Terminal(end)
        }
        Some(SubMode::LeavingSelect(end)) if kind == Some(SelectionKind::Connector) => {
            Context::Leaving(end)
        }
        Some(SubMode::AttachSelect { end, .. }) if kind == Some(SelectionKind::Connector) => {
            Context::Attach(end)
        }
        Some(
            SubMode::StrokeSelect
            | SubMode::OrderMode
            | SubMode::Endpoint(_)
            | SubMode::TerminalSelect(_)
            | SubMode::LeavingSelect(_)
            | SubMode::AttachSelect { .. },
        ) => Context::Inconsistent,
    }
}

/// The contextual blocks active for `state`.
pub(crate) fn context_blocks(
    state: &DiagramState,
) -> impl Iterator<Item = &'static ContextBlock> + use<> {
    let kind = state.selected_shape().map(SelectionKind::of);
    let is_box = kind == Some(SelectionKind::Box);
    let is_horizontal_line = kind == Some(SelectionKind::HorizontalLine);
    let is_vertical_line = kind == Some(SelectionKind::VerticalLine);
    let is_line = is_horizontal_line || is_vertical_line;
    let is_connector = kind == Some(SelectionKind::Connector);
    let is_box_or_line = matches!(
        kind,
        Some(SelectionKind::Box | SelectionKind::HorizontalLine | SelectionKind::VerticalLine)
    );
    let is_movable_connector = matches!(
        state.selected_shape(),
        Some(Shape::Connector { from, to, .. })
            if matches!(from.at, EndpointAt::Absolute { .. })
                || matches!(to.at, EndpointAt::Absolute { .. })
    );
    let has_shapes = state.session().diagram().shape_count() > 0;
    let has_attachable_shapes = state
        .session()
        .diagram()
        .shapes()
        .any(|(_, shape)| DiagramState::is_attachable(shape));

    let (slice, active_endpoint, is_endpoint_detached, is_endpoint_attached, is_endpoint_reset): (
        &'static [ContextBlock],
        _,
        _,
        _,
        _,
    ) = match context(state) {
        Context::Idle => (IDLE, None, false, false, false),
        Context::Selection => (SELECTION, None, false, false, false),
        Context::Stroke => (STROKE_SELECT, None, false, false, false),
        Context::Order => (ORDER_MODE, None, false, false, false),
        Context::Endpoint(end) => {
            let (detached, attached, reset) = match state.selected_shape() {
                Some(Shape::Connector { from, to, .. }) => {
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
            (ENDPOINT, Some(end), detached, attached, reset)
        }
        Context::Terminal(_) => (TERMINAL_SELECT, None, false, false, false),
        Context::Leaving(_) => (LEAVING_SELECT, None, false, false, false),
        Context::Attach(_) => (ATTACH_SELECT, None, false, false, false),
        Context::Inconsistent => (&[], None, false, false, false),
    };

    slice.iter().filter(move |block| match block.condition {
        Condition::Always => true,
        Condition::Box => is_box,
        Condition::HorizontalLine => is_horizontal_line,
        Condition::VerticalLine => is_vertical_line,
        Condition::Line => is_line,
        Condition::Movable => is_box_or_line || is_movable_connector,
        Condition::Connector => is_connector,
        Condition::EndpointDetached => is_endpoint_detached,
        Condition::EndpointAttached => is_endpoint_attached,
        Condition::EndpointReset => is_endpoint_reset,
        Condition::AttachableEndpoint => is_endpoint_detached && has_attachable_shapes,
        Condition::HasShapes => has_shapes,
        Condition::ActiveEndpointIs(target_end) => active_endpoint == Some(target_end),
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

fn selected_id(state: &DiagramState) -> Option<ShapeId> {
    let id = state.selected();
    debug_assert!(
        id.is_some(),
        "a selection block matched without a selection"
    );
    id
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EndpointMeaning {
    Move,
    AdjustOut(i16),
    Reset,
    Detach,
}

fn dispatch_endpoint_meaning(
    meaning: EndpointMeaning,
    state: &DiagramState,
    key: &KeyEvent,
) -> Reply {
    let Some(SubMode::Endpoint(end)) = state.sub_mode() else {
        return Reply::Ignored;
    };
    let Some(id) = selected_id(state) else {
        return Reply::Ignored;
    };

    match meaning {
        EndpointMeaning::Move => {
            let Some((dx, dy)) = arrow_direction(key.code) else {
                debug_assert!(false, "arrow pattern matched a non-arrow key");
                return Reply::Ignored;
            };
            Reply::Act(Action::Command(Command::MoveEndpoint {
                id,
                end,
                by: (dx, dy).into(),
                out: 0,
            }))
        }
        EndpointMeaning::AdjustOut(out) => Reply::Act(Action::Command(Command::MoveEndpoint {
            id,
            end,
            by: Delta::ZERO,
            out,
        })),
        EndpointMeaning::Reset => Reply::Act(Action::Command(Command::ResetEndpoint { id, end })),
        EndpointMeaning::Detach => Reply::Act(Action::Command(Command::DetachEndpoint { id, end })),
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
            let Some((dx, dy)) = arrow_direction(key.code) else {
                debug_assert!(false, "arrow pattern matched a non-arrow key");
                return Reply::Ignored;
            };
            let Some(id) = selected_id(state) else {
                return Reply::Ignored;
            };
            Reply::Act(Action::Command(Command::Move {
                id,
                by: (dx, dy).into(),
            }))
        }
        ContextMeaning::Resize => {
            let Some((dx, dy)) = arrow_direction(key.code) else {
                debug_assert!(false, "arrow pattern matched a non-arrow key");
                return Reply::Ignored;
            };
            let Some(id) = selected_id(state) else {
                return Reply::Ignored;
            };
            Reply::Act(Action::Command(Command::Resize {
                id,
                by: (dx, dy).into(),
            }))
        }
        ContextMeaning::RotateLine => {
            let Some(id) = selected_id(state) else {
                return Reply::Ignored;
            };
            Reply::Act(Action::Command(Command::RotateLine { id }))
        }
        ContextMeaning::InvertTerminals => {
            let Some(id) = selected_id(state) else {
                return Reply::Ignored;
            };
            Reply::Act(Action::Command(Command::InvertTerminals { id }))
        }
        ContextMeaning::Remove => {
            let Some(id) = selected_id(state) else {
                return Reply::Ignored;
            };
            Reply::Act(Action::Command(Command::Remove { id }))
        }
        ContextMeaning::Forward => {
            let Some(id) = selected_id(state) else {
                return Reply::Ignored;
            };
            Reply::Act(Action::Command(Command::Forward { id }))
        }
        ContextMeaning::Backward => {
            let Some(id) = selected_id(state) else {
                return Reply::Ignored;
            };
            Reply::Act(Action::Command(Command::Backward { id }))
        }
        ContextMeaning::ToFront => {
            let Some(id) = selected_id(state) else {
                return Reply::Ignored;
            };
            Reply::Act(Action::Command(Command::ToFront { id }))
        }
        ContextMeaning::ToBack => {
            let Some(id) = selected_id(state) else {
                return Reply::Ignored;
            };
            Reply::Act(Action::Command(Command::ToBack { id }))
        }
        ContextMeaning::SetStroke(name) => {
            let Some(id) = selected_id(state) else {
                return Reply::Ignored;
            };
            state.set_sub_mode(None);
            Reply::Act(Action::Command(Command::SetStroke {
                id,
                stroke: monospace_diagram::Stroke::from(name),
            }))
        }
        ContextMeaning::EnterLeavingSelect => {
            let Some(SubMode::Endpoint(end)) = state.sub_mode() else {
                return Reply::Ignored;
            };
            state.set_sub_mode(Some(SubMode::LeavingSelect(end)));
            Reply::Repaint
        }
        ContextMeaning::EnterTerminalSelect => {
            let Some(SubMode::Endpoint(end)) = state.sub_mode() else {
                return Reply::Ignored;
            };
            state.set_sub_mode(Some(SubMode::TerminalSelect(end)));
            Reply::Repaint
        }
        ContextMeaning::EnterAttachSelect => {
            if state.enter_attach_select() {
                Reply::Repaint
            } else {
                Reply::Ignored
            }
        }
        ContextMeaning::FocusNextAttachTarget => {
            if state.focus_next_attach_target() {
                Reply::Repaint
            } else {
                Reply::Consumed
            }
        }
        ContextMeaning::FocusPreviousAttachTarget => {
            if state.focus_previous_attach_target() {
                Reply::Repaint
            } else {
                Reply::Consumed
            }
        }
        ContextMeaning::AttachEndpoint(anchor) => {
            let Some(SubMode::AttachSelect { end, target }) = state.sub_mode() else {
                return Reply::Ignored;
            };
            let Some(id) = selected_id(state) else {
                return Reply::Ignored;
            };
            state.set_sub_mode(Some(SubMode::Endpoint(end)));
            Reply::Act(Action::Command(Command::Hang {
                id,
                end,
                target,
                anchor,
            }))
        }
        ContextMeaning::EnterSubMode(sub_mode) => {
            state.set_sub_mode(Some(sub_mode));
            Reply::Repaint
        }
        ContextMeaning::LeaveSubMode => {
            state.set_sub_mode(state.sub_mode().and_then(SubMode::parent));
            Reply::Repaint
        }
        ContextMeaning::SetEndpointTerminal(choice) => {
            let Some(SubMode::TerminalSelect(end)) = state.sub_mode() else {
                return Reply::Ignored;
            };
            let Some(id) = selected_id(state) else {
                return Reply::Ignored;
            };
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
            let Some(id) = selected_id(state) else {
                return Reply::Ignored;
            };
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
        ContextMeaning::Duplicate => {
            if state.duplicate_selected().is_some() {
                Reply::Repaint
            } else {
                Reply::Ignored
            }
        }
        ContextMeaning::Pick => Reply::Ignored,
        ContextMeaning::MoveEndpoint => {
            dispatch_endpoint_meaning(EndpointMeaning::Move, state, key)
        }
        ContextMeaning::AdjustEndpointOut(out) => {
            dispatch_endpoint_meaning(EndpointMeaning::AdjustOut(out), state, key)
        }
        ContextMeaning::ResetEndpoint => {
            dispatch_endpoint_meaning(EndpointMeaning::Reset, state, key)
        }
        ContextMeaning::DetachEndpoint => {
            dispatch_endpoint_meaning(EndpointMeaning::Detach, state, key)
        }
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
        for table in ALL_TABLES {
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

    use monospace_core::ShapeId;
    use std::num::NonZeroU16;

    const EMPTY: &str = r#"{ "next_id": 1, "shapes": [] }"#;

    const A_BOX: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                      "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
    }"#;

    const A_LINE: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "line", "id": 1, "at": { "x": 2, "y": 1 },
                      "len": 6, "orientation": "horizontal", "stroke": "light" } ]
    }"#;

    const A_VERTICAL_LINE: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "line", "id": 1, "at": { "x": 2, "y": 1 },
                      "len": 3, "orientation": "vertical", "stroke": "light" } ]
    }"#;

    const A_CONNECTOR: &str = r#"{
        "next_id": 2,
        "shapes": [ { "kind": "connector", "id": 1,
                      "from": { "at": { "kind": "point", "x": 2, "y": 1 },
                                "leaving": "right", "terminal": { "kind": "arm" } },
                      "to": { "at": { "kind": "point", "x": 10, "y": 1 },
                              "leaving": "left", "terminal": { "kind": "arm" } },
                      "stroke": "light" } ]
    }"#;

    const A_CONNECTOR_AND_A_BOX: &str = r#"{
        "next_id": 3,
        "shapes": [
            { "kind": "connector", "id": 1,
              "from": { "at": { "kind": "point", "x": 2, "y": 1 },
                        "leaving": "right", "terminal": { "kind": "arm" } },
              "to": { "at": { "kind": "point", "x": 10, "y": 1 },
                      "leaving": "left", "terminal": { "kind": "arm" } },
              "stroke": "light" },
            { "kind": "box", "id": 2, "at": { "x": 10, "y": 10 },
              "size": { "width": 4, "height": 3 }, "stroke": "light" }
        ]
    }"#;

    /// A connector attached at both ends, so a bare arrow has nothing fixed to move.
    const A_HANGING_CONNECTOR: &str = r#"{
        "next_id": 3,
        "shapes": [
            { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
              "size": { "width": 4, "height": 3 }, "stroke": "light" },
            { "kind": "box", "id": 2, "at": { "x": 12, "y": 0 },
              "size": { "width": 4, "height": 3 }, "stroke": "light" },
            { "kind": "connector", "id": 3,
              "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right",
                                "offset": { "dx": 0, "dy": 0 }, "out": 0 },
                        "terminal": { "kind": "arm" } },
              "to": { "at": { "kind": "reference", "shape": 2, "anchor": "left",
                              "offset": { "dx": 0, "dy": 0 }, "out": 0 },
                      "terminal": { "kind": "arm" } },
              "stroke": "light" }
        ]
    }"#;

    fn a_state_over(json: &str) -> DiagramState {
        let diagram =
            monospace_description::parse(json).expect("the test's description is readable");
        DiagramState::new(monospace_editing::Session::new(diagram))
    }

    #[test]
    fn context_reports_expected_context_for_all_valid_states() {
        let id = ShapeId::new(NonZeroU16::new(1).expect("1 is non-zero"));

        // Idle on empty diagram
        let empty_idle = a_state_over(EMPTY);
        assert_eq!(context(&empty_idle), Context::Idle);

        // Idle on diagram with shapes
        let box_idle = a_state_over(A_BOX);
        assert_eq!(context(&box_idle), Context::Idle);

        // Selection: box
        let mut box_selected = a_state_over(A_BOX);
        box_selected.select(Some(id));
        assert_eq!(context(&box_selected), Context::Selection);

        // Selection: horizontal line
        let mut line_selected = a_state_over(A_LINE);
        line_selected.select(Some(id));
        assert_eq!(context(&line_selected), Context::Selection);

        // Selection: vertical line
        let mut vertical_line_selected = a_state_over(A_VERTICAL_LINE);
        vertical_line_selected.select(Some(id));
        assert_eq!(context(&vertical_line_selected), Context::Selection);

        // Selection: connector
        let mut conn_selected = a_state_over(A_CONNECTOR);
        conn_selected.select(Some(id));
        assert_eq!(context(&conn_selected), Context::Selection);

        // Stroke select
        let mut stroke_state = a_state_over(A_BOX);
        stroke_state.select(Some(id));
        stroke_state.set_sub_mode(Some(SubMode::StrokeSelect));
        assert_eq!(context(&stroke_state), Context::Stroke);

        // Order mode
        let mut order_state = a_state_over(A_BOX);
        order_state.select(Some(id));
        order_state.set_sub_mode(Some(SubMode::OrderMode));
        assert_eq!(context(&order_state), Context::Order);

        // Endpoint From / To
        let mut ep_from = a_state_over(A_CONNECTOR);
        ep_from.select(Some(id));
        ep_from.set_sub_mode(Some(SubMode::Endpoint(End::From)));
        assert_eq!(context(&ep_from), Context::Endpoint(End::From));

        let mut ep_to = a_state_over(A_CONNECTOR);
        ep_to.select(Some(id));
        ep_to.set_sub_mode(Some(SubMode::Endpoint(End::To)));
        assert_eq!(context(&ep_to), Context::Endpoint(End::To));

        // Terminal From / To
        let mut term_from = a_state_over(A_CONNECTOR);
        term_from.select(Some(id));
        term_from.set_sub_mode(Some(SubMode::TerminalSelect(End::From)));
        assert_eq!(context(&term_from), Context::Terminal(End::From));

        let mut term_to = a_state_over(A_CONNECTOR);
        term_to.select(Some(id));
        term_to.set_sub_mode(Some(SubMode::TerminalSelect(End::To)));
        assert_eq!(context(&term_to), Context::Terminal(End::To));

        // Leaving From / To
        let mut leaving_from = a_state_over(A_CONNECTOR);
        leaving_from.select(Some(id));
        leaving_from.set_sub_mode(Some(SubMode::LeavingSelect(End::From)));
        assert_eq!(context(&leaving_from), Context::Leaving(End::From));

        let mut leaving_to = a_state_over(A_CONNECTOR);
        leaving_to.select(Some(id));
        leaving_to.set_sub_mode(Some(SubMode::LeavingSelect(End::To)));
        assert_eq!(context(&leaving_to), Context::Leaving(End::To));

        // Inconsistent
        let mut inconsistent = a_state_over(A_BOX);
        inconsistent.set_sub_mode(Some(SubMode::StrokeSelect));
        assert_eq!(context(&inconsistent), Context::Inconsistent);
    }

    #[test]
    fn format_command_line_in_idle_and_selection() {
        let id1 = ShapeId::new(NonZeroU16::new(1).expect("1 is non-zero"));

        // 1. Idle empty
        let empty_idle = a_state_over(EMPTY);
        assert_eq!(
            format_command_line(&empty_idle),
            "CREATE: [b] Box [l] Line [c] Connector"
        );

        // 2. Idle populated
        let populated_idle = a_state_over(A_BOX);
        assert_eq!(
            format_command_line(&populated_idle),
            "CREATE: [b] Box [l] Line [c] Connector · SELECT: [Tab] Next [Shift+Tab] Prev [Click] Pick"
        );

        // 3. Selection: Box
        let mut box_selected = a_state_over(A_BOX);
        box_selected.select(Some(id1));
        assert_eq!(
            format_command_line(&box_selected),
            "MOVE: ↑↓←→ · RESIZE: Shift+↑↓←→ · TEXT: [t] Edit · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [d] Duplicate · [Del] Delete · [Esc] Deselect"
        );

        // 4. Selection: Horizontal Line
        let mut horizontal_line_selected = a_state_over(A_LINE);
        horizontal_line_selected.select(Some(id1));
        assert_eq!(
            format_command_line(&horizontal_line_selected),
            "MOVE: ↑↓←→ · RESIZE: Shift+←→ · ROTATE: [r] · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [d] Duplicate · [Del] Delete · [Esc] Deselect"
        );

        // 5. Selection: Vertical Line
        let mut vertical_line_selected = a_state_over(A_VERTICAL_LINE);
        vertical_line_selected.select(Some(id1));
        assert_eq!(
            format_command_line(&vertical_line_selected),
            "MOVE: ↑↓←→ · RESIZE: Shift+↑↓ · ROTATE: [r] · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [d] Duplicate · [Del] Delete · [Esc] Deselect"
        );

        // 6. Selection: Connector with a fixed endpoint offers MOVE first
        let mut conn_selected = a_state_over(A_CONNECTOR);
        conn_selected.select(Some(id1));
        assert_eq!(
            format_command_line(&conn_selected),
            "MOVE: ↑↓←→ · ENDPOINTS: [1] From [2] To · INVERT: [i] · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [d] Duplicate · [Del] Delete · [Esc] Deselect"
        );

        // 7. Selection: Connector with two references has nothing fixed to move
        let id3 = ShapeId::new(NonZeroU16::new(3).expect("3 is non-zero"));
        let mut hanging_selected = a_state_over(A_HANGING_CONNECTOR);
        hanging_selected.select(Some(id3));
        assert_eq!(
            format_command_line(&hanging_selected),
            "ENDPOINTS: [1] From [2] To · INVERT: [i] · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [d] Duplicate · [Del] Delete · [Esc] Deselect"
        );
    }

    #[test]
    fn format_command_line_in_sub_modes() {
        let id1 = ShapeId::new(NonZeroU16::new(1).expect("1 is non-zero"));

        // Stroke select
        let mut stroke_state = a_state_over(A_BOX);
        stroke_state.select(Some(id1));
        stroke_state.set_sub_mode(Some(SubMode::StrokeSelect));
        assert_eq!(
            format_command_line(&stroke_state),
            "STROKE: [1] light ┌─┐ [2] double ╔═╗ [3] light-round ╭─╮ [4] heavy ┏━┓ [5] ascii +-+ · [Esc] Cancel"
        );

        // Order mode
        let mut order_state = a_state_over(A_BOX);
        order_state.select(Some(id1));
        order_state.set_sub_mode(Some(SubMode::OrderMode));
        assert_eq!(
            format_command_line(&order_state),
            "ORDER: [Home] Front [End] Back [PgUp] Forward [PgDn] Backward · [Esc] Done"
        );

        // Terminal select From / To
        let mut term_from = a_state_over(A_CONNECTOR);
        term_from.select(Some(id1));
        term_from.set_sub_mode(Some(SubMode::TerminalSelect(End::From)));
        assert_eq!(
            format_command_line(&term_from),
            "TERMINAL: [1] arm ─ [2] solid ▶ [3] outline ▷ [4] ascii > [5] diamond ◆ · [Esc] Cancel"
        );

        let mut term_to = a_state_over(A_CONNECTOR);
        term_to.select(Some(id1));
        term_to.set_sub_mode(Some(SubMode::TerminalSelect(End::To)));
        assert_eq!(
            format_command_line(&term_to),
            "TERMINAL: [1] arm ─ [2] solid ▶ [3] outline ▷ [4] ascii > [5] diamond ◆ · [Esc] Cancel"
        );

        // Leaving select From / To
        let mut leaving_from = a_state_over(A_CONNECTOR);
        leaving_from.select(Some(id1));
        leaving_from.set_sub_mode(Some(SubMode::LeavingSelect(End::From)));
        assert_eq!(
            format_command_line(&leaving_from),
            "LEAVING: [1] ↑ North [2] → East [3] ↓ South [4] ← West · [Esc] Cancel"
        );

        let mut leaving_to = a_state_over(A_CONNECTOR);
        leaving_to.select(Some(id1));
        leaving_to.set_sub_mode(Some(SubMode::LeavingSelect(End::To)));
        assert_eq!(
            format_command_line(&leaving_to),
            "LEAVING: [1] ↑ North [2] → East [3] ↓ South [4] ← West · [Esc] Cancel"
        );
    }

    #[test]
    fn format_command_line_in_endpoint_sub_mode_and_inconsistent() {
        let id1 = ShapeId::new(NonZeroU16::new(1).expect("1 is non-zero"));
        let id3 = ShapeId::new(NonZeroU16::new(3).expect("3 is non-zero"));

        // Endpoint From (detached)
        let mut ep_from = a_state_over(A_CONNECTOR);
        ep_from.select(Some(id1));
        ep_from.set_sub_mode(Some(SubMode::Endpoint(End::From)));
        assert_eq!(
            format_command_line(&ep_from),
            "MOVE: ↑↓←→ · LEAVING: [d] · TERMINAL: [t] · [2] To · [Esc] Done"
        );

        // Endpoint To (detached)
        let mut ep_to = a_state_over(A_CONNECTOR);
        ep_to.select(Some(id1));
        ep_to.set_sub_mode(Some(SubMode::Endpoint(End::To)));
        assert_eq!(
            format_command_line(&ep_to),
            "MOVE: ↑↓←→ · LEAVING: [d] · TERMINAL: [t] · [1] From · [Esc] Done"
        );

        // Endpoint From (attached, zero offset/out -> reset hidden)
        let mut ep_from_att = a_state_over(AN_ATTACHED_CONNECTOR);
        ep_from_att.select(Some(id3));
        ep_from_att.set_sub_mode(Some(SubMode::Endpoint(End::From)));
        assert_eq!(
            format_command_line(&ep_from_att),
            "OFFSET: ↑↓←→ · OUT: [+] [-] · [0] Detach · TERMINAL: [t] · [2] To · [Esc] Done"
        );

        // Endpoint To (attached, non-zero offset/out -> reset shown)
        let mut ep_to_att = a_state_over(AN_ATTACHED_CONNECTOR);
        ep_to_att.select(Some(id3));
        ep_to_att.set_sub_mode(Some(SubMode::Endpoint(End::To)));
        assert_eq!(
            format_command_line(&ep_to_att),
            "OFFSET: ↑↓←→ · OUT: [+] [-] · [r] Reset · [0] Detach · TERMINAL: [t] · [1] From · [Esc] Done"
        );

        // Inconsistent
        let mut inconsistent = a_state_over(A_BOX);
        inconsistent.set_sub_mode(Some(SubMode::StrokeSelect));
        assert_eq!(format_command_line(&inconsistent), "");
    }

    #[test]
    fn inconsistent_state_with_selected_box_and_endpoint_sub_mode() {
        let id = ShapeId::new(NonZeroU16::new(1).expect("1 is non-zero"));
        let mut state = a_state_over(A_BOX);
        state.select(Some(id));
        state.set_sub_mode(Some(SubMode::Endpoint(End::From)));

        assert_eq!(context(&state), Context::Inconsistent);
        assert_eq!(format_command_line(&state), "");

        let arrow = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(dispatch_key(&mut state, &arrow), Reply::Ignored);
    }

    #[test]
    fn inconsistent_state_with_active_sub_mode_and_no_selection() {
        for sub_mode in [
            SubMode::StrokeSelect,
            SubMode::OrderMode,
            SubMode::Endpoint(End::From),
            SubMode::Endpoint(End::To),
            SubMode::TerminalSelect(End::From),
            SubMode::LeavingSelect(End::From),
        ] {
            let mut state = a_state_over(A_BOX);
            state.set_sub_mode(Some(sub_mode));

            assert_eq!(
                context(&state),
                Context::Inconsistent,
                "expected Inconsistent for {sub_mode:?} without selection"
            );
            assert_eq!(format_command_line(&state), "");

            let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
            assert_eq!(
                dispatch_key(&mut state, &key),
                Reply::Ignored,
                "expected Ignored for {sub_mode:?} without selection"
            );
        }
    }

    #[test]
    fn inconsistent_state_with_endpoint_sub_mode_on_non_connector_returns_ignored() {
        for sub_mode in [
            SubMode::Endpoint(End::From),
            SubMode::Endpoint(End::To),
            SubMode::TerminalSelect(End::From),
            SubMode::LeavingSelect(End::From),
        ] {
            let mut state = a_state_over(A_BOX);
            state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
            state.set_sub_mode(Some(sub_mode));

            let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
            assert_eq!(
                dispatch_key(&mut state, &key),
                Reply::Ignored,
                "expected Ignored for {sub_mode:?} on a box"
            );
        }
    }

    #[test]
    fn endpoint_sub_mode_dispatch_key_switches_endpoints_and_enters_selectors() {
        let id1 = ShapeId::new(NonZeroU16::new(1).expect("1 is non-zero"));

        let mut state = a_state_over(A_CONNECTOR);
        state.select(Some(id1));
        state.set_sub_mode(Some(SubMode::Endpoint(End::From)));

        let key_1 = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
        let key_2 = KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE);
        let key_t = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
        let key_d = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);

        assert_eq!(dispatch_key(&mut state, &key_1), Reply::Ignored);
        assert_eq!(state.sub_mode(), Some(SubMode::Endpoint(End::From)));

        assert_eq!(dispatch_key(&mut state, &key_t), Reply::Repaint);
        assert_eq!(state.sub_mode(), Some(SubMode::TerminalSelect(End::From)));

        state.set_sub_mode(Some(SubMode::Endpoint(End::From)));
        assert_eq!(dispatch_key(&mut state, &key_d), Reply::Repaint);
        assert_eq!(state.sub_mode(), Some(SubMode::LeavingSelect(End::From)));

        state.set_sub_mode(Some(SubMode::Endpoint(End::From)));
        assert_eq!(dispatch_key(&mut state, &key_2), Reply::Repaint);
        assert_eq!(state.sub_mode(), Some(SubMode::Endpoint(End::To)));

        assert_eq!(dispatch_key(&mut state, &key_2), Reply::Ignored);
        assert_eq!(state.sub_mode(), Some(SubMode::Endpoint(End::To)));

        assert_eq!(dispatch_key(&mut state, &key_t), Reply::Repaint);
        assert_eq!(state.sub_mode(), Some(SubMode::TerminalSelect(End::To)));

        state.set_sub_mode(Some(SubMode::Endpoint(End::To)));
        assert_eq!(dispatch_key(&mut state, &key_d), Reply::Repaint);
        assert_eq!(state.sub_mode(), Some(SubMode::LeavingSelect(End::To)));

        state.set_sub_mode(Some(SubMode::Endpoint(End::To)));
        assert_eq!(dispatch_key(&mut state, &key_1), Reply::Repaint);
        assert_eq!(state.sub_mode(), Some(SubMode::Endpoint(End::From)));
    }

    const AN_ATTACHED_CONNECTOR: &str = r#"{
        "next_id": 4,
        "shapes": [
            { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 4 }, "stroke": "light" },
            { "kind": "box", "id": 2, "at": { "x": 20, "y": 0 }, "size": { "width": 8, "height": 4 }, "stroke": "light" },
            { "kind": "connector", "id": 3,
              "from": { "at": { "kind": "reference", "shape": 1, "anchor": "top", "offset": { "dx": 0, "dy": 0 }, "out": 0 },
                        "terminal": { "kind": "arm" } },
              "to":   { "at": { "kind": "reference", "shape": 2, "anchor": "left", "offset": { "dx": 0, "dy": 1 }, "out": 2 },
                        "terminal": { "kind": "symbol", "style": "solid" } },
              "stroke": "light" }
        ]
    }"#;

    fn all_constructible_states() -> Vec<DiagramState> {
        let id1 = ShapeId::new(NonZeroU16::new(1).expect("1 is non-zero"));
        let id3 = ShapeId::new(NonZeroU16::new(3).expect("3 is non-zero"));

        let mut states = Vec::new();

        // Idle on empty diagram
        states.push(a_state_over(EMPTY));

        // Idle on diagram with shapes
        states.push(a_state_over(A_BOX));

        // Selection: box
        let mut s = a_state_over(A_BOX);
        s.select(Some(id1));
        states.push(s);

        // Selection: horizontal line
        let mut s = a_state_over(A_LINE);
        s.select(Some(id1));
        states.push(s);

        // Selection: vertical line
        let mut s = a_state_over(A_VERTICAL_LINE);
        s.select(Some(id1));
        states.push(s);

        // Selection: connector
        let mut s = a_state_over(A_CONNECTOR);
        s.select(Some(id1));
        states.push(s);

        // Stroke select
        let mut s = a_state_over(A_BOX);
        s.select(Some(id1));
        s.set_sub_mode(Some(SubMode::StrokeSelect));
        states.push(s);

        // Order mode
        let mut s = a_state_over(A_BOX);
        s.select(Some(id1));
        s.set_sub_mode(Some(SubMode::OrderMode));
        states.push(s);

        // Endpoint From (detached)
        let mut s = a_state_over(A_CONNECTOR);
        s.select(Some(id1));
        s.set_sub_mode(Some(SubMode::Endpoint(End::From)));
        states.push(s);

        // Endpoint To (detached)
        let mut s = a_state_over(A_CONNECTOR);
        s.select(Some(id1));
        s.set_sub_mode(Some(SubMode::Endpoint(End::To)));
        states.push(s);

        // Endpoint From (attached)
        let mut s = a_state_over(AN_ATTACHED_CONNECTOR);
        s.select(Some(id3));
        s.set_sub_mode(Some(SubMode::Endpoint(End::From)));
        states.push(s);

        // Endpoint To (attached, reset true)
        let mut s = a_state_over(AN_ATTACHED_CONNECTOR);
        s.select(Some(id3));
        s.set_sub_mode(Some(SubMode::Endpoint(End::To)));
        states.push(s);

        // Terminal select From
        let mut s = a_state_over(A_CONNECTOR);
        s.select(Some(id1));
        s.set_sub_mode(Some(SubMode::TerminalSelect(End::From)));
        states.push(s);

        // Terminal select To
        let mut s = a_state_over(A_CONNECTOR);
        s.select(Some(id1));
        s.set_sub_mode(Some(SubMode::TerminalSelect(End::To)));
        states.push(s);

        // Leaving select From
        let mut s = a_state_over(A_CONNECTOR);
        s.select(Some(id1));
        s.set_sub_mode(Some(SubMode::LeavingSelect(End::From)));
        states.push(s);

        // Leaving select To
        let mut s = a_state_over(A_CONNECTOR);
        s.select(Some(id1));
        s.set_sub_mode(Some(SubMode::LeavingSelect(End::To)));
        states.push(s);

        // Attach select From
        let id2 = ShapeId::new(NonZeroU16::new(2).expect("2 is non-zero"));
        let mut s = a_state_over(A_CONNECTOR_AND_A_BOX);
        s.select(Some(id1));
        s.set_sub_mode(Some(SubMode::AttachSelect {
            end: End::From,
            target: id2,
        }));
        states.push(s);

        // Attach select To
        let mut s = a_state_over(A_CONNECTOR_AND_A_BOX);
        s.select(Some(id1));
        s.set_sub_mode(Some(SubMode::AttachSelect {
            end: End::To,
            target: id2,
        }));
        states.push(s);

        states
    }

    #[test]
    fn no_key_is_accepted_by_more_than_one_action_in_any_context() {
        for state in all_constructible_states() {
            let mut seen = std::collections::HashSet::new();
            for block in context_blocks(&state) {
                for action in block.actions {
                    for key in action.trigger.samples() {
                        assert!(
                            seen.insert(key),
                            "key {key:?} is accepted by more than one active action in context {:?}",
                            context(&state)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn all_tables_and_context_blocks_cover_each_other() {
        let states = all_constructible_states();

        // 1. Every declared table in ALL_TABLES is produced in at least one constructible context.
        for &table in ALL_TABLES {
            let produced = states.iter().any(|state| {
                let blocks: Vec<_> = context_blocks(state).collect();
                !blocks.is_empty()
                    && blocks
                        .iter()
                        .all(|b| table.as_ptr_range().contains(&std::ptr::from_ref(*b)))
            });
            assert!(
                produced,
                "table at {:?} was never produced by context_blocks in any constructible context",
                table.as_ptr()
            );
        }

        // 2. Every block produced by context_blocks in any constructible context belongs to a declared table in ALL_TABLES.
        for state in &states {
            let blocks: Vec<_> = context_blocks(state).collect();
            assert!(
                !blocks.is_empty(),
                "constructible state in context {:?} produced no context blocks",
                context(state)
            );
            let belongs_to_known_table = ALL_TABLES.iter().any(|table| {
                blocks
                    .iter()
                    .all(|b| table.as_ptr_range().contains(&std::ptr::from_ref(*b)))
            });
            assert!(
                belongs_to_known_table,
                "context_blocks produced blocks in context {:?} that do not belong to any table in ALL_TABLES",
                context(state)
            );
        }
    }

    #[test]
    fn every_offered_stroke_and_terminal_exists_in_the_catalog() {
        let catalog = monospace_glyph_sets::all();

        for block in STROKE_SELECT {
            for action in block.actions {
                let ContextMeaning::SetStroke(name) = &action.meaning else {
                    continue;
                };
                let key = monospace_core::GlyphKey {
                    top: None,
                    right: None,
                    bottom: None,
                    left: Some(monospace_core::Stroke::from(*name)),
                };
                assert!(
                    catalog.glyph(&key).is_some(),
                    "stroke {name} has no glyph in the catalog"
                );
            }
        }

        let styles = catalog.symbol_styles();

        for block in TERMINAL_SELECT {
            for action in block.actions {
                let ContextMeaning::SetEndpointTerminal(choice) = &action.meaning else {
                    continue;
                };
                match (*choice).to_terminal() {
                    monospace_core::Terminal::Arm => {}
                    monospace_core::Terminal::Glyph { glyph } => {
                        let exists = styles.iter().any(|style| {
                            monospace_core::Symbol::ALL.iter().any(|&symbol| {
                                let key = monospace_core::SymbolCell {
                                    symbol,
                                    style: (*style).clone(),
                                };
                                catalog.symbol(&key) == Some(&glyph)
                            })
                        });
                        assert!(
                            exists,
                            "glyph {} is missing from the catalog",
                            glyph.as_str()
                        );
                    }
                    monospace_core::Terminal::Symbol { style } => {
                        assert!(
                            styles.iter().any(|known| known.0 == style.0),
                            "terminal style {} is missing from the catalog",
                            style.0
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn format_command_line_in_attach_select() {
        let id1 = ShapeId::new(NonZeroU16::new(1).expect("1 is non-zero"));
        let target = ShapeId::new(NonZeroU16::new(2).expect("2 is non-zero"));
        let mut state = a_state_over(A_CONNECTOR_AND_A_BOX);
        state.select(Some(id1));

        state.set_sub_mode(Some(SubMode::AttachSelect {
            end: End::From,
            target,
        }));
        assert_eq!(
            crate::status::status_line(&state),
            "ATTACH for 1. From of Connector #1 · Detached at (2, 1), leaving right · Target: Box #2 · 1 of 1"
        );
        assert_eq!(
            format_command_line(&state),
            "TARGET: [Tab] Next [Shift+Tab] Prev · ANCHOR: [1] Top [2] Right [3] Bottom [4] Left · [Esc] Cancel"
        );

        state.set_sub_mode(Some(SubMode::AttachSelect {
            end: End::To,
            target,
        }));
        assert_eq!(
            crate::status::status_line(&state),
            "ATTACH for 2. To of Connector #1 · Detached at (10, 1), leaving left · Target: Box #2 · 1 of 1"
        );
        assert_eq!(
            format_command_line(&state),
            "TARGET: [Tab] Next [Shift+Tab] Prev · ANCHOR: [1] Top [2] Right [3] Bottom [4] Left · [Esc] Cancel"
        );
    }

    #[test]
    fn attach_is_offered_only_for_a_detached_endpoint_with_a_candidate() {
        let id1 = ShapeId::new(NonZeroU16::new(1).expect("1 is non-zero"));

        // 1. Present with detached connector + box
        let mut state_detached_with_box = a_state_over(A_CONNECTOR_AND_A_BOX);
        state_detached_with_box.select(Some(id1));
        state_detached_with_box.set_sub_mode(Some(SubMode::Endpoint(End::From)));
        let cmd = format_command_line(&state_detached_with_box);
        assert!(cmd.contains("ATTACH: [a]"), "expected ATTACH in {cmd}");

        // 2. Absent with attached connector + box
        let id3 = ShapeId::new(NonZeroU16::new(3).expect("3 is non-zero"));
        let mut state_attached_with_box = a_state_over(AN_ATTACHED_CONNECTOR);
        state_attached_with_box.select(Some(id3));
        state_attached_with_box.set_sub_mode(Some(SubMode::Endpoint(End::From)));
        let cmd = format_command_line(&state_attached_with_box);
        assert!(!cmd.contains("ATTACH"), "expected no ATTACH in {cmd}");

        // 3. Absent with detached connector without box
        let mut state_detached_without_box = a_state_over(A_CONNECTOR);
        state_detached_without_box.select(Some(id1));
        state_detached_without_box.set_sub_mode(Some(SubMode::Endpoint(End::From)));
        let cmd = format_command_line(&state_detached_without_box);
        assert!(!cmd.contains("ATTACH"), "expected no ATTACH in {cmd}");
    }

    #[test]
    fn context_reports_attach_select() {
        let id1 = ShapeId::new(NonZeroU16::new(1).expect("1 is non-zero"));
        let target = ShapeId::new(NonZeroU16::new(2).expect("2 is non-zero"));

        let mut s_from = a_state_over(A_CONNECTOR_AND_A_BOX);
        s_from.select(Some(id1));
        s_from.set_sub_mode(Some(SubMode::AttachSelect {
            end: End::From,
            target,
        }));
        assert_eq!(context(&s_from), Context::Attach(End::From));

        let mut s_to = a_state_over(A_CONNECTOR_AND_A_BOX);
        s_to.select(Some(id1));
        s_to.set_sub_mode(Some(SubMode::AttachSelect {
            end: End::To,
            target,
        }));
        assert_eq!(context(&s_to), Context::Attach(End::To));

        // Inconsistent if selection is a box with AttachSelect
        let mut inconsistent = a_state_over(A_CONNECTOR_AND_A_BOX);
        inconsistent.select(Some(target));
        inconsistent.set_sub_mode(Some(SubMode::AttachSelect {
            end: End::From,
            target,
        }));
        assert_eq!(context(&inconsistent), Context::Inconsistent);
    }

    #[test]
    fn attach_select_dispatch_focuses_and_attaches() {
        let text = r#"{
            "next_id": 4,
            "shapes": [
                { "kind": "connector", "id": 1,
                  "from": { "at": { "kind": "point", "x": 2, "y": 1 },
                            "leaving": "right", "terminal": { "kind": "arm" } },
                  "to": { "at": { "kind": "point", "x": 10, "y": 1 },
                          "leaving": "left", "terminal": { "kind": "arm" } },
                  "stroke": "light" },
                { "kind": "box", "id": 2, "at": { "x": 10, "y": 10 },
                  "size": { "width": 4, "height": 3 }, "stroke": "light" },
                { "kind": "line", "id": 3, "at": { "x": 20, "y": 10 },
                  "len": 5, "orientation": "horizontal", "stroke": "light" }
            ]
        }"#;
        let mut state = a_state_over(text);
        let id1 = ShapeId::new(NonZeroU16::new(1).unwrap());
        let id2 = ShapeId::new(NonZeroU16::new(2).unwrap());
        let id3 = ShapeId::new(NonZeroU16::new(3).unwrap());

        state.select(Some(id1));
        state.set_sub_mode(Some(SubMode::Endpoint(End::From)));

        // Enter attach select with 'a'
        let key_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(dispatch_key(&mut state, &key_a), Reply::Repaint);
        assert_eq!(
            state.sub_mode(),
            Some(SubMode::AttachSelect {
                end: End::From,
                target: id2
            })
        );

        // Tab focuses next candidate (id3)
        let key_tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(dispatch_key(&mut state, &key_tab), Reply::Repaint);
        assert_eq!(
            state.sub_mode(),
            Some(SubMode::AttachSelect {
                end: End::From,
                target: id3
            })
        );

        // BackTab focuses previous candidate (id2)
        let key_back_tab = KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
        assert_eq!(dispatch_key(&mut state, &key_back_tab), Reply::Repaint);
        assert_eq!(
            state.sub_mode(),
            Some(SubMode::AttachSelect {
                end: End::From,
                target: id2
            })
        );

        // Key '1' attaches to Top
        let key_1 = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
        let reply = dispatch_key(&mut state, &key_1);
        assert_eq!(
            reply,
            Reply::Act(Action::Command(Command::Hang {
                id: id1,
                end: End::From,
                target: id2,
                anchor: Anchor::Top,
            }))
        );
        // SubMode is returned to Endpoint
        assert_eq!(state.sub_mode(), Some(SubMode::Endpoint(End::From)));
    }

    #[test]
    fn attach_select_esc_returns_to_endpoint_without_a_step() {
        let mut state = a_state_over(A_CONNECTOR_AND_A_BOX);
        let id1 = ShapeId::new(NonZeroU16::new(1).unwrap());
        let id2 = ShapeId::new(NonZeroU16::new(2).unwrap());

        state.select(Some(id1));
        state.set_sub_mode(Some(SubMode::AttachSelect {
            end: End::From,
            target: id2,
        }));

        let can_undo = state.session().can_undo();
        let key_esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(dispatch_key(&mut state, &key_esc), Reply::Repaint);
        assert_eq!(state.sub_mode(), Some(SubMode::Endpoint(End::From)));
        assert_eq!(state.session().can_undo(), can_undo);
    }
}
