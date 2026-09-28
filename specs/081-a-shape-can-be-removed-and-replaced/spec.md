<!-- The feature branch is named after the issue title, truncated by `cargo xtask spec`. -->
<!-- The maintainer's decision draft is a Spanish file outside this repository. -->
<!-- cspell:ignore decisiones -->

# Feature Specification: A shape can be removed and replaced

**Feature Branch**: `081-a-shape-can-be-removed-and-replaced-deciding` | **Issue**:
[#81](https://github.com/andresmoschini/monospace/issues/81) | **Created**: 2026-09-27 | **Status**:
Draft

**Input**: Issue #81, "A shape can be removed and replaced" — with identities in place, the rest of
the changes a diagram accepts become expressible. This slice takes a shape out, and puts a different
shape under an identity while it keeps its place in the order; and because a replacement is how a
figure moves, it also brings the displacement that builds the new value and the one query a caller
needs to read a figure back before replacing it. Part of
[issue #62](https://github.com/andresmoschini/monospace/issues/62), on top of
[#80](https://github.com/andresmoschini/monospace/issues/80).

## What this slice implements

Four sections of [`docs/diagram-model.md`](../../docs/diagram-model.md), each amended on this branch
ahead of this spec: a slice needing a rule the model lacks has the model grow it before the code
does.

- [§1 _Vocabulary_](../../docs/diagram-model.md#1-vocabulary) — one row added, how far a figure
  moves along each axis, and one corrected: a diagram's `Shape` holds its kind, its parameters and
  its position, while its identity and its place in the order are the diagram's, which is what
  _Changing a diagram_ already said and what B4's reader hands back.
- [§4 _Positions_](../../docs/diagram-model.md#4-positions) — displacing a figure moves every
  position it holds, and reaches a reference's offsets rather than anything that reference resolves
  to.
- [§9 _Changing a diagram_](../../docs/diagram-model.md#9-changing-a-diagram) — the `remove` and
  `replace` rows, the rule that no change can fail, and the replacement that moves, which is the
  figure displaced by a delta: a box and a line through their own position, a connector through both
  endpoints at once, and a connector is not the exception. That paragraph said only that moving is a
  replacement and left the new value's construction open.
- [§11 _Open questions_](../../docs/diagram-model.md#11-open-questions) — "How does a shape move?"
  reduces to the reference case, the only part of it with no type to displace yet.

The model already states the rule this slice's `remove` inherits — removing a shape leaves every
reference to it unresolved, and those shapes stop being drawn. Nothing here adds to it, and nothing
here can observe it; see **Testing expectations**. This spec restates none of the model, and takes
none of the decisions: the seven this slice needs are on the sheet `/speckit-plan` part one writes,
from the maintainer's draft at `monospace-drafts/81-decisiones.md`.

## Clarifications

### Session 2026-09-28

- Q: Where does the displacement in the demonstration's third picture come from — how far the figure
  moves, and who decides that amount? → A: A fixed value the demonstration's own code carries,
  chosen so the shipped description shows it and marked in a comment as a demonstration-only
  assumption, the way the reorder's is. The description format gains no field for it and the binary
  gains no argument for it, so a file's picture is exactly the one it was and the shipped
  description needs no change.
- Q: The checklist says the reader is justified because the demonstration needs it. Does the
  demonstration read a figure back? → A: No, and B4 never said it did. The demonstration keeps
  naming `#1` by hand, as it does today. B4 is justified by the general argument in **Input**: a
  caller holding only an identity cannot displace or replace a figure without reading it back first.
  The direction that is missing is the other one — there is no way to get a diagram's identities
  back at all, by listing them or by asking a position which shape decided it — and that is what B4
  scenario 3 leaves open and issue #86 settles.
- Q: The slice is named for removing and replacing a shape, but the shipped run shows neither — it
  shows a reorder and a displacement. Is that deliberate? → A: No. The shipped demonstration grows a
  fourth picture, showing the same back-most shape taken out. It is appended after the pair that
  contrasts the reorder with the displacement rather than interleaved with them, so all four
  pictures are about one figure: it moves, it is displaced, and then it is gone.
- Q: When the reader hands a figure back, does that figure carry the shape's identity? → A: No. It
  is the bare value the caller added, and the identity is the diagram's — held beside the figure
  rather than inside it, as it has been since spec 079. The model's _Vocabulary_ row for `Shape`
  claimed otherwise and is corrected on this branch, next to the `Delta` row it gains. Putting the
  identity inside the value would also put a name in an immutable figure whose name changes the
  moment a different figure is put under it.

### Session 2026-09-27

- Q: Issue #81's body defers movement to "the slice that implements movement". Does this slice
  implement it? → A: Yes, in full — `remove`, `replace`, the displacement, and the reader. The
  alternative was `remove` and `replace` alone, which leaves §9's displacement paragraph
  unimplemented and the demonstration at two pictures.
- Q: When do the four model amendments land? → A: On this branch, ahead of this spec, in a commit of
  their own. §4's new paragraph is what makes displacing a connector's reference visible rather than
  a silent change of meaning in issue #82.
- Q: `specs/080-.../contracts/diagram-api.md` says an identity has no constructor, and
  `ShapeId::new` is public. Which is right? → A: The code. The 080 contract is corrected on this
  branch in a commit of its own, so this slice's contract does not cite a claim the code
  contradicts.

## Behavior

### B1 — A shape can be taken out

1. **Given** a diagram holding several figures, **When** one is taken out by its identity, **Then**
   the picture drawn afterwards no longer contains it, and the figures that stayed draw exactly what
   they drew.
2. **Given** an identity the diagram does not hold, **When** a shape is taken out, **Then** nothing
   changes — no error, no report, no panic — the same answer _Positions_ gives a reference to a
   shape that is not there.
3. **Given** a shape that has been taken out, **When** any shape is added afterwards, **Then** the
   addition is named with a fresh identity and the removed one's is never handed out again.

### B2 — A different shape can be put under an identity

1. **Given** a diagram holding a box, **When** a wider box is put under that identity, **Then** the
   picture is the one that same wider box added on its own produces. The diagram never reaches into
   a box and widens it: the identity and the place in the order survive, and everything the figure
   owns is the new figure's.
2. **Given** a box under an identity, **When** a line is put under it, **Then** the figure is the
   line, kind included, and nothing of the previous figure survives:

   ```text
   ┌──┐
   │░░│
   └──┘
   ```

   ```text
   ────


   ```

   Hypothetical — hand-drawn, not generated. One window: the filled box as written, and the line
   that replaced it at the same place in the same order. The contract test carries both as generated
   pictures, which is what turns this requirement into a checked one.

3. **Given** a figure that overlaps another, **When** it is put back under its own identity
   unchanged, **Then** the overlap resolves as it did, because a replacement is not a reorder.
4. **Given** an identity the diagram does not hold, **When** a shape is put under it, **Then**
   nothing changes and the shape handed in is not added either: there is no way to name a shape into
   existence.

### B3 — A figure is displaced by a delta

1. **Given** a box, **When** it is displaced two cells to the right, **Then** the picture is the
   same box drawn two cells to the right.
2. **Given** a connector and a delta of no horizontal amount and two cells down, **Then** both
   endpoints moved two cells down, and the picture is the connector as written, two rows lower. It
   is not the exception to displacement, and displacing it is not a silent no-op:

   ```text
   ┌──┐
   │  ├───►
   │  │
   └──┘
   ```

   ```text
   ┌──┐
   │  │
   │  ├───►
   └──┘
   ```

   Hypothetical — hand-drawn, not generated. One window: one box that does not move, and the
   connector before and after the displacement. The contract test carries both as generated
   pictures, the second with each endpoint at `y + 2`, so the arrow row moves down the box and
   nothing else in the window does.

3. **Given** any figure, **When** it is displaced, **Then** no cell of the diagram has changed. A
   displacement builds a value and changes nothing; putting it back under the identity is what
   changes the diagram.
4. **Given** a figure displaced by nothing at all, **When** the value comes back, **Then** it is the
   same figure, and replacing it draws what it drew.

### B4 — A caller can read a figure back by its identity

1. **Given** a diagram holding a figure, **When** the caller asks for the identity its addition
   handed back, **Then** it gets that figure, and can compare it by value with the one it added.
   What comes back is the bare figure the caller added: the identity and the place in the order are
   the diagram's, held beside the figure rather than inside it, and nothing about the figure names
   where it sits.
2. **Given** an identity the diagram does not hold, **When** the caller asks, **Then** it gets
   nothing and nothing about the diagram changed.
3. **Given** this reader, **When** a caller looks for a way to ask what a diagram holds — a listing,
   an order, a count — **Then** there is none. One query by identity is the whole of it, and drawing
   stays the only way anything else is observed.

### B5 — The shipped demonstration shows a removal beside a displacement and a reorder

1. **Given** the shipped demonstration, **When** the application is run with no arguments, **Then**
   it prints four captioned pictures: the description as written, the same description with its
   back-most shape moved one place toward the front, that same shape displaced, and that same shape
   taken out.
2. **Given** the second and third pictures, **When** they are read together, **Then** the figure the
   third displaces is the one the second moved, so the contrast between what a reorder changes and
   what a displacement changes is direct rather than something a reader works out.
3. **Given** the third picture, **When** it is compared with the second, **Then** they differ, and
   every cell outside the displaced figure's own is the same in both.
4. **Given** the fourth picture, **When** it is read with the first three, **Then** the figure it no
   longer holds is the one the second moved and the third displaced, so all four are about one
   figure and the removal is the last of three changes rather than a fourth unrelated one.
5. **Given** the fourth picture, **When** it is compared with the third, **Then** they differ only
   in the cells that figure occupied: each is the cell the figure behind it decides, or empty where
   no figure decides one. Taking a shape out leaves a gap, not a hole punched in what was around it.
6. **Given** a path, **When** the application is given one, **Then** it prints one picture and
   nothing else, exactly as today. That is what `cargo xtask render` embeds, and it is why the split
   exists.
7. **Given** a description holding no shapes, or one, **When** it is demonstrated, **Then** all four
   pictures are identical and nothing fails.
8. **Given** the shipped demonstration, **When** the third picture is built, **Then** the amount the
   figure is displaced by is a fixed value the demonstration's own code carries, chosen so the
   shipped description shows it and marked in a comment as a demonstration-only assumption, the way
   the reorder's is. The description format gains no field for it and the binary gains no argument
   for it, so a file's picture is exactly the one it was and the shipped description needs no
   change.
9. **Given** the shipped demonstration, **When** it names the shape it moves, displaces and takes
   out, **Then** it names it with `#1` built by hand, as it does today, and does not read the figure
   back to obtain it. B4's reader is for a caller holding only an identity, and the demonstration is
   not one.

## Edge cases

- One shape taken out leaves an empty diagram, and drawing one leaves the buffer as it was.
- An identity from another diagram names nothing here, so nothing changes: identities are unique
  within a diagram and say nothing across two.
- A shape taken out and one added afterwards is named with the next identity, never the vacated one
  — the counter only rises, so an identity is never reissued.
- A figure put back under its own identity unchanged draws what it drew; a displaced-by-nothing
  figure comes back equal to itself.
- A figure displaced past the window has what falls inside drawn and the rest clipped, as any figure
  is, and a figure displaced onto cells another already holds composes with it by the same rule two
  figures sharing a cell always obey — the order did not move.
- A shape taken out that something would have referenced: no figure can hold a reference yet, so the
  answer is the general one. The model's rule for what was referenced arrives with issue #82.

## What this slice does not decide

- **What displacing a figure that holds a reference means.** _Positions_ states the first answer and
  _Open questions_ keeps the question, because nothing can hold one yet. Settled by: the first slice
  that has a reference to displace, issue #82.
- **How a caller discovers what a diagram holds.** The reader goes one way only: one query by an
  identity the caller already has, and nothing coming back. Neither a listing of a diagram's
  identities nor an answer to "which shape decided this position" exists, though the buffer's
  ownership record already holds what the second would need. Settled by: the first consumer needing
  a diagram's identities rather than one figure — issue #86 for the position.
- **Whether taking a shape out hands anything back.** It hands back nothing, and there is no history
  and nothing to undo. Settled by: a consumer that must undo a removal, a drawing application being
  the obvious one.
- **Whether a caller chooses an identity or edits one.** Still the model's open question, unchanged
  here. Settled by: the first slice where a caller has a name worth keeping.
- **A move of its own, beside the replacement that moves.** _Changing a diagram_ counts five changes
  and this leaves it at five. Settled by: a consumer that displaces repeatedly and for which
  rebuilding the value is the wrong shape — an editor being the obvious candidate.

## Testing expectations

- **Contract** — `remove`, by drawing: a diagram of several figures, drawn before and after one is
  taken out, differs, and the figures that stayed draw what they drew.
- **Contract** — `replace`, by drawing: a box put back as a wider box and as a line — the pair B2
  shows — each pinned against the picture that figure added on its own produces; the place in the
  order survives, asserted by a figure put back unchanged over an overlap; and a replacement naming
  nothing changes nothing and adds nothing.
- **Contract** — a displacement, by drawing, for a box and for a connector: each equals the picture
  the same figure added at the displaced position produces, which is what pins a connector as
  displaced through both endpoints rather than the exception, or the silent no-op, it would
  otherwise be; and a displacement changes nothing until the value is put back under its identity.
- **Contract** — the reader itself, the only rule here no picture can show: it returns the figure
  its addition named, and nothing for an identity the diagram does not hold.
- **Contract** — the identity counter only rises: take `#1` out, add a figure, and the identity
  handed back is the next one rather than `#1`.
- **Contract** — the demonstration: four captioned pictures on a bare run — the first the
  description as written, the second about the order, the third about the position and differing
  from the second only in the displaced figure, the fourth about the figure that is gone and
  differing from the third only in the cells it occupied; and a path still printing one picture and
  nothing else. The test pins the pictures, not the captions' wording.
- Unit tests for both changes, the displacement and the reader are the minimum; a rule above with no
  test named against it is unfinished. No characterization test is called for: every rule here is
  about a picture or a value, and a range too wide to read by hand is not what this slice produces.
- **Two rules are accepted with nothing to verify them, named as such per principle IV:** the
  model's rule that taking a shape out leaves every reference to it unresolved, and the rule
  _Positions_ now states about displacing a reference. No figure can hold a reference yet, so
  neither is reachable from a test. Both are in the model now because that document is design intent
  rather than observed behavior, which is what its own header says, and both can be asserted once
  the slice that introduces the type lands.

## Success criteria

- **SC-001**: A caller holding a diagram can take any shape out and draw what is left, and can put a
  different figure under that shape's identity and draw that, without rebuilding the diagram from
  its description.
- **SC-002**: A box put back as a wider box and as a line both draw the figure handed in, and the
  identity and the place in the order are the only things that survive either.
- **SC-003**: Every way of naming a shape that is not there — an identity from another diagram, one
  for a diagram holding nothing — leaves the diagram drawing what it drew, adds nothing, and fails
  no run.
- **SC-004**: A figure displaced two cells down draws where the same figure added there would draw,
  connector included, and no kind of figure is left where it was.
- **SC-005**: Taking a shape out and adding one afterwards never reissues the removed shape's
  identity.
- **SC-006**: A bare run of the application shows the same figures four times, the second differing
  from the first only in which of two overlapping figures wins their shared cells, the third only in
  where that same figure sits, and the fourth not holding that figure at all — so a person who runs
  it tells a reorder, a displacement and a removal from one another without reading a test.
- **SC-007**: Given a path, the application still prints one picture and nothing else, which is what
  `cargo xtask render` embeds in a document.
- **SC-008**: `cargo xtask check` is green, including `monospace-diagram` compiling for
  `wasm32-unknown-unknown`.
