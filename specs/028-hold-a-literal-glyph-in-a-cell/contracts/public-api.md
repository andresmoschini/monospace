# Public API contract: Hold a literal glyph in a cell

**Feature**: [spec.md](../spec.md) | **Plan**: [plan.md](../plan.md) | **Date**: 2026-09-09

The surface below is the input draft's
([spec 0005](../../../docs/specs/0005-hold-a-literal-glyph-in-a-cell.md), _Public surface_), carried
here unchanged. Phase 0 found no reason to move it; what it did find is that the shape needs
ADR-0026 before it ships, and that is [R1](../research.md). The reasoning for each item lives in the
draft and in the ADR, not in this file — this file says what the crate exposes when the feature is
done.

## Added

```rust
pub struct StrokeCell {
    pub base: Stroke,
    pub top: Arm,
    pub right: Arm,
    pub bottom: Arm,
    pub left: Arm,
}

pub enum Cell {
    Strokes(StrokeCell),
    Literal(Glyph),
}

impl From<StrokeCell> for Cell { /* ... */ }
```

- `StrokeCell` is today's `Cell`, renamed. Same fields, same visibility, same absence of validation.
- Both variants are public, so `Cell::Literal(glyph)` is available to any caller. That is safe
  because `Glyph` validated what matters one layer down; the draft's open question about
  constructing only through functions stays open, and what would settle it is a third kind of cell.
- `From<StrokeCell> for Cell` is what keeps the front end's nine stamps per box from doubling in
  width. Nothing converts the other way, since it would have to fail.

## Changed

| Item                | Before                    | After                                  |
| ------------------- | ------------------------- | -------------------------------------- |
| `Cell`              | a struct with five fields | a sum of `Strokes` and `Literal`       |
| `Cell::is_decided`  | the four arms             | the four arms, or `true` for a literal |
| `Buffer::stamp`     | takes the struct          | takes the sum                          |
| `Buffer::cell`      | answers `Option<&Cell>`   | the same, where `Cell` is now the sum  |
| `lib.rs` re-exports | `Arm`, `Cell`             | `Arm`, `Cell`, `StrokeCell`            |

`Buffer::stamp` and `Buffer::cell` keep their signatures word for word. What widened is the type
inside them, so every caller changes and no signature does.

**This is a breaking change to `Cell`**, and deliberately so: it stops being a struct with public
fields, so every construction site becomes `StrokeCell { .. }.into()` and every reader matches. No
crate outside this repository depends on these crates
([ADR-0002](../../../docs/decisions/0002-no-minimum-supported-rust-version.md)), so the cost is
counted inside the workspace: 33 construction sites, 32 of them in test modules.

For exactly one commit — the structural one — `pub type Cell = StrokeCell;` stands in the sum's
place so that nothing has to be edited to compile. It is gone by the end of the feature. See
[R7](../research.md).

## Unchanged

`Arm`, `Stroke`, `Glyph`, `GlyphKey`, `GlyphCatalog`, `Pos`, `Size`, `StampMode`, `render`, and
every rule in the built-in Light table. No new dependency, and `Glyph`'s invariant is neither
widened nor narrowed.

## Behavioral guarantees a caller can rely on

1. A cell holding a glyph renders as exactly that glyph, whatever the catalog holds — no key is
   built and no lookup happens (FR-002).
2. Nothing connects into a cell holding a glyph. A figure that writes arms over one inherits its
   refusal on every side it does not decide itself (FR-006).
3. A cell holding a glyph is decided, so `Below` leaves it alone and a front-to-back walk may stop
   at it (FR-009).
4. Stamping a stack front to back with `Below` and back to front with `Above` produces the same
   buffer, with literals in the stack as well as stroke cells (FR-008).
5. A diagram containing no literal renders byte for byte as it did before this feature (FR-004).

## Documentation contract

Every item above carries rustdoc, and two sentences are required rather than optional:

- `Cell`'s documentation says **which of the two kinds wins where they meet** (FR-011). That is the
  sentence a reader would otherwise reconstruct from the merge, and it is the one thing about this
  type that is not obvious from its shape.
- `Cell::is_decided`'s documentation says a literal is decided on all four sides by definition, and
  keeps its existing link to ADR-0017.

`StrokeCell` inherits the documentation today's `Cell` carries, including its link to
[ADR-0012](../../../docs/decisions/0012-one-stroke-per-cell.md) for why an arm has no stroke of its
own. The module documentation of `cell.rs` changes from "a base stroke and four arms" to name both
kinds.
