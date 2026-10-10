---
status: draft
date: 2026-10-10
---

# An editing layer above the diagram model

## Why now

[`docs/diagram-model.md`](../docs/diagram-model.md) settles that a diagram is a value, that holding
one is not the same as editing one, and that its five operations are all of them. It also says so by
declining to keep any history: a removal "hands back nothing. There is no history and nothing to
undo". Something has to live on the other side of that sentence, because every caller holding a
`&mut Diagram` today changes one and none of them can give the change back.

**The layer is wanted for its vocabulary and not only for its history.** The diagram's five
operations are complete and coarse at once, and the coarseness is free there and expensive here:
`replace` is one operation serving four meanings — a figure displaced, resized, restyled or changed
into another kind — and a caller who drags a figure and a caller who drags its border mean different
things and did different work. A caller is not the diagram, and a vocabulary written for the diagram
is a vocabulary that has to be translated by every consumer that is not one.

Nothing here is drawn and nothing is read from a file, which is what keeps the slice small: the
subject is what a caller can mean and what it costs to mean it, and that is testable without a
terminal, a screen or a file.

**The demonstration is why this is one change and not two.** It is a scripted run of the diagram's
own operations, so it is the one caller in the repository this layer fits without anything invented,
and it is what turns the ownership rule from a sentence into something exercised. That the building
stage is larger than the crate alone is a cost worth naming here rather than discovering at the pull
request.

**This spec is over the ceiling `specs/README.md` sets, and the demonstration is why.** The crate
and its document fit under it; adding the first caller does not. The format's own first answer is to
split, and the answer given here is not to, because the demonstration is what proves the other half.
Rewording would not have brought it under: the words come back on the next `cargo xtask fix`.

## Scope

### In

- `docs/editing-model.md`, created, owning what it means to change a diagram and what holds one
  while it is being changed. It lands here rather than in the building stage because both existing
  model documents are design intent written before the code that implements them, and this one
  records a decision — a vocabulary of meanings rather than of operations — that the code cannot be
  written until it is made.
- `docs/diagram-model.md`, one sentence: the paragraph that says what it does not describe gains a
  pointer to the layer that does, the same move `docs/model.md` makes towards it.
- The crate `monospace-editing`, holding a session and the commands performed against it, and
  `Clone` on `Diagram`, which is the one method it needs that the diagram does not have.
- Four commands, and no more: `Move`, `Hang`, `Forward` and `Remove`.
- `-p monospace-editing` added to the gate's `wasm` step, the crate being portable by naming rather
  than by exclusion.
- `monospace-cli`'s demonstration running on a session and walking back to its first picture.
- The two sentences that say every crate but the CLI stays portable, replaced by what the gate's
  `wasm` step actually checks: the table in `CONTRIBUTING.md`, its _What is in scope_, and one line
  of `README.md`.
- `specs/197-an-editing-layer-above-the-diagram-model.md`, this file.

### Out

- **Commands the demonstration does not need.** Adding a figure, resizing one, restyling one and
  moving one toward the back of the order are all meanings this layer could carry and none of them
  has a caller. A command added later is additive — the session already does whatever a command says
  — so leaving them out defers work rather than foreclosing it.
- **What is selected and what is on the screen.** A selection is a thing a caller holds and a screen
  is a thing a caller draws, and neither is a record of what a diagram used to be.
- **Reading and writing a diagram.** The description format stays private to `monospace-cli`. It
  later becomes a layer above the diagram that knows nothing of this one.
- **Merging a gesture into one step.** D3 defers it and `docs/editing-model.md` records what reopens
  it. Recording the command beside the diagram is what leaves the rule writable later, so what is
  out is the rule and not the room for it.
- **A command as text.** A command carrying its own fields is what makes a line of a script
  possible, and nothing here reads or writes one.
- **A bound on how many steps are kept**, and **whether a session knows where its diagram came
  from**. Both are in the model's _Deliberately unresolved_, with what would settle each.
- **WebAssembly bindings, a web front end, non-terminal GUIs, persistence and collaboration**, which
  `CONTRIBUTING.md` names as renegotiated rather than quietly widened.

