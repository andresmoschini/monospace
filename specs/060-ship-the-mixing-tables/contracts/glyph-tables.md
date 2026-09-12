# Contract: the four mixing tables

This project has no network or process boundary; its external interface is the public Rust API
`monospace-glyph-sets` exposes to whoever depends on it. This extends the contract features 054 and
056 already recorded — see 054's `contracts/glyph-set-extension-point.md` for what `monospace-core`
itself promises, and 056's `contracts/glyph-tables.md` for the four single-stroke tables. This file
covers only what is new: four more functions of the same shape, each mixing two strokes.

## What `monospace-glyph-sets` promises to whoever depends on it

```rust
let light_double: monospace_core::GlyphCatalog = monospace_glyph_sets::mixing_light_and_double();
let light_heavy: monospace_core::GlyphCatalog = monospace_glyph_sets::mixing_light_and_heavy();
let light_round_double: monospace_core::GlyphCatalog =
    monospace_glyph_sets::mixing_light_round_and_double();
let light_round_heavy: monospace_core::GlyphCatalog =
    monospace_glyph_sets::mixing_light_round_and_heavy();

// A mixing table combines with the two single-stroke tables its rows draw on to answer the
// combinations it covers with its own character, and to keep degrading everything else (FR-006):
let catalog = monospace_core::GlyphCatalog::union([
    monospace_glyph_sets::light_round(),
    monospace_glyph_sets::double(),
    light_round_double,
]);

// Any combination of the nine tables `monospace-glyph-sets` now ships answers every key from
// whichever table defines it, regardless of the order they went into the union (FR-007):
let full = monospace_core::GlyphCatalog::union([
    GlyphCatalog::light(),
    monospace_glyph_sets::ascii(),
    monospace_glyph_sets::double(),
    monospace_glyph_sets::heavy(),
    monospace_glyph_sets::light_round(),
    light_double,
    light_heavy,
    light_round_double,
    light_round_heavy,
]);
```

Guarantees:

- `mixing_light_and_double()` returns a `GlyphCatalog` holding exactly the eighteen rows
  `docs/glyph-sets.md` records under _Mixing Light and Double_ — no row added, none omitted, none
  altered (FR-001). Analogously, `mixing_light_and_heavy()` for _Mixing Light and Heavy_'s fifty
  rows (FR-002), `mixing_light_round_and_double()` for _Mixing Light Round and Double_'s eighteen
  (FR-003) and `mixing_light_round_and_heavy()` for _Mixing Light Round and Heavy_'s fifty (FR-004).
- A catalog built from a mixing table and the two single-stroke tables its rows draw on answers
  every combination the mixing table covers with the character it publishes, and continues to
  degrade every combination it does not cover exactly as a catalog without the mixing table would
  (FR-006).
- Each of the four is reachable the same way every table already in this crate is: a zero-argument
  public function returning a `GlyphCatalog`, built through `monospace-core`'s public API alone,
  with no privilege `ascii()` lacks (FR-005).
- None of the four mixing tables defines a key any of the other eight tables this crate now ships
  also defines. A catalog built from any combination of the nine answers every key from whichever
  table defines it, unaffected by the order the tables were added (FR-007).

## What is explicitly not promised

- No function is added that takes a table name, a stroke, or any other argument to choose which
  table to build, or that picks a mixing table automatically for a given pair of strokes — each
  table keeps its own zero-argument function, exactly as every existing table already does.
- No change to `GlyphCatalog::union`, `GlyphCatalog::from_rules`, `GlyphKey`, `Stroke`, or anything
  else in `monospace-core` — this feature's contract is additive data in `monospace-glyph-sets`
  alone.
- No command-line way to choose a catalog is added anywhere (FR-010) — the CLI's shipped
  demonstration is the only caller this feature wires up, and it is wired to the full union shown
  above, not to a chooser.
