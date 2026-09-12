# Contract: mixed arm strokes

This project has no network or process boundary; its external interface is the public Rust API
`monospace-core` and `monospace-glyph-sets` expose to whoever depends on them, and the command-line
output `monospace-cli` produces. This feature changes one existing part of the first (`Arm`,
`StrokeCell`, `Cell::glyph_str`), extends the second the same way features 054 and 056 already did
(see that feature's `contracts/glyph-set-extension-point.md` and `contracts/glyph-tables.md`), and
changes the third in one place.

## What `monospace-core` promises to whoever depends on it

```rust
use monospace_core::{Arm, GlyphCatalog, Stroke, StrokeCell};

let cell = StrokeCell {
    base: Stroke::from("light"),
    top: Arm::Set(Stroke::from("light")),
    right: Arm::Set(Stroke::from("heavy")),
    bottom: Arm::Set(Stroke::from("light")),
    left: Arm::Set(Stroke::from("heavy")),
};

// key() reads each arm's own stroke; a catalog holding a Light-with-Heavy mixing rule answers it:
let mixed = GlyphCatalog::union([monospace_glyph_sets::light_heavy()]);
assert_eq!(mixed.glyph(&cell.key()).map(|g| g.as_str()), Some("┿"));

// A cell whose arms all carry the base stroke renders exactly as it always has:
let uniform = StrokeCell { base: Stroke::from("light"), top: Arm::Set(Stroke::from("light")),
    right: Arm::Set(Stroke::from("light")), bottom: Arm::Set(Stroke::from("light")),
    left: Arm::Set(Stroke::from("light")) };
```

Guarantees:

- `Arm::Set` carries the `Stroke` that runs to that side (FR-001). `Closed` and `Unset` are
  unchanged: no stroke runs there, and they read the same as each other when a key is built.
- The character a cell resolves to is looked up from each arm's own stroke first (FR-002). When that
  exact combination has no rule, every connected arm is moved to the cell's `base` stroke and looked
  up again (FR-004, unchanged in meaning and scope from
  [ADR-0009](../../docs/decisions/0009-degrade-a-cell-to-its-base-stroke.md)); when neither matches,
  there is no glyph.
- When a figure is stamped over another, an arm the upper figure leaves `Unset` keeps the stroke it
  was drawn with — it is not redrawn in the upper figure's `base` (FR-003).
- Which figure decides an arm, which figure sets the base stroke, the two stamp modes, and the three
  arm states are unchanged (FR-005): stamping front to back with `Below` still produces the same
  buffer as stamping back to front with `Above`.
- A cell whose arms all carry its own base stroke — every cell any version of this project could
  build before this feature — renders exactly the character it rendered before this feature
  (SC-006).

## What `monospace-glyph-sets` promises to whoever depends on it

```rust
let catalog = monospace_core::GlyphCatalog::union([
    monospace_glyph_sets::ascii(),
    monospace_glyph_sets::double(),
    monospace_glyph_sets::heavy(),
    monospace_glyph_sets::light_round(),
    monospace_glyph_sets::light_double(),
    monospace_glyph_sets::light_heavy(),
    monospace_glyph_sets::light_round_double(),
    monospace_glyph_sets::light_round_heavy(),
]);
```

Guarantees:

- `light_double()`, `light_heavy()`, `light_round_double()` and `light_round_heavy()` each return a
  `GlyphCatalog` holding exactly the rows `docs/glyph-sets.md` records under the matching _Mixing …_
  section — 18, 50, 18 and 50 respectively (FR-006–FR-009, SC-001).
- Each is reachable the same way `double()`, `heavy()` and `light_round()` already are: a
  zero-argument public function returning a `GlyphCatalog`, built through `monospace-core`'s public
  API alone, with no privilege a table written outside this project would not equally have (FR-010).
- None of the nine tables this crate and `monospace-core` now ship between them (five single-stroke,
  four mixing) defines a key any other of the nine defines, so a catalog built from any selection of
  them, in any order, answers every key from the table that names it (FR-011, SC-007).

## What `monospace-cli` promises

- The shipped demonstration's catalog is built from all nine tables above (FR-013); its shipped demo
  file, `assets/demo.json`, is unchanged (FR-015). Running the CLI with no arguments produces the
  block `spec.md`'s User Story 2 names, differing from before this feature at exactly the eight
  crossing positions its light/double and light/heavy figure pairs share — accepted on observation,
  not pinned by a test (FR-014, SC-003).

## What is explicitly not promised

- No CLI flag or argument is added to choose a catalog (FR-016) — the demonstration's catalog is
  still fixed at the source level, just built from four more functions.
- No single-stroke table is added or changed, and no figure group in the demonstration is added,
  moved or removed — only the characters at existing crossings change (FR-016).
- No fallback chain between strokes, and no third lookup beyond the exact key and the base-stroke
  key: a mixture no table records renders exactly as it renders today, whether or not some other
  mixing table happens to be loaded (FR-004, SC-005).
- `GlyphCatalog::from_rules` and `GlyphCatalog::union` are unchanged — this feature's contract is a
  changed `Arm`/`StrokeCell`/`Cell` in `monospace-core`, additive data in `monospace-glyph-sets`,
  and one changed call site in `monospace-cli`.
