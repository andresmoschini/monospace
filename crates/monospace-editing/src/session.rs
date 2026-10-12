//! The session: a diagram being edited, the steps behind and ahead of it, and how far along them
//! it stands. See the _Vocabulary_ rows for `Session` and for `Position`, and §4 and §6 of
//! [`docs/editing-model.md`](../../../docs/editing-model.md).

use monospace_diagram::Diagram;
use rpds::Vector;

use crate::{Command, Step};

/// A diagram being edited, the steps behind and ahead of it, and a position among them.
///
/// **The diagram is a field of its own and nothing hands it out mutably.** No method here returns
/// `&mut Diagram` and none returns a `Diagram` either, so a caller has no way to change what a
/// session owns except by performing a command against it, and no second way to say what
/// [`diagram`](Session::diagram) already says. The claim is about the public surface, and the field
/// below is private and written once: a command changes the diagram by way of the steps, never by
/// way of the field.
///
/// **The field holds the diagram the session was created over, and the steps hold every state after
/// it.** A step is the diagram a command *left*, so the state before the first command is the one
/// state no step holds, and it is the field — which is why nothing writes the field again, and why
/// the diagram in hand is either that field or a step and never anything else. One diagram per step
/// and one to start with, which is the model's "memory proportional to the diagram times the number
/// of steps", and each of them held once.
///
/// **Giving a step back moves the position and changes nothing else.** [`undo`](Session::undo) and
/// [`redo`](Session::redo) are one line each for that reason, and neither removes a step or
/// rebuilds a diagram: the state that comes back is the state that was there, kept since the command
/// that made it. Nothing is inverted, and every step stays where it is while the history walks over
/// them, which is what makes the walk go both ways.
///
/// **The history is a line, and `position` is where in it the session stands.** It is how many
/// steps are behind rather than an index into a pair of stacks, so it cannot disagree with either
/// of them: there is something behind exactly when `position` is above zero, and something ahead
/// exactly when it is below `steps.len()`. A command carried out from behind the end discards what
/// was ahead, which is the only end a line can discard from without changing what giving a step
/// back means.
///
/// # The absence, shown rather than described
///
/// The rule that no method here hands out a mutable diagram is held by nothing in this file, so it
/// is worth one documentation example that says it out loud: everything up to the last line compiles, and the
/// last line is a method this crate does not have.
///
/// ```compile_fail
/// use monospace_core::{Buffer, Pos, Size};
/// use monospace_diagram::Diagram;
/// use monospace_editing::Session;
///
/// let mut session = Session::new(Diagram::new());
/// session.diagram().draw(&mut Buffer::new(Pos { x: 0, y: 0 }, Size { width: 1, height: 1 }));
///
/// session.diagram_mut();
/// ```
pub struct Session {
    /// The diagram this session was created over, and the one it hands out until a command has been
    /// carried out.
    ///
    /// **Written once, in [`new`](Session::new), and never again** — a stronger statement than the
    /// field being private, and it is what lets what the session hands out be read off the position
    /// alone: this field is the whole of the answer at the oldest position, and one step is the
    /// whole of it everywhere else. Private rather than `pub(crate)` because there is no longer
    /// anything another module needs to do to it.
    initial_diagram: Diagram,

    /// The steps behind `position` and the steps ahead of it, in the order they were made. **Step
    /// `j` is the diagram as it was left by command `j + 1`**, so the step behind the position is
    /// the state the session hands out and the step ahead of it is what a command starting from
    /// here would put behind. The state before the first command is not here, and `position == 0`
    /// is what says so.
    ///
    /// Held in the same persistent sequence the diagram holds its shapes in, and it is a
    /// [`Vector`] rather than a `Vec` for the reason that costs something: a session holds one
    /// history and hands it out step by step, so nothing is shared between two copies of it, and a
    /// `Vec` would do the same work with a shorter reach. What it does buy is that dropping the
    /// steps ahead of the position does not move the steps behind it — see
    /// [`perform`](Session::perform), which is where that is paid for.
    steps: Vector<Step>,

    /// How many of `steps` are behind the state the session hands out.
    position: usize,
}

impl Session {
    /// A session over `diagram`, holding it and no history behind it.
    ///
    /// The diagram is taken by value, so the caller has handed it over rather than lent it, and a
    /// copy the caller kept is a diagram of its own that this session never sees. **Where it came
    /// from is not asked and not refused**: a diagram is a value and any of them can be opened,
    /// which §8 of the model lists as deliberately unresolved rather than as missing.
    #[must_use]
    pub fn new(initial_diagram: Diagram) -> Session {
        Self {
            initial_diagram,
            steps: Vector::new(),
            position: 0,
        }
    }

