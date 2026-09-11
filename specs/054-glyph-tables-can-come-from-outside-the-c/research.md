# Phase 0 Research: Glyph tables can come from outside the core

The spec leaves the shape of the extension point to this stage (see its Assumptions). Everything
else needed to start implementing is already settled by the spec, ADR-0036 and the model, so this
file covers exactly the open question and the small choices it drags in.

## The extension point's shape

**Decision**: Add a `GlyphSet` value type to `monospace-core`, holding an ordered group of
`(GlyphKey, Glyph)` rows, and a `GlyphCatalogBuilder` that consumes `GlyphSet` values one at a time,
in the order they are given, inserting each row only when its key is not already claimed.

```rust
pub struct GlyphSet { /* private: Vec<(GlyphKey, Glyph)> */ }
impl GlyphSet {
    pub fn new(rows: impl IntoIterator<Item = (GlyphKey, Glyph)>) -> Self;
    pub fn light() -> Self; // the table monospace-core already ships
}

pub struct GlyphCatalogBuilder { /* private: HashMap<GlyphKey, Glyph> */ }
impl GlyphCatalogBuilder {
    #[must_use] pub fn with(self, set: GlyphSet) -> Self; // first claim wins
    #[must_use] pub fn build(self) -> GlyphCatalog;
}

impl GlyphCatalog {
    pub fn builder() -> GlyphCatalogBuilder;
    pub fn light() -> Self; // unchanged signature; now Self::builder().with(GlyphSet::light()).build()
}
```

`monospace-glyph-sets` then needs nothing from the core beyond `GlyphKey`, `Glyph`, `Stroke` and
`GlyphSet`, all already public or made public by this feature, to expose its own:

```rust
pub fn ascii() -> monospace_core::GlyphSet;
```

**Rationale**:

- It names the thing the model already names. `docs/model.md` calls the entity `GlyphSet`; giving it
  a concrete Rust type lets `monospace-glyph-sets` return something with a name in its own public
  API instead of an opaque `impl Iterator<Item = (GlyphKey, Glyph)>` the caller would have to spell
  out.
- The builder is the settled shape for "assemble a value from a variable number of parts, in an
  order the caller states" in Rust — nothing here invents a pattern the ecosystem does not already
  use for this.
- `GlyphSet` stays opaque: it exposes no accessor to its rows or their origin, so there is no way to
  ask a `GlyphSet` — let alone a built `GlyphCatalog` — which table a rule came from (FR-003). Rows
  go in through `new`, a `GlyphCatalogBuilder` drains them into one `HashMap`, and nothing
  downstream keeps the grouping.
- First-claim-wins falls out of `HashMap::entry(..).or_insert(..)` for free: exactly the rule
  `docs/model.md` already states under _Strokes, glyph sets and the catalog_, so the builder
  implements an existing decision rather than making a new one.
- `GlyphSet::light()` is added so the CLI can put the core's own table into the same builder as an
  outside one (FR-014); `GlyphCatalog::light()` keeps its signature and existing callers, now
  implemented in terms of it, so nothing that calls it today needs to change.

**Alternatives considered**:

- **A free function taking an ordered `Vec<GlyphSet>`** (`GlyphCatalog::from_sets(sets)`) — rejected
  because a caller who wants to add sets conditionally (a feature flag, a CLI option added later)
  would have to build the `Vec` by hand first; the builder chains that construction instead, and
  costs nothing extra for the CLI's fixed two-table case.
- **A trait such as `IntoGlyphSet`** implemented by whatever an outside crate wants to contribute —
  rejected as a concept with no behavior it doesn't already have: `GlyphSet::new` from an iterator
  of `(GlyphKey, Glyph)` is already everything a table needs to hand over, since `GlyphKey`'s fields
  are public and `Glyph::new` is already the public way to make one. A trait would add a name to
  learn without adding a capability.
- **No `GlyphSet` type at all**, just
  `GlyphCatalogBuilder::with(rows: impl IntoIterator<Item = (GlyphKey, Glyph)>)` — rejected because
  it leaves the model's `GlyphSet` entity with no corresponding type in the one crate that could
  give it one, and because `monospace-glyph-sets`'s public API would then have to write out the
  iterator type inline rather than naming what it returns.

## No new dependency

`monospace-glyph-sets` needs nothing beyond `monospace-core` itself. Nothing in this feature calls
for a crate the workspace does not already have, matching the spec's own Assumptions, so the
dependency-freshness rule has no version to check this time.

## Where the ASCII rows come from

The fifteen rows are copied verbatim from `docs/glyph-sets.md`'s _ASCII_ table, in the same row
order, using the same private `Row` tuple pattern `monospace-core` already uses for `LIGHT` — a
`const` array of
`(Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, &'static str)`
converted into a `GlyphSet` once. This is the existing, settled pattern in the codebase; nothing new
is invented for it.
