# Monospace

> ASCII diagramming, built one honest commit at a time.

**Monospace** is a set of applications and libraries for creating diagrams using ASCII characters —
think boxes, arrows, flowcharts, and architecture sketches, all rendered in plain monospaced text.

## Why this project exists

This isn't primarily about shipping a diagramming tool. It's a deliberate exercise in learning how
to work effectively with **Spec-Driven Development** using Claude, and in practicing solid **Rust
architecture, design, and idioms** along the way. The diagrams are the vehicle; the process is the
point.

The idea, part of the domain logic, the design and the model come from a private project of the same
author that will not be published — see [the brief](docs/brief.md) for the full note. The Rust
architecture, the implementation and the decisions recorded under `docs/decisions/` are this
repository's own, even where some carry over an approach already worked out in that project.

## How we're building it

- **Methodology:** Spec-Driven Development, iterating with Claude at every step.
- **Language:** Rust.
- **Cadence:** small, incremental commits. Each one aims to leave the project in a working,
  demonstrable state — no long-lived broken branches, no giant reveals.

The scope, the constraints and what is deliberately left out are in [the brief](docs/brief.md).

## Roadmap

The plan is to grow outward from a solid core:

1. **Core library (Rust)** — `monospace-core`, the diagramming engine.
2. **Simple CLI application** — `monospace-cli`, a minimal, non-interactive console app to exercise
   the library and produce diagrams from the terminal.
3. **Interactive TUI application** — `monospace`, a richer terminal UI with mouse support, editing,
   and drag-and-drop diagram manipulation.
4. **WebAssembly library** — compile the core to WASM for use in the browser.
5. **Web application** — a front-end built on top of the WASM library.

Each stage builds on a working foundation from the one before it.

## Current status

🚧 **Foundations, not features.** The workspace is laid out, `monospace-cli` prints a line it asks
`monospace-core` for, and every commit is checked by a ten-step quality gate that the pre-commit
hook and CI run identically. There is no diagramming yet — the next increment is the first slice of
real domain logic.

What exists is the scaffolding: formatting, linting, spelling, documentation and tests, all enforced
rather than merely intended. Including the check that keeps the core compiling for WebAssembly, so
stage 4 stays reachable instead of becoming a rewrite.

## Getting started

```sh
rustup toolchain install   # reads rust-toolchain.toml
cargo xtask setup          # installs the Node tooling the gate needs

cargo run -p monospace-cli # the one thing it can do so far
cargo xtask check          # the whole quality gate
```

[CONTRIBUTING.md](CONTRIBUTING.md) covers all of it properly, including what each check owns and
what to do when one fails.

### Layout

| Path                    | What it is                                                      |
| ----------------------- | --------------------------------------------------------------- |
| `crates/monospace-core` | The library. All domain logic lives here, and nothing else.     |
| `crates/monospace-cli`  | The command-line application. Holds no logic of its own.        |
| `xtask/`                | Repository automation. `cargo xtask check` is the gate.         |
| `docs/brief.md`         | Scope, principles, and what is deliberately out of scope.       |
| `docs/decisions/`       | Why things are the way they are, recorded as they were decided. |

## Guiding principles

- **Demonstrable over complete.** Every commit should do _something_ you could show someone, even if
  small.
- **Process over product.** Getting good at Spec-Driven Development with Claude, and at writing
  idiomatic, well-architected Rust, matters more than the diagramming tool itself.
- **Iterate in the open.** Specs, decisions, and course corrections are part of the history, not
  hidden behind a single squashed commit.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

The MIT License is a permissive open source license that allows you to use, copy, modify, and
distribute the code freely, with only the requirement to include the original copyright notice.

---

_This project is a learning journey as much as a piece of software. Expect the specs, the code, and
this README to evolve together._
