//! The single source of truth for contextual keybindings and bar labels.
//!
//! **One definition for offer and acceptance** (Representation Rule, #288): the contextual
//! status bar displays the actions offered by the current context, and the region dispatches
//! exactly the keys the table offers.
//!
//! **Two tables, one mechanism**: `SELECTION` is the context of a selected figure and
//! `STROKE_SELECT` the stroke sub-mode; `context_blocks` chooses between them from the state, and
//! a block's condition says whether it is active. `format_command_line` renders what
//! `dispatch_key` answers, so a key cannot be offered without being accepted; `PICK` is the one
//! label the row carries with no key behind it.

use monospace_core::Orientation;
use monospace_diagram::Shape;
use ratatui_crossterm::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::diagram_state::{DiagramState, SubMode};
use crate::vocabulary::{Action, Reply};

/// What the command row offers while no figure is selected.
const PICK: &str = "SELECT: [Click] Pick";

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
    /// A single bare character key.
    Char(char),
}

impl KeyPattern {
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
            Self::Char(c) => {
                event.modifiers == KeyModifiers::NONE && event.code == KeyCode::Char(c)
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
            Self::Char(c) => vec![(KeyCode::Char(c), KeyModifiers::NONE)],
        }
    }
}

/// What a contextual keypress means.
pub(crate) enum ContextMeaning {
    /// Dispatches a direct application action.
    Act(Action),
    /// Dispatches a directional action taking `(dx, dy)`.
    Directional(fn(i16, i16) -> Action),
    /// Sets the stroke of the selected figure to the catalog name.
    SetStroke(&'static str),
    /// Enters an inline sub-mode.
    EnterSubMode(SubMode),
    /// Leaves an inline sub-mode back to normal selection.
    LeaveSubMode,
    /// Deselects the active figure.
    ///
    /// **Always owes a frame**: the action is offered only while a figure is selected, so the
    /// table reaches it with something to drop.
    Deselect,
}

/// One contextual action offered by a block.
pub(crate) struct ContextAction {
    pub(crate) pattern: KeyPattern,
    pub(crate) display: &'static str,
    pub(crate) meaning: ContextMeaning,
}

/// A group of related contextual actions sharing a category prefix and condition.
pub(crate) struct ContextBlock {
    pub(crate) category: Option<&'static str>,
    pub(crate) condition: Condition,
    pub(crate) actions: &'static [ContextAction],
}

static SELECTION: &[ContextBlock] = &[
    ContextBlock {
        category: Some("MOVE"),
        condition: Condition::Always,
        actions: &[ContextAction {
            pattern: KeyPattern::Arrows,
            display: "↑↓←→",
            meaning: ContextMeaning::Directional(Action::Move),
        }],
    },
    ContextBlock {
        category: Some("RESIZE"),
        condition: Condition::Box,
        actions: &[ContextAction {
            pattern: KeyPattern::ShiftArrows,
            display: "Shift+↑↓←→",
            meaning: ContextMeaning::Directional(Action::Resize),
        }],
    },
    ContextBlock {
        category: Some("RESIZE"),
        condition: Condition::HorizontalLine,
        actions: &[ContextAction {
            pattern: KeyPattern::ShiftHorizontalArrows,
            display: "Shift+←→",
            meaning: ContextMeaning::Directional(Action::Resize),
        }],
    },
    ContextBlock {
        category: Some("RESIZE"),
        condition: Condition::VerticalLine,
        actions: &[ContextAction {
            pattern: KeyPattern::ShiftVerticalArrows,
            display: "Shift+↑↓",
            meaning: ContextMeaning::Directional(Action::Resize),
        }],
    },
    ContextBlock {
        category: Some("TEXT"),
        condition: Condition::Box,
        actions: &[ContextAction {
            pattern: KeyPattern::Char('t'),
            display: "[t] Edit",
            meaning: ContextMeaning::Act(Action::EditText),
        }],
    },
    ContextBlock {
        category: Some("STROKE"),
        condition: Condition::Always,
        actions: &[ContextAction {
            pattern: KeyPattern::Char('s'),
            display: "[s]",
            meaning: ContextMeaning::EnterSubMode(SubMode::StrokeSelect),
        }],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            pattern: KeyPattern::Code(KeyCode::Delete),
            display: "[Del] Delete",
            meaning: ContextMeaning::Act(Action::Delete),
        }],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            pattern: KeyPattern::Code(KeyCode::Esc),
            display: "[Esc] Deselect",
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
                pattern: KeyPattern::Char('1'),
                display: "[1] light ┌─┐",
                meaning: ContextMeaning::SetStroke("light"),
            },
            ContextAction {
                pattern: KeyPattern::Char('2'),
                display: "[2] double ╔═╗",
                meaning: ContextMeaning::SetStroke("double"),
            },
            ContextAction {
                pattern: KeyPattern::Char('3'),
                display: "[3] light-round ╭─╮",
                meaning: ContextMeaning::SetStroke("light-round"),
            },
            ContextAction {
                pattern: KeyPattern::Char('4'),
                display: "[4] heavy ┏━┓",
                meaning: ContextMeaning::SetStroke("heavy"),
            },
            ContextAction {
                pattern: KeyPattern::Char('5'),
                display: "[5] ascii +-+",
                meaning: ContextMeaning::SetStroke("ascii"),
            },
        ],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            pattern: KeyPattern::Code(KeyCode::Esc),
            display: "[Esc] Cancel",
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

    let slice: &'static [ContextBlock] = match sub_mode {
        Some(SubMode::StrokeSelect) if selected => STROKE_SELECT,
        _ if selected => SELECTION,
        _ => &[],
    };

    slice.iter().filter(move |block| match block.condition {
        Condition::Always => true,
        Condition::Box => is_box,
        Condition::HorizontalLine => is_horizontal_line,
        Condition::VerticalLine => is_vertical_line,
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
            result.push_str(action.display);
        }
    }
    if result.is_empty() {
        PICK.to_owned()
    } else {
        result
    }
}

/// Dispatches a keyboard event in the diagram region using the contextual table.
pub(crate) fn dispatch_key(state: &mut DiagramState, key: &KeyEvent) -> Reply {
    if key.kind != KeyEventKind::Press {
        return Reply::Ignored;
    }

    for block in context_blocks(state) {
        for action in block.actions {
            if action.pattern.matches(key) {
                return match action.meaning {
                    ContextMeaning::Act(ref act) => Reply::Act(act.clone()),
                    ContextMeaning::Directional(f) => {
                        let (dx, dy) =
                            arrow_direction(key.code).expect("arrow pattern matched arrow key");
                        Reply::Act(f(dx, dy))
                    }
                    ContextMeaning::SetStroke(name) => {
                        state.set_sub_mode(None);
                        Reply::Act(Action::SetStroke(monospace_diagram::Stroke::from(name)))
                    }
                    ContextMeaning::EnterSubMode(sub_mode) => {
                        state.set_sub_mode(Some(sub_mode));
                        Reply::Repaint
                    }
                    ContextMeaning::LeaveSubMode => {
                        state.set_sub_mode(None);
                        Reply::Repaint
                    }
                    ContextMeaning::Deselect => {
                        state.select(None);
                        Reply::Repaint
                    }
                };
            }
        }
    }

    Reply::Ignored
}
