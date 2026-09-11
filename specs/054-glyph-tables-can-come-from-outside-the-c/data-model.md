# Phase 1 Data Model: Glyph tables can come from outside the core

This feature's "data" is a small set of Rust types, not persisted records. This file names what each
one holds, where it lives, and what it does and doesn't let a caller do — the field-by-field mapping
`research.md`'s decision needs before code exists.

## `GlyphCatalog` (existing type, gains two constructors)

No field change: still a `HashMap<GlyphKey, Glyph>` behind an opaque type, still answering a key
with one lookup. A table from outside the core and a catalog several tables were merged into are
both values of this one type — that is the whole of `research.md`'s decision, so nothing new is
added alongside it.

**New constructors**:

- `GlyphCatalog::from_rules(rules: impl IntoIterator<Item = (GlyphKey, Glyph)>) -> Self` — the
  extension point (FR-001). Builds a catalog from one ordered group of rules, first claim wins
  across that group. Everything on the right-hand side of a call — `GlyphKey`'s public fields,
  `Glyph::new` — is already public, so this is usable from outside `monospace-core` with no private
  item.
- `GlyphCatalog::union(catalogs: impl IntoIterator<Item = Self>) -> Self` — combines already-built
  catalogs in the order given; where two claim the same key, the earlier one in the order wins
  (FR-002). Implemented by folding each catalog's rules into one `HashMap` with
  `entry(key).or_insert(glyph)`, so a catalog already built by `from_rules`, by `light()`, or by an
  earlier `union` composes the same way as any other.

**Existing constructor, reimplemented**:

- `GlyphCatalog::light() -> Self` keeps its signature; now reads `Self::from_rules(LIGHT)` where
  `LIGHT` is the private row table already in `monospace-core` (FR-006). Every existing caller keeps
  compiling and keeps producing the same catalog (FR-005, SC-008).

**Invariants**: every row handed to `from_rules` is already a valid `(GlyphKey, Glyph)` pair — a
`Glyph` cannot be invalid by construction — so there is nothing to validate at this layer. An empty
`rules` iterator, or a `catalogs` list of zero or one catalog, is valid and produces a catalog that
answers accordingly (Edge Cases: "a catalog built from no tables at all", "a table with no rows").

**What stays impossible**: no operation on `GlyphCatalog` reports which `from_rules` call or which
element of a `union` answered a given key (FR-003, SC-007) — merging happens by draining one
`HashMap` into another, and nothing is kept that could answer that question.

## `monospace_glyph_sets::ascii` (new crate, sole public item)

`pub fn ascii() -> monospace_core::GlyphCatalog` — a catalog built from exactly the fifteen rows
[`docs/glyph-sets.md`](../../docs/glyph-sets.md) records under _ASCII_ — no row added, none omitted,
none altered (FR-011). Built the same way `GlyphCatalog::light()` is built inside the core: a
private `const` row table converted once via `GlyphCatalog::from_rules`. No type is defined in this
crate; it reaches everything it needs through `monospace-core`'s public API (FR-007, FR-009).

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
  Glyph     (existing, ::new)          ─┴──> GlyphCatalog::from_rules(rows) ──> GlyphCatalog
  GlyphCatalog::light()                 ──> GlyphCatalog::from_rules(LIGHT)

monospace_glyph_sets::ascii() ──> GlyphCatalog::from_rules(ASCII) ──> GlyphCatalog

GlyphCatalog::union([ GlyphCatalog::light(), monospace_glyph_sets::ascii() ])  ──> GlyphCatalog

monospace-cli::description::Description::render()
  GlyphCatalog::union([GlyphCatalog::light(), monospace_glyph_sets::ascii()])
  ──> used by monospace_core::render(...)
```
