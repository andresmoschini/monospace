# Feature Specification: Give a glyph a type of its own

**Feature Branch**: `006-give-a-glyph-a-type`

**Created**: 2026-09-08

**Status**: Draft

**Input**: Maintainer request, made in Spanish and recorded here in English: give glyphs a type of
their own — a validated `Glyph` that replaces `char` throughout the catalog and the rendering
engine, so that a glyph can be a whole grapheme cluster and never a control character. The input
document is [spec 0004](../../docs/specs/0004-give-a-glyph-a-type-of-its-own.md), abandoned as a
draft when the spec home moved to Spec Kit; its behavior rules, its table of examples and its public
surface are already agreed. The decision behind them is
[ADR-0019](../../docs/decisions/0019-represent-a-glyph-as-a-grapheme-cluster.md). This is a slice
that enables other features rather than a feature of its own, and its acceptance criterion is that
the rendered output does not change.

## Clarifications

### Session 2026-09-08

- Q: What exactly counts as a control character when refusing a glyph? → A: Unicode's `Cc` category,
  which is what `char::is_control()` answers, and nothing wider.
- Q: Are two glyphs that look alike but are encoded differently the same glyph? → A: No. Text is not
  normalized, equality is by text rather than by appearance, and the rustdoc says so.

## User Scenarios & Testing _(mandatory)_

The two stories are the two increments this slice arrives in, in this order. Each one leaves the
workspace green and the rendered output untouched, and the second one changes no signature the first
one introduced — which is what makes them separable rather than one change described twice.

### User Story 1 - A validated glyph of one character (Priority: P1)

A caller needs somewhere to put "the thing a cell renders to", and today the only place is a
character. A character accepts `\n` and `\t`, neither of which occupies a cell, so it cannot say
what a glyph is allowed to be. This story is the type that says it: one character, never a control
character, with a single way in and nothing to inspect on refusal. It replaces the character in the
catalog and in the renderer at the same time, so the type is in use rather than merely declared, and
the rules this library ships get checked while the catalog is built.

**Why this priority**: it is the whole of what the two queued slices are waiting for — a cell that
holds a literal glyph, and glyph sets loaded from a file. Both put a glyph at a boundary this
library has never had, and both need a type that already exists to receive it.

**Independent Test**: construct a glyph from each single-character row of the table under
_Examples_, assert which are accepted; then build the built-in Light catalog, ask it for every key a
light cell can produce, and render the front end's diagram — the text is identical to what it was
before.

**Acceptance Scenarios**:

1. **Given** the text `"│"`, **When** a glyph is constructed from it, **Then** it is accepted and
   reads back as `"│"`.
2. **Given** the text `""`, `"ab"` or `"\n"`, **When** a glyph is constructed from it, **Then** it
   is refused, and the refusal carries nothing to inspect.
3. **Given** the built-in Light catalog and the key with `light` on top and bottom and nothing on
   the sides, **When** the key is looked up, **Then** the answer is the glyph for `"│"`.
4. **Given** the built-in Light catalog, **When** each of the fifteen rules of the Light table is
   looked up, **Then** every one of them answers a glyph; **and When** a key mentioning a stroke it
   has no rule for is looked up, **Then** there is no answer, exactly as before.
5. **Given** a rule shipped inside this library that is not a valid glyph, **When** the catalog is
   built, **Then** the failure is loud rather than a position that silently renders as a space.
6. **Given** the front end, **When** it is run before and after this story, **Then** the two outputs
   are identical byte for byte, and `crates/monospace-cli/tests/cli.rs` never appears in the diff.

---

### User Story 2 - The invariant widens to a grapheme cluster (Priority: P2)

A character is smaller than one thing a reader sees. It cannot hold `é` written as `e` followed by a
combining acute, a flag, or any emoji built from more than one code point, so a caller writing a
literal or a file declaring one would be refused for text that occupies exactly one cell. This story
widens what is inside the glyph to one grapheme cluster, which is the unit a cell actually holds,
and keeps the refusal of every control character — the two halves of the invariant are not
redundant, and `"\r\n"` is the input that proves it.

**Why this priority**: it is the half that costs a dependency, and it is separable because the first
story already settled the shape. It is second rather than first so that its diff is exactly what the
dependency buys and nothing else.

**Independent Test**: construct a glyph from each multi-code-point row of the table under _Examples_
and assert it is accepted; assert `"\r\n"` is still refused; then confirm the front end's output and
every existing assertion are untouched by the widening.

**Acceptance Scenarios**:

1. **Given** the text `"e\u{301}"` — `é` as two code points — **When** a glyph is constructed from
   it, **Then** it is accepted as one glyph.
2. **Given** a regional-indicator pair or an emoji with a modifier, **When** a glyph is constructed
   from it, **Then** it is accepted as one glyph.
