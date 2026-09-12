# Phase 1 Data Model: Allow render cells with mixed arms' strokes

This feature's data is two kinds: a changed shape for one existing type (`Arm`), and four more
`const` row tables of the same shape feature 056 already established. This file names what each
holds and what it does and doesn't let a caller do — the mapping `research.md`'s decisions need
before code exists.

## `Arm` (existing type, changed shape)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arm {
    Set(Stroke),
    Closed,
    Unset,
}
```

Was `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` with a payload-less `Set`. `Copy` is dropped:
`Stroke` is an owned `String`
([ADR-0015](../../docs/decisions/0015-represent-a-stroke-as-a-string.md)), so a `Set` arm must be
cloned rather than copied wherever a cell is duplicated. `Closed` and `Unset` are unchanged. Every
existing construction site supplies the same stroke already given to that cell's `base` — no site
yet exists where the two differ, since nothing built before this feature could construct one.

## `StrokeCell` (existing type, unchanged fields, two methods)

Fields (`base: Stroke`, `top: Arm`, `right: Arm`, `bottom: Arm`, `left: Arm`) do not change.

```rust
pub fn key(&self) -> GlyphKey          // reads each arm's own stroke — FR-001, FR-002
pub fn degraded_key(&self) -> GlyphKey // reads self.base for every Set side — FR-004
```

`key()` changes from reading `self.base` on every `Set` side to reading the stroke each `Arm::Set`
now carries. `degraded_key()` is new: the same shape as `key()`, but every `Set` side answers
`self.base.clone()` regardless of what its own arm carries. Both leave `Closed` and `Unset` sides as
`None`, per _Rendering_'s rule that the two read the same at key-building time.

**Invariant**: a `StrokeCell` whose arms all carry `self.base`'s own value produces the same
`GlyphKey` from both methods — this is the case FR-004 and SC-006 require to be silent: nothing
about a diagram that mixes no strokes changes.

## `Cell::glyph_str` (existing method, one more lookup)

```rust
Cell::Strokes(cell) => glyphs
    .glyph(&cell.key())
    .or_else(|| glyphs.glyph(&cell.degraded_key()))
    .map(Glyph::as_str),
```

Was a single `glyphs.glyph(&cell.key())`. `Cell::Literal`'s branch is unchanged: no key is built, no
catalog is consulted, per _Rendering_'s rule that degradation never applies to a literal.

**State transitions**: none — this is a pure function from a `Cell` and a `GlyphCatalog` reference
to `Option<&str>`, called once per rendered position, with no memory between calls.

## The four mixing tables (new private `const` data)

Each `&'static [Row]`, reusing the existing private `Row` tuple `monospace-glyph-sets` already
defines — no field change, no new variant:

| Table                | Rows | Source section in `docs/glyph-sets.md` |
| -------------------- | ---: | -------------------------------------- |
| `LIGHT_DOUBLE`       |   18 | _Mixing Light and Double_              |
| `LIGHT_HEAVY`        |   50 | _Mixing Light and Heavy_               |
| `LIGHT_ROUND_DOUBLE` |   18 | _Mixing Light Round and Double_        |
| `LIGHT_ROUND_HEAVY`  |   50 | _Mixing Light Round and Heavy_         |

Copied verbatim, in the same row order, from the named section. Each row names two different strokes
across its four sides (never one, never three or four distinct) — that is what "mixing" means for
these tables — and the character the document's fifth column records.

**Invariant (FR-011)**: no key any of these four rows produce collides with a key any of the five
single-stroke tables (`ascii`, Light, `double`, `heavy`, `light_round`) or either other mixing table
produces. A single-stroke table's rows only ever name one stroke; a mixing table's rows only ever
name two — the two families cannot collide on that shape alone, and the four mixing tables pair
different strokes from each other, so no two of the nine ever contest the same key.

## `light_double`, `light_heavy`, `light_round_double`, `light_round_heavy` (new public functions)

```rust
#[must_use]
pub fn light_double() -> GlyphCatalog        // build("Mixing Light and Double", LIGHT_DOUBLE)
#[must_use]
pub fn light_heavy() -> GlyphCatalog         // build("Mixing Light and Heavy", LIGHT_HEAVY)
#[must_use]
pub fn light_round_double() -> GlyphCatalog  // build("Mixing Light Round and Double", ...)
#[must_use]
pub fn light_round_heavy() -> GlyphCatalog   // build("Mixing Light Round and Heavy", ...)
```

Same shape as the existing `double()`/`heavy()`/`light_round()` (FR-010): no argument, no privilege,
`#[must_use]`, built through the existing shared `build` helper. FR-016 bounds these four
explicitly: this feature adds no argument that selects a subset of a table, filters by stroke, or
merges with another — combining tables stays entirely `GlyphCatalog::union`'s job, unchanged.

## `Description::render` (existing method, one changed call)

```rust
let catalog = GlyphCatalog::union([
    GlyphCatalog::light(),
    monospace_glyph_sets::ascii(),
    monospace_glyph_sets::double(),
    monospace_glyph_sets::heavy(),
    monospace_glyph_sets::light_round(),
    monospace_glyph_sets::light_double(),        // new
    monospace_glyph_sets::light_heavy(),         // new
    monospace_glyph_sets::light_round_double(),  // new
    monospace_glyph_sets::light_round_heavy(),   // new
]);
```

The only change in `monospace-cli`. No new field, no new type, no CLI argument (FR-016): the
demonstration's catalog is still built the same way, with four more entries in the same list.

## Relationships at a glance

```text
Arm::Set(Stroke)  ──read by──>  StrokeCell::key()          ──>  GlyphKey (exact, per-arm)
StrokeCell::base  ──read by──>  StrokeCell::degraded_key()  ──>  GlyphKey (uniform, base stroke)

Cell::glyph_str:  key() lookup  ──miss──>  degraded_key() lookup  ──miss──>  None (renders as space)

docs/glyph-sets.md
  _Mixing Light and Double_        ──copied verbatim──> LIGHT_DOUBLE:       &[Row; 18]
  _Mixing Light and Heavy_         ──copied verbatim──> LIGHT_HEAVY:        &[Row; 50]
  _Mixing Light Round and Double_  ──copied verbatim──> LIGHT_ROUND_DOUBLE: &[Row; 18]
  _Mixing Light Round and Heavy_   ──copied verbatim──> LIGHT_ROUND_HEAVY:  &[Row; 50]

LIGHT_DOUBLE, LIGHT_HEAVY, LIGHT_ROUND_DOUBLE, LIGHT_ROUND_HEAVY
  ──build(name, rows)──> GlyphCatalog::from_rules(..) ──> GlyphCatalog

light_double(), light_heavy(), light_round_double(), light_round_heavy()   # same shape, one call each

Description::render()'s GlyphCatalog::union([...])  ──gains the four above──>  the CLI demo's catalog
```

## The documents (no new entity, corrected prose)

`docs/model.md`'s _Strokes, glyph sets and the catalog_ currently says the mixing sets are "left
loaded from a file when someone asks for them"; `docs/glyph-sets.md`'s opening note currently says
they "remain reference only, with nothing loading them yet." Both are corrected to name the four
mixing tables alongside ASCII, Double, Heavy and Light Round as data `monospace-glyph-sets` ships
(FR-012, and the `docs/model.md` correction `research.md` names). Neither document's tables of rows
change — only the sentence describing where the mixing tables come from.
