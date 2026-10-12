//! The session: a diagram being edited, the steps behind and ahead of it, and how far along them
//! it stands. See the _Vocabulary_ rows for `Session` and for `Position`, and §4 and §6 of
//! [`docs/editing-model.md`](../../../docs/editing-model.md).

use monospace_diagram::Diagram;
use rpds::Vector;

use crate::{Command, Step, utilities::kept_first};

/// A diagram being edited, the steps behind and ahead of it, and a position among them.
///
/// **Nothing here changes a session, and the type system says so rather than a rule this file
/// keeps.** No method takes `&mut self`: every one of them is asked over `&self` and hands a new
/// [`Session`] back, so a caller that forgets to keep what it was given changes nothing and a
/// caller that keeps it still cannot have changed this one. **The guarantee §4 of the model makes —
/// the diagram a session owns is changed through the session and nowhere else — is what the absence
/// of a `&mut self` anywhere in this surface means**, and it is a stronger answer than the one a
/// private field was: nothing in this crate can break it, because there is no borrow that would
/// let it. The field below is private and written once on top of that: a command changes the
/// diagram by way of the steps, never by way of the field.
///
/// **What a session is made of is shareable, and the sharing is what pays for the copies.** Two
/// sessions standing at different positions in the same history hold the same `Vector<Step>` nodes
/// and the same `Diagram`s inside them, and a new one costs a new position and at most a new
/// leaf. Nothing in the layer needs two sessions at once today — the demonstration walks one — but
/// this is the property a caller gets for free the first time one does, and it is why the steps
/// are a persistent sequence and not a `Vec`.
///
/// **The field holds the diagram the session was created over, and the steps hold every state after
/// it.** A step is the diagram a command *left*, so the state before the first command is the one
/// state no step holds, and it is the field — which is why nothing writes the field again, and why
/// the diagram in hand is either that field or a step and never anything else. One diagram per step
/// and one to start with, which is the model's "memory proportional to the diagram times the number
/// of steps", and each of them held once.
///
/// **Giving a step back moves the position and changes nothing else.** [`undo`](Session::undo) and
/// [`redo`](Session::redo) are one expression each for that reason, and neither removes a step or
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
/// The rule that nothing here changes a session is held by nothing in this file, so it is worth one
/// documentation example that says it out loud: everything up to the last line compiles, and the
/// last line is a method this crate does not have.
///
/// ```compile_fail
/// use monospace_core::{Buffer, Pos, Size};
/// use monospace_diagram::Diagram;
/// use monospace_editing::Session;
///
/// let session = Session::new(Diagram::new());
/// session.diagram().draw(&mut Buffer::new(Pos { x: 0, y: 0 }, Size { width: 1, height: 1 }));
///
/// session.diagram_mut();
/// ```
#[derive(Clone)]
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
    /// Held in the same persistent sequence the diagram holds its shapes in, so two sessions
    /// standing at different positions in one history are sharing it rather than copying it — see
    /// [`perform`](Session::perform), which is the one place it is written to.
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
    pub fn new(initial_diagram: Diagram) -> Self {
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

    /// Gives the step behind the position back: the diagram it holds is what the session this
    /// returns holds again.
    ///
    /// **Nothing here is changed, and the same is true of [`redo`](Session::redo) and
    /// [`perform`](Session::perform).** What comes back is a session standing one place earlier over
    /// the same steps, which is a new position rather than a new history: every step is the node
    /// this one was holding, and no diagram is copied. **At the oldest position there is nothing
    /// behind, and what comes back is this session unchanged** — no error, no report and no panic,
    /// which is the answer the diagram gives a figure it does not hold and the reason `can_undo` is
    /// asked rather than a result waited for.
    #[must_use]
    pub fn undo(&self) -> Self {
        if !self.can_undo() {
            return self.clone();
        }

        Self {
            initial_diagram: self.initial_diagram.clone(),
            steps: self.steps.clone(),
            position: self.position - 1,
        }
    }

    /// Carries the step ahead of the position out again: the diagram it holds is what the session
    /// this returns holds again.
    ///
    /// **This session is not changed**, for the reason [`undo`](Session::undo) gives. **At the
    /// newest position there is nothing ahead, and what comes back is this session unchanged** — the
    /// same silence in the other direction, and asked about by
    /// [`can_redo`](Session::can_redo) rather than waited for.
    #[must_use]
    pub fn redo(&self) -> Self {
        if !self.can_redo() {
            return self.clone();
        }

        Self {
            initial_diagram: self.initial_diagram.clone(),
            steps: self.steps.clone(),
            position: self.position + 1,
        }
    }

    /// Carries out `command` against the diagram this session holds: records what the command
    /// leaves behind as a step, and hands back a session standing over that.
    ///
    /// **This session is not changed, and discarding what comes back changes nothing.** That is
    /// what keeping a session out of reach of a `&mut` buys, and it is why this is the one method
    /// here whose result has to be kept rather than merely read.
    ///
    /// **The step is the diagram the command left, and that order is what fixes what the field
    /// holds.** The state the first command found is the one state no step holds, and it is the
    /// field — which is why nothing writes the field again, and why the diagram in hand is either
    /// that field or a step and never a third thing. **The other order would work too**, and it
    /// would make the field hold the newest state rather than the first: a field whose contents
    /// change with every command, and a newest state kept outside the history, which is what makes
    /// the answer at the end of the history a case of its own instead of a step like any other.
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
    #[must_use]
    pub fn perform(&self, command: Command) -> Self {
        let made = command.apply_to(self.diagram());
        let mut steps = kept_first(&self.steps, self.position);
        steps.push_back_mut(Step::new(command, made));

        Self {
            initial_diagram: self.initial_diagram.clone(),
            steps,
            position: self.position + 1,
        }
    }
}