3. **Given** the text `"\r\n"`, **When** a glyph is constructed from it, **Then** it is refused,
   even though it is a single grapheme cluster.
4. **Given** the text `"ab"`, **When** a glyph is constructed from it, **Then** it is still refused
   — now as two clusters rather than as two characters.
5. **Given** the public surface the first story introduced, **When** the widening lands, **Then** no
   signature and no call site has changed, and no row of the built-in table has moved.
6. **Given** the front end, **When** it is run before and after this story, **Then** the two outputs
   are identical byte for byte.

---

### Edge Cases

| Case                                                        | Expected                                                                                                         | Settled by |
| ----------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | ---------- |
| Empty text                                                  | Refused: nothing is not a glyph                                                                                  | P1         |
| A control character on its own, such as `"\n"`              | Refused                                                                                                          | P1         |
| More than one character, such as `"ab"`                     | Refused as two characters, then as two clusters                                                                  | P1, P2     |
| A character wider than one column, such as `"漢"`           | Accepted, and it shifts the rest of its row by a column — documented, not enforced                               | P1         |
| A cluster of several code points                            | Accepted: one glyph                                                                                              | P2         |
| A cluster of format characters only, such as a lone joiner  | Accepted, and it occupies no column — the same documented imperfection as a wide grapheme, with the sign changed | P1         |
| The same letter in two normal forms, `"é"` and `"e\u{301}"` | Both accepted, and they compare unequal: no normalization happens here                                           | P2         |
| A control character inside one cluster, `"\r\n"`            | Refused, even though it is a single cluster by UAX #29                                                           | P2         |
| A key the catalog does not answer                           | A space, unchanged from before the slice                                                                         | P1         |
| A rule this library ships that is not a valid glyph         | A loud failure while the catalog is built, not a space in the output                                             | P1         |

## Requirements _(mandatory)_

### Functional Requirements

The rules below are the ones already agreed in the input document, renumbered and tagged with the
story that delivers each. The mapping to the input's _Behavior_ list is given where it is not one to
one.

- **FR-001** (P1): A glyph MUST be constructible from text that is exactly one character containing
  no control character, and MUST NOT be constructible from anything else. (Input rule 1, narrowed to
  what P1 accepts.)
- **FR-002** (P1): Empty text MUST be refused. (Input rule 2.)
- **FR-003** (P1): Text of more than one character MUST be refused. (Input rule 3, narrowed; FR-011
  widens it.)
- **FR-004** (P1): Any control character MUST be refused. A control character is one in Unicode's
  `Cc` category — what `char::is_control()` answers — and the definition MUST NOT be widened to
  format characters, which would refuse the joiner that holds a multi-code-point emoji together and
  contradict FR-011. (Input rule 5, in part; FR-013 carries the rest.)
- **FR-005** (P1): An invalid glyph MUST NOT be representable: the payload stays private and
  construction is the only way in, so nothing downstream has anything to check.
- **FR-006** (P1): Refusal MUST carry no detail. There is one way to fail and nothing to say about
  it beyond "not a glyph"; the slice that loads a file is where a rejection becomes a message
  someone reads, and the context that message needs belongs to the loader.
- **FR-007** (P1): The catalog MUST hold glyphs rather than characters, and MUST answer a key with a
  borrow of the glyph it holds rather than a copy.
