---
status: accepted
date: 2026-09-09
decision-makers: Andrés Moschini
---

# Represent a cell as a sum of a stroke cell and a literal glyph

## Context and Problem Statement

Today's `Cell` is a base stroke and four arms, and the character it draws is always derived from
them through the catalog. [`docs/model.md`](../model.md) now has a second thing a cell can be: **a
literal glyph**, the character it renders to, chosen rather than derived. A letter is one; a fill
that occludes what it covers is the other, and today's only approximation — four `Closed` arms — can
only ever be a space, since the Light table has no rule for the empty key.

Feature 028 has to give `Cell` a shape that can hold either one. The two kinds are mutually
exclusive by the model's own words: a literal "has no arms of its own," and which of the two kinds a
cell ends up being "belongs to the figure in front, exactly as the base stroke does." Nothing in the
model describes a cell that is both, or a rule for what such a cell would mean. The shape chosen has
to make that exclusivity either enforced or merely conventional, and the difference reaches every
function that reads a cell: the merge, the key builder, and every test that constructs one.

## Decision Drivers

- The maintainer's rule, wider than this feature: **as far as possible, the model should make
  invalid states impossible to represent.**
- Making an invalid state unrepresentable is not a principle this repository has written down under
  that name, but it is exactly what principle I,
  [Process over product](../../.specify/memory/constitution.md#i-process-over-product-non-negotiable),
  asks for when a shortcut and a better design disagree: the type that closes off the invalid state
  is the one worth the churn.
- FR-003 requires that a cell holding a literal "have no arms to read, and MUST NOT be constructible
  with arms of its own." A requirement a type can discharge on its own needs no runtime check to
  keep it true.
- [ADR-0011](0011-expose-cell-for-testing-stamping.md)'s driver about public surface: a shape chosen
  here is read by every construction site in the crate, so the cost of a wrong shape is not local.

## Considered Options

- **A** — A sum: `Cell::Strokes(StrokeCell)` and `Cell::Literal(Glyph)`.
- **B** — A struct carrying arms and `Option<Glyph>` together.
- **C** — A sum with inline variant fields: `Cell::Strokes { base, top, right, bottom, left }`.
- **D** — A trait with two implementors.

## Decision Outcome

Chosen option: **A**, because it is the only one of the four that makes "a cell that is a letter and
also has a stroke reaching its top side" a value with nowhere to exist, rather than a value the type
admits and every reader has to rule out by convention.

The example that rules out B: `Cell { base, top: Set, .., literal: Some(glyph) }`. Under option B
that value compiles, and `merge`, `render` and every future function that reads a cell has to invent
an answer for what it means — an answer the model does not give, because the model has no such
state. Under option A there is no field to put a stroke and a glyph in at once, so FR-003 needs no
check: the compiler enforces it before any test could.

### Consequences

- Good, because "which kind wins where two cells meet" is answered once, by the type: `merge` keeps
  taking a `StrokeCell` and a private helper in `buffer.rs` answers `Closed` for a literal's four
  sides, rather than every caller re-deriving the answer from an optional field.
- Good, because the four `Closed` sides a literal contributes when composing live in the merge, not
  in the type — storing them would let a caller build a literal with an open side, which is a state
  the model does not have either.
- Bad, because `Cell` stops being a struct with public fields. Every one of the 33 construction
  sites in the workspace changes, and every reader that matched on the old shape now matches on the
  sum. [research.md](../../specs/028-hold-a-literal-glyph-in-a-cell/research.md), R7 records how the
  structural half of that cost is kept out of the commit that also adds the literal.
- Neutral, because both variants are public, so `Cell::Literal(glyph)` is constructible directly.
  That is safe today because `Glyph` already validated what matters one layer down; it stops being
  safe the day a second literal-shaped kind of cell needs an invariant across variants, which is an
  open question this record does not close.

### Confirmation

Enforced by the compiler for the exclusivity itself: there is no `Cell` value holding both a stroke
cell and a glyph, or neither, because the sum has no such variant. `cargo xtask check` verifies
nothing about the choice of shape directly — it has no opinion about which of the four options was
taken — but every test written against the sum (SC-003, SC-004) would fail to compile against a
struct-shaped `Cell`, which is as close to a mechanical check on this decision as the gate gets.

## Pros and Cons of the Options

### A — A sum: `Cell::Strokes(StrokeCell)` and `Cell::Literal(Glyph)`

- Good, because "which one wins" is answered once, by the type. The merge keeps taking a stroke
  cell.
- Bad, because `Cell` stops being a struct with public fields: every construction site changes, and
  every reader matches.

### B — A struct carrying arms and `Option<Glyph>` together

- Good, because no call site moves; the diff is small.
- Bad, because every reader has to ask which half wins, and nothing answers. A cell with both a
  stroke and a glyph is representable, and FR-003 becomes a runtime rule rather than a type fact.

### C — A sum with inline variant fields: `Cell::Strokes { base, top, right, bottom, left }`

- Good, because it is one type instead of two — no separate `StrokeCell` to name.
- Bad, because the merge and the key builder both grow a pattern to reach four fields they take as a
  unit today, and the merge of two stroke cells can no longer keep the signature it already has.

### D — A trait with two implementors

- Good, because it stays open to a third kind of cell without touching the existing two.
- Bad, because it costs dynamic dispatch and a boxed cell per position, for a set of kinds the model
  closes at two. Nothing in the model or in this feature's scope anticipates a third kind.

## Reversibility

Expensive. The name is in the public API and every construction site in the workspace reads it, so
reversing this — moving back to a struct, or to a trait — touches the same 33 call sites this
decision already costs, in the opposite direction, plus whatever code by then depends on matching a
`Cell::Strokes`/`Cell::Literal` sum. Nothing outside this repository depends on these crates
([ADR-0002](0002-no-minimum-supported-rust-version.md)), so the cost is paid once, inside the
workspace, and not by an external consumer's semantic-versioning promise.

## Confidence

High (~85%).

What would change it: a third kind of cell arriving with an invariant that spans it and one of the
existing two — the open question [spec 0005](../specs/0005-hold-a-literal-glyph-in-a-cell.md) left
about constructing `Cell` only through functions. That is the case option D was built for, and it is
the one this record cannot rule out from here.

What would not change it: the size of the diff. 33 construction sites was known before this decision
was taken, and the type alias in
[research.md](../../specs/028-hold-a-literal-glyph-in-a-cell/research.md), R7 is what keeps that
cost from landing in the same commit as the literal itself, rather than a reason to prefer a shape
that avoids the cost altogether.

## More Information

- [`docs/model.md`](../model.md), _The cell_, subsection _A cell can be a literal instead_, and
  _Stamping_, for the rules this shape has to carry.
- [research.md](../../specs/028-hold-a-literal-glyph-in-a-cell/research.md), R1, for the full
  options table and the alternative of recording this in the learning log instead, rejected by the
  same test [the decisions README](README.md) states for every record in this directory: expensive
  to undo, or a reader will ask why.
- [ADR-0008](0008-compose-overlapping-cells-with-three-state-arms.md), which this feature's
  composition rules rest on without changing.
- [ADR-0017](0017-ask-the-cell-whether-it-is-decided.md), for `is_decided`, which this feature
  widens rather than replaces.
