# Feature Specification: Taking a shape out leaves what hangs from it not drawn, and the question is written down

**Feature Branch**: `142-taking-a-shape-out-leaves-the-figures-th-deciding` | **Issue**:
[#142](https://github.com/andresmoschini/monospace/issues/142) | **Created**: 2026-10-01 |
**Status**: Draft

**Input**: Issue #142 — a box a connector hangs from is taken out, and the connector's reference
names an identity the diagram no longer holds, so the connector stops being drawn and nothing says
why. That silence is right while a reference may legitimately not resolve yet, and wrong for a shape
that is gone. The issue's own answer is to freeze every position holding a reference to the removed
shape at the absolute point it was resolving to, so everything hanging would stay drawn exactly
where it was. **This slice takes that answer and does not implement it.** It leaves the rule the
model already states, puts the question where a reader will meet it, and grows the shipped
demonstration by the one picture that makes the gap visible today. Part of
[#62](https://github.com/andresmoschini/monospace/issues/62), on top of
[#81](https://github.com/andresmoschini/monospace/issues/81),
[#83](https://github.com/andresmoschini/monospace/issues/83),
[#88](https://github.com/andresmoschini/monospace/issues/88) and
[#143](https://github.com/andresmoschini/monospace/issues/143).

## What this slice implements

Three sections of [`docs/diagram-model.md`](../../docs/diagram-model.md), the first **untouched**
and the other two **amended**.

- [§4 _Positions_](../../docs/diagram-model.md#4-positions) — nothing added and nothing corrected.
  Its two-row table already answers a removal: a shape the diagram does not hold resolves to
  nothing, and a figure whose position does not resolve is not drawn. This slice is that paragraph
  shown in a picture, and a rule it does not change.
- [§9 _Changing a diagram_](../../docs/diagram-model.md#9-changing-a-diagram) — **amended**. The
  removal paragraph keeps its rule and gains a line naming where the open question lives, so a
  reader who takes the rule for settled is told otherwise at the point they would take it.
- [§11 _Open questions_](../../docs/diagram-model.md#11-open-questions) — **two questions added**:
  the one the issue asks, and the one behind it. §11 carries each in a line; B3 carries the second
  one step by step, because it is invisible until somebody has tried to answer the first.

One thing this adds that the model does not describe, because it is not the model's: the shipped
demonstration grows a seventh captioned picture. That is `monospace-cli`'s own code rather than a
field in the description format, for the reason 143's slice put its sixth picture there
([ADR-0035](../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md)), and
`assets/demo.json` grows nothing.

## Clarifications

### Session 2026-10-01

- Q: [P1] The slice takes no decision, so what does it leave behind as proof? → A: **The question
  written down, plus a seventh picture in the demonstration.** `cargo run -p monospace-cli` prints
  the arrow and the box it hung from both gone, so the gap is something a reader sees rather than
  something they have to take on faith. The narrower option was rejected because a rule recorded
  only in a test is invisible to the person the rule is about, and this repository's evidence for a
  rule has been a picture in the shipped run since 080.
- Q: [P2] Is the question one or two? → A: **Two, written separately, and the second one step by
  step.** Answering the first alone gets you to "freeze the hanging positions at the point they
  resolved to", and the second question is then found rather than avoided: freezing means reaching
  into figures the removal did not name, and the model says in §9 that nothing is rewritten and
  nothing cascades. B3 is the evidence that it is one question and not a preference, and it is in
  the spec rather than on the sheet because a sheet entry that weighs a decision nobody has seen the
  shape of is a sheet entry nobody can answer.
- Q: [P3] Does §9's removal paragraph get a note, or is the question left to §11? → A: **A note.**
  The paragraph is where a reader takes the rule for settled, and §11 is where a reader goes when
  they are looking for what is unsettled. Nothing in the note claims the other rules of this model
  are settled — no rule here is final, which is what the constitution's _Understanding changes_
  already says — it says only that this one has a question standing next to it.

## Behavior

The arrangement every scenario below is about, drawn once so the pictures can be read against it: a
box four cells by three at the origin, a box three cells by three at `{8, 0}`, and a connector whose
`from` is a reference to the first box's right side carrying an offset of nothing, leaving
rightward, with its `to` at `{8, 1}` leaving leftward. Both terminals are arm terminals.

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 12, "height": 3 } },
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "box", "id": "#2", "at": { "x": 8, "y": 0 }, "size": { "width": 3, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": "#3",
      "from": { "at": { "kind": "reference", "shape": "#1", "anchor": "right",
          "offset": { "dx": 0, "dy": 0 } },
        "leaving": "right", "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "point", "x": 8, "y": 1 }, "leaving": "left",
        "terminal": { "kind": "arm" } },
      "stroke": "light" } ] }
-->

```text
┌──┐    ┌─┐
│  ├────┤ │
└──┘    └─┘
```

<!-- /render -->

### B1 — Taking a shape out draws nothing that hung from it

1. **Given** the arrangement above, **When** the box the arrow hangs from is taken out, **Then** the
   arrow is not drawn at all — the end that is a plain point, and could still resolve on its own,
   goes with it — and the other box is drawn exactly as it would have been:

   ```text
           ┌─┐
           │ │
           └─┘
   ```

   Measured, not generated, and a `<!-- render: -->` marker cannot reach it: a description file adds
   shapes and has no field that takes one out, so nothing a file carries describes a removal. The
   carrier for a picture of a change is the crate's gallery
   ([ADR-0064](../../docs/decisions/0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md)).

2. **Given** the arrangement above with that box taken out, **When** a box is put back under the
   same identity, **Then** the arrow draws again and the whole picture is byte for byte the one
   above it. Measured on this branch, and the reason a later reader should check it rather than
   assume it: §9 says re-adding a shape with the same identity would make the references resolve
   again, and this is the slice that finds out whether it can.
3. **Given** the arrangement above with that box taken out, **When** a shape is then added the
   ordinary way, **Then** the identity handed back is not the removed one — `#4` where `#1` was
   taken out — the new shape is drawn where the box stood, and the arrow is still not drawn. Nothing
   about the hole is repaired, and a caller who re-adds without naming the identity sees a diagram
   that looks the same and hangs from nothing.

### B2 — The demonstration grows by one picture

1. **Given** the shipped demonstration, **When** the application is run with no arguments, **Then**
   it prints **seven** captioned pictures, and the seventh is the sixth with the box the arrow hangs
   from taken out:

   ```text
   With the arrow displaced as well:             With the box the arrow hangs from taken out:
                     ┌──┐    ┌──┐    +--+                          ┌──┐    ┌──┐    +--+
     ┌──┐ ┌──┐       │ ┌┼─┐  │ ++-+  | ┌┼─┐        ┌──┐ ┌──┐       │ ┌┼─┐  │ ++-+  | ┌┼─┐
     │░░│ │░░│  ┌──┐ └─┼┘ │  └─+┘ |  +-┼+ │        │░░│ │░░│       └─┼┘ │  └─+┘ |  +-┼+ │
     └─┬┘ └──┘  │░░│   └──┘    +--+    └──┘        └─┬┘ └──┘         └──┘    +--+    └──┘
       │        └──┘                                 │
   ┌───┼───        ───┐                          ┌───┼───
   │   │              │  ▲                       │   │
   │                  └──┘                       │
   ```

   Measured, not generated, and by the step added to `monospace-cli` and run on 2026-10-01. The two
   blocks are the sixth and the seventh at the same size, and **the difference is twenty-two cells
   and no others**: the box's twelve and the arrow's ten, counted against each other rather than
   read off either. The caption beside the seventh is the one this slice proposes and no test pins
   it.

2. **Given** the first six pictures, **When** they are compared with the ones printed today,
   **Then** all six are byte for byte what they are, the first is still the one a path prints on its
   own, and a path still prints one picture and nothing else. The demonstration grows by one picture
   and by nothing else.
3. **Given** the demonstration as shipped, **When** it is read, **Then** `assets/demo.json` is byte
   for byte the file it is today: the seventh picture is a change `monospace-cli` makes in its own
   code, beside the six it already makes, and the format grows no field.

### B3 — Two rows of the model's table, and a removal is one of them

§4's table has two rows, and the three routes below are how one arrives at each. They are written
out step by step because the second question in §11 is worth nothing until somebody can see that the
three are indistinguishable from outside.

1. **Route A — the shape was never there.** **Given** a connector whose `from` names an identity
   nothing holds, **When** the diagram is drawn, **Then** the arrow is not drawn and every other
   figure is drawn as it would have been. This is row one, and
   [ADR-0041](../../docs/decisions/0041-resolve-a-position-through-a-reference.md) settled it: a
   diagram is built in whatever order its user chooses, and a reference to a shape that is coming is
   a normal state rather than a fault.
2. **Route B — the shape was there and was taken out.** **Given** the same connector over the same
   box, **When** the box is taken out, **Then** the arrow is not drawn and every other figure is
   drawn as it would have been. This is row one as well, reached from the other side, and it is B1.
   **The diagram cannot tell A from B**: both are a reference naming an identity `get` does not
   find, and neither leaves a trace.
3. **Route C — the identity is still held, and the figure there answers no side.** **Given** the
   same connector over the same box, **When** a connector is put under that same identity in place
   of the box, **Then** the arrow is not drawn either, and `get` still finds the identity the
   reference names. This is row two, and it is the sharpest of the three: not a missing shape but a
   present one that cannot answer.
4. **Given** those three routes, **When** each is drawn, **Then** the arrow's six cells are gone in
   all three — the four `{4, 1}` through `{7, 1}` its route writes and the two borders `{3, 1}` and
   `{8, 1}` it turns — the whole picture is the far box alone in A and B, and in C the only cells
   beyond that are the replacement's own. **The six is measured, and read as the difference between
   this arrangement and the same arrangement with no connector in it rather than off either
   picture.** The ten belongs to B2.1's arrow, where measurement confirms it, and two arrangements
   sharing a count is how a number from one reached the other. **The three are not compared as
   buffers**, because C is not one — the test compares the arrow's footprint and says so. What the
   three show is that **nothing in the diagram records which route happened**, and that is the whole
   of §11's second question: any answer that treats B differently from A has to begin by remembering
   something the diagram does not currently remember.

## Edge cases

- **A shape nothing references.** Taking it out changes nothing at all, and the demonstration's
  fourth picture already shows it: `#1` is not a shape any position names. Derived from the rule
  rather than decided by it, and it is the reason the arrangement B1 and B2 are about is **not**
  reachable in the shipped demonstration today — the seventh picture is what makes it reachable.
- **Two connectors from one box.** Both are not drawn, and nothing else changes. Measured.
- **A figure put back under the removed identity that answers no side.** Still not drawn, and the
  identity being present changes nothing. That is B3.3 read from the other side.
- **A figure put back under the removed identity as a kind that does answer the side** — a line
  where a box stood. The arrow draws again, hanging from the line's far end rather than where the
  box answered. Measured. §9's "re-adding a shape with the same identity" is about a shape drawing,
  not about it being the same shape.
- **A gap large enough to saturate.** Still nothing: `Position::resolve` returns before the
  addition, so a reference that resolved to nothing resolves to nothing however large its offset,
  and the figure is not drawn either way. ADR-0041's rule, unchanged.
- **The arrow taken out before the box.** The box goes on drawing, and the arrow was not what the
  removal was about. Two removals in either order leave the same picture, which is the order rule
  rather than a question about references.
- **A shape put back under the removed identity, displaced.** The arrow draws again and hangs from
  wherever the new figure stands, which is
  [#83](https://github.com/andresmoschini/monospace/issues/83) and
  [#143](https://github.com/andresmoschini/monospace/issues/143) unchanged. A displacement is a
  property of one figure, and a figure that is not in the diagram cannot be displaced.

## What this slice does not decide

- **Whether anything should happen at all.** That is §11's first question and it is the issue's own:
  should what hung from a removed shape stay drawn where it stood. The issue proposes freezing each
  hanging position at the absolute point it was resolving to, and this slice does not do that, does
  not refuse it, and does not cost it anything — the positions carrying those references are values
  today and rewriting them is arithmetic a later slice can do. Settled by: the first consumer that
  takes a shape out and expects the picture to keep what hung from it. There is no such consumer,
  and the one coming is an editor.
- **How anything would tell a removal from a shape that is not there.** §11's second question, and
  B3 is its evidence. **It cannot be answered first**: whatever the first answer turns out to be,
  this is a consequence of it, and a sheet entry that answered them in the other order would be
  answering a question whose answer had not been chosen. So they go on one sheet or on two in that
  order, and the sheet says which — and that is a question about the shape of the work, not a
  decision about the model, which is why it belongs in _Decisions_ of this plan rather than in the
  behavior above.
- **Where the record goes.**
  [ADR-0041](../../docs/decisions/0041-resolve-a-position-through-a-reference.md) chose this silence
  at `load-bearing`, and whether a rule about removals revises it or sits beside it is the sheet's.
  The constitution's test settles it when the sheet reaches it: if a record cannot be cited without
  citing the other, the two are one record.
- **Telling anybody why the arrow is not there.** The issue says "nothing says why", and that half
  is [#88](https://github.com/andresmoschini/monospace/issues/88)'s: a way to ask a diagram which of
  its shapes it could not draw. Adding it here would put a diagnostic query in a slice whose subject
  is a rule, and ADR-0041 named the query as the thing to add once a consumer existed — a consumer
  #88 has and this slice does not.
- **What a reference to a shape that was never there should do.** Settled, and B3 puts that route
  beside the others because it is the evidence for the second question rather than because it is in
  doubt. Nothing, no error, no report: see ADR-0041.
- **Whether a reference widens to any shape's position**
  ([#89](https://github.com/andresmoschini/monospace/issues/89)), and **editing an identity after
  the fact**. §11's other two questions, and nothing in this slice's arrival settles either.

## Testing expectations

- **Contract** — the rule as it is, asked and then drawn: taking out a shape a connector hangs from
  leaves the figure not drawn, every other figure byte for byte what it would have been, and no run
  fails. The end that resolves is named in the test beside the end that does not, because
  `a_connector_with_an_endpoint_that_does_not_resolve_is_not_drawn_at_all` already holds this by
  value and the only thing new here is that a removal reaches it.
- **Contract** — the three routes of B3 as one test, comparing the arrow's footprint across the
  three rather than the whole buffer, with B3.4's reason written at the comparison. An
  implementation that treated a removal differently from an identity nothing holds would draw two of
  the three and fail, which is the point: this is the first test that can fail on a decision nobody
  has taken.
- **Contract** — the put-back and the ordinal, both directions: after a removal, `add` hands back an
  identity no shape holds, and `add_under` with the removed identity makes the figure draw again
  byte for byte what it drew before. Neither is stated in the crate today, and B1.3 is a caller
  walking into the first one.
- **Contract** — the demonstration: seven captioned pictures, the seventh the sixth with exactly the
  box's twelve cells and the arrow's ten cells gone and no other cell changed, the first six byte
  for byte what they are, `assets/demo.json` unchanged, and a path still printing one picture. No
  caption's wording is pinned. The twenty-two is asserted as a count against the sixth picture, so a
  rule that started drawing a fragment of the route would fail rather than shrink the block quietly.
- **Gallery** — one block added to `an_endpoint_hangs_from_a_side_and_follows_it`, its second line
  `change: the box taken out`, beside three that already mutate one diagram between them and so need
  no new machinery. **A picture of a change is what no description file and no generated-picture
  marker can reach**, which is why B1.1's is hand-drawn, and the gallery is the one carrier left
  ([ADR-0064](../../docs/decisions/0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md)).
  It is the one picture this slice adds that fails loudly rather than going stale.
- **Characterization** — none, and none moves. The connector sweep builds its descriptions from
  shape lists and reads no shipped file, and a figure vanishing from a diagram is a range small
  enough to read rather than one too wide to assert by hand.
- **What this slice rewrites rather than contradicts.** Two places, and each names what falsified
  it. The first names this issue: the rustdoc on
  `a_figure_put_back_under_the_referenced_identity_draws_the_connector_again` reads "`remove` frees
  an identity permanently — `add` never hands one out twice, and there is no `add_under` — so a
  removal followed by an addition cannot put anything back under the removed one's identity, and the
  reference stays unresolved for good." **Two of its three claims are false.** `add_under` arrived
  with [#148](https://github.com/andresmoschini/monospace/issues/148) and is `pub`; and measured on
  this branch, a removal followed by `add_under` puts the box back and the arrow draws again byte
  for byte, which is B1.2. The third claim is true, and it is why the case still goes through
  `replace` — the reason given in the first half of that paragraph survives while the reason given
  in the second half does not. Rewritten here rather than left standing beside a behavior it denies,
  and `add_under`'s own rustdoc already says what it is for.
- **What this slice rewrites rather than contradicts, second place** — `diagram.rs:2350-2355`, on
  `a_figure_added_under_a_spelled_identity_is_what_a_hanging_endpoint_finds`, reads "a diagram
  offers no way to name a shape into existence, so a spelled identity can only ever be the one an
  `add` is about to issue". **False for the same reason and by the same method**: `add_under` is
  `pub`, and that sentence is what the case above it is built to reach. It is
  [#148](https://github.com/andresmoschini/monospace/issues/148)'s subject rather than this slice's,
  and it is corrected here because `add_under` falsified both claims in the same way — correcting
  one and leaving this one standing would put a paragraph beside a public method that contradicts
  it, on the branch that measured it false.

## Success criteria

- **SC-001**: Running the application with no arguments prints a seventh captioned picture in which
  the arrow is gone because the box it hung from is gone, and that picture is the sixth with exactly
  the box's twelve cells and the arrow's ten cells removed and no other cell changed — while the
  first six pictures are byte for byte what they were, `assets/demo.json` is untouched, and a path
  still prints one picture and nothing else.
- **SC-002**: The gap is written down in the two places a reader looks: §9's removal paragraph says
  what a removal does and points at the open question, and §11 carries both questions — the second
  written as three routes to one picture rather than as a sentence, so it can be answered by
  somebody who has not read this issue.
- **SC-003**: The rule as it is today cannot change silently. It is pinned by tests that fail on a
  removal answering differently from a missing identity, a gallery block that fails if the picture
  of a removal moves, and both paragraphs the crate carries that `add_under` falsified are
  corrected, so a slice that takes the freeze has to say it is taking it rather than discover it in
  a picture.
- **SC-004**: `cargo xtask check` is green, including `monospace-diagram` compiling for
  `wasm32-unknown-unknown`.
