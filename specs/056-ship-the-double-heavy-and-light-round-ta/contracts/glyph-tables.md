# Contract: the three new glyph tables

This project has no network or process boundary; its external interface is the public Rust API
`monospace-glyph-sets` exposes to whoever depends on it. This extends the contract feature 054
already recorded for `ascii()` — see that feature's `contracts/glyph-set-extension-point.md` for
what `monospace-core` itself promises. This file covers only what is new: three more functions of
the same shape.

## What `monospace-glyph-sets` promises to whoever depends on it

```rust
let double_table: monospace_core::GlyphCatalog = monospace_glyph_sets::double();
let heavy_table: monospace_core::GlyphCatalog = monospace_glyph_sets::heavy();
let light_round_table: monospace_core::GlyphCatalog = monospace_glyph_sets::light_round();

// Any of the four single-stroke tables the crate now ships can be combined, in any order,
// with no key contested between them (FR-008, SC-006):
let catalog = monospace_core::GlyphCatalog::union([
    monospace_glyph_sets::ascii(),
    double_table,
    heavy_table,
    light_round_table,
]);
```

Guarantees:

- `double()` returns a `GlyphCatalog` holding exactly the fifteen rows `docs/glyph-sets.md` records
  under _Double_ — no row added, none omitted, none altered (FR-001). Analogously, `heavy()` for
  _Heavy_ (FR-002) and `light_round()` for _Light Round_, including its four corner rows that read
  differently from Light's (FR-003).
- Each of the three, used alone, answers all fifteen non-empty combinations of its own stroke
  (FR-004, FR-005, FR-006) and produces no character outside its own fifteen glyphs and space when
  used to render (SC-005).
- Each is reachable the same way `ascii()` already is: a zero-argument public function returning a
  `GlyphCatalog`, built through `monospace-core`'s public API alone, with no privilege `ascii()`
  lacks (FR-007).
- No two of the four single-stroke tables the crate now ships (ASCII, Double, Heavy, Light Round)
  define a key with the same stroke name, so a catalog built from any combination of them lets every
  key render from whichever table names its stroke, regardless of the order they went into the
  catalog (FR-008, SC-006).

## What is explicitly not promised

- No function is added that takes a table name, a stroke, or any other argument to choose which of
  the four tables to build — each table keeps its own zero-argument function, exactly as `ascii()`
  already does.
- No change to `GlyphCatalog::union`, `GlyphCatalog::from_rules`, or anything else in
  `monospace-core` — this feature's contract is additive data in `monospace-glyph-sets` alone.
- No change to the CLI's default catalog or its shipped demonstration (FR-010) — this contract
  governs library code newly reachable, not what the CLI chooses to build from it.
- The four mixing sets (_Mixing Light and Double_, _Mixing Light and Heavy_, _Mixing Light Round and
  Double_, _Mixing Light Round and Heavy_) gain no function here and remain reference-only (FR-009).
