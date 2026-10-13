//! The session: a diagram being edited, the steps behind and ahead of it, and how far along them
//! it stands. See the _Vocabulary_ rows for `Session` and for `Position`, and §4 and §6 of
//! [`docs/editing-model.md`](../../../docs/editing-model.md).

use monospace_diagram::Diagram;
use rpds::VectorSync;

use crate::utilities::kept_first;
use crate::{Command, Step};

/// A diagram being edited, the steps behind and ahead of it, and a position among them.
///
/// **Nothing here changes a session, and the type system says so rather than a rule this file
/// keeps.** No method takes `&mut self`: every one of them is asked over `&self` and hands a new
/// [`Session`] back, so a caller that forgets to keep what it was given changes nothing, and a
/// caller that keeps it still cannot have changed this one. **The guarantee §4 of the model makes —
/// the diagram a session owns is changed through the session and nowhere else — is what the absence
/// of a `&mut self` anywhere in this surface means**, and it is a stronger answer than the one a
/// private field was: nothing in this crate can break it, because there is no borrow that would let
/// it. **The price is that a caller has to keep what it is given**: discarding the result of
/// [`perform`](Session::perform) changes nothing at all, which is a warning rather than a failure.
///
/// **The field holds the newest diagram there is, and the position says which diagram is in hand.**
/// A step is pushed *before* a command is carried out, so what the steps hold is every state that
/// is not the newest one, and the newest one has nowhere else to be: it is what
/// [`diagram`](Session::diagram) answers with at the end of the history, and it is the only state
/// no step holds. The two are the same number of diagrams either way — the model's "memory
/// proportional to the diagram times the number of steps" — and holding the newest separately is
/// what lets the state a command produced be carried out again after being given back. It is not
/// recomputed and it is not derived: a step written before the command it belongs to cannot say
/// what the command produced, so the diagram the command produced is the one place that knows.
///
/// **What a session is made of is shareable, and the sharing is what pays for the copies.** Two
/// sessions standing at different positions in one history hold the same `VectorSync<Step>` and the
/// same `Diagram`s, and a new one costs a new position and at most a new leaf. Nothing in the layer
/// needs two sessions at once today — the demonstration walks one — but this is the property a
/// caller gets for free the first time one does, and it is why the steps are a persistent sequence
/// and not a `Vec`.
///
/// **Giving a step back moves the position and changes nothing else.** [`undo`](Session::undo) and
/// [`redo`](Session::redo) are one expression each for that reason, and neither removes a step or
/// rebuilds a diagram: the state that comes back is the state that was there, held since before the
/// command that displaced it. Nothing is inverted, and the newest diagram stays where it is
/// while the history walks over it, which is what makes the walk go both ways.
///
/// **The history is a line, and `position` is where in it the session stands.** It is how many
/// steps are behind rather than an index into a pair of stacks, so it cannot disagree with either
/// of them: there is something behind exactly when `position` is above zero, and something ahead
/// exactly when it is below `steps.len()`. A command carried out from behind the end discards what
/// was ahead, which is the only end a line can discard from without changing what giving a step
/// back means.
///
/// **The steps are held in the same persistent sequence the diagram holds its figures in**, so a
/// session standing at one place in a history shares every step with one standing at another
/// rather than copying them — and a step is a whole diagram beside a command, which is what makes
/// the difference between sharing and copying the difference between 949 B and 40 122 B per step.
/// That sequence has no truncation, so what was ahead is dropped by a walk of `drop_last_mut`
/// rather than in one step: each of those is a trie descent where a `Vec` was one, and it is paid
/// only on the branch that carries a command out from behind the end. See `kept_first`, which is
/// where that walk is written and tested.
///
/// # The absence, shown rather than described
///
/// The rule that nothing here changes a session is held by nothing in this file, so it
/// is worth one documentation example that says it out loud: everything up to the last line compiles, and the
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
    /// The newest diagram there is: what the session was created over until a command changes it,
    /// and what it holds again at the end of the history.
    ///
    /// **Private, and written rather than handed around.** It was `pub(crate)` while
    /// `Command::perform` was the way in from another module; with `perform` a method of the
    /// session, nothing outside this file needs to change it and nothing outside it can.
    diagram: Diagram,

    /// The steps behind `position` and the steps ahead of it, in the order they were made. **Step
    /// `j` is the diagram as it was when command `j + 1` was carried out**, so the step at
    /// `position` is the state the session hands out and the step behind it is the state a
    /// command found. The newest state is not here, and `position == steps.len()` is what says so.
    ///
    /// Held in the same persistent sequence the diagram holds its figures in, so two sessions over
    /// one history are sharing it rather than copying it — see [`record`](Session::record), which
    /// is the one place it is written to.
    steps: VectorSync<Step>,

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
            steps: VectorSync::new_sync(),
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

    /// Gives the step behind the position back: the diagram it holds is what the session this
    /// returns holds again.
    ///
    /// **Nothing here is changed, and the same is true of [`redo`](Session::redo) and
    /// [`perform`](Session::perform).** What comes back is a session standing one place earlier
    /// over the same steps: a new position rather than a new history, every step a node this one
    /// was already holding, and no diagram copied to produce it.
    ///
    /// **At the oldest position there is nothing behind, and what comes back is this session
    /// unchanged** — no error, no report and no panic, which is the answer the diagram gives a
    /// figure it does not hold and the reason `can_undo` is asked rather than a result waited for.
    #[must_use]
    pub fn undo(&self) -> Self {
        if !self.can_undo() {
            return self.clone();
        }

        Self {
            diagram: self.diagram.clone(),
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
            diagram: self.diagram.clone(),
            steps: self.steps.clone(),
            position: self.position + 1,
        }
    }

    /// Carries `command` out against the diagram this session holds, and hands back the session
    /// standing over what it left: records the diagram the command finds as a step, discards
    /// whatever was ahead of the position, and adopts the state at the position as the diagram
    /// being edited.
    ///
    /// **This session is not changed, and discarding what comes back changes nothing.** That is
    /// what keeping a session out of reach of a `&mut` buys, and it is why this is the one method
    /// here whose result has to be kept rather than merely read.
    ///
    /// **The step holds the diagram the command found, read before the command is carried out and
    /// kept as a copy of it.** That is what makes it the diagram as it was rather than a result of
    /// being told what became of it — the model's reason for not inverting a command at all, since
    /// `Diagram::remove` freezes what hung from the figure it takes and nothing re-hangs it
    /// afterwards. **The order of the two lines below is not what makes that true** — the step is
    /// handed a copy, so it holds the same state either way — and the copy is there rather than the
    /// order because the command is consumed by the step and cannot be asked twice.
    ///
    /// **Nothing is inverted, and the steps are what make that unnecessary.** `Diagram::remove`
    /// freezes every reference naming the figure it takes and states that nothing re-hangs them,
    /// so an inverse of that command cannot be derived — it could only be reconstructed from what
    /// the caller happened to capture beforehand. Every state the history passes through is kept
    /// instead, and giving a change back is one of them becoming the diagram again.
    ///
    /// **What was ahead of the position is discarded**, which is what makes the history a line
    /// rather than a tree. Branching earns its keep when a state somebody discarded is worth
    /// returning to, and nothing here says one is: a caller that moves on from a state it gave back
    /// has said the state was not worth keeping. What would reopen it is the first caller that
    /// wants one back, and §6 of the model says so. **It is `kept_first` rather than a
    /// `Vec::truncate`**, because the sequence holding the steps is not one that can be truncated
    /// in place; the walk is where the cost of that choice is written down rather than paid
    /// silently here.
    ///
    /// **What the command becomes is carried out here and nowhere else.** The mapping from a
    /// meaning to one of the diagram's five operations is the one in [`Command`], and the session
    /// is what carries it out because only what was asked knows whether what it was asked to do
    /// was one gesture or several: a figure dragged across a screen is one thing to the person
    /// dragging it and many displacements to the diagram. Nothing reads that answer yet — §8 of
    /// the model says what is left to write — and the field it will be read from is the step's
    /// command, which is recorded beside the diagram from this first version.
    ///
    /// **It is recorded whether or not the command changed anything**, including a command naming
    /// a figure this diagram does not hold. The session does not know whether the caller meant to
    /// do something, and a history that skipped a step the caller took is worse than one longer
    /// than it needed to be.
    #[must_use]
    pub fn perform(&self, command: Command) -> Self {
        let found = self.diagram().clone();
        let diagram = command.apply_to(&found);

        let mut steps = kept_first(&self.steps, self.position);
        steps.push_back_mut(Step::new(command, found));

        Self {
            diagram,
            steps,
            position: self.position + 1,
        }
    }
}
