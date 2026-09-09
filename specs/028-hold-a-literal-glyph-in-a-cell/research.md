# Phase 0 research: Hold a literal glyph in a cell

**Feature**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Date**: 2026-09-09

What a cell is, how two of them compose and what a literal renders to are not researched here: they
are settled in _The cell_, _Stamping_ and _Rendering_ of [`docs/model.md`](../../docs/model.md), and
they rest on
[ADR-0008](../../docs/decisions/0008-compose-overlapping-cells-with-three-state-arms.md). The input
draft ([spec 0005](../../docs/specs/0005-hold-a-literal-glyph-in-a-cell.md)) is where the proposed
shape and its reasoning come from. What follows is investigation local to this feature, and each
item says whether it is settled here or verified at implementation.

## R1 — The one decision this plan may not take

**Decision**: taken by the maintainer on 2026-09-09, after this plan was presented: the sum, option
A below. The reason given is the one that goes in ADR-0026's drivers, because it is broader than
this feature — _as far as possible, the model should make invalid states impossible to represent_.
This file records that the decision exists and who took it; ADR-0026 owns the reasoning, the
rejected options and the consequences, and it is the first task.

**Why an ADR and not a line in the learning log**: principle VI's test has two halves and this
clears both. It is expensive to undo — the name is in the public API, and 33 construction sites read
it — and "why is this an enum rather than a struct with an optional glyph?" is exactly the question
a reader arrives with, since the struct version is the shorter code. So the record comes before the
code that depends on it.

**The example the ADR should carry**, because it is what the maintainer's rule rules out:
`Cell { base, top: Set, .., literal: Some(glyph) }` — a cell that is a letter and also has a stroke
reaching its top side. The model has no such state; option B compiles it, and every function then
has to invent an answer for it. Option A gives it nowhere to exist, which is what makes FR-003 need
no check.

**What the ADR has to weigh**, gathered here so it has material rather than a blank page:

| Option                                                              | For                                                                                   | Against                                                                                             |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| A sum: `Cell::Strokes(StrokeCell)` and `Cell::Literal(Glyph)`       | "Which one wins" is answered once, by the type. The merge keeps taking a stroke cell. | `Cell` stops being a struct with public fields: every construction site changes, and readers match. |
| A struct carrying arms and `Option<Glyph>` together                 | No call site moves; the diff is small.                                                | Every reader has to ask which half wins, and nothing answers. A cell with both is representable.    |
| A sum with inline variant fields: `Cell::Strokes { base, top, .. }` | One type instead of two.                                                              | The merge and the key builder both grow a pattern to reach four fields they take as a unit today.   |
| A trait with two implementors                                       | Open to a third kind.                                                                 | Dynamic dispatch and a boxed cell per position, for a set of kinds the model closes at two.         |

**Alternatives considered** for the process rather than the design: recording it in the learning log
instead of an ADR. Rejected by the same test — the log is for what is cheap to undo, and this is the
shape of a public type that everything else in the crate reads.

## R2 — How much of the composition already exists

**Finding**: three of the model's five rows in _Stamping_ need no new code at all, because a literal
is decided on all four sides and `Buffer::stamp` already branches on that.

- `Above` with a decided stamp returns the stamp outright
  ([ADR-0018](../../docs/decisions/0018-mirror-the-decided-skip-in-above.md)). A literal is decided,
  so "arms, stamping a literal, `Above`" and "a literal, stamping a literal, `Above`" both land
  there and produce the incoming literal.
- `Below` onto a decided target returns without writing
  ([ADR-0017](../../docs/decisions/0017-ask-the-cell-whether-it-is-decided.md)). A literal target is
  decided, so both `Below` rows with a literal underneath leave it alone.

**Consequence for the plan**: the new work inside the merge is one case — a literal in the _bottom_
role, contributing four `Closed` sides to a stroke cell that is being written over it. Which is why
`Cell::is_decided` answering `true` for a literal (FR-009) is not a convenience: it is what makes
the two existing branches produce the rows the model asks for.

**Verified at implementation**: that the two branches stay _optimizations_ rather than becoming
load-bearing. Both ADRs claim deleting them leaves every buffer byte-identical, and that claim now
covers literals too. [quickstart.md](quickstart.md) has the procedure: delete one, run the suite,
restore.

## R3 — Where a literal's four closed sides live

**Decision**: in the merge, not in the type. `merge` keeps its shape — the topmost figure's kind
wins, and each side falls through to the one behind — and reads the bottom cell's four sides through
a private helper that answers `Closed` for a literal.

**Rationale**: the model says a literal "has no arms of its own" and behaves as four `Closed` sides
_for composing_. Storing them would let a caller build a literal with an open side, which is a state
the model does not have, and it would make the type carry a field nothing reads. A private helper in
`buffer.rs` keeps the sentence in the model literally true of the code.

The merge stays total: given a literal on top it returns that literal, which is the model's "which
of the two kinds a cell ends up being belongs to the figure in front". That case is unreachable
through `stamp` today, because R2's two branches catch every decided cell before the merge — and
writing it as a returning branch rather than an `unreachable!()` is what keeps those branches
deletable, and therefore optimizations.

