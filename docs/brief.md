# Monospace — Initial Project Brief / Spec

**Status:** Draft — pre-implementation

**Data:** 2026-09-04

**Purpose:** This document is the starting spec for Spec-Driven Development. It defines intent, scope, and constraints before any feature-level spec is written. Feature specs will branch off from this document and should stay consistent with it.

---

## 1. Vision

Monospace is a set of applications and libraries for creating diagrams using ASCII characters — boxes, arrows, connectors, flowcharts, and simple architecture sketches, rendered entirely in plain monospaced text.

The project's real objective is **not** to ship a polished diagramming tool. It exists to build hands-on proficiency in:

- **Spec-Driven Development**, working iteratively with Claude as a collaborator.
- **Rust architecture, design, and idiomatic practice**.

The diagramming domain was chosen because it's small enough to reason about end-to-end, but rich enough to raise real design questions (parsing, layout, rendering, extensibility).

## 2. Guiding Principles

These principles constrain every future spec and commit:

1. **Demonstrable over complete.** Every commit that lands leaves the project building, with all validations green and `cargo run` producing output. From the first commit this may be trivial (a version banner); from the first domain increment onwards it should show real behaviour, however small.
1. **Process over product.** When there's tension between "the fastest way to a feature" and "the way that teaches good Rust design or good spec-driven practice," prefer the latter.
2. **Incremental scope.** Specs should be sliced thin. Prefer several small specs over one large one.
3. **Traceable decisions.** Architectural and design choices should be documented as they're made, not reconstructed after the fact.

## 3. Scope

### In scope (this phase)

- A **Rust core library** (`monospace-core`) implementing the diagramming logic: input parsing, layout/positioning, and ASCII rendering.
- A **simple CLI application** (`monospace-cli`) — minimal and non-interactive — that uses `monospace-core` to produce diagrams from the terminal (e.g. read a description, render, print/write output).

### Explicitly out of scope (for now)

- WebAssembly bindings.
- Web front-end application.
- Non-terminal GUI interfaces.
- Persistence, collaboration, or export formats beyond plain text.

### Future phases (not started, informs today's design)

1. Core library `monospace-core` (Rust) ✅ current focus
2. Simple CLI application `monospace-cli` ✅ current focus
3. Interactive TUI application `monospace` (mouse, editing, drag-and-drop) — next up
4. WebAssembly library — compiled from `monospace-core`
5. Web application — built on the WASM library

The split between phases 2 and 3 is deliberate: the simple CLI proves the core library's API is usable with minimal ceremony, before taking on the added complexity of an interactive terminal UI. Because phases 4–5 will also reuse `monospace-core`, the core should avoid CLI-specific, TUI-specific, or terminal-specific assumptions leaking into its public API.

## 4. Functional Direction (high-level, to be refined into feature specs)

The following are candidate capabilities for the core library and CLI. Each should become its own spec before implementation — they are listed here only to establish direction, not as commitments.

- Define a minimal diagram description (nodes, connections, labels).
- Compute a layout for nodes on a 2D character grid.
- Render nodes as boxes using box-drawing or plain ASCII characters.
- Render connections between nodes as lines/arrows.
- Read a diagram description from a file or stdin via the CLI.
- Output the rendered diagram to stdout or a file.

## 5. Non-Functional Constraints

- **Language:** Rust (stable toolchain).
- **Dependencies:** kept minimal and justified. Prefer the standard library for domain logic, especially early on, to maximize the design-learning value. For infrastructure concerns (e.g. CLI parsing, TUI rendering), use idiomatic, well-established crates rather than reinventing them.
- **Testing:** each feature spec should define its own testing expectations (unit tests at minimum for core logic).
- **Documentation:** public library APIs should be documented with rustdoc as they're introduced, not retrofitted later.

## 6. Working Process

1. Write a small spec for the next slice of functionality.
2. Discuss/iterate on the spec with Claude before implementing.
3. Implement against the spec.
4. Commit with a message that reflects the demonstrable state achieved.
5. Append an entry to `docs/learning-log.md` for the increment, with two or three bullets covering: something learned about Rust design/idioms, something learned about the Spec-Driven Development process, and (optionally) a decision or trade-off worth remembering later.

## 7. Open Questions (domain)

To be resolved in early feature specs, not here:

- What minimal diagram description format will the core accept (custom DSL vs. structured input)?
- How will node layout be computed (fixed grid vs. computed positions)?
- What's the initial set of supported shapes/connectors?
- Which Rust TUI crate/framework (if any) will back the interactive application, and what does that imply for how the core library exposes mutable state for editing?

## 8. Open Questions to Be Resolved in Early Feature Specs (tooling)

- **MSRV:** what minimum supported Rust version will the project target, and how will it be enforced (CI check, `rust-version` in `Cargo.toml`)?
- **Toolchain:** which Rust toolchain channel (stable/beta/nightly) and edition will be used, and will any nightly-only features be relied upon?
- **`rust-toolchain.toml`:** will the repository pin an exact toolchain via `rust-toolchain.toml`, and if so, how will it be kept in sync with MSRV?
- **Workspace structure:** will `monospace-core`, `monospace-cli`, and `monospace` (and later the WASM crate) live in a single Cargo workspace, and how will crates/directories be laid out (e.g. `crates/monospace-core`, `crates/monospace-cli`, `crates/monospace-tui`)?

---

*This brief sets direction, not final answers. It should be revisited and amended as the project's early iterations surface better understanding — but changes to it should be deliberate, not incidental.*