## The decision

**D1 — a command carries the meaning, and the fields that go with it.**

**Answer:** `Move { id, by }`, not `Replace(id, shape)`. **Why not** the diagram's five operations
restated as an enum: `replace` is one operation serving four meanings, and a caller who displaces a
figure and a caller who resizes one did different work and would say so differently. **Why not** the
command taking a finished figure: it throws away what the caller knew — the delta, not the result of
applying it — and the delta is what a person reading a list of commands recognizes, and what a line
of a script has to be written in either way. **Answered by** the maintainer, in the session that
wrote this.

**D2 — a command is given the session, and not the diagram.**

**Answer:** `Command::perform(&self, session: &mut Session)`. **Why not**
`perform(&self, diagram: &mut Diagram)` with the session doing the bookkeeping around it: then
nothing in the command can know whether what it was asked to do was one gesture or several, and that
is the only question standing between the session and a usable undo for a drag. **Why not** a
session method that dispatches: the same thing said from the other side, and it puts the decision
where the caller is not. **What it costs:** the command and the session cannot be told apart, which
D2 accepts rather than solves — the collaboration between them is the point. **Answered by** the
maintainer, in the session that wrote this.

**D3 — a step holds the diagram whole and the command beside it, and nothing is inverted.**

**Answer:** a step is the diagram as it was and the command that replaced it; undo is that diagram
becoming the diagram again. **Why not** the inverse of each command: `Diagram::remove` freezes every
reference naming the figure it takes and states that nothing re-hangs them, so an inverse cannot be
derived from the command — only reconstructed from what a caller captured beforehand, and a command
whose author forgot is wrong in a way nothing reports. **Why not** the diagram alone: it records
what is true, never what was done to it, so whatever eventually tells a drag from a teleportation
has nothing to read in a pair of diagrams. **Why not** the difference between two diagrams: the same
ambiguity as a snapshot, computed on every command, which is a cost on the write path paid to save
memory this domain has no use for. **What it costs:** memory proportional to the diagram times the
number of steps. **Answered by** the maintainer, in the session that wrote this.

**D4 — one command is one step, and the session merges nothing.**

**Answer:** a caller asks, and what it asked for is one thing the caller may give back. The session
records and does not interpret. **Why not** writing the rule now: no caller produces a gesture that
undo makes useless yet, and D2 has already settled where the answer lives — the command is asked,
and it is the intent rather than the trace of it. **Why not** a rule reading a run of operations and
deciding they were one: a figure dragged across the screen and a figure nudged one cell a hundred
times produce the same operations, and nothing in them says which happened. **What reopens it:** the
first caller that produces a gesture, which a drag is. **Answered by** the maintainer, in the
session that wrote this.

**D5 — the history is linear.**

**Answer:** a session holds a position among its steps, giving one back moves it, and a command
carried out behind the end discards the steps ahead. **Why not** a tree: branching earns its keep
when a state somebody discarded is worth returning to, and nothing here says one is. **What reopens
it:** the first time a caller wants a state the history threw away. **Answered by** the maintainer,
in the session that wrote this.

**D6 — the crate is `monospace-editing`, and the document is `docs/editing-model.md`.**

**Answer:** both named for the activity rather than for either half of it. **Why not**
`monospace-commands`: the commands are the vocabulary, not the history, and a crate named for half
of what it holds is a crate whose contents surprise a reader. **Why not** `monospace-application`:
the interactive program is the application, and the word belongs to the layer above. **Why not**
`monospace-session`: that names the mechanism rather than the layer, and the mechanism is what the
layer was misnamed after. **Why not** a module of the TUI with no document of its own: every
decision here is observable from outside — whether undo is available, what a command means, whether
the diagram can be changed at all — and `docs/diagram-model.md` §11 puts a rule in a module's
rustdoc only "where nothing outside it can observe the choice". **Answered by** the maintainer, in
the session that wrote this.

**D7 — the gate's `wasm` step names the crate, and the prose stops claiming what it excludes.**

