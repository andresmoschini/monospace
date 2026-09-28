<!-- The maintainer's decision draft is a Spanish file outside this repository. -->
<!-- cspell:ignore decisiones -->

# Phase 0 research: a shape can be removed and replaced

Seven questions. Four are open; three record what the maintainer's draft at
`monospace-drafts/81-decisiones.md` §3 already answered, with the code that settles them. None takes
an ADR: each answer lives inside one crate and is undone by changing the code that gives it. The
domain-level answers are on [decisions.md](decisions.md) and are not repeated.

## Q1: `Delta` and `displaced_by` — the type, the method, and the connector

**Decision**: `Delta { pub dx: i32, pub dy: i32 }` in a new `crates/monospace-diagram/src/delta.rs`,
deriving `Debug, Clone, Copy, PartialEq, Eq` to match `Pos` (geometry.rs:7), plus
`From<(i32, i32)>`; and `pub fn displaced_by(&self, by: Delta) -> Self` on the diagram's `Shape`,
matching on the variant so that `Box` and `Line` displace `at` while `Connector` displaces `from.at`
and `to.at` together, copying `leaving` and `terminal` through. `monospace-core` gains nothing.

**Rationale**: the model gives `Delta` a row — a horizontal and a vertical amount (§1) — and `Size`
cannot be it, being `u32` and meaning an extent; `Pos` reused would be one type meaning two things
without saying so. The connector needs no special case because its route is derived from its
endpoints and never described by the caller (connector.rs:144), so displacing both is sufficient.
The method needs no `Clone` on `Shape`: it clones per field, as `Shape::draw` already does
(shape.rs:102).

**Alternatives considered**: displacing a connector through one endpoint — a no-op on the route, and
the `// TODO: implement it` the previous system shipped; and D1's alternative on the core's trait,
which the enum would forward to anyway.

## Q2: How does a displacement move a position, when the workspace has no arithmetic at all?

**Decision**: no operator, no core method. `displaced_by` reads `at.x`/`at.y` — both `pub` on `Pos`
(geometry.rs:9-12) — and writes `Pos { x: at.x.saturating_add(by.dx), .. }` per axis, returning
`Self` rather than `Option<Self>`.

**Rationale**: `impl (Add|Sub|AddAssign|SubAssign)<` matches nothing across `crates/**/*.rs` — no
precedent to follow, none to break — and `Pos`'s public fields are why no core change is needed at
all. Saturating rather than wrapping or `None`, because the two are not equally honest: coordinates
may be negative (geometry.rs:6), so a wrapped `i32::MAX + 1` lands on `i32::MIN` and inside a window
a caller could really hold, while a saturated coordinate is past the end of any window a `u32` width
can describe and draws nothing whatever the window.

**Alternatives considered**: `checked_add` returning `Option<Pos>`, which is the core's convention —
`pos_at` (line.rs:31), `offset` (connector.rs:199), `checked_add_unsigned` (box_shape.rs:34) all
return `None` and draw nothing — but it makes an unreachable failure a second `if let` for the
caller composing `get` and `replace`. `wrapping_add`, per above.

## Q3: How do `get`, `remove` and `replace` find their shape?

**Decision**: the linear search `forward`/`backward` already do (diagram.rs:67), through one private
`fn find(&self, id: &ShapeId) -> Option<usize>`. `remove` drops the entry, `replace` overwrites
`.shape` in place — which keeps the identity and the place in the order for free. Both take
`&ShapeId`.

**Rationale**: `Placed` is a crate-private pair of an identity and a shape (diagram.rs:30), so both
are expressible without widening anything or naming a place in the order a second way. `add` already
increments `next` before use (diagram.rs:58), so SC-005 needs no code beyond not decrementing it.

**Alternatives considered**: `Vec::retain`, also linear, hiding which entry went; swapping the
removed entry with the last, changing the order for a removal the model says changes only the
holding; a `HashMap` keyed by identity, which `ShapeId` cannot be — it derives no `Hash`.

