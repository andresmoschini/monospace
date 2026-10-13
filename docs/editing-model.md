# The editing model

**Status:** Design intent, with one slice implemented. What §1 through §7 describe is
[`monospace-editing`](../crates/monospace-editing), and
[`specs/197-an-editing-layer-above-the-diagram-model.md`](../specs/197-an-editing-layer-above-the-diagram-model.md)
is the specification that slice was built against. §8 _Deliberately unresolved_ and §10 _Open
questions_ are what a later change answers. Amend this document when reality contradicts it, and say
so in the commit.

**Created:** 2026-10-10

This document describes the layer above [the diagram model](diagram-model.md): the things a caller
means to do to a diagram, and what holds one while it is being changed and how any state already
seen is given back. [`diagram-model.md`](diagram-model.md) owns what a diagram is and the five
operations it answers to, and it is deliberately not extended here — the two layers are separate
crates and separate documents for the same reason. A diagram is a value: it is constructed, copied,
drawn and dropped. This layer is a live thing that contains one.

What this document does not describe: what is selected, what is on the screen, how a diagram is read
from a file or written to one, and anything about a terminal. Those are layers above or beside this
one, and none of them is known here. The file format is the first of them:
[`monospace-description`](../crates/monospace-description) reads a diagram in and knows nothing of
what a session is, and this layer knows nothing of it — the two hang off
[`monospace-diagram`](diagram-model.md) and neither depends on the other.

## 1. Vocabulary

| Term       | Meaning                                                                                    |
| ---------- | ------------------------------------------------------------------------------------------ |
| `Diagram`  | Shapes in an order, drawable, changeable; the source of truth, and a value                 |
| `Command`  | One thing a caller means to do to a diagram                                                |
| `Session`  | A diagram being edited, the steps behind and ahead of it, and a position among them        |
| `Step`     | One entry in a session's history: the diagram as it was left, and the command that left it |
| `Position` | How far along its history a session is; how many steps a caller may still have given back  |

**A command is not one of the diagram's five changes, and the two words are kept apart on purpose.**
A change is an operation the diagram answers to; a command is a meaning a caller means. Several
commands become one change, and one change answers to meanings that are not the same — which is the
whole of §2.

## 2. A command means something the diagram does not name

The diagram has five operations and they are complete: nothing else changes a diagram. They are also
**coarse**, and coarseness that costs nothing in a diagram costs something here, because a caller is
not the diagram. `replace` is one operation serving four meanings — a figure displaced, a figure
resized, a figure's stroke changed, a figure's kind changed — and a caller who drags a figure and a
caller who drags its border mean different things and did different work.

So a command carries **its own fields rather than a finished shape**:

```rust
Command::Replace(ShapeId, Shape)   // "here is the figure it is now"
Command::Move { id, by: Delta }    // "move it by this"
```

The first restates the diagram's operation and throws away what the caller knew. The second keeps
the delta, which is the thing a person reading a list of them would recognize — and the thing a
script would have to write either way, since a line of a script is made of meanings and not of a
blob handed to a method.

**Each command says which operation it becomes.** That mapping is the layer's work and it is not
free: a command that resizes has to know how to rebuild a figure of each kind at another size, and
to answer what resizing a figure that has no size means. The diagram never has to answer either
question, because it is handed the finished figure and never asks what became of it.

### The initial set

The set is **open**, and a command added to it is additive: the session already does whatever a
command says, and one more meaning is one more arm.

| Command                     | What it means                                     | The change it becomes              |
| --------------------------- | ------------------------------------------------- | ---------------------------------- |
| `Move { id, by }`           | the figure goes so much, along each axis          | `replace` with it displaced        |
| `Hang { id, from, anchor }` | one end of a connector hangs from a figure's side | `replace` with a rebuilt connector |
| `Forward { id }`            | one place toward the front of the order           | `forward`                          |
| `Remove { id }`             | the figure is taken out                           | `remove`                           |

These four are what [`docs/model.md`](model.md) calls _The initial set_ for shapes: enough to work
with, and no promise that four is the number. `Hang` is the one that shows what the layer is for —
it reads the connector that is there, keeps what it does not change, and rebuilds the one end that
changes, which is logic a caller otherwise writes and gets subtly wrong.

## 3. A command operates on the session

**A command is given the session, not the diagram.** That is what lets a command decide what a step
is, because only the caller knows that what it was asked to do was one gesture and not several: a
figure dragged across the screen is one thing to the person dragging it and many displacements to
the diagram.

This is why the history does not need a rule that guesses. A rule reading a run of operations would
have to tell one drag from a hundred nudges of the same figure, and **the two produce the same
operations** — nothing in them says which happened. A command is the intent rather than the trace of
it, so the question is asked of something that knows the answer. What the session does with the
answer is §5, and what it does today is also §5.