**Alternatives considered**:

- `merge(top: StrokeCell, bottom: &Cell) -> StrokeCell`, making R2's argument a type-level
  invariant. Rejected: the call sites would need a match whose other arm cannot happen, so the
  impossible case moves from a harmless branch to an `unreachable!()`, and the two shortcuts stop
  being deletable.
- Storing four `Closed` arms in the literal variant. Rejected above.

## R4 — What the renderer has to change, and what it must not

**Finding**: a literal's text is borrowed from the buffer; every other glyph's text is borrowed from
the catalog. `glyph_at` returns `&'a str` tied to the catalog alone today, so the two sources have
to meet in a single lifetime for it to return either.

**Decision**: unify them —
`fn glyph_at<'a>(buffer: &'a Buffer, glyphs: &'a GlyphCatalog, ..) -> &'a str`. `render` holds both
borrows for its whole body, so the caller is unaffected and its public signature does not move.
`key_of` narrows to take a stroke cell, since the key of a literal is never built.

**Rationale**: it costs a lifetime annotation and no allocation. Rendering already copies each
`&str` into the output string once, which is where the cost was and stays.

**Alternatives considered**: returning `String` or `Cow<str>` from the per-position helper.
Rejected: an allocation per rendered cell, for a borrow the caller already holds.

**Verified at implementation**: that the annotation is sufficient — that nothing forces `render` to
name a lifetime of its own. If it turns out to, the fallback is the same signature with two named
lifetimes and no behavior difference.

## R5 — What to call the kind that is not a literal

**Decision**: `StrokeCell`, with the variant `Cell::Strokes`. Confirmed by the maintainer on
2026-09-09.

**Rationale**: the arms are the mechanism, not the subject. They are how this model represents
strokes meeting at a position — a low-level device — and what the layer above cares about is the
strokes themselves. A name taken from the mechanism would describe how the type is built; this one
describes what it is for. It is also the name the input draft already gave it, so nothing is renamed
for taste.

**Alternatives considered**: `ArmCell` with `Cell::Arms`, following the row labels of _Stamping_,
which say "Arms" and "A literal". The argument for it was that the model owns the domain vocabulary;
against it, those labels name a row rather than a type, `ArmCell` describes half of what the struct
holds — the base stroke is not an arm — and it names the low-level device rather than the thing.
Rejected for the reason above.

## R6 — No dependency, and where the fill character comes from

**Finding**: nothing is added. `Glyph` and `unicode-segmentation` arrived with feature 006, and this
feature validates nothing of its own — a cell holds a `Glyph` that was already checked on
construction (FR-012).

The front end needs one glyph from a literal string. `Glyph::new` answers an `Option`, so the call
site is `Glyph::new("░").expect(..)` with a message saying why it cannot fail. Verified by reading
`Cargo.toml`: the workspace enables `clippy::pedantic` and `missing_docs`, and none of the
restriction lints that would forbid `expect` are on. `clippy::missing_panics_doc` is in pedantic but
fires on public items, and the front end's helpers are private to its binary.

**Alternatives considered**: a constant in the core for the shade. Rejected — the core gains no
opinion about what anything is filled with, which is the spec's _Out of scope_.

## R7 — The commit split, and how the structural commit avoids touching tests

**Decision**: two code commits. First a `refactor` that renames the struct to `StrokeCell` and
leaves `pub type Cell = StrokeCell;` in its place; then a `feat` that takes the name `Cell` for the
sum, deletes the alias, and adds the literal with its rules, the render branch and the tests. A
second `feat` follows for the front end's fill, which is the spec's second story.

**Rationale**: the rename reaches 33 construction sites, 32 of them in test modules. Without the
alias the structural commit has to edit tests to compile, and principle V says a structural commit
modifies no test — so the plan would open by arguing for an exception. With the alias it edits none,
and the mechanical `StrokeCell { .. }.into()` edits travel with the behavioral commit that forces
them, where a `feat` is allowed to touch tests.

**Measured, not assumed**: a type alias to a named-field struct works in struct-literal expressions,
in patterns and in comparisons. Compiled and run with the pinned rustc 1.98.1 under edition 2024
before this was written down, as a throwaway file outside the repository:

```rust
pub struct StrokeCell { pub base: String, pub top: u8 }
pub type Cell = StrokeCell;
let c = Cell { base: "light".to_string(), top: 1 };  // literal: accepted
let Cell { base, top } = &c;                          // pattern: accepted
```

**Alternatives considered**:

- One commit for everything. Rejected: it buries what a literal costs under a rename that touches
  every construction site in the workspace, which is the reason the input draft split it too.
- The rename without the alias, justified in Complexity Tracking. Rejected: the alias costs one line
  for one commit and needs no justification at all.

**Cost recorded**: `Cell` exists as a public type alias for exactly one commit. Anyone reading only
that commit sees a name that means what it always meant, which is the point.
