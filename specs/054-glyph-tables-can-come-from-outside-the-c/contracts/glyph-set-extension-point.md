# Contract: the glyph-set extension point

This project has no network or process boundary; its external interface is the public Rust API two
crates expose to whoever depends on them. This is that contract — what `monospace-core` promises an
outside table, and what `monospace-glyph-sets` promises the CLI (or anyone else). See
`data-model.md` for the types themselves and `research.md` for why they take this shape.

## `monospace-core` promises to a table it does not ship

```rust
// Build a rule: a key on one side, a glyph on the other. Both are already public today.
let key = GlyphKey { top: Some(Stroke::from("mine")), right: None, bottom: Some(Stroke::from("mine")), left: None };
let glyph = Glyph::new("x").expect("one grapheme cluster, no control character");

// Group rules into a table.
let mine = GlyphSet::new([(key, glyph) /* , more rows */]);

// Put it in a catalog, in a stated order, alongside anything else.
let catalog = GlyphCatalog::builder()
    .with(GlyphSet::light())
    .with(mine)
    .build();
```

Guarantees:

- Everything on the right-hand side of `GlyphSet::new`'s call is reachable through
  `monospace_core`'s public API — no private item is needed (FR-001, FR-009).
- `.with` calls are order-sensitive only where two sets claim the same key; a set's rows that no
  other set claims answer identically no matter where it sits in the chain (Acceptance Scenario
  1.3).
- Once `.build()` returns, no operation on `GlyphCatalog` reports which `.with` call, and so which
  set, answered a given key (FR-003, SC-007).
- A `GlyphSet` with no rows, or the same `GlyphSet` value given to `.with` twice, changes nothing
  about the resulting catalog beyond what its rows already claimed once (Edge Cases).

## `monospace-glyph-sets` promises to whoever depends on it

```rust
let ascii_table: monospace_core::GlyphSet = monospace_glyph_sets::ascii();
```

Guarantees:

- `ascii()` returns a `GlyphSet` holding exactly the fifteen rows `docs/glyph-sets.md` records under
  _ASCII_ — no row added, none omitted, none altered (FR-011).
- A catalog built from `ascii()` alone answers all fifteen non-empty combinations of the `ascii`
  stroke (FR-012, SC-005), and produces no character outside printable ASCII when used to render
  (SC-004).
- The crate depends on nothing but `monospace-core`'s public API, compiles for
  `wasm32-unknown-unknown` (FR-010), and defines no type of its own that `monospace-core` does not
  already give it a name for.

## What is explicitly not promised

- No operation exists to ask a built `GlyphCatalog`, or a `GlyphSet` once handed to `.with`, which
  table a rule came from (FR-003, SC-007) — this is a limit, not an oversight.
- Neither crate gains a way to remove a rule, replace one already claimed, or merge two catalogs
  after they are built; `.with` on a fresh `GlyphCatalogBuilder` is the only composition operation.
- No command-line flag is added anywhere in this feature to choose which tables go into a catalog
  (FR-019) — this contract governs library code, not the CLI's argument surface.
