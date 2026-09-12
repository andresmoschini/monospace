# Quickstart: validating the Double, Heavy and Light Round tables

Prerequisites: the toolchain in `rust-toolchain.toml` (installs itself), no other setup — this
feature adds no dependency.

## Build and unit-test the crate

```sh
cargo build --workspace
cargo test --workspace
```

Expected: everything compiles, and the new unit tests pass alongside the existing ones — no existing
test in `monospace-glyph-sets` or elsewhere changes.

## User Story 1 — draw with the Double table

```sh
cargo test -p monospace-glyph-sets double
```

Expected: a catalog built from `monospace_glyph_sets::double()` alone answers all fifteen of its own
non-empty combinations (SC-001), and a box rendered against it uses only the Double table's own
box-drawing characters and space (SC-005).

## User Story 2 — draw with the Heavy table

```sh
cargo test -p monospace-glyph-sets heavy
```

Expected: the same two properties, for `heavy()` and the Heavy table (SC-002, SC-005).

## User Story 3 — draw with the Light Round table

```sh
cargo test -p monospace-glyph-sets light_round
```

Expected: the same two properties, for `light_round()` and the Light Round table, including its four
rounded corners reading distinctly from Light's square ones (SC-003, SC-005).

## Mixing tables (FR-008, SC-006)

```sh
cargo test -p monospace-glyph-sets union
```

Expected: a catalog built from two or more of ASCII, Double, Heavy and Light Round answers a key
from each table correctly, and the result does not depend on the order the tables went into it,
since none of the four contests a key another already holds.

## Row-for-row against the document (SC-004)

Manual check when writing or reviewing the three new `const` tables: every row in `DOUBLE`, `HEAVY`
and `LIGHT_ROUND` is compared, key by key, against the matching row in `docs/glyph-sets.md`'s
_Double_, _Heavy_ and _Light Round_ tables. The unit tests above prove completeness and the
character set; this comparison is what proves each row is the _correct_ character, not just _a_
character.

## Nothing else changed (FR-010, SC-007)

```sh
cargo run -p monospace-cli
cargo test -p monospace-cli
```

Expected: the shipped demonstration prints byte-identically to before this feature, and the existing
CLI subprocess tests pass unchanged — this feature adds functions nothing yet calls by default.

## Portability

```sh
cargo check -p monospace-glyph-sets --target wasm32-unknown-unknown
```

Expected: compiles for the WebAssembly target, as it already did before this feature — the crate was
already in the gate's `wasm` step from feature 054; this only adds data to it.

## The whole gate

```sh
cargo xtask check
```

Expected: green, on a fresh clone, per principle IV.
