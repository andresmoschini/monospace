# Quickstart: validating the mixing tables

Prerequisites: the toolchain in `rust-toolchain.toml` (installs itself), no other setup — this
feature adds no dependency.

## Build and unit-test the crate

```sh
cargo build --workspace
cargo test --workspace
```

Expected: everything compiles, and the new unit tests pass alongside the existing ones — no existing
test in `monospace-glyph-sets` changes (only `monospace-cli`'s demo-crossing test changes, below).

## User Story 1 — draw with the Light and Heavy mixing table

```sh
cargo test -p monospace-glyph-sets mixing_light_and_heavy
```

Expected: every one of the fifty combinations _Mixing Light and Heavy_ records produces the
character the table publishes (SC-002), and a catalog built from `light()`, `heavy()` and
`mixing_light_and_heavy()` alone renders a diagram with no light/heavy overlap byte-identically to
one built from `light()` and `heavy()` alone (acceptance scenario 2).

## User Story 2 — draw with the Light and Double mixing table

```sh
cargo test -p monospace-glyph-sets mixing_light_and_double
```

Expected: all eighteen covered combinations produce the table's character (SC-001); at least one of
the thirty-two uncovered combinations still degrades to one stroke, exactly as a catalog without the
table would (SC-005).

## User Story 3 — draw with the Light Round and Heavy mixing table

```sh
cargo test -p monospace-glyph-sets mixing_light_round_and_heavy
```

Expected: all fifty combinations produce the character the table publishes (SC-004), matching
`mixing_light_and_heavy` with `light-round` in place of `light`.

## User Story 4 — draw with the Light Round and Double mixing table

```sh
cargo test -p monospace-glyph-sets mixing_light_round_and_double
```

Expected: all eighteen covered combinations produce the table's character (SC-003); the analogous
uncovered combination still degrades (SC-005).

## No key collision across all nine tables (FR-007)

```sh
cargo test -p monospace-glyph-sets union
```

Expected: a catalog built from any combination of the five single-stroke tables and the four mixing
tables answers every key from whichever table defines it, regardless of the order the tables went
into it.

## The shipped demonstration (FR-008, FR-009, SC-006)

```sh
cargo run -p monospace-cli
cargo test -p monospace-cli
```

Expected: the demonstration's output differs from before this feature at exactly the positions where
two of its shapes overlap on a combination one of the four mixing tables covers, and nowhere else
(SC-006) — in particular, the two crossings corrected from Light+Light-Round and Heavy+Double to
Light Round+Double and Light Round+Heavy now show a mixed character instead of a degraded one, and
so do the existing Light+Double and Light+Heavy crossings. The updated
`crossings_between_the_new_tables_degrade_to_whichever_figure_is_in_front` test (renamed to reflect
that these crossings now mix rather than degrade) asserts the new characters at all six positions.

## Nothing outside the four pairs changed (FR-010, edge case)

A diagram that touches none of the four mixing tables' keys renders byte-identically to before this
feature. There is no dedicated fixture for this in the plan; it is what the unchanged parts of
`crates/monospace-cli/tests/cli.rs` (the ASCII/Light crossings, the hand-written single-box and
three-shape tests) already continue to prove by staying green.

## Row-for-row against the document (mirrors feature 056's SC-004 practice)

Manual check when writing or reviewing the four new `const` tables: every row in
`MIXING_LIGHT_AND_DOUBLE`, `MIXING_LIGHT_AND_HEAVY`, `MIXING_LIGHT_ROUND_AND_DOUBLE` and
`MIXING_LIGHT_ROUND_AND_HEAVY` is compared, key by key, against the matching row in
`docs/glyph-sets.md`'s four _Mixing ..._ sections. The unit tests above prove completeness and
non-collision; this comparison is what proves each row is the _correct_ character, not just _a_
character.

## Portability

```sh
cargo check -p monospace-glyph-sets --target wasm32-unknown-unknown
```

Expected: compiles for the WebAssembly target, unchanged from before this feature — only data is
added to a crate already in the gate's `wasm` step.

## The whole gate

```sh
cargo xtask check
```

Expected: green, on a fresh clone, per principle IV.
