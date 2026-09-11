# Phase 1 Data Model: Glyph tables can come from outside the core

This feature's "data" is a small set of Rust types, not persisted records. This file names what each
one holds, where it lives, and what it does and doesn't let a caller do — the field-by-field mapping
`research.md`'s decision needs before code exists.

## `GlyphSet` (new, `monospace-core::glyph`)

A group of glyph rules, in the order they were given — the model's `GlyphSet` entity, now a type.

| Field          | Type                     | Notes                                                                      |
| -------------- | ------------------------ | -------------------------------------------------------------------------- |
| (private) rows | `Vec<(GlyphKey, Glyph)>` | Not exposed. No accessor reads a `GlyphSet`'s rows or its origin (FR-003). |

**Construction**:

- `GlyphSet::new(rows: impl IntoIterator<Item = (GlyphKey, Glyph)>) -> Self` — the only way anything
  outside `monospace-core` builds one, using only `GlyphKey` (public fields) and `Glyph::new`
  (already public). This is the whole of FR-001's extension point.
- `GlyphSet::light() -> Self` — the fifteen rows of the Light table in
  [`docs/glyph-sets.md`](../../docs/glyph-sets.md), the same data `LIGHT` already holds today, now
  reachable as a `GlyphSet` rather than only as a whole built catalog.

**Invariants**: every row's glyph is already a valid `Glyph`, so a `GlyphSet` cannot hold an invalid
one — there is nothing left to validate at this layer. A `GlyphSet` with no rows is valid and
contributes nothing (Edge Case: "a table with no rows").

**Relationships**: consumed by `GlyphCatalogBuilder::with`, which drains a `GlyphSet`'s rows into
the catalog being built and then discards the grouping.

## `GlyphCatalogBuilder` (new, `monospace-core::glyph`)

Assembles a `GlyphCatalog` from `GlyphSet` values added in a stated order.

| Field           | Type                       | Notes                                                                            |
| --------------- | -------------------------- | -------------------------------------------------------------------------------- |
| (private) rules | `HashMap<GlyphKey, Glyph>` | The catalog under construction; same representation `GlyphCatalog` already uses. |

**Operations**:

- `GlyphCatalog::builder() -> GlyphCatalogBuilder` — the only way to get one; starts empty.
- `GlyphCatalogBuilder::with(self, set: GlyphSet) -> Self` — consumes `self` and `set`, inserts each
  of the set's rows via `HashMap::entry(key).or_insert(glyph)`, and returns the builder so calls
  chain. A key already present from an earlier `with` call keeps its earlier answer (FR-002, Edge
  Cases: "two tables that claim the same key", "the same table put into one catalog twice").
- `GlyphCatalogBuilder::build(self) -> GlyphCatalog` — consumes the builder, producing a
  `GlyphCatalog` indistinguishable in shape from one built any other way.

**State transitions**: none — a builder is built up once, by value, through owned `self` in every
method, and consumed by `build`. There is no way to inspect or reuse a builder after `build` is
called, and no way to ask which `with` call answered a given key once built (FR-003, SC-007).

## `GlyphCatalog` (existing, unchanged in shape)

No field changes. `GlyphCatalog::light()` keeps its existing signature and now reads:

```rust
pub fn light() -> Self {
    Self::builder().with(GlyphSet::light()).build()
}
```

so every existing caller — `monospace-core`'s own tests, `monospace-cli`'s test module — keeps
compiling and keeps producing the same catalog (FR-005, SC-008).

## `monospace_glyph_sets::ascii` (new crate, sole public item)

`pub fn ascii() -> monospace_core::GlyphSet` — the ASCII table's fifteen rows from
[`docs/glyph-sets.md`](../../docs/glyph-sets.md), _ASCII_, built the same way `GlyphSet::light()` is
built inside the core: a private `const` row table converted once. No type is defined in this crate;
it reaches everything it needs through `monospace-core`'s public API (FR-007, FR-009, FR-011).

## The demo description (`crates/monospace-cli/assets/demo.json`)

No schema change — `description.rs`'s JSON format already accepts any stroke name as a string
(FR-016, FR-019). What changes is content:

- New shapes whose `"stroke"` is `"ascii"`, alongside the existing `"light"` ones, on the same
  canvas.
- Two new crossings between an ASCII figure and a Light figure: one drawn with the ASCII figure on
  top (`mode: "above"` relative to the Light one already on the canvas), one with the Light figure
  on top, so the shared cell in each reads from whichever table answers for the front figure
  (FR-017, SC-009).
- The canvas's `size` may grow to fit the additions; that is a value in the file, not a behavior
  change (spec Assumptions).

## Relationships at a glance

```text
monospace-core::glyph
  GlyphKey  (existing, public fields) ──┐
  Glyph     (existing, ::new)          ─┼──> GlyphSet::new(rows) ──> GlyphSet
  GlyphSet::light()                    ─┘

GlyphSet ──> GlyphCatalogBuilder::with(set) [repeatable, ordered] ──> GlyphCatalogBuilder::build()
          ──> GlyphCatalog (unchanged shape)

monospace_glyph_sets::ascii() ──> GlyphSet   (built the same way GlyphSet::light() is)

monospace-cli::description::Description::render()
  GlyphCatalog::builder()
    .with(GlyphSet::light())
    .with(monospace_glyph_sets::ascii())
    .build()
  ──> used by monospace_core::render(...)
```