- **FR-008** (P1): The built-in Light catalog MUST answer every key the Light table in
  [`docs/glyph-sets.md`](../../docs/glyph-sets.md) gives a glyph for, with that glyph, and MUST
  answer no other key. (Spec 0001's rule 10, unchanged except in what the answer is made of.)
- **FR-009** (P1): Building the built-in Light catalog MUST fail loudly if one of the rules this
  library ships is not a valid glyph. That is a bug in shipped data rather than a condition a caller
  can act on, and it is the one place where failing loudly beats a refusal that renders as a space.
  (Input rule 7.)
- **FR-010** (P1): Rendering MUST collect glyphs rather than characters, and a position with no
  cell, no answer, or outside the window MUST still render as a single space.
- **FR-011** (P2): A grapheme cluster built from several code points MUST be accepted as one glyph:
  `é` as `e` followed by U+0301, a regional-indicator pair, an emoji with a modifier. (Input rule
  4.)
- **FR-012** (P2): Text of more than one grapheme cluster MUST be refused. This replaces FR-003:
  `"ab"` stays refused, for a different reason. (Input rule 3.)
- **FR-013** (P2): A control character MUST be refused even when it is part of a single grapheme
  cluster. `"\r\n"` is one cluster by UAX #29 and is refused anyway, which is why the invariant has
  two halves rather than one. (Input rule 5, completed.)
- **FR-014** (P2): The widening MUST change no public signature, no call site and no row of the
  built-in table. Its diff is the invariant and the dependency, and nothing else.
- **FR-015** (P1, P2): For any catalog whose glyphs are single characters, rendering MUST produce
  exactly what it produced before this slice — after each story, not only after both. (Input rule
  8.)
- **FR-016** (P1): Every new public item MUST carry rustdoc, and the glyph's own documentation MUST
  say what it accepts and, in the same breath, that width is not part of the invariant — a wide
  grapheme is one glyph and will shift its row. The caveat is needed from P1, since a single wide
  character is already accepted. It MUST also say that equality is by text rather than by
  appearance, which is the trap FR-018 leaves in place.
- **FR-017** (P2): Spec 0001's promise of lines of exactly `width` **characters** MUST be read from
  here on as `width` **glyphs**. It is the same rectangle and no longer the same character count.
  Nothing observable changes in this slice, since every built-in glyph is one character; the slice
  that first stores a multi-character glyph is the one that would otherwise have broken the promise
  silently.
- **FR-018** (P1): Text MUST NOT be normalized on construction. Two glyphs are equal when their text
  is equal, so `é` as U+00E9 and `é` as `e` followed by U+0301 are two glyphs that render alike and
  compare unequal. Which normal form text arrives in belongs to the caller, who knows where the text
  came from; normalizing here would need a decision ADR-0019 did not take, and a second dependency
  to carry it out.

### Key Entities

- **Glyph**: what a cell renders to — one grapheme cluster, non-empty, with no control character in
  it. Its text can be read back; its storage is private. Two of them are equal when their text is
  equal. Width and normal form are both deliberately outside the invariant.
- **GlyphCatalog**: every rule in play, now mapping a key to a glyph instead of to a character.
  Nothing else about it changes: one lookup, and no way to count or enumerate its rules.
- **GlyphKey**: unchanged. A stroke name or nothing on each of the four sides.
- **Stroke**: untouched. It keeps its public field and its lack of validation; ADR-0019 records why
  a stroke and a glyph look alike and are not.

## Public surface _(already agreed)_

Carried from the input document, which agreed it against
[ADR-0019](../../docs/decisions/0019-represent-a-glyph-as-a-grapheme-cluster.md). It is the same
after either story — that is what FR-014 asserts, and the reason the two stories are separable:

```rust
pub struct Glyph(/* private */);

impl Glyph {
    pub fn new(text: &str) -> Option<Self>;
    pub fn as_str(&self) -> &str;
}

impl GlyphCatalog {
    pub fn glyph(&self, key: &GlyphKey) -> Option<&Glyph>;
}
```

- **`new` takes text rather than a character even in P1**, which is what keeps P2 from moving a
  single call site.
- **`new` returns `Option`, not `Result`**, for the reason FR-006 gives.
- **`as_str` rather than a character accessor**, because a glyph is not always one character and the
  only thing this library does with one is write it into the rendered string.
- **`glyph` answers with a borrow**, because the catalog owns its rules and a glyph is no longer
  `Copy`; answering by value would clone once per rendered cell.
- **`Glyph` derives `Debug`, `Clone`, `PartialEq` and `Eq`** — the first and the comparisons are
  what tests assert through, `Clone` because the inside owns its storage. Not `Copy`, which is the
  cost ADR-0019 records.

## Examples

**What is a glyph.** Every row is a test, named after what it asserts.

| Text         | Constructed | Why                                                  | Settled by |
| ------------ | ----------- | ---------------------------------------------------- | ---------- |
| `"│"`        | Accepted    | one cluster, one character                           | P1         |
| `""`         | Refused     | FR-002                                               | P1         |
| `"ab"`       | Refused     | two characters, then two clusters — FR-003, FR-012   | P1, P2     |
| `"\n"`       | Refused     | FR-004                                               | P1         |
| `"e\u{301}"` | Accepted    | `é` decomposed: two code points, one cluster, FR-011 | P2         |
| `"🇦🇷"`       | Accepted    | a regional-indicator pair is one cluster             | P2         |
| `"\r\n"`     | Refused     | one cluster by UAX #29, and control characters       | P2         |

The last row is why the invariant has two halves rather than one. `"\r\n"` passes the cluster test
and has to be refused anyway, so "exactly one cluster" alone would have let through the one input
that breaks the rendered rectangle instead of merely shifting it. In P1 it is refused for the
uninteresting reason that it is two characters; the case only becomes load-bearing in P2.

**From a key to a glyph.** The example spec 0001 named "From a cell to a character", now one step
longer. The key with `light` on top and bottom and nothing on the sides answers a glyph whose text
is `"│"`:

```rust
let catalog = GlyphCatalog::light();
let key = GlyphKey {
    top: Some(Stroke::from("light")),
    right: None,
    bottom: Some(Stroke::from("light")),
    left: None,
};

assert_eq!(catalog.glyph(&key).map(Glyph::as_str), Some("│"));
```

This is the shape every existing assertion on a looked-up glyph takes after P1: `Some('│')` becomes
`Some("│")` through the text accessor. The edit is mechanical, and P2 does not touch it again.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: `cargo run -p monospace-cli` prints exactly what it printed before the slice, byte for
  byte, measured after each story rather than only at the end, and
  `crates/monospace-cli/tests/cli.rs` does not appear in the diff of either.
- **SC-002**: every one of the seven rows of the table under _Examples_ is covered by a test named
  after what it asserts.
- **SC-003**: one test walks all fifteen rules of the built-in Light table and asserts each one
  constructs as a glyph. It is what FR-009's loud failure protects, and the reason it never fires.
- **SC-004**: the existing catalog and rendering tests keep asserting what they assert, and no test
  is added or removed in the commit that mechanically edits them.
- **SC-005**: `cargo xtask check` passes after each story, on a fresh clone as well as in the
  working copy.
- **SC-006**: the second story's diff touches no public signature and no call site introduced by the
  first — countable, and the measure of whether the split was worth taking.
- **SC-007**: the dependency ADR-0019 accepts is added with an exact version whose publication date
  is at least seven days old, and the version and that date are named in the commit that adds it.
- **SC-008**: the two boundary cases the clarifications settled each have a test naming them — a
  cluster of format characters only is accepted, and the two normal forms of `é` both construct and
  compare unequal.

## Assumptions

- **The model needs no amendment.** Verified: the `Glyph` entry in _Vocabulary_ of
  [`docs/model.md`](../../docs/model.md) already reads "what a cell renders to: one grapheme
  cluster", committed with ADR-0019. This slice implements _Strokes, glyph sets and the catalog_ and
  _Rendering_ exactly as they already stand — what a catalog is, how a key is answered, and the
  two-step lookup are untouched.
- **P1 is deliberately narrower than the model for one increment.** Between the two stories the
  implementation accepts less than "one grapheme cluster". That is a temporary state inside one
  feature, not a model change, and P2 closes it.
- **No decision is taken here.** ADR-0019 owns the representation and its costs, including the
  dependency; this spec is what that decision looks like when implemented.
- **Which dependency, and which version, is settled at plan time.** ADR-0019 names the crate; the
  exact version and the check that its publication date is at least seven days old belong to the
  plan's research, and SC-007 is what records them.
- **The front end is not touched.** Verified: it names no glyph and no character today, so threading
  the type through the core reaches it only through the text it prints.
- **"Never a control character" is absolute, and it means `Cc` and only `Cc`.** No control character
  is a glyph, whichever cluster it arrives in, and there is no allow-list for a benign one. It is
  also not widened: refusing a zero-width cluster while accepting a double-width one would enforce
  half of the width problem and document the other half, and width is what ADR-0019 deliberately
  left outside the invariant. If that ever stops being acceptable, what returns is that ADR's
  rejected option D — one cluster _and_ one column — with an ADR of its own.

## Out of scope

Every item names where it is handled instead.

- **A cell that holds a literal glyph.** The next slice, and one of the two consumers this exists
  for. Nothing here puts a glyph anywhere near a cell.
- **Loading glyph sets from a file.** A later slice, and the other consumer. It is what turns a
  refusal into a message someone reads.
- **Column width.** ADR-0019 leaves it outside the invariant deliberately: a wide grapheme is
  accepted and shifts the rest of its row by a column. Documented, not enforced, and not this
  slice's to revisit.
- **Anything about strokes.** A stroke keeps its public field and its lack of validation.
- **An inline representation for a glyph's storage.** ADR-0019 accepts an allocation per glyph and
  says the answer, if one is ever needed, is inside this same type where no caller sees it. What
  would settle it is the first slice that stores text in volume, and a measurement rather than a
  guess.
- **Everything spec 0001 left out and this spec does not name** — degradation, a catalog from more
  than one set, walking a region — stays out, with the destinations that spec gave.

The near miss is the rendered output, and FR-015 with SC-001 are where it is nailed down: not one
character of it moves, in the front end or in any test, after either story. This slice widens what
the library can hold, not what it holds.

## Handoff to the plan

Two things this spec surfaces and does not own:

- **P1 mechanically edits existing test assertions**, which _Structural and behavioral change never
  share a commit_ in [the constitution](../../.specify/memory/constitution.md) does not allow of a
  structural commit. The plan's Complexity Tracking is where that departure is justified, since the
  constitution gives that table exactly one job.
- **FR-009's loud failure requires a documented panic.** `clippy::missing_panics_doc` is enforced by
  the gate and will ask for it on catalog construction the moment the panic exists.
