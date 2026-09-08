# Roadmap

**Status:** Living document — direction, amended as phases land and as capabilities turn into specs.
Nothing here is a commitment, and nothing here is a rule: the rules are in
[the constitution](../.specify/memory/constitution.md).

**Created:** 2026-09-08

## Phases

The plan grows outward from a solid core. Each stage builds on a working foundation from the one
before it.

| #   | What                                       | State         |
| --- | ------------------------------------------ | ------------- |
| 1   | `monospace-core`, the library              | current focus |
| 2   | `monospace-cli`, minimal, non-interactive  | current focus |
| 3   | `monospace`, the interactive TUI           | next up       |
| 4   | WebAssembly library, from `monospace-core` | not started   |
| 5   | Web application, on the WASM library       | not started   |

**Why the CLI comes before the TUI.** The split between phases 2 and 3 is deliberate: a minimal
non-interactive consumer proves the core library's API is usable with no ceremony, before the added
complexity of an interactive terminal UI can hide an awkward API behind its own machinery.

**Why phases 4 and 5 shape today's design.** Both reuse `monospace-core`, which is why its public
API carries no CLI, TUI or terminal assumptions. That is a principle rather than an aspiration —
[The core stays portable](../.specify/memory/constitution.md#vii-the-core-stays-portable) — and the
quality gate's `wasm` step is what enforces it.

**Open for phase 3:** which Rust TUI crate or framework backs it, if any. It is a tooling choice and
becomes an ADR when phase 3 starts, not before. The part of that question which belongs to the
domain — what the core promises about editing something already stamped — is in
[the model's open questions](model.md#10-open-questions).

## Candidate capabilities

Direction, not commitments. Each becomes its own spec before anything is implemented; they are
listed here to establish where the work is heading.

- Define a minimal diagram description: nodes, connections, labels.
- Compute a layout for nodes on a 2D character grid.
- Render nodes as boxes, using box-drawing or plain ASCII characters.
- Render connections between nodes as lines and arrows.
- Read a diagram description from a file or stdin, through the CLI.
- Write the rendered diagram to stdout or a file.

Underneath every one of them sit two mechanisms they all need: a buffer of cells that figures write
into, and the rendering of that buffer to characters. Both are described in [the model](model.md),
which specs slice rather than restate, and which owns the vocabulary they use.
