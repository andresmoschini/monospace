# Quickstart: validating glyph tables from outside the core

Prerequisites: the toolchain in `rust-toolchain.toml` (installs itself), no other setup — this
feature adds no dependency.

## Build and unit-test both libraries

```sh
cargo build --workspace
cargo test --workspace
```

Expected: everything compiles, and the new unit tests pass alongside the existing ones —
`monospace-core`'s builder tests (ordering, first-claim-wins) and `monospace-glyph-sets`'s table
test (the ASCII table answers all fifteen of its own non-empty combinations, the same shape as
`monospace-core`'s existing Light test). No existing test in either crate changes.

## User Story 1 — a table from outside answers keys like any other

Covered by unit tests in `monospace-core` that build a `GlyphCatalog` from a `GlyphSet` defined in
the test module itself (not `monospace-core`'s own data) together with `GlyphSet::light()`, per
`contracts/glyph-set-extension-point.md`. Run:

```sh
cargo test -p monospace-core
```

Expected: a key only the outside set holds, a key only Light holds, and a key both claim (checked
both orders) all answer as the spec's acceptance scenarios describe (SC-006).

## User Story 2 — a diagram drawn in ASCII alone

```sh
cargo test -p monospace-glyph-sets
```

Expected: a catalog built from `monospace_glyph_sets::ascii()` alone answers every one of its
fifteen rows (SC-005), and a rendered box's every character is `+`, `-`, `|` or a space — checked
over the whole output, not sampled (SC-004).

## User Story 3 — the shipped demonstration

```sh
cargo run -p monospace-cli
```

Expected: one diagram, printed with no arguments, containing both box-drawing characters (from
Light) and `+`/`-`/`|` characters (from ASCII) — SC-003. Reading the printed output at the two
crossing cells confirms SC-009: the crossing with the ASCII figure in front shows an ASCII character
there, and the crossing with the Light figure in front shows a Light character there, at the same
relative position in each figure.

```sh
cargo test -p monospace-cli
```

Expected: the existing subprocess tests continue to pass unchanged in behavior (SC-008 for every
diagram that is not the demo), and the tests extended for this feature confirm the demo's new
content.

## Portability (FR-010)

```sh
cargo check -p monospace-core -p monospace-glyph-sets --target wasm32-unknown-unknown
```

Expected: both compile for the WebAssembly target. This is the same check `cargo xtask check` runs
as its `wasm` step once the step's package list is extended.

## The whole gate

```sh
cargo xtask check
```

Expected: green, on a fresh clone, exactly as principle IV requires before the gate is trusted for
this change.
