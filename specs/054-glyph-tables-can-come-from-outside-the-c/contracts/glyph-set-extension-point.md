# Contract: the glyph-set extension point

This project has no network or process boundary; its external interface is the public Rust API two
crates expose to whoever depends on them. This is that contract — what `monospace-core` promises an
outside table, and what `monospace-glyph-sets` promises the CLI (or anyone else). See
`data-model.md` for the type itself and `research.md` for why it takes this shape.

## `monospace-core` promises to a table it does not ship

```rust
// Build a rule: a key on one side, a glyph on the other. Both are already public today.
let key = GlyphKey { top: Some(Stroke::from("mine")), right: None, bottom: Some(Stroke::from("mine")), left: None };
let glyph = Glyph::new("x").expect("one grapheme cluster, no control character");

// A table from outside the core is a catalog built from its own rows.
let mine = GlyphCatalog::from_rules([(key, glyph) /* , more rows */]);

// Combine it with anything else, in a stated order.
let catalog = GlyphCatalog::union([GlyphCatalog::light(), mine]);
```

Guarantees:

- Everything on the right-hand side of `GlyphCatalog::from_rules`'s call is reachable through
  `monospace_core`'s public API — no private item is needed (FR-001, FR-009).
- `union`'s order matters only where two catalogs claim the same key; a catalog's rows that no other
  catalog claims answer identically no matter where it sits in the list (Acceptance Scenario 1.3).
- Once `union` returns, no operation on the resulting `GlyphCatalog` reports which catalog in the
  list, and so which table, answered a given key (FR-003, SC-007) — it is a `GlyphCatalog` like any
  other, built the same way `light()` builds one.
- A catalog with no rows, or the same catalog given to `union` twice, changes nothing about the
  result beyond what its rows already claimed once (Edge Cases).

## `monospace-glyph-sets` promises to whoever depends on it

```rust
let ascii_table: monospace_core::GlyphCatalog = monospace_glyph_sets::ascii();
```

Guarantees:

- `ascii()` returns a `GlyphCatalog` holding exactly the fifteen rows `docs/glyph-sets.md` records
  under _ASCII_ — no row added, none omitted, none altered (FR-011).
- That catalog alone answers all fifteen non-empty combinations of the `ascii` stroke (FR-012,
  SC-005), and produces no character outside printable ASCII when used to render (SC-004).
- The crate depends on nothing but `monospace-core`'s public API, compiles for
  `wasm32-unknown-unknown` (FR-010), and defines no type of its own — the value it hands back is
  already a type `monospace-core` names.

## What is explicitly not promised

- No operation exists to ask a `GlyphCatalog` which table a rule came from (FR-003, SC-007) — this
  is a limit, not an oversight.
- Neither crate gains a way to remove a rule from a catalog, replace one already claimed, or update
  a catalog in place; `from_rules` and `union` each produce a new value, and nothing mutates an
  existing one.
- No command-line flag is added anywhere in this feature to choose which tables go into a catalog
  (FR-019) — this contract governs library code, not the CLI's argument surface.
