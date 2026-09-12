# Quickstart: validating mixed arm strokes

Prerequisites: the toolchain in `rust-toolchain.toml` (installs itself), no other setup — this
feature adds no dependency.

## Build and unit-test the workspace

```sh
cargo build --workspace
cargo test --workspace
```

Expected: everything compiles, and the new unit tests pass alongside the existing ones. Every
existing test that constructs an `Arm::Set` changes (a `Copy` variant gaining a payload cannot be
done without touching them); no existing test's _expected output_ changes.

## User Story 1 — draw the junction two strokes make

```sh
cargo test -p monospace-core
```

Expected: a cell whose arms carry `light` on top/bottom and `heavy` on left/right renders `┿` (not
either stroke's own cross) once a catalog holds Light, Heavy and their mixing table; the same arms
with the strokes swapped between sides render `╂`; a cell whose arms all carry one stroke renders
exactly what it renders today; front-to-back and back-to-front stamping still produce the same
buffer (Acceptance Scenarios 1–5).

## User Story 2 — see the difference in the shipped demonstration

```sh
cargo run -p monospace-cli
```

Expected: the last figure group's light/double and light/heavy crossings print `╪ ╫ ┿ ╂` where they
print `╬ ┼ ╬ ╋`-family uniform characters today; every other character is byte-identical to before.
This is the one check FR-014/SC-003 name — accepted on observation, not by test. Compare the output
by hand against the expected block in `spec.md`'s User Story 2, and record what was observed in
`docs/learning-log.md` once the CLI-wiring commit lands.

## User Story 3 — build a catalog from a mixing table

```sh
cargo test -p monospace-glyph-sets
```

Expected: each of `light_double()`, `light_heavy()`, `light_round_double()`, `light_round_heavy()`
holds exactly its documented row count (18/50/18/50, SC-001); their spot-checked rows — including
`╪ ╫` and `┿ ╂` — render the character `docs/glyph-sets.md` publishes (SC-002); a catalog built from
all nine tables this project ships answers every key correctly regardless of the order they are
combined in, checked in at least two orders (FR-011, SC-007).

## User Story 4 — keep uncovered junctions degrading as they do today

```sh
cargo test -p monospace-core degrade
```

Expected: a light/double combination the mixing table does not record renders the same character
with and without that table in the catalog (SC-005); a Light/Light-Round or Heavy/Double mix, which
no table pairs, still degrades — the fifth and sixth figure pairs in the demonstration stay
unchanged; a diagram that mixes no strokes renders byte-identically before and after this feature
(SC-006).

## Row-for-row against the document

Manual check when writing or reviewing the four new `const` tables: every row in `LIGHT_DOUBLE`,
`LIGHT_HEAVY`, `LIGHT_ROUND_DOUBLE` and `LIGHT_ROUND_HEAVY` is compared, key by key, against the
matching row in `docs/glyph-sets.md`'s four _Mixing …_ sections. The unit tests above prove row
counts and a sample of characters; this comparison is what proves the transcription itself is
faithful.

## The shipped demo file is untouched (FR-015)

```sh
git diff --stat crates/monospace-cli/assets/demo.json
```

Expected: no output — the file is byte-identical to `main`, verified by diff, once the feature is
complete (SC-004).

## Portability

```sh
cargo check -p monospace-core --target wasm32-unknown-unknown
cargo check -p monospace-glyph-sets --target wasm32-unknown-unknown
```

Expected: both compile for the WebAssembly target, as they already did before this feature — an
owned `Stroke` per arm and four more `const` tables need nothing new here.

## The whole gate

```sh
cargo xtask check
```

Expected: green, on a fresh clone, per principle IV — after each of the three commits this feature
lands as, not only at the end.