**Answer:** the building stage adds `-p monospace-editing` to the step, and this change replaces the
two sentences that say every crate but the CLI with what the step checks. **Why not** naming the
exclusions in the step: a list of what is excluded is a second list to keep in step with the first,
and the omission is what keeps a crate added later out. **Why not** deferring the prose to the
building stage: the claim is already false today, because `xtask` is a crate the step does not name.
**Answered by** the maintainer, in the session that wrote this.

## Model slice

- `docs/editing-model.md`, created. It owns the layer: what a command means and why it carries its
  own fields, that a command is given the session, what a step is, what undo restores, and what the
  layer does not reach. It lands there because D1, D2, D3, D4, D5 and D6 are the model's kind of
  rule rather than a module's, and a rule that lives only in a new crate's rustdoc is a rule with
  one reader — which is the next change to need it.
- `docs/diagram-model.md`, one sentence. The paragraph beginning _What this document does not
  describe_ names `docs/editing-model.md` and keeps the claim it makes about itself.
- `docs/model.md`: none. No open question in either document above is answered here.

## Public surface

The deciding stage adds none of this; it is what the building stage publishes.

```rust
// monospace-editing
pub struct Session { /* private */ }

impl Session {
    pub fn new(diagram: Diagram) -> Session;
    pub fn diagram(&self) -> &Diagram;
    pub fn undo(&mut self);
    pub fn redo(&mut self);
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
}

impl Command {
    pub fn perform(&self, session: &mut Session);
}

pub enum Command {
    Move { id: ShapeId, by: Delta },
    Hang { id: ShapeId, from: ShapeId, anchor: Anchor },
    Forward { id: ShapeId },
    Remove { id: ShapeId },
}

// monospace-diagram
impl Clone for Diagram {}
```

**There is no `Session::apply`.** D2 is visible in the surface rather than in prose: the command is
what acts, and it is handed the session to act on. A session method taking a command would be the
same design said from the other side and would put the decision D2 is about back where it cannot be
made.

No method returns `&mut Diagram`, and none returns a `Diagram` either: `Diagram` is `Clone`, so a
caller that wants to keep one takes it, and a method here would be a second way to say the same
thing.

## Behavior

1. A session owns the diagram it was created over.
2. The diagram a session owns is changed only by a command performed against it.
3. No method of a session hands out a mutable reference to the diagram it owns.
4. Every command becomes one of the diagram's five operations, and no command reaches the diagram by
   any other way.
5. A command is carried out and the diagram as it was is recorded as a step, whether or not the
   command altered the diagram.
6. Every command is a step of its own, and the session merges none.
7. Undo restores the diagram the step behind the position holds, whole.
8. Redo restores the diagram the step ahead of the position holds, whole.
9. Undoing with no step behind changes nothing, and redoing with no step ahead changes nothing.
10. A command carried out with steps ahead of the position discards them.
11. Whether undo and redo are available is answerable without carrying either out.
12. The diagram is drawn by borrowing it, and a drawing of what a session holds is equal to a
    drawing of the diagram it was created over whenever no command has been carried out.
13. Nothing of a session reaches a file, a screen, or what is selected.
14. The demonstration wraps its diagram in a session once and performs every step through it, so its
    seven pictures before the walk-back are the diagram model's own and this change contributes only
    the lines under them.
15. The demonstration walks back one picture per step after its last, and its final picture is the
    one it began with.
16. The demonstration given a path draws one picture and nothing else, and walks nowhere.

## Examples

**A whole run over one figure.** Hypothetical: no code produces this yet, and none of it has been
observed. The window is `window(8, 3)` throughout and the figure is `small_box(0, 0, no fill)`, so
every picture below is what the diagram model already draws — this change contributes the line under
each one and nothing above it.

```text
  session over [small_box(0,0,no fill)]

  no command
  ┌──┐
  │  │
  └──┘
  steps: 0   undo: no   redo: no

  Move(1, by 4 right)
      ┌──┐
      │  │
      └──┘
  steps: 1   undo: yes  redo: no

  undo
  ┌──┐
  │  │
  └──┘
  steps: 1   undo: no   redo: yes

  redo
      ┌──┐
      │  │
      └──┘
  steps: 1   undo: yes  redo: no

  Move(7, by 1 right)  —  a figure the diagram does not hold
      ┌──┐
      │  │
      └──┘
  steps: 2   undo: yes  redo: no

  undo
  undo
  ┌──┐
  │  │
  └──┘
  steps: 2   undo: no   redo: yes
```

