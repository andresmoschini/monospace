# Phase 0 Research: Glyph tables can come from outside the core

The spec leaves the shape of the extension point to this stage (see its Assumptions). Everything
else needed to start implementing is already settled by the spec, ADR-0036 and the model, so this
file covers exactly the open question and the small choices it drags in.

## The extension point's shape

**Decision**: Add no new type. A table and a catalog already answer to the same contract — a lookup
from a `GlyphKey` to a `Glyph` — so a table from outside is just a `GlyphCatalog` built from its own
rows, and combining tables is building one `GlyphCatalog` out of others. Two associated functions on
the existing type carry the whole extension point:

```rust
impl GlyphCatalog {
    /// Builds a catalog from an ordered group of rules. First claim wins.
    pub fn from_rules(rules: impl IntoIterator<Item = (GlyphKey, Glyph)>) -> Self;

    /// Builds a catalog by merging other catalogs, in the order given. Where two claim the same
    /// key, the earlier one in the order wins.
    #[must_use]
    pub fn union(catalogs: impl IntoIterator<Item = Self>) -> Self;

    // Existing, unchanged signature; now `Self::from_rules(LIGHT)`.
    pub fn light() -> Self;
}
```

`monospace-glyph-sets` needs nothing from the core beyond `GlyphKey` (public fields), `Glyph::new`
(already public) and `GlyphCatalog::from_rules`, to expose its own:

```rust
pub fn ascii() -> monospace_core::GlyphCatalog;
```

and the CLI combines the two by:

```rust
GlyphCatalog::union([GlyphCatalog::light(), monospace_glyph_sets::ascii()])
```

**Rationale**:

- No new type earns its keep here. A "table" and a "catalog" already have the same shape — a set of
  rules answering one lookup — so giving the table its own type (`GlyphSet`, considered and dropped;
  see below) would be two names for one contract. `GlyphCatalog` already documents "answering a key
  is one lookup" and already hides where a rule came from; a single-table catalog is not a special
  case of that, it is the ordinary case with one contributor instead of several.
- `union` replaces a builder. A builder earns its keep when construction has to happen in steps
  interleaved with other logic (conditionally adding sets, say); this feature always builds the same
  two-catalog list at once, so a plain function over an ordered collection says the same thing with
  one call instead of a chain, and with one fewer type to document and test.
- First-claim-wins falls out of `HashMap::entry(..).or_insert(..)` in both `from_rules` (across a
  single table's own rows) and `union` (across catalogs), so both functions implement the one rule
  `docs/model.md` already states under _Strokes, glyph sets and the catalog_ rather than inventing a
  second one.
- Nothing new is added to answer "which rule came from where": a `GlyphCatalog` built by `union`
  looks exactly like one built by `from_rules` or by `light()`, because it is the same type built
  the same way, which is the simplest way to keep FR-003 true.

**Alternatives considered**:

- **A distinct `GlyphSet` type, with a `GlyphCatalogBuilder` to assemble a `GlyphCatalog` from
  several of them** — the design this research originally landed on. Dropped: it adds two public
  types to express what `from_rules` and `union` on the one existing type already express, and a
  `GlyphSet` would carry no capability a `GlyphCatalog` doesn't already have — it would just be a
  `GlyphCatalog` with a different name and a narrower job. Two types are a cost that has to buy
  something; here it bought nothing.
- **A trait such as `IntoGlyphSet`**, implemented by whatever an outside crate wants to contribute —
  rejected for the same reason it was rejected before: `(GlyphKey, Glyph)` pairs are already
  everything a table needs to hand over, through types that are already public.
- **`GlyphCatalog::from_rules` alone, no `union`**, leaving the CLI to build one catalog directly
  from the concatenation of Light's rows and ASCII's rows — rejected because it would need a way to
  get at Light's rows outside of a full catalog (the very thing dropping `GlyphSet` avoids
  reintroducing), and because "merge catalogs that already exist" is the operation the spec's
  acceptance scenarios actually describe (a catalog built from each of two _tables_, not from one
  concatenated list).

## No new dependency

`monospace-glyph-sets` needs nothing beyond `monospace-core` itself. Nothing in this feature calls
for a crate the workspace does not already have, matching the spec's own Assumptions, so the
dependency-freshness rule has no version to check this time.

## Where the ASCII rows come from

The fifteen rows are copied verbatim from `docs/glyph-sets.md`'s _ASCII_ table, in the same row
order, using the same private `Row` tuple pattern `monospace-core` already uses for `LIGHT` — a
`const` array of
`(Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, &'static str)`
converted into a `GlyphCatalog` once via `GlyphCatalog::from_rules`. This is the existing, settled
pattern in the codebase; nothing new is invented for it.
