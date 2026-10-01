<!-- The feature branch is named after the issue title, truncated by `cargo xtask spec`. That -->
<!-- truncation landed inside a word, so the spell checker is told to expect it. -->
<!-- cspell:ignore referen -->

# Feature Specification: Displacing a figure that holds a reference moves it

**Feature Branch**: `143-displacing-a-figure-that-holds-a-referen-deciding` | **Issue**:
[#143](https://github.com/andresmoschini/monospace/issues/143) | **Created**: 2026-10-01 |
**Status**: Draft

**Input**: Issue #143 — a connector with an endpoint hanging from a box is displaced, and the
displacement has no coordinates to add to: that endpoint's position is a reference, and a reference
held no offsets until [#83](https://github.com/andresmoschini/monospace/issues/83). So the figure
does not move at all, which is the one outcome a caller cannot tell from a bug. With the offsets
landed, this slice decides what displacing a figure that holds a reference means — §11's third open
question, whose own trigger has now fired — and makes the figure move. Part of
[#62](https://github.com/andresmoschini/monospace/issues/62), on top of
[#82](https://github.com/andresmoschini/monospace/issues/82) and
[#83](https://github.com/andresmoschini/monospace/issues/83).

## What this slice implements

Three sections of [`docs/diagram-model.md`](../../docs/diagram-model.md), **none amended** — §4
already states the rule, exactly as 083's slice needed no amendment.

- [§4 _Positions_](../../docs/diagram-model.md#4-positions) — the displacement paragraph, the one
  083 named as the slice that was left. This slice makes it true.
- [§9 _Changing a diagram_](../../docs/diagram-model.md#9-changing-a-diagram) — "What a displacement
  means for a position that is a reference is stated under _Positions_", which is this slice.
- [§11 _Open questions_](../../docs/diagram-model.md#11-open-questions) — _What does displacing a
  figure that holds a reference mean?_ comes out. Its own trigger — "the first slice that has a
  reference to displace" — has fired twice, in 082 and in 083, and this is that slice. What replaces
  it is a sentence naming what that first slice settled and what it left: that the two directions
  are not the same rule, and that moving a set is still open because nothing moves a set.

Nothing here amends §1 or §10. §1's `Delta` row already says one delta is both how far a figure
moves and how far a reference stands from a side, which is the whole reason the rule is one addition
rather than a second kind of displacement; and §10 grows a property only when a property has been
decided, which the pair under **Edge cases** has not.

Deciding this is domain-level: a caller can see where a figure lands, so the answer is the
maintainer's and takes a record in [`docs/decisions/`](../../docs/decisions/README.md) beside
[ADR-0041](../../docs/decisions/0041-resolve-a-position-through-a-reference.md), which is the record
the same position rests on today. It is **P1** below, and the spec states the rule §4 states rather
than choosing between the alternatives the sheet weighs.

## Clarifications

### Session 2026-10-01

- Q: [P1] Where does the displacement reach — a reference's offsets, or the place the reference
  resolves to? → A: **The offsets**, which is what §4 states. Not the place it resolves to, which
  would move another figure and contradict both "a displacement is a property of one figure rather
  than of a diagram" in §4 and "nothing cascades" in §9; and not a refusal, which would leave
  [#62](https://github.com/andresmoschini/monospace/issues/62)'s second-from-last bullet
  unimplementable. §11's question comes out and the record is the ADR this slice needs.
- Q: [P2] A caller that displaces both the figure a reference hangs from and the figure holding it —
  what does it get? → A: **Not this slice's question, and not yet a decision.** Nothing displaces
  more than one figure: there is no selection, no group, and no consumer that moves a set, so no
  caller can reach the arrangement. The arithmetic is derived rather than chosen and sits under
  **Edge cases**; what would force an answer is the first consumer that displaces more than one
  figure at a time, which is a selection.
- Q: [P3] Does the slice reach the description format, or stop at `monospace-diagram`? → A: **The
  demonstration moves, and it costs no format change at all.** The bare run already displaces
  figures in `monospace-cli`'s own code — `main.rs` carries its deltas deliberately, "rather than a
  field in the description format or an argument on the binary" (B5.8,
  [ADR-0035](../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md)) — so the
  evidence is a **sixth** captioned picture: the fifth rehangs the arrow at both ends, and
  displacing it there grows both offsets at once while both boxes stand still. Measured today, that
  step draws a picture **byte for byte identical** to the fifth, because both endpoints are
  references and a displacement reaches neither: the shipped demonstration reproduces the issue's
  own complaint.

## Behavior

The arrangement every scenario below is about, drawn once so the pictures can be read against it: a
box four cells by three at the origin, and a connector whose `from` is a reference to that box's
right side carrying an offset of nothing, leaving rightward, with its `to` at `{7, 1}` leaving
leftward. Both terminals are arm terminals.

```text
┌──┐
│  ├────
└──┘
```

Hypothetical — nothing displaces anything in a description, so no code draws this one; it is the
same arrangement [082's slice](../082-a-connector-endpoint-hangs-from-a-box-s/spec.md) and 083's
already draw with the endpoint written as the point it resolves to.

### B1 — A displacement reaches a reference's offsets

1. **Given** the arrangement above, **When** the connector is displaced two cells down, **Then**
   every position it holds lands two cells lower and the figure draws as a translation of itself:

   ```text
   ┌──┐
   │  │
   └──┘
      ─────
   ```

   Hypothetical — measured, not generated: drawn today by building the two positions the rule
   yields, a reference to the same side with an offset of `(0, 2)` and a `to` at `{7, 3}`, because
   no code displaces into them. Which identity the reference names and which side it hangs from are
   exactly what they were; only the two amounts change.

2. **Given** the arrangement above, **When** the connector is displaced four cells right, **Then**
   its endpoint stands four cells to the right of the border the box still draws, the box does not
   move, and the gap between them grew by four. A displacement is a property of one figure, and
   which figure that is has to be visible from the picture.
3. **Given** a figure holding two references rather than one, **When** it is displaced, **Then**
   both offsets grow by the same amount and the figure is translated rigidly — the rule above,
   twice, and one test.

### B2 — The two directions are one slice and two rules

1. **Given** the arrangement above, **When** the **box** is displaced four cells right, **Then** the
   endpoint follows the side it hangs from, the gap between border and endpoint is unchanged, and
   the route keeps its length:

   ```text
       ┌──┐
       │ ┌┼┐
       └─┴┴┘
   ```

   Hypothetical — measured, not generated, and this is
   [#83](https://github.com/andresmoschini/monospace/issues/83)'s rule unchanged: the connector here
   is degenerate, because the endpoint arrived where the free end already stood.

2. **Given** the arrangement above, **When** the **connector** is displaced two cells down, **Then**
   the figure does not bend: today it does, which is the whole of what this slice replaces.

   ```text
   ┌──┐
   │  ├─┐
   └──┘ │
        └──
   ```

   Measured, not generated — what `monospace-diagram` draws on 2026-10-01 for this exact
   displacement: the endpoint stays welded to the border, the free end drops two cells, and the
   route takes a bend to join them. Every figure here is one the caller asked to move and none of
   them did.

3. **Given** B2.1 and B2.2 as one test, **When** it runs, **Then** displacing the figure a reference
   hangs from moves the endpoint, and displacing the figure that holds one moves the endpoint too.
   Both rules are in one test because an implementation that reached the same place in both
   directions — by rewriting the shape a reference names, say — would satisfy each on its own and
   draw neither picture above.

### B3 — The arrow moves, and two pictures carry the evidence

1. **Given** the shipped demonstration's fifth picture — the arrow rehung from the box it already
   pointed at, and that box displaced four cells right — **When** the arrow itself is then displaced
   two cells down, **Then** both of its offsets grow by two, both boxes stand exactly where they
   stood, and the sixth picture is the fifth with the arrow two rows lower:

   ```text
                     ┌──┐    ┌──┐    +--+
     ┌──┐ ┌──┐       │ ┌┼─┐  │ ++-+  | ┌┼─┐
     │░░│ │░░│  ┌──┐ └─┼┘ │  └─+┘ |  +-┼+ │
     └─┬┘ └──┘  │░░│   └──┘    +--+    └──┘
       │        └──┘
   ┌───┼───        ───┐
   │   │              │  ▲
                     └──┘
   ```

   Hypothetical — measured, not generated. **Today's answer is the fifth picture again, byte for
   byte**, which is how the measurement was taken: the step added to `monospace-cli` first and run,
   and the two blocks compared. Both endpoints are references at that point in the demonstration,
   which is why a displacement reaches neither and the picture is unchanged — the silent no-op this
   slice exists to end, reproduced in the shipped demonstration. The block above is what the rule
   yields, built by growing both offsets by two and drawing that.

2. **Given** the demonstration as shipped, **When** it is read, **Then** `assets/demo.json` is byte
   for byte the file it is today: the sixth picture is a change `monospace-cli` makes in its own
   code, beside the four it already makes, and the format grows no field.
3. **Given** the first five pictures, **When** they are compared with the ones printed today,
   **Then** all five are byte for byte what they are, the first of them still the one a path prints
   on its own, and the run still prints one picture for a path and nothing else. The demonstration
   grows by one picture and by nothing else.
4. **Given** the crate's gallery test `an_endpoint_hangs_from_a_side_and_follows_it`, **When** it
   runs, **Then** it holds three blocks — the arrangement as written, the box displaced four cells
   right, and the connector displaced two cells down — and the third is **drawn by `displaced_by`
   rather than built by hand**, so a rule that stops holding drops the snapshot instead of nothing.
   It is the arrangement the two blocks beside it already use, with the direction this slice adds
   next to the one that already worked.

## Edge cases

- A reference naming a shape the diagram does not hold, or an anchor its kind does not answer: the
  offsets still grow, because a displacement builds a value and cannot fail, and the figure still
  draws nothing. A reference that resolved to nothing before the displacement resolves to nothing
  after it — and a connector answers no anchor at all, so a chain of references stays one link long.
- An offset already carrying the endpoint inside the box it hangs from: drawn there, composing with
  the border as [#83](https://github.com/andresmoschini/monospace/issues/83)'s edge case says, and a
  displacement grows that gap like any other.
- A displacement of nothing: a reference comes back equal to itself, which is a statement about the
  arithmetic and not an exception.
- **The box displaced two down and then the connector displaced two down**: the connector ends up
  four down with its gap grown by two, because each displacement belongs to one figure and both
  figures moved. Derived from the rule rather than decided by it, and stated here so a reader is not
  surprised by it and so a later slice that changes it has to say so rather than discover it: **no
  displacement moves an anchor and its holder together and keeps the gap.** Moving the box carries
  the endpoint; moving the endpoint grows the gap.
- A displacement large enough to saturate an offset: the offset gets the same saturating addition
  every other coordinate here gets, so the endpoint travels to the end of the window's coordinates
  and **stays there** — a later displacement back does not return it, since the arithmetic does not
  remember where it was. An absolute position saturating draws nothing; an offset saturating draws a
  very far away endpoint, which is the one asymmetry between the two.
- An endpoint pushed outside the window by the gap growing: what falls inside is drawn and the rest
  clipped, as any figure is, and nothing reports it.

## What this slice does not decide

- **Whether moving a set of figures keeps their gaps.** The edge case above is what a caller gets
  who displaces two figures in a row, and no caller does: there is no selection, no group and no
  consumer that moves more than one figure, so nothing forces an answer. Settled by: the first
  consumer that displaces more than one figure at a time, which is a selection.
- **Whether an attachment decides the direction a connector leaves in, and whether a reference may
  reach anything but a connector's endpoint.** A displacement reaches the offsets and leaves
  `leaving` and `terminal` exactly as they were, so a displaced connector may leave a side in a
  direction that draws something nobody wants. The rule is stated about a position, so it lands
  wherever a reference later appears, and whether a chain longer than one link composes or collides
  and what a cycle means are [#89](https://github.com/andresmoschini/monospace/issues/89)'s.
- **Saying "one cell out from that side" without naming the axis.** The displacement reads the
  offset in the screen axes, which keeps it one addition, so a gap that means a distance from the
  side has no spelling and this slice gives the offset no second meaning.
  [#146](https://github.com/andresmoschini/monospace/issues/146).
- **Reporting a figure the diagram could not draw.** A displacement can move a figure out of the
  window or grow a gap that reaches nothing, and neither is a fault to report here:
  [#88](https://github.com/andresmoschini/monospace/issues/88).
- **How a caller reaches a displacement at all.** `Shape::displaced_by` and `replace` are the
  surface 082 established, and §9's table of five changes does not grow a sixth one.

## Testing expectations

- **Contract** — the rule, asked and then drawn: displacing a connector whose `from` holds a
  reference grows that reference's offset by the delta on each axis, leaves its identity and its
  anchor alone, and moves the absolute endpoint by the same delta. Compared by value, with an
  `assert_ne!` on the value that moved — a displacement that changed nothing would satisfy "a
  reference comes back equal to itself" by doing exactly that, which is the bug this slice exists to
  end — and then by drawing against the connector built from the two positions the rule yields,
  which is B1.1 and B1.2.
- **Contract** — both directions in one test, B2.3, for the reason it gives.
- **Contract** — a reference that resolves to nothing still resolves to nothing after a
  displacement, every other shape draws unchanged, and no run fails. The offsets having grown is
  asserted too: a displacement that left them alone would satisfy this test by doing what it does
  today.
- **Contract** — the two derived arrangements, as the edge cases state them: an offset that has
  saturated stays saturated and a displacement back does not return it; and displacing the figure a
  reference hangs from and then the figure holding it leaves the connector twice as far down with
  its gap grown. Both pinned rather than described, so a later slice that changes either has to say
  so rather than discover it in a picture.
- **Contract** — the demonstration: six captioned pictures, the sixth the fifth with the arrow two
  rows lower and both boxes where they were, and the first five byte for byte what they are today. A
  test pins that the sixth differs from the fifth **only** in the cells the arrow holds before and
  after, which is the shape 083's own fifth-picture test uses and the only claim that says the boxes
  stood still. No caption's wording is pinned, and a path still prints one picture.
- **Contract** — the gallery: `an_endpoint_hangs_from_a_side_and_follows_it` holds a third block,
  the arrangement above with the connector displaced two cells down, reached through `displaced_by`
  and not by building the two positions by hand. Its two existing blocks are byte for byte what they
  are. This is the one picture in the slice a broken rule drops rather than leaves stale, so it is
  pinned as a snapshot rather than drawn into a document.
- **Characterization** — none, and none moves. The connector sweep builds its descriptions from
  shape lists and never reads the shipped demonstration, so no case in it moves.
- **What this slice rewrites rather than contradicts.** Four places declare the no-op this slice
  reverses and all four become false with B1, so they are rewritten in the slice rather than left
  contradicting it. `position.rs` carries two rustdoc paragraphs calling it "a deliberate no-op
  rather than an omission" and "a silent no-op on purpose" and naming this issue as the decision
  that would change it, and one test named
  `a_displacement_moves_a_point_and_leaves_a_reference_alone` pins it by value. `shape.rs:179-183`
  carries the fourth, and it is the one worth naming: `Shape::displaced_by` is `pub` where
  `Position::displaced_by` is `pub(crate)`, so it is the paragraph a caller reads first, and it is
  the only one of the four that asks the question rather than answering it — it ends on "what
  displacing such a figure should mean in general is #143's to settle".

## Success criteria

- **SC-001**: Displacing a figure that holds a reference moves every position it holds by the same
  amount, so the figure draws as a translation of itself and never as the bent route it draws today.
- **SC-002**: The two directions are distinguishable from outside and neither broke: displacing the
  figure a reference hangs from still carries the endpoint with it and leaves the gap unchanged —
  the rule [#83](https://github.com/andresmoschini/monospace/issues/83) pinned, asserted beside
  SC-001 rather than separately — while displacing the figure that holds the reference moves the
  endpoint and leaves the other figure where it was.
- **SC-003**: A figure holding a reference that resolves to nothing is still not drawn after the
  displacement, every other shape is unchanged, and no run fails.
- **SC-004**: The shipped demonstration gains a sixth picture in which the arrow is displaced two
  rows and both boxes stand still, `assets/demo.json` and the description format are untouched, the
  first five pictures are byte for byte what they were, and a path still prints one picture and
  nothing else — so the evidence for the rule is in the shipped run and the cost of carrying it is
  one picture rather than a new field in a format twenty-four markers read. **And the crate's
  gallery gains the same rule beside the two blocks it already holds**, drawn by the code this slice
  changes, so a rule that breaks drops a snapshot there and nothing in a document — the one asks
  whether the defect was real, the other what the rule draws.
- **SC-005**: Every tracked picture is byte for byte what it was. No `<!-- render: -->` marker draws
  the demonstration's fifty-by-thirteen canvas — the largest is twenty-four by nine — so none of
  them is stale, and none moves.
- **SC-006**: `cargo xtask check` is green, including `monospace-diagram` compiling for
  `wasm32-unknown-unknown`.