    /// The diagram this session holds, borrowed.
    ///
    /// **The step behind the position, and the diagram this session was created over when nothing is
    /// behind it** — read rather than rebuilt, and whole. Nothing is copied and nothing can be
    /// changed through it, which is what makes it the answer for a caller that repaints on every
    /// event. A caller that wants to keep one copies it — `Diagram` is `Clone` — and the copy is a
    /// diagram like any other: changeable, free of the session and unconnected to it.
    #[must_use]
    pub fn diagram(&self) -> &Diagram {
        let behind = self.position.checked_sub(1).and_then(|i| self.steps.get(i));
        behind.map_or(&self.initial_diagram, |step| &step.diagram)
    }

    /// Whether there is a step behind the position, and so whether [`undo`](Session::undo) would
    /// change anything.
    ///
    /// **Answerable without carrying it out**, which is what lets a caller grey out a control
    /// rather than guess — the same answer the diagram gives a reference to a figure that is not
    /// there.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.position > 0
    }

    /// Whether there is a step ahead of the position, and so whether [`redo`](Session::redo)
    /// would change anything.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.position < self.steps.len()
    }

    /// Gives the step behind the position back: the diagram it holds is what this session holds
    /// again.
    ///
    /// **At the oldest position there is nothing behind, and nothing is given back.** No error, no
    /// report and no panic, which is the answer the diagram gives a figure it does not hold and
    /// the reason `can_undo` is asked rather than a result waited for.
    pub fn undo(&mut self) {
        if self.can_undo() {
            self.position -= 1;
        }
    }

    /// Carries the step ahead of the position out again: the diagram it holds is what this
    /// session holds again.
    ///
    /// **At the newest position there is nothing ahead, and nothing is carried out.** The same
    /// silence as [`undo`](Session::undo) in the other direction, and asked about by
    /// [`can_redo`](Session::can_redo) rather than waited for.
    pub fn redo(&mut self) {
        if self.can_redo() {
            self.position += 1;
        }
    }

    /// Carries out `command` against the diagram this session holds: records what the command
    /// leaves behind as a step, and makes that what this session holds.
    ///
    /// **The step is taken after the command runs, and that order is what fixes what the field
    /// holds.** A step is the diagram a command *left*, so the state the first command found is the
    /// one state no step holds, and it is the field — which is why nothing writes the field again,
    /// and why the diagram in hand is either the field or a step and never a third thing. **The
    /// other order would work too**, and it would make the field hold the newest state rather than
    /// the first: a field whose contents change with every command, and a newest state kept outside
    /// the history, which is what makes the answer at the end of the history a case of its own
    /// instead of a step like any other.
    ///
    /// **What the command becomes is carried out here and nowhere else.** The mapping from a
    /// meaning to one of the diagram's five operations is the one in [`Command`], and the session is
    /// what carries it out because only what was asked knows whether what it was asked to do was one
    /// gesture or several: a figure dragged across a screen is one thing to the person dragging it
    /// and many displacements to the diagram. Nothing reads that answer yet — §8 of the model says
    /// what is left to write — and the field it will be read from is the step's command, which is
    /// recorded beside the diagram from this first version.
    ///
    /// **Nothing is inverted, and the steps are what make that unnecessary.** `Diagram::remove`
    /// freezes every reference naming the figure it takes and states that nothing re-hangs them, so
    /// an inverse of that command cannot be derived — it could only be reconstructed from what the
    /// caller happened to capture beforehand. Every state the history passes through is kept instead,
    /// and giving a change back is one of them becoming the diagram again.
    ///
    /// **What was ahead of the position is discarded**, which is what makes the history a line
    /// rather than a tree. Branching earns its keep when a state somebody discarded is worth
    /// returning to, and nothing here says one is: a caller that moves on from a state it gave back
    /// has said the state was not worth keeping. What would reopen it is the first caller that
    /// wants one back, and §6 of the model says so.
    ///
    /// **The step is recorded whatever the command did**, including when it named a figure this
    /// diagram does not hold and changed nothing. The session does not know whether the caller meant
    /// to do something, and a history that skipped a step the caller took is worse than one longer
    /// than it needed to be.
    pub fn perform(&mut self, command: Command) {
        let made = command.apply_to(self.diagram());
        // **Dropping the steps ahead is a rewrite rather than one call**, because `rpds::Vector`
        // offers `drop_last_mut` and no way to say "keep the first n". Each dropped step is one
        // step nobody asked to keep, so the walk is over the same elements a truncation would have
        // released, and it stops at the position — which is what makes the history a line rather
        // than a tree.
        while self.steps.len() > self.position {
            self.steps.drop_last_mut();
        }
        self.steps.push_back_mut(Step::new(command, made));
        self.position += 1;
    }
}
