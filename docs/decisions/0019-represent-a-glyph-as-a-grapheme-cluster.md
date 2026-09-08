---
status: accepted
date: 2026-09-07
decision-makers: Andrés Moschini
---

# Represent a glyph as a validated grapheme cluster

## Context and Problem Statement

A glyph is a `char` today. The catalog holds `HashMap<GlyphKey, char>`, `glyph` returns
`Option<char>`, and `render` collects those characters into a string. It works because every rule in
the built-in table is one character and because nothing outside this repository has ever produced a
glyph.

Two slices are queued that change both halves of that sentence. A cell will be able to hold a
literal glyph instead of deriving one from its arms, which puts glyphs on the writing side of the
API for the first time, arriving from a caller. And glyph sets will be loaded from files, which puts
them on the reading side, arriving from outside the process. In both cases something has to say what
a glyph is allowed to be.

`char` answers that badly in two directions at once. It accepts `\n` and `\t`, which occupy no cell
and break the rectangle `render` promises rather than merely spoiling it. And it rejects `é` written
as `e` followed by a combining acute, a flag, and every emoji built from more than one code point. A
`char` is a Unicode scalar value, which is smaller than "one thing a reader sees" and unrelated to
"one column wide".

## Decision Drivers

- `render` promises a rectangle of exactly `width` by `height`
  ([spec 0001](../specs/0001-stamp-cells-and-render-them.md), rule 6). A glyph that is not one cell
  of output breaks a tested contract, not just the picture.
- Glyphs are about to arrive from outside, so whatever the invariant is has to be checkable at a
  boundary rather than remembered.
