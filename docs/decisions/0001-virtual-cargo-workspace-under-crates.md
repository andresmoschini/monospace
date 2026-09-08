---
status: accepted
date: 2026-09-04
decision-makers: Andrés Moschini
---

# Lay out the project as a virtual Cargo workspace under `crates/`

## Context and Problem Statement

The repository is currently a single binary package named `monospace`, containing only
`src/main.rs`. The brief commits to four artifacts across five phases: `monospace-core`,
`monospace-cli`, an interactive TUI application, and a WebAssembly library compiled from the core.
Only the first two are in scope now.

The brief also states a constraint that outlives this phase: because phases 4 and 5 will reuse
`monospace-core`, the core must avoid letting CLI-specific, TUI-specific or terminal-specific
assumptions leak into its public API. That is a claim about a boundary, and a boundary that exists
only in prose is a boundary that erodes.

Deciding the layout now, with no domain code written, is the cheapest this decision will ever be.
Deferring it does not avoid the cost; it postpones it to a point where files have to be moved and
imports rewritten.

## Decision Drivers

- The brief's "process over product" principle: when the fast path and the path that teaches better
  Rust design diverge, prefer the second. Multi-crate design — API boundaries, dependency graphs,
  features versus crates — is the central Rust architecture topic of this project.
- The core must stay compilable for `wasm32-unknown-unknown`, which means it must not acquire
  terminal or CLI dependencies, not even optional ones behind default features.
- This phase's actual goal is a quality gate. Its configuration should live in one place rather than
  be duplicated per crate.
- The brief's "demonstrable over complete" principle pulls the other way: a workspace enables no new
  capability today.

## Considered Options

- **A** — Single package with `src/lib.rs` and `src/main.rs`; defer the workspace.
- **B** — Virtual workspace with members under `crates/`.
- **C** — Virtual workspace with members flat at the repository root.

## Decision Outcome

Chosen option: **B, a virtual workspace with members under `crates/`**, because it is the only
option that makes the core's dependency isolation structural rather than a matter of discipline, and
because `[workspace.lints]` is the right home for the quality gate this phase exists to build.

The decision fixes four things:

1. **Layout.** The root `Cargo.toml` is a virtual manifest: `[workspace]` with no `[package]`.
   Publishable crates live in `crates/monospace-core` and `crates/monospace-cli`. Repository tooling
   lives at `xtask/`, outside `crates/`, and is marked `publish = false` — `crates/` means
   "artifacts that ship", and mixing tooling into it blurs that.
2. **Names.** `monospace-cli` ships a binary called `monospace-cli`. The name `monospace` is
   reserved for the interactive TUI application of phase 3. This resolves a collision between the
   brief, which named the phase-3 TUI `monospace`, and `CLAUDE.md`, which had `monospace-cli`
   shipping a binary called `monospace`; two crates in one workspace cannot both produce that
   binary.
3. **No `default-members`.** In a virtual manifest with no `default-members`, a bare `cargo build`,
   `cargo test` or `cargo doc` already operates on every member, so local commands and the commands
   CI runs cover the same set. Setting `default-members` would narrow the bare commands to a subset
   while CI kept covering everything, which is the standard way a quality gate starts lying.
4. **Shared metadata and lints** are centralized in `[workspace.package]` and `[workspace.lints]`,
   with members inheriting via `field.workspace = true`.

The brief's principle 1 previously implied that a bare `cargo run` must produce output. That is
relaxed to `cargo run -p monospace-cli`, and the brief is amended accordingly.

### Consequences

- Good, because the core's dependency list is enforced by Cargo rather than by review. A CLI
  argument parser cannot reach the core's dependency graph by accident, which keeps the WebAssembly
  phase reachable without feature gymnastics.
- Good, because lint configuration, shared metadata and, later, shared dependency versions live in
  exactly one file.
- Good, because it frees the crate name `monospace` for the TUI. Since publishing to crates.io is
  intended eventually, names are worth allocating deliberately.
- Bad, because `cargo run` needs `-p`. Measured during the migration, this is narrower than it first
  appeared: bare `cargo build`, `cargo test` and `cargo doc` cover every member already, and while
  the workspace holds a single binary even a bare `cargo run` resolves it. It breaks as soon as a
  second binary exists — which is when `xtask` lands — because Cargo can no longer determine which
  binary to run. Option A would have avoided this for the life of the project.
