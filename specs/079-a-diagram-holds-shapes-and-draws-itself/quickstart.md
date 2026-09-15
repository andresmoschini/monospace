# Quickstart: validating a diagram that holds shapes and draws itself

Prerequisites: the toolchain in `rust-toolchain.toml`, which installs itself, and
`cargo xtask setup` once for the Node tooling the gate uses. This feature adds no dependency, so
there is nothing else to install.

## Before touching anything: capture the demonstration

SC-003 and TE-007 are confirmed by comparison, and the "before" side of it only exists now.

```sh
cargo run -p monospace-cli > /tmp/demo-before.txt
```

Keep that file until the end. Nothing automatic pins this output.

## Build and test the workspace

```sh
cargo build --workspace
cargo test --workspace
```

Expected: everything compiles, `monospace-diagram`'s new unit tests pass, and the only existing
tests that changed are the CLI's — fixtures that lost a `mode` field, and one test removed with the
field it checked.

## User Story 1 — a figure survives being drawn

Covered by unit tests in `monospace-diagram`, against
[contracts/diagram-api.md](contracts/diagram-api.md):

```sh
cargo test -p monospace-diagram
```

Expected: a diagram holding a box and a line draws into a window and, drawn again into an equal
window, produces an equal buffer (TE-003); an empty diagram leaves its buffer untouched; a box
hanging over the edge of the window draws what falls inside and nothing else, with no error
(TE-002); and each of the three kinds draws exactly what the core shape it constructs draws with the
same parameters (TE-005).

## User Story 2 — the order decides who wins an overlap

Same command, different tests. Expected: two overlapping boxes drawn front to back with `Below`
produce the same buffer as the same two core shapes stamped back to front with `Above` (TE-001); the
same two shapes in opposite orders produce different buffers (TE-004); two crossing lines make a
junction rather than one interrupting the other; and a filled box in front of a line hides it where
they overlap.

## User Story 3 — the application draws through a diagram

```sh
cargo test -p monospace-cli
cargo run -p monospace-cli > /tmp/demo-after.txt
diff /tmp/demo-before.txt /tmp/demo-after.txt
```

Expected: the tests pass, and `diff` reports nothing — the demonstration's output is byte-identical
although `assets/demo.json` has changed and the format it is written in no longer has `mode`
(SC-003). Record the observation in `docs/learning-log.md` when the increment closes; that is all
TE-007 has behind it.

To see the format itself, write a file per
[contracts/description-format.md](contracts/description-format.md) and pass its path:

```sh
cargo run -p monospace-cli -- path/to/description.json
```

Two overlapping boxes rendered this way match the two corresponding core shapes stamped back to
front with `Above` — TE-006, which the subprocess tests in `crates/monospace-cli/tests/cli.rs`
cover.

## The gate

```sh
cargo xtask check
```

Expected: green, including the `wasm` step now checking `monospace-diagram` against
`wasm32-unknown-unknown` alongside the two crates it checks today (FR-003, SC-004). To confirm the
step really reaches the new crate rather than passing by default, break it once on purpose: put
`#[cfg(target_arch = "wasm32")] compile_error!("reached");` in its `lib.rs`, watch `wasm` fail while
`build` and `test` stay green, and take it back out. That is the only way to tell a step that checks
the crate from one that never sees it, since a crate this pure has nothing it could fail on by
accident.
