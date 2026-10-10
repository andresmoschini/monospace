//! One entry in a session's history: the diagram as it was, and the command that replaced it.
//! See the _Vocabulary_ row for `Step` and §5 of
//! [`docs/editing-model.md`](../../../docs/editing-model.md).

use monospace_diagram::Diagram;

use crate::Command;

/// What a session pushes before it carries out a command.
///
/// **Both halves are needed and a diagram alone cannot stand in for the other.** A diagram records
/// what is true and never what was done to it — a figure dragged one cell and the same figure
/// teleported across the diagram differ as gestures and are identical as states — so whatever
/// eventually tells those apart would have nothing to read in a pair of diagrams. That is what
/// the command beside it is for, and why it is stored rather than derived.
///
/// **The diagram it holds is the one the session held before the command, and nothing is
/// computed from the command to get it.** Inverting a command is not available and would not be
/// right: `Diagram::remove` freezes every reference naming the figure it takes and states that
/// nothing re-hangs them, so an inverse of that command cannot be derived — it could only be
/// reconstructed from what the caller captured beforehand, and a command whose author forgot is
/// wrong in a way nothing reports. So the state is kept and the command is kept beside it.
///
/// **It is private, and not because either half is secret.** Nothing hands a step out today, and a
/// caller able to read one would be able to hold a diagram and change it — which is a way around
/// the session, and the one rule this crate exists to make impossible.
pub(crate) struct Step {
    /// The diagram as it was when the command was carried out. This is the state undo restores
    /// and the state redo restores, whole and without being computed from anything.
    pub(crate) diagram: Diagram,

    /// The command that was carried out.
    ///
    /// **Written and never read, which is the whole of what the decision D3 asked for.** Nothing
    /// in the crate asks a step what it was made of, and the layer does not pretend otherwise: the
    /// place this answer would be read is §3 of the model, and the session already stores what it
    /// would be read from. `expect` rather than `allow` so that the day something does read it,
    /// the gate says so instead of the attribute quietly outliving its reason.
    #[expect(
        dead_code,
        reason = "the model requires it beside the diagram; nothing reads it yet"
    )]
    pub(crate) command: Command,
}

impl Step {
    /// The step a command leaves behind when it is carried out over `diagram`.
    pub(crate) fn new(command: Command, diagram: Diagram) -> Self {
        Self { diagram, command }
    }
}
