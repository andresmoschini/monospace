<!-- The feature branch is named after the issue title, truncated by `cargo xtask spec`. -->

# Feature Specification: A reference carries a horizontal and a vertical offset

**Feature Branch**: `083-a-reference-carries-a-horizontal-and-a-v-deciding` | **Issue**:
[#83](https://github.com/andresmoschini/monospace/issues/83) | **Created**: 2026-09-29 | **Status**:
Draft

**Input**: Issue #83 — a reference names a shape and one of its four sides, so an endpoint attached
to a box lands exactly on that side and an arrow cannot be started a character away from it. This
slice gives a reference the two offsets the model's _Vocabulary_ already gives it, which is its last
field, and lets a description name one. What displacing a figure that holds a reference means is
deliberately not here: it is [#143](https://github.com/andresmoschini/monospace/issues/143), and
this slice is what it waits for. Part of
[#62](https://github.com/andresmoschini/monospace/issues/62), on top of
[#82](https://github.com/andresmoschini/monospace/issues/82).

## What this slice implements

Four sections of [`docs/diagram-model.md`](../../docs/diagram-model.md), none amended: the model
already describes a reference as three fields and says one resolves by adding the offsets, so this
slice is that sentence becoming true.

- [§1 _Vocabulary_](../../docs/diagram-model.md#1-vocabulary) — the `Reference` row names three
  fields, and this slice lands the two offsets.
- [§4 _Positions_](../../docs/diagram-model.md#4-positions) — a reference resolves to its anchor
  plus the two offsets. Left: the displacement paragraph, which is #143.
- [§6 _Attachment_](../../docs/diagram-model.md#6-attachment) — an endpoint's position may be a
  reference, and is the only one that may be. The offset is what lets it stand off the side.
- [§11 _Open questions_](../../docs/diagram-model.md#11-open-questions) — what displacing a figure
  holding a reference means stays open, and **What this slice does not decide** says why.

One thing this adds that the model does not describe, because it is not the model's: a description
file may name a reference. That format belongs to the application
([ADR-0035](../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md)), so this is a
consumer of the model and not a change to it.

This spec restates none of the model and takes none of the decisions: what type carries the two
amounts, how a description spells a reference, and how the arithmetic is reached are the sheet
`/speckit-plan` part one writes, beside
[ADR-0041](../../docs/decisions/0041-resolve-a-position-through-a-reference.md).

## Clarifications

### Session 2026-09-29

- Q: The offset sits in the screen axes or in the anchor's own frame, where "one cell out from that
  side" is a single value rather than four. → A: **The screen axes** — a horizontal and a vertical
  amount, both signed, added to the point the anchor resolves to. It is what §1 and §4 say, and it
  lets #143 reach a reference's offsets by adding a displacement to the two fields rather than by
  converting it first, which would give the same method a second meaning. The convenience the
  anchor's frame buys is not refused, it is written down in
  [#146](https://github.com/andresmoschini/monospace/issues/146).
- Q: What the demonstration does, given that a picture is the only place an offset is visible. → A:
  The connector's far endpoint stops being a point and becomes a reference to the bottom side of the
  box above it, with an offset of one in each axis — the point it already stands on, so **the
  rendered picture does not change by a character**. The evidence is in the description rather than
  in the picture, which means the description has to be able to name a reference, and it reverses
  the answer 082 gave: `monospace-cli` reads one out of a file.

## Behavior

### B1 — A reference carries two offsets, and the endpoint stands off the side

1. **Given** a connector whose `from` endpoint is a reference to a box's right side carrying a
   horizontal offset of two and a vertical one of nothing, **When** the diagram is drawn, **Then**
   the endpoint lands two cells to the right of the middle of that side, and the picture is the one
   a connector drawn from that absolute point draws:

   ```text
   ┌──┐
   │  │ ─────
   └──┘
   ```

   Hypothetical — measured, not generated. What today's code draws for a connector whose `from` sits
   at the point this reference resolves to, which is what the referenced diagram has to draw too.

2. **Given** a reference to a box's bottom side carrying a vertical offset of one and a horizontal
   one of nothing, **When** the diagram is drawn, **Then** the endpoint stands one whole cell clear
   below the box. This is the arrangement that has no picture at all today, and the second block is
   what drawing it with no offset gives:

   ```text
   ┌──┐
   │  │
   └──┘
    │
   ┌┘
   └─
   ```

   ```text
   ┌──┐
   │  │
   └┬─┘
    │
   ┌┘
   └─
   ```

   Hypothetical — measured, not generated: the same connector, its `from` one row lower in the first
   than in the second. The second is the picture 082 already draws, and it is what the first is
   measured against.

3. **Given** a reference carrying an offset of nothing in both directions, **When** the diagram is
   drawn, **Then** the endpoint lands exactly on the middle of the side and every diagram 082 draws
   draws identically.

### B2 — The offset is a gap from the side, so it travels with it

1. **Given** the diagram B1.1 draws, **When** the box is replaced by the same box five cells to the
   right, **Then** the endpoint lands two cells to the right of the middle of the _new_ side, the
   free end stays where it was, and the route is drawn as it is drawn for those two points:

   ```text
        ┌──┐
        │  │ ────
        └──┘
   ```

   Hypothetical — measured, not generated: the same connector and the same offset, with the box on
   its new side. The offset is the gap from the border rather than a point, which is what this
   second block measures.

2. **Given** a reference to a shape the diagram does not hold, or to an anchor that shape does not
   answer, **When** the diagram is drawn, **Then** the whole figure holding it is not drawn however
   large its offset, every other shape draws exactly what it drew, and nothing errors, reports or
   panics. A large offset on a reference that resolves to nothing is still nothing, not somewhere.

3. **Given** a reference to a box that is then replaced by a line, **When** the diagram is drawn,
   **Then** the offset is added to whatever the line's side middle is — its own middle, on a
   horizontal line, asked for twice as its top and as its bottom.

### B3 — A displacement still leaves a hanging endpoint where it was

1. **Given** a connector with one endpoint a reference and one an absolute point, **When** the
   connector is displaced, **Then** the absolute endpoint moves and the referenced one does not move
   at all, exactly as 082 pinned it and as the pair of pictures in its B4.2 shows. The offsets this
   slice adds do not reach that displacement, and a slice that wired them would be taking
   [#143](https://github.com/andresmoschini/monospace/issues/143)'s decision.

### B4 — A description can name a reference and its two offsets

1. **Given** a description whose connector holds a reference instead of a point, **When** the
   application is given the file, **Then** it draws the diagram that reference resolves to, and a
   path given on the command line behaves exactly as it does today otherwise.
2. **Given** a description naming a reference to a shape it does not hold, **When** it is read,
   **Then** the figure holding it is absent from the picture, every other shape is drawn exactly as
   it would have been, and the run succeeds. This is B2.2 arriving on the wire, and it is the cost
   [ADR-0041](../../docs/decisions/0041-resolve-a-position-through-a-reference.md) already accepts:
   a file naming nothing that is there draws a diagram with a silent hole in it.
3. **Given** a description whose reference carries no offset, **When** it is read, **Then** it draws
   what a description naming the point it resolves to draws, byte for byte.

### B5 — The demonstration's far end hangs from a box, and the picture does not move

1. **Given** the shipped demonstration, **When** the application is run with no arguments, **Then**
   it still prints the same five captioned pictures it prints today, character for character, and
   the connector's far endpoint — the one standing at the point the shipped description spells
   outright — is now a reference to the bottom side of the box above it, carrying an offset of one
   in each axis. That box is the fifth entry, and its bottom side middle plus that offset is the
   point the terminal already stood on, which is the whole of the claim: the evidence is in the
   description rather than in the picture, and the picture not moving is what proves the two offsets
   add the way they say they do.
2. **Given** a path, **When** the application is given one, **Then** it prints one picture and
   nothing else, exactly as today, which is what `cargo xtask render` embeds.

## Edge cases

- An offset that puts the endpoint inside the box it hangs from — a horizontal offset to the _left_
  on a right side — draws it there, and the two figures compose in the shared cell by the rule two
  figures sharing a cell always obey, rather than the offset being refused:

  ```text
  ┌──┐
  │ ─┼──────
  └──┘
  ```

  Hypothetical — measured, not generated: the same connector drawn from a point one cell inside the
  box, where its arm terminal and the box's own border meet.

- An offset of nothing in one direction and something in the other: the two are independent, and
  neither is checked against the side it is measured from. A negative amount is a point on the far
  side of the anchor, which is the first edge case.
- A box one cell wide or one cell tall: its four side middles coincide in pairs by the general rule,
  and the offset is added to whichever point the pair names.
- A connector: it answers no side at all, so an offset added to nothing is still nothing — the
  restriction doing its work, and a chain of references still one link long.
- An endpoint pushed outside the window: what falls inside is drawn and the rest clipped, as any
  figure is.

## What this slice does not decide

- **What displacing a figure that holds a reference means.** B3 is the no-op 082 pinned, and §4
  states the rule this slice makes implementable: a displacement reaches the reference's offsets. It
  does not do it, because that is [#143](https://github.com/andresmoschini/monospace/issues/143).
- **How a description names the shape a reference points at, and whether a caller can name an
  identity at all.** The format names its shapes by the place they are listed, and a reference has
  to name one; whether the name written down is the identity the diagram issues or the position in
  the list is the sheet's. B5 writes a fifth entry out by hand. The model's own trigger for
  answering the second is the first slice reading a diagram from a file, and B4 is that slice.
- **Whether a caller can ask where a position resolves to.** The anchors exist to be resolved
  through and drawing is the only consumer, and the offsets do not make a second one appear. Settled
  by: the first consumer that needs the number without drawing the diagram.
- **A way to say "one cell out from that side" without naming the axis**, which the first answer
  refused: [#146](https://github.com/andresmoschini/monospace/issues/146). The constraint that issue
  carries is that it is a spelling and not a second meaning — taken at the boundary, producing the
  two screen amounts — so #143's displacement keeps its one reading.
- **Whether an attachment decides the direction a connector leaves in.** §6 says the direction is
  the caller's, and an offset widens what a caller can write without changing that. Settled by:
  [#89](https://github.com/andresmoschini/monospace/issues/89).

## Testing expectations

- **Contract** — the offset arithmetic, asked rather than drawn: a reference to a box's right side
  with a horizontal offset of two resolves to the point two cells right of that side's middle, and a
  vertical offset of one on a reference to the bottom resolves to the point one row below it. An
  `assert_ne!` on the no-offset case is what keeps a resolve that added nothing from passing.
- **Contract** — by drawing: the reference with a horizontal offset of two draws exactly what a
  connector drawn from the absolute point it resolves to draws, and the box displaced with the
  offset unchanged draws exactly what a connector drawn from the new absolute point draws — the pair
  B1.1 and B2.1 show. A box replaced by a line under an offset is the third: the offset is added to
  the line's own middle, not to the box's old side middle.
- **Contract** — non-resolution with an offset, by drawing: a reference to an absent shape and a
  reference to an anchor a kind does not answer, each carrying an offset large enough to be
  somewhere, and each asserting the figure is absent from the output and every other shape is
  unchanged. B4.2 is the same rule arriving through a file rather than through code.
- **Contract** — the demonstration: the five pictures are the ones it prints today, byte for byte,
  with the far endpoint of the tenth entry now a reference rather than a point, and the picture a
  path prints unchanged. The test pins the pictures and not the captions' wording.
- **Characterization** — none. Every rule this slice adds is a rule of the model stated in one
  sentence, which is what a contract test is for.

## Success criteria

- **SC-001**: An endpoint attached to a shape's side can be placed any number of cells away from
  that side, and draws the same picture an endpoint at the point it resolves to draws.
- **SC-002**: An offset is a gap from the side rather than a point: displacing the shape it hangs
  from carries the endpoint with it, and the gap between the border and the endpoint is unchanged.
- **SC-003**: An endpoint one whole cell clear of a box's border can be drawn, which is the
  arrangement that has no picture at all today.
- **SC-004**: A reference that resolves to nothing is not made to resolve by a large offset: the
  figure holding it is absent from the output, every other shape is unchanged, and no run fails —
  whether it came from code or from a file.
- **SC-005**: A bare run of the application prints the same five pictures it printed before, and the
  connector's far endpoint hangs from a box through a reference carrying an offset of one in each
  axis, so the evidence is in the description and the arithmetic is proved by the picture not
  moving.
- **SC-006**: Displacing a figure that holds a reference still leaves the hanging endpoint exactly
  where it was, which is the behavior 082 named and the one a caller relies on until #143.
- **SC-007**: `cargo xtask check` is green, including `monospace-diagram` compiling for
  `wasm32-unknown-unknown`.