- Bad, because a virtual manifest has no edition to infer the dependency resolver from, so
  `resolver = "3"` has to be declared explicitly or Cargo falls back to the version 1 resolver.
  Cargo does warn when it is missing, so the mistake is visible rather than silent — but that
  warning comes from Cargo rather than from `rustc`, so denying warnings in the quality gate will
  not turn it into a failure.
- Neutral, because `xtask` becomes a workspace member and is therefore formatted, linted and tested
  like production code. That is the intent, but it does mean tooling code is held to the same bar.

### Confirmation

Enforced mechanically, not by review: the quality gate runs
`cargo check -p monospace-core --target wasm32-unknown-unknown`. If a terminal-specific or
CLI-specific dependency reaches the core, that check fails. The boundary the brief asserts in prose
is therefore verified on every commit rather than asserted.

The layout itself is confirmed by the absence of a `[package]` section in the root manifest: with a
virtual manifest, an accidental root-level crate cannot compile unnoticed.

## Pros and Cons of the Options

### A — Single package with `src/lib.rs` and `src/main.rs`

The binary consumes the library only through its public API, so the boundary is real and checked by
the compiler, not merely a convention.

- Good, because there is no ceremony: `cargo run`, `cargo test` and `cargo doc` work with no flags.
- Good, because it is the smallest structure that satisfies today's requirements, which is what
  "demonstrable over complete" argues for.
- Bad, because CLI dependencies become dependencies of the whole package, and therefore of the
  library's dependency graph. Reaching WebAssembly later would mean `default-features = false` and
  `#[cfg(feature = ...)]`, trading structural complexity for feature complexity — the worse of the
  two, because features are invisible in the file tree and combinatorial in testing.
- Bad, because `[workspace.lints]` is unavailable, so the lint configuration has to move the moment
  a second crate appears.
- Bad, because the crate name `monospace` stays bound to the CLI, which conflicts with the brief's
  plan for phase 3.

### B — Virtual workspace with members under `crates/`

- Good, because each crate declares only the dependencies it needs.
- Good, because the repository root stays legible: with four or five crates plus `docs/`, `xtask/`
  and `.github/`, the `crates/` prefix separates "what ships" from "everything else".
- Good, because it is the layout most multi-crate Rust projects use, so it needs no explanation to
  someone arriving at the repository.
- Bad, because of the `-p` friction and the explicit resolver, both noted above.
- Bad, because it is more structure than today's code justifies. The mitigation is timing: with zero
  domain code, the migration is a file move rather than a refactor.

### C — Virtual workspace with members flat at the repository root

Identical to B in Cargo semantics; the difference is purely how the tree reads.

- Good, because paths are shorter and there is no intermediate directory that holds nothing itself.
- Bad, because with five crates plus `docs/`, `xtask/`, `.github/` and `.githooks/`, the root
  becomes a flat list in which code and non-code are indistinguishable.

## Reversibility

**Cheap now, and only now.** Moving between A, B and C is `git mv` plus editing two manifests while
there is no domain code. That cost grows with every file added, and grows discontinuously once
anything is published.

**Expensive later, but not permanent:** the dependency split. If the core accumulates CLI
dependencies and stops compiling for WebAssembly, undoing that is a real refactor rather than a
move. This is the risk the Confirmation check above exists to prevent.

**Effectively permanent:** the crate names, from the moment anything is published to crates.io. The
name reservation in point 2 above is made now specifically because it cannot be made later.

## Confidence

High (~90%).

The one objection to this option was that a bare `cargo run` no longer works, which conflicted with
the brief's principle 1. That objection was resolved by relaxing the brief rather than by contorting
the layout with `default-members`, which removes the reason the confidence was not higher.

What remains unknown, and what it would change: whether the phase-3 TUI is a separate crate or the
CLI grows into it. If the CLI grows into the TUI, the crate count drops from four to three and
option A becomes more defensible — though not enough to reverse this, because the WebAssembly phase
alone justifies keeping the core's dependency graph clean.

What would prove this wrong: the `-p` friction turning into a habit of running the wrong command, or
`crates/` still holding exactly two members a year from now. Either would mean the structure was
bought for a future that never arrived.

## More Information

- [The roadmap](../roadmap.md#phases) for the phase plan, and
  [The core stays portable](../../.specify/memory/constitution.md#vii-the-core-stays-portable) for
  the constraint on the core's public API. Both were sections 3 and 5 of the brief when this record
  was written.
- The WebAssembly check that enforces the core's isolation is part of the quality gate; see
  `CONTRIBUTING.md`.