- [Constraints and Dependencies](../../.specify/memory/constitution.md#constraints-and-dependencies):
  prefer the standard library for domain logic, especially early on, to maximize the design-learning
  value. The brief's §5 when this was written.
- The vocabulary already has the word and no type behind it: [`docs/model.md`](../model.md) defines
  `Glyph` as what a cell renders to.

## Considered Options

- **A** — Keep `char` everywhere, with no type of its own.
- **B** — `Glyph(char)`, a wrapper type that rejects control characters.
- **C** — `Glyph(String)`, validating exactly one grapheme cluster, which needs
  `unicode-segmentation`.
- **D** — `Glyph(String)`, validating one grapheme cluster **and** one column, which needs
  `unicode-width` as well.

## Decision Outcome

Chosen option: **C, one validated grapheme cluster**, because a cell holds one thing a reader sees,
and the grapheme cluster is the only unit that means that — a scalar value is smaller than it, and a
column is a property of the terminal rather than of the text.

The invariant is exactly one grapheme cluster, non-empty, containing no control character. The two
halves are not redundant: `\r\n` is a single grapheme cluster by UAX #29 and has to be refused
anyway.

Width is deliberately not part of the invariant. A wide grapheme — most CJK, most emoji — is
accepted and shifts the rest of its row by a column. That is a known, documented imperfection, and
the alternative was to reject exactly the characters that motivated widening past `char` in the
first place.

The dependency is `unicode-segmentation`, from `unicode-rs`, which has no runtime dependencies of
its own. Grapheme segmentation is UAX #29 — combining marks, ZWJ, variation selectors, regional
indicators, Hangul, emoji modifiers — and its tables move with each Unicode release. Implementing it
here would mean owning rules this project has no interest in owning, which is the same trade-off
[issue #13](https://github.com/andresmoschini/monospace/issues/13) already weighs for link checking:
a maintained table somewhere else, or a copy here that goes quietly stale.

### Consequences

- Good, because the invariant has one home. `Glyph::new` is the only way to make one, so nothing
  that reaches a cell of output can skip the check.
- Good, because the spec that loads sets from a file inherits a validated type instead of inventing
  validation while it is also learning to parse.
- Good, because the built-in tables get checked too. Fifteen rules today and nine sets eventually,
  all of them data that nothing currently looks at.
- Bad, because it is the first dependency in `monospace-core`, against the brief's preference for
  the standard library in domain logic. It is recorded here so that the next one is weighed against
  a precedent rather than waved through as a habit.
- Bad, because a glyph stops being `Copy` and costs an allocation each. A page of text would be a
  page of allocations. The wrapper is what makes an inline representation a later change that no
  caller sees.
- Bad, because the catalog and the renderer change shape: `glyph` answers with a borrow rather than
  a copied `char`, and `render` collects string slices.
- Neutral, because `Stroke` keeps its public field and no validation. The two look alike and are
  not: a stroke is a name, and a name has nothing to check, while a glyph is a cell of output and
  has.

### Confirmation

By construction: the payload is private and `Glyph::new` is the only constructor, so an invalid
glyph cannot exist to be found. What is left to test is the constructor itself, and
[spec 0004](../specs/0004-give-a-glyph-a-type-of-its-own.md) carries a case per rule plus one that
walks every built-in rule of the Light table and asserts it constructs.

## Pros and Cons of the Options

### A — Keep `char`

- Good, because it is `Copy`, allocates nothing, and there is nothing to build.
- Bad, because it admits `\n` and `\t`, which break the rendered rectangle rather than shifting it.
- Bad, because it cannot hold `é` decomposed, a flag, or an emoji built from several code points.
- Bad, because the invariant stays prose in a doc comment, which is where invariants stop being
  enforced.

### B — `Glyph(char)`

- Good, because it keeps `Copy`, needs no dependency, and still rejects control characters.
- Good, because it gives the invariant a home and leaves the inside changeable.
- Bad, because it cannot hold a multi-scalar grapheme, so this same decision comes back the first
  time somebody writes an emoji — and by then there are callers written against the narrow type.

### C — `Glyph(String)`, one grapheme cluster

- Good, because the unit matches what a cell is: one thing a reader sees.
- Good, because the check is at the boundary, where the two coming slices need it.
- Bad, because it costs a dependency and an allocation per glyph.

### D — One grapheme cluster and one column

- Good, because the invariant would match what the project is named after.
- Bad, because it rejects most emoji, which is the thing widening past `char` was for.
- Bad, because it needs a second dependency and a table deciding what one column means, which
  terminals disagree about in practice.

## Reversibility

Cheap in the direction that matters. The payload is private, so the inside can become an inline
string, or narrow back to a `char`, without a single caller changing — which is the whole reason the
type wraps a `String` rather than being one.

What is not cheap is removing the type. Every later caller that produces or consumes a glyph is
written against it, and taking it away means putting the invariant back into prose. Dropping the
dependency is the same move as narrowing the invariant, and by then it is a behavior change for
anything that stored a multi-scalar glyph.

## Confidence

High (85%).

What would change it: text arriving in volume, where an allocation per cell becomes measurable. The
answer there is an inline representation inside this same type, not a different decision, which is
why it does not change this one now.

What would prove it wrong later: terminals disagreeing enough about grapheme clusters that "one
grapheme, one cell" stops describing what a reader sees. That would push the unit towards the column
after all, and bring D back with a real reason instead of a theoretical one.

## More Information

- [ADR-0015](0015-represent-a-stroke-as-a-string.md), the same shape of question one level down, and
  the reason a stroke is a public `String` while a glyph is not.
- [`docs/model.md`](../model.md), whose vocabulary entry for `Glyph` this decision rewrites.
- [Constraints and Dependencies](../../.specify/memory/constitution.md#constraints-and-dependencies)
  for the preference this dependency is weighed against, and
  [issue #13](https://github.com/andresmoschini/monospace/issues/13) for the same trade-off arising
  over link checking. Both were sections of the brief, §5 and §9, when this was written.
- The slice that implements it: [spec 0004](../specs/0004-give-a-glyph-a-type-of-its-own.md),
  abandoned as a draft when the spec home moved and now the input to the Spec Kit feature that
  replaces it. Nothing implements this decision yet.