Three of those lines carry rules on their own. The picture that does not move between the fourth and
the fifth blocks is rule 5: a command naming nothing is recorded all the same, because the session
does not know the caller meant to do something and a history that skipped a step the caller took is
worse than one longer than it needed to be. The `undo: no` on the seventh block is rule 9 at the end
of the history rather than in the middle of it. The second block is rule 12 — the picture is the
diagram model's, drawn through a borrow, and the layer's whole contribution to it is that the fifth
and eighth blocks are the same picture again.

**The demonstration's shape.** Also hypothetical, and for the same reason. It is written as the
count of pictures rather than as the pictures, because the pictures are the diagram model's and this
change contributes none — what is listed is what a reader of `cargo run -p monospace-cli` would
count.

```text
  as written, and the seven steps the demonstration already takes     8 pictures
  walking back, one picture per step, until the first picture again   7 pictures
                                                                          ---
                                                                          15
```

The last of the fifteen is the first of the eight, and that equality is the whole of rules 14
and 15. Rule 16 is what keeps the other fourteen out of the form given a path: that one draws a
single picture and reaches for nothing.

## What proves it

None of these tests exists yet; the deciding stage adds no code. The names are what the building
stage is held to.

| Rule | Test                                                              |
| ---- | ----------------------------------------------------------------- |
| 1    | `a_session_owns_the_diagram_it_was_created_over`                  |
| 2    | `the_diagram_changes_only_by_a_command_performed_on_it`           |
| 3    | `no_method_of_a_session_hands_out_a_mutable_diagram`              |
| 4    | `each_command_becomes_one_of_the_diagrams_five_operations`        |
| 4    | `no_command_reaches_the_diagram_by_another_way`                   |
| 5    | `a_command_naming_nothing_is_still_a_step`                        |
| 6    | `two_commands_are_two_steps_and_are_never_merged`                 |
| 7    | `undo_restores_the_diagram_the_step_behind_holds`                 |
| 8    | `redo_restores_the_diagram_the_step_ahead_holds`                  |
| 9    | `undo_with_nothing_behind_changes_nothing`                        |
| 9    | `redo_with_nothing_ahead_changes_nothing`                         |
| 10   | `a_command_with_steps_ahead_discards_them`                        |
| 11   | `whether_undo_is_available_is_answerable_without_carrying_it_out` |
| 12   | `drawing_what_a_session_holds_needs_no_copy`                      |
| 14   | `the_demonstration_runs_every_step_through_a_session`             |
| 15   | `the_demonstration_walks_back_to_the_picture_it_began_with`       |
| 16   | `a_path_draws_one_picture_and_walks_nowhere`                      |
| 13   | nothing holds this, and it is named here rather than claimed done |

Rule 13 is not a rule a test can read: it is a statement about what the crate depends on and what it
knows, and a `Cargo.toml` is where a reader looks for it.

Three things are measured rather than tested. **That the `wasm` step has teeth** for the new crate:
`-p monospace-editing` added to the step on purpose, the step run, its failure recorded, the
addition removed — reported in the building pull request. **That the crate compiles for
`wasm32-unknown-unknown`**, which the step does once it is named. **That the demonstration's
pictures are the ones it prints today**, which is what rules 14 and 15 are checked against: the
seven forward pictures are moved by nothing this change does, and the walk-back adds seven more. The
sweep is reported in the building pull request, because the deciding stage moves no snapshot and its
honest answer today is `Unchanged.`

## Open questions

None. What surfaced while writing this became another issue rather than a section here:
[#198](https://github.com/andresmoschini/monospace/issues/198) asks whether a command is an enum or
a trait, which nothing here needs an answer to — four commands have one obvious shape — and which
the first script decides, since a script can only use the commands that exist unless something can
add one.