## Q4: What does `Shape` derive, and what does that cost?

**Decision**: `Shape` gains `Clone, PartialEq, Eq` alongside its `Debug` (shape.rs:38).

**Rationale**: B4.1 asks a caller to compare what `get` returned with what it added, and B3.4 asks a
displaced-by-nothing figure to come back equal to itself; neither is expressible without
`PartialEq`, and those two tests are the cheapest way to say it. Every field already supports it:
`Pos`, `Size`, `Direction`, `Orientation`, `Stroke` (stroke.rs:5), `Glyph` (glyph.rs:26) and
`Terminal` (connector.rs:118) all derive `Debug, Clone, PartialEq, Eq`.

**Alternatives considered**: asserting the figures through drawn pictures, which the spec prefers
everywhere else and which cannot express `get` at all; a hand-written `PartialEq` for rules a derive
would get wrong, of which there are none.

## Q5: Is `get` justified, now that the demonstration does not read back?

**Decision**: yes, on the general argument, and the sheet says so rather than claiming a consumer
that does not exist. This is the draft's own risk 2 arriving.

**Rationale**: the draft justified it by the demonstration; the 2026-09-28 clarification settled
that the demonstration names `#1` by hand (B5.9) and never reads back, because
`Description::into_diagram` discards the identities `add` returns (description.rs:231). What
survives is the spec's **Input** argument — a caller holding only an identity cannot displace or
replace anything without reading the figure first, and the draft's §1 composition is exactly that
call — so D2's confidence is medium-high, not high.

**Alternatives considered**: dropping `get` and letting the caller hold its own figures, leaving a
caller that received a diagram unable to act on one; `ids()` instead, which answers a different
question, issue #86.

## Q6: What does the demonstration grow into, and what moves with it?

**Decision**: four captioned pictures, the fourth appended after the pair rather than interleaved
(B5.4); picture three displaces the figure picture two moved, by a fixed delta the demonstration's
own code carries and marks in a comment as a demonstration-only assumption (B5.8) — the treatment
the reorder's already gets at main.rs:98.

**Rationale**: the draft said three; the clarification answered that the removal is deliberate, so
the draft's count is superseded and the sheet does not re-ask it. Three pictures move in the tests:
`demonstrated_pictures` (main.rs:186) splits once on `\n\n` into a two-tuple and becomes an n-way
split, and since the new pictures are appended below rather than inserted, the coordinates `char_at`
reads at `y + 1` (tests/cli.rs:338) do not move — worth saying rather than leaving to be found.

**Alternatives considered**: reading the identity back through `into_diagram` rather than naming
`#1`, a change to the CLI's internal conversion for a figure the description already names by
position; a field in the description format, which B5.8 forbids and
[ADR-0035](../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md) with it.

## Q7: Which of 080's records contradicts the code about `ShapeId`?

**Decision**: none to correct. The one artifact a caller reads is already right, and the four that
still say "no constructor" are 080's record of what was agreed at the time.

**Rationale**: `4a851ac` settled this three commits ago, in its own message: a contract describes
what a caller sees _now_, so one contradicting the code is worse than none, while "a merged spec is
not rewritten to match a correction found later". The spec's clarification names the contract, and
the contract is what needed it — `contracts/diagram-api.md:16-19` now shows
`pub fn new(text: impl Into<String>)` and 29-33 says it is a way to spell an identity rather than to
have one issued, which is what the code does: `ShapeId::new` is `pub` at diagram.rs:18 and the
demonstration writes `#1` with it at main.rs:99. The other four — `data-model.md:22-23`,
`plan.md:12`, `research.md:12,16`, `tasks.md:62-63` — record what 080 agreed, and the maintainer has
already ruled on them.

**Alternatives considered**: correcting the four as well, which is what this question first proposed
and which `4a851ac`'s stated rule rejects; and 081's contract citing the code rather than the file,
which is what part two should do either way.
