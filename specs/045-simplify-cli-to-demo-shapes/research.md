# Research: Simplify CLI to demo shapes

## Decision: parse the description with `serde` + `serde_json`

- **Decision**: Add `serde` (with the `derive` feature) 1.0.229 and `serde_json` 1.0.151 as
  dependencies of `monospace-cli` only. Both published 2026-07, well past the seven-day freshness
  rule; confirmed with the maintainer before adding (constitution, _Dependencies_).
- **Rationale**: `serde_json` is the conventional way to read structured JSON in Rust
  ([CLAUDE.md](../../CLAUDE.md), _Working with me_), and deriving `Deserialize` on the description
  types gets both parsing and the error locations FR-013/FR-014 need for free: a `serde_json::Error`
  carries a line and column for a syntax error, and the same type is returned for a data error — a
  missing field, a wrong type, or an unrecognized shape kind when the enum uses an internally tagged
  representation (`#[serde(tag = "kind")]`), whose message names the unrecognized tag. One error
  type covers both FR-013 and FR-014 instead of two parsing paths.
- **Alternatives considered**:
  - A hand-rolled JSON parser: rejected, reinvents what a well-established crate already does
    correctly, against the constitution's preference for idiomatic crates over reinvention for an
    infrastructure concern.
  - `serde_json::Value` matched by hand instead of a derived `Deserialize`: rejected, pushes field
    presence and type checking into this feature's own code where `serde`'s derive already does it,
    and loses the automatic error locations.

## Decision: no new dependency for argument parsing

- **Decision**: Read the single optional path argument with `std::env::args()`. No argument-parsing
  crate (e.g. `clap`) is added.
- **Rationale**: the spec's own assumptions rule out flags, subcommands and options beyond one
  optional path (_Assumptions_, "No new command-line surface beyond one optional path"). Counting
  `std::env::args().skip(1)` and matching on zero, one or more arguments is the whole of what FR-001
  and the "more than one argument" edge case need; a parsing crate would be a second dependency ask
  for a problem the standard library already covers.
- **Alternatives considered**: `clap` — rejected as disproportionate to a single positional
  argument, and every dependency is its own ask to the maintainer.

## Decision: embed the demonstration file with `include_str!`

- **Decision**: Ship the demonstration description as `crates/monospace-cli/assets/demo.json` and
  embed its text at compile time with `include_str!`, rather than reading it from a runtime path
  relative to the executable or the working directory.
- **Rationale**: FR-022 requires the no-argument run to work from any working directory and from a
  binary copied outside a checkout, which a runtime-relative path cannot guarantee on every
  platform. `include_str!` resolves against the source file's location at compile time, so the text
  travels inside the binary itself. FR-023 ("the only source of what the binary carries") holds
  because the constant is the file's text unmodified — parsed through the same `Description` type as
  any other path, so passing that file's path explicitly (User Story 2, scenario 3) parses
  byte-identical text and produces byte-identical output.
- **Alternatives considered**:
  - A path relative to `CARGO_MANIFEST_DIR` resolved at runtime: rejected, only works when run from
    inside a checkout with the source tree present, which FR-022 explicitly rules out.
  - Writing the demonstration as Rust literals instead of a JSON file: rejected, fails FR-004 and
    FR-023 — it would not be "readable and editable as text by someone who has not read the source
    code" and would duplicate the description format's own shape in a second notation.

## Decision: one `serde_json::Error` covers every failure path except a missing file

- **Decision**: `std::fs::read_to_string` failing is reported as FR-012 (path named, to stderr).
  Every other failure — malformed JSON (FR-013) or a well-formed document that is not a valid
  diagram (FR-014) — is a `serde_json::Error` from a single `serde_json::from_str::<Description>`
  call, reported by its own `Display` output, which already names the location and, for an
  internally tagged enum, the offending variant name.
- **Rationale**: keeps `main` a linear read → parse → render pipeline with one error type per stage
  instead of a hand-written distinction between "malformed" and "invalid", which `serde_json`
  already draws internally more precisely than this feature needs to.
- **Alternatives considered**: a custom error enum distinguishing I/O, syntax and semantic errors —
  rejected as an abstraction the two-stage pipeline (read, then parse) does not need; `main` matches
  on two `Result`s, not three error kinds.

## Decision: the description format's shapes map directly onto the core's public shape types

- **Decision**: `Description`, `Canvas` and `ShapeDescription` (with `Box`, `Line`, `Arrow`
  variants) are new types private to `monospace-cli`, each field a direct JSON rendering of a field
  already public on `monospace_core::{BoxShape, Line, Arrow, Endpoint}`, `Pos`, `Size`, `Stroke`,
  `Glyph`, `Orientation`, `Direction` and `StampMode`. Converting one to the matching core shape is
  a field-by-field construction, not a computation.
- **Rationale**: FR-007 requires every parameter a Rust caller has to be reachable from the file;
  mirroring the existing public fields is the only way to guarantee the count in SC-003 matches
  without inventing a parallel vocabulary. FR-019/ADR-0035 keep this mapping entirely inside
  `monospace-cli`: `monospace-core` gains nothing.
- **Alternatives considered**: a description schema that only covers what the shipped demonstration
  happens to use — rejected, would not satisfy FR-007's "every parameter" requirement and would make
  SC-003's count a guess rather than a check against the core's actual public fields.