**A command still changes nothing the caller did not ask for.** Deciding what to record is not
deciding what to do: the operation a command becomes is the diagram's, unchanged, and the fact that
this layer can group two commands into one step is not a license for it to change one on the way.

## 4. The session owns the diagram

A session's diagram is held in a field of its own — the one it was created over, written once in
`new` and never again — or in a step of the history, and the only thing that changes it is a command
the session was given. **No method of a session hands out a `&mut Diagram`, and no method of a
session takes a `&mut Session` at all.** Every method is asked over `&self` and hands a new session
back, so there is no borrow a caller could be given that would let it change one in place.

**That is the enforcement, and it is worth being exact about how it works now.** A `&mut Diagram`
anywhere in the surface would be the old thin version of this rule — one field and one absence,
voidable by a method returning a mutable reference, with nothing failing when it was. The absence of
a `&mut self` is not that: a caller cannot reach a session's internals because the type hands out no
route to them, and the one failure this layer exists to make impossible — a session recording
faithfully while a caller changes its diagram behind it — has no spelling left in this crate. **The
price is that a caller has to keep what it is given**: discarding the result of `perform` changes
nothing at all, which is a warning rather than a failure, and a warning the surface cannot make for
a caller who wrote `let _ =`.

**A session is a value in the same way a diagram is, and it is asked the same way.** A diagram hands
back a new diagram from every one of its operations rather than changing the one it was given, and a
session over a diagram does the same. **What it buys is shareable history**: two sessions standing
at different positions in one history hold the same steps and the same diagrams, and a new one costs
a new position and at most a new leaf. What it costs is one node split per operation, which is why
this is written down rather than assumed — the layer has no caller that keeps two sessions yet, and
the first one that does is what settles whether the persistent sequence earns its keep.

**This is a claim about one diagram, not about diagrams.** `monospace-diagram` keeps its whole
public API, and every one of its five operations stays as usable as it is today by anything that
owns a diagram of its own. The guarantee is that a diagram a session owns is changed through the
session and nowhere else. A caller that wants to edit without a history holds a `Diagram` and edits
it, which is what happens today and is not deprecated by this layer.

## 5. Steps

A step is a diagram as it was left, together with the command that left it. The session pushes one
after it carries out a command, and **the one state no step holds is the diagram the session was
created over**, which is a field of its own and is written once.

**Both halves are needed, and a diagram alone cannot stand in for the other.** A diagram records
what is true, never what was done to it — a figure dragged one cell and the same figure teleported
across the diagram differ as gestures and are identical as states — so whatever eventually tells
those apart has nothing to read in a pair of diagrams. That is what the command beside it is for,
and it is why the command is stored rather than derived.

**The state a command found is the step before it.** A step therefore says what a command produced
rather than what it started from, and what it started from is one place further back — which is what
makes undo a walk over the sequence rather than a subtraction from it.

**Holding the diagram the session was created over in the field keeps one rule rather than two.**
The diagram in hand is that field or a step, decided by the position alone, and the field is written
once, so what it holds is the same for the whole life of the session. **The other order works too**
and would make the field hold the newest state rather than the first: a second place a diagram can
be, one whose contents change with every command, and an answer at the end of the history that is a
case of its own because the newest state is the one state kept outside it.

**One command is one step, and the session merges nothing.** A caller asks, and what it asked for is
one thing the caller may give back. The session records and does not interpret: it holds what it was
told and what the diagram looked like afterwards, and it draws no conclusion from the pair. §3 says
why that conclusion is available and _Deliberately unresolved_ says what is left to write.

**A command that names nothing is a step like any other.** The diagram's operations cannot fail, so
a command naming a figure the diagram does not hold changes nothing — and the session records it all
the same, because the session does not know whether the caller meant to do something. The
alternative, recording only commands that altered the diagram, would make the history shorter and
would make undo skip a step the caller took, and a history that does not match what was done is
worse than one longer than it needed to be.

## 6. Giving a change back

**Giving a step back restores the state one step further back, whole.** Nothing is computed and
nothing is inverted: the step carries the state, and undo is that state becoming the diagram again.
**At the oldest position what comes back is not a step at all** but the diagram the session was
created over, which is the one state the history does not hold — so the walk has one end that is a
field rather than an entry, and a caller cannot tell the two apart from what it is handed. The
diagram gives back its five operations for callers that hold one of their own; it is not what a
session undoes, and asking it to would be asking the value to remember. **What comes back is a
session standing one place earlier over the same steps** — a new position rather than a new history,
and no diagram copied to produce it.

