//! The session: a diagram being edited, the steps behind and ahead of it, and how far along them
//! it stands. See the _Vocabulary_ rows for `Session` and for `Position`, and §4 and §6 of
//! [`docs/editing-model.md`](../../../docs/editing-model.md).

use monospace_diagram::Diagram;

use crate::{Command, Step};

/// A diagram being edited, the steps behind and ahead of it, and a position among them.
///
/// **The diagram is a field of its own and nothing hands it out mutably.** No method here returns
/// `&mut Diagram` and none returns a `Diagram` either, so a caller has no way to change what a
/// session owns except by performing a command against it, and no second way to say what
/// [`diagram`](Session::diagram) already says. The claim is about the public surface, and the
/// field below is `pub(crate)` for one reason and one: it is the way in from
/// [`Command::perform`](crate::Command::perform), which is where a command changes a diagram on
/// purpose.
///
/// **The field holds the newest diagram there is, and the position says which diagram is in
/// hand.** A step is pushed *before* a command is carried out, so what the steps hold is every
/// state that is not the newest one, and the newest one has nowhere else to be: it is what
/// [`diagram`](Session::diagram) answers with at the end of the history, and it is the only state
/// no step holds. The two are the same number of diagrams either way — the model's "memory
/// proportional to the diagram times the number of steps" — and holding the newest separately is
/// what lets the state a command produced be carried out again after being given back. It is not
/// recomputed and it is not derived: a step written before the command it belongs to cannot say
/// what the command produced, so the diagram the command produced is the one place that knows.
///
/// **Giving a step back moves the position and changes nothing else.** [`undo`](Session::undo) and
/// [`redo`](Session::redo) are one line each for that reason, and neither removes a step or
/// rebuilds a diagram: the state that comes back is the state that was there, held since before
/// the command that displaced it. Nothing is inverted, and the newest diagram stays where it is
/// while the history walks over it, which is what makes the walk go both ways.
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
    /// The newest diagram there is: what the session was created over until a command changes it,
    /// and what it holds again at the end of the history.
    ///
    /// `pub(crate)` rather than private because [`Command::perform`](crate::Command::perform) is
    /// in another module and is the only thing allowed to change it. That is the entire
    /// enforcement, and it is worth being exact about how thin it is: one field and one absence.
    /// A public method returning `&mut Diagram` would void it, and nothing would fail — the
    /// session would keep recording faithfully while a caller changed its diagram behind it,
    /// which is the one failure this layer exists to make impossible.
    pub(crate) diagram: Diagram,

    /// The steps behind `position` and the steps ahead of it, in the order they were made. **Step
    /// `j` is the diagram as it was when command `j + 1` was carried out**, so the step at
    /// `position` is the state the session hands out and the step behind it is the state a
    /// command found. The newest state is not here, and `position == steps.len()` is what says so.
    steps: Vec<Step>,

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
    pub fn new(diagram: Diagram) -> Session {
        Self {
            diagram,
            steps: Vec::new(),
            position: 0,
        }
    }

    /// The diagram this session holds, borrowed.
    ///
    /// **The newest one at the end of the history, and otherwise the one the step at the position
    /// holds** — the same state the step is the record of, read rather than rebuilt. Nothing is
    /// copied and nothing can be changed through it, which is what makes it the answer for a
    /// caller that repaints on every event. A caller that wants to keep one copies it — `Diagram`
    /// is `Clone` — and the copy is a diagram like any other: changeable, free of the session and
    /// unconnected to it.
    #[must_use]
    pub fn diagram(&self) -> &Diagram {
        self.steps
            .get(self.position)
            .map_or(&self.diagram, |step| &step.diagram)
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

    /// Takes `command` as the one about to be carried out: records the diagram it finds as a step,
    /// discards whatever was ahead of the position, and adopts the state at the position as the
    /// diagram being edited.
    ///
    /// **The step is pushed before the command runs**, which is what makes it the diagram as it
    /// was rather than a result of being told what became of it — the model's reason for not
    /// inverting a command at all, since `Diagram::remove` freezes what hung from the figure it
    /// takes and nothing re-hangs it afterwards.
    ///
    /// **The truncation is what makes the history a line rather than a tree.** Branching earns
    /// its keep when a state somebody discarded is worth returning to, and nothing here says one
    /// is: a caller that moves on from a state it gave back has said the state was not worth
    /// keeping. What would reopen it is the first caller that wants one back, and §6 of the model
    /// says so.
    ///
    /// **It is recorded whether or not the command changes anything**, including a command naming
    /// a figure this diagram does not hold. The session does not know whether the caller meant to
    /// do something, and a history that skipped a step the caller took is worse than one longer
    /// than it needed to be.
    pub(crate) fn record(&mut self, command: Command) {
        let found = self.diagram().clone();
        self.steps.truncate(self.position);
        self.steps.push(Step::new(command, found.clone()));
        self.diagram = found;
        self.position += 1;
    }
}
