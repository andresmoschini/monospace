//! The single source of truth for contextual keybindings and bar labels.
//!
//! **One definition for offer and acceptance** (Representation Rule, #288): the contextual
//! status bar displays the actions offered by the current context, and the region dispatches
//! exactly the keys the table offers.

use monospace_diagram::Shape;

use crate::diagram_state::DiagramState;

/// Condition required for a contextual block to be active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Condition {
    /// Always active in this context.
    Always,
    /// Active only if the selected figure has a width and height to resize.
    Size,
}

/// One contextual action offered by a block.
pub(crate) struct ContextAction {
    pub(crate) display: &'static str,
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
            display: "↑↓←→"
        }],
    },
    ContextBlock {
        category: Some("RESIZE"),
        condition: Condition::Size,
        actions: &[ContextAction {
            display: "Shift+↑↓←→",
        }],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            display: "[Del] Delete",
        }],
    },
    ContextBlock {
        category: None,
        condition: Condition::Always,
        actions: &[ContextAction {
            display: "[Esc] Deselect",
        }],
    },
];

/// The contextual blocks active for `state`.
pub(crate) fn context_blocks(
    state: &DiagramState,
) -> impl Iterator<Item = &'static ContextBlock> + use<> {
    let has_size = state.selected_shape().is_some_and(Shape::has_size);
    let selected = state.selected_shape().is_some();

    SELECTION.iter().filter(move |block| {
        if !selected {
            return false;
        }
        match block.condition {
            Condition::Always => true,
            Condition::Size => has_size,
        }
    })
}

/// Formats the contextual command row (row 1) from the diagram state.
pub(crate) fn format_command_line(state: &DiagramState) -> String {
    if state.selected_shape().is_none() {
        return "SELECT: [Click] Pick".to_owned();
    }

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
        "SELECT: [Click] Pick".to_owned()
    } else {
        result
    }
}