**The history is linear.** A session at a position has steps behind it and steps ahead of it, and
giving a step back moves the position rather than removing anything. A command carried out while the
position is behind the end discards the steps ahead, which is what makes the history a line rather
than a tree: there is one way back from any point, and only the newest one is kept.

**Nothing is given back when there is nothing to give back.** At the oldest position there is no
step behind, and at the newest there is none ahead, and asking in either direction changes nothing —
the same answer the diagram gives a reference to a shape that is not there. Both are answerable
without carrying it out, which is what lets a caller grey out a control rather than guess.

## 7. What a session hands out

Two things, and they are not the same kind of thing.

A **reference to the diagram**, for drawing. Nothing is copied and nothing can be changed through
it, which is what makes it the answer for the caller that repaints on every event.

A **copy of the diagram**, for somebody who will keep it. `Diagram` is `Clone`, so a caller that
wants one takes it, and a method on a session would be a second way to say the same thing. The copy
is a diagram like any other: changeable, free of the session, and unconnected to it. **What "free of
the session" costs to be true of is worth saying now**: the copy shares every figure with the
original rather than duplicating it, which it may do because nothing in a diagram is mutable — the
session still cannot be reached through the copy, and dropping the copy frees nothing the original
holds.

**A copy of the session joins it, and that copy shares its steps as well as its figures.** Two
sessions over one history are two positions rather than two histories, and what a caller keeps a
session for — walking back over it, standing at another place in the same walk — is what the sharing
is for. Nothing in this layer needs two yet, and the property is here because the first caller that
does will not have to ask for it.

**"The diagram" is therefore two things, and a sentence that uses the word is ambiguous until it
says which.** The one a session owns is live and has a history; a copy is a value that does not. The
ambiguity is worth one sentence rather than two names, because both are diagrams in every sense
`monospace-diagram` gives the word.

## 8. Deliberately unresolved

- **When one command is a step rather than several.** §3 settles where the answer lives — the
  command is given the session, so the command is asked — and leaves it unwritten, because no caller
  produces a gesture yet. It would come back when one does, which a drag is.
- **How many steps a session keeps.** Nothing here bounds the history, and a diagram is a value
  rather than a handful of bytes, so an unbounded history is an unbounded amount of memory. What
  would settle it: the first diagram big enough for the copies to be felt rather than measured. A
  bound discards from the far end, which is the only end it can discard from without changing what
  undo means. **What the change that made a session a value did to this figure**: a session no
  longer copies its history, so 1000 steps cost 949 B each rather than 40 122 B, and a 300-shape
  session holds 0.9 MiB rather than 38.3 MiB. **The growth is still linear in the number of
  commands**, which is the whole of why the question is still open here — a bound is still the
  answer to a different question, when a history is too long to be worth keeping at any cost, and
  this section is not where that is decided.
- **Whether a session knows where its diagram came from.** Which file, whether it has unsaved
  changes, whether it has a name at all — none of it is here, and none of it is refused either.
- **How a command is written as text.** A command carrying its own fields is what makes a line of a
  script possible, and nothing here reads or writes one. What would settle it: the first caller that
  wants to replay a list of commands rather than type them.

## 9. Properties worth testing

- Giving back every step in order reaches the diagram the session was created over.
- Redoing them reaches the diagram the session held before any of it was given back.
- A command naming a figure the diagram does not hold leaves the diagram equal and is still a step.
- A command carried out after a step was given back leaves nothing ahead of the position to give.
- Each command is what one of the diagram's five operations becomes, and no command reaches the
  diagram by any other way. **Not the other way round**: `add` and `backward` are two of the five
  and the initial set of commands reaches neither, because adding a figure is not one thing a caller
  means to do to one and moving one toward the back of the order is not another.
- Drawing what a session holds and drawing the diagram it was created over produce equal buffers
  when no command has been carried out.
- A copy a session handed out does not change when the session's diagram does.

## 10. Open questions

Each of these is waiting for an answer, and the answer is prose in this document, or in the module
that owns what the question is about where nothing outside it can observe the choice. A feature spec
that needs one of them answered amends this document first and then implements the slice.

- **What is a command that means nothing about a figure the diagram does not hold?** §5 answers it:
  the step is recorded anyway. What it does not answer is whether the step is worth keeping, which
  is the same question as how far back the history reaches and is settled there.
- **Can a session be handed a diagram that has already been edited?** A diagram is a value, so one
  can be produced any way and opened. Whether the session should care where its diagram came from is
  _Deliberately unresolved_, and this question is what settles it.
- **Does a command know what it did?** §3 gives it the session and no way to be asked afterwards,
  which is enough for deciding what to record and not enough for a caller that wants to know what
  happened. Nothing needs the answer yet.
