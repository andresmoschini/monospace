# Phase 1 Data Model: Ship the Double, Heavy and Light Round tables

This feature's "data" is Rust `const` tables and the functions that turn them into a `GlyphCatalog`,
not persisted records. This file names what each one holds and what it does and doesn't let a caller
do — the mapping `research.md`'s decision needs before code exists.

## `Row` (existing private type, unchanged)

Already defined in `monospace-glyph-sets/src/lib.rs` for `ascii()`'s `ASCII` table:

```rust
type Row = (
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    &'static str,
);
```

Reused as-is for the three new tables — no field change, no new variant.

## `DOUBLE`, `HEAVY`, `LIGHT_ROUND` (new private `const` tables)

Each `&'static [Row]` of exactly fifteen entries, copied verbatim, in the same row order, from
`docs/glyph-sets.md`'s _Double_, _Heavy_ and _Light Round_ tables respectively (FR-001, FR-002,
FR-003). Every entry's stroke name is `"double"`, `"heavy"` or `"light-round"` on whichever of the
four sides the document's table names it, `None` where the document leaves the cell blank, and the
character the document's fifth column records.

**Invariant carried over from `ASCII`**: every row is a valid `(GlyphKey, Glyph)` pair once
converted — `Glyph::new` on the character succeeds — so there is nothing to validate beyond what
`build`'s existing panic already covers (see below). None of the fifteen rows in any of the three
tables is the empty key (no side set); that is what "every non-empty combination" already means in
the spec and in `docs/model.md`.

## `build` (new private helper, extracted from `ascii()`)

```rust
fn build(table: &str, rows: &[Row]) -> GlyphCatalog
```

Converts one row table into a `GlyphCatalog` via the existing `GlyphCatalog::from_rules` extension
point (feature 054): each `Row` becomes a `(GlyphKey, Glyph)` pair, `Stroke::from` on each non-empty
side, and `Glyph::new` on the character, panicking with `table` in the message if a row's character
is not a valid glyph. `ascii()` is reimplemented as `build("ASCII", ASCII)`; nothing about its
signature, its return value, or its panic condition changes, only where the conversion logic lives
(structural, not behavioral — see `research.md`).

## `double`, `heavy`, `light_round` (new public functions)

```rust
#[must_use]
pub fn double() -> GlyphCatalog       // build("Double", DOUBLE)       — FR-001, FR-004
#[must_use]
pub fn heavy() -> GlyphCatalog        // build("Heavy", HEAVY)         — FR-002, FR-005
#[must_use]
pub fn light_round() -> GlyphCatalog  // build("Light Round", LIGHT_ROUND) — FR-003, FR-006
```

Same shape as the existing `pub fn ascii() -> GlyphCatalog` (FR-007): no argument, no privileged
access to anything `monospace-core` does not already expose publicly, `#[must_use]` because
discarding the built catalog is always a mistake, exactly as it already is for `ascii()`.

**What stays impossible**: none of the four functions accepts a table to merge with, a stroke name
to filter by, or any other parameter — combining tables remains entirely the caller's job, done with
the existing `GlyphCatalog::union` (feature 054), unchanged by this feature (FR-008).

## Relationships at a glance

```text
docs/glyph-sets.md
  _Double_ table       ──copied verbatim──> DOUBLE:      &[Row; 15]
  _Heavy_ table        ──copied verbatim──> HEAVY:       &[Row; 15]
  _Light Round_ table  ──copied verbatim──> LIGHT_ROUND: &[Row; 15]

DOUBLE, HEAVY, LIGHT_ROUND, ASCII ──build(name, rows)──> GlyphCatalog::from_rules(..) ──> GlyphCatalog

double(), heavy(), light_round(), ascii()   # same shape, one call each

GlyphCatalog::union([double(), heavy(), light_round(), ascii()])   # caller's choice, unchanged
  ──> a key from each answers independently (FR-008, SC-006)
```

## The documents (no new entity, corrected prose)

`docs/model.md`'s _Strokes, glyph sets and the catalog_ and `docs/glyph-sets.md`'s opening statement
both currently say only Light and ASCII ship as built-in data. Both are corrected to name Double,
Heavy and Light Round alongside them (FR-011, FR-012). Neither document's tables of rows change —
only the sentence describing which tables are data versus reference-only.
