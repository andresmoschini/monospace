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
author that will not be published — see [the model](docs/model.md) for the full note. The Rust
architecture, the implementation and the decisions recorded under `docs/decisions/` are this
repository's own, even where some carry over an approach already worked out in that project.

## How we're building it

- **Methodology:** Spec-Driven Development, iterating with Claude at every step.
- **Language:** Rust.
- **Cadence:** small, incremental commits. Each one aims to leave the project in a working,
  demonstrable state — no long-lived broken branches, no giant reveals.

The scope, the constraints and what is deliberately left out are in
[the constitution](.specify/memory/constitution.md).

## Roadmap

Core library, then a minimal CLI, then an interactive TUI, then WebAssembly, then the web app — each
stage on a working foundation from the one before it. [The roadmap](docs/roadmap.md) has the phases,
why the CLI comes before the TUI, and the capabilities the work is heading towards.

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
| `docs/model.md`         | The domain's design, its provenance, and its open questions.    |
| `docs/roadmap.md`       | The phases, and the capabilities the work heads towards.        |
| `docs/decisions/`       | Why things are the way they are, recorded as they were decided. |

## Guiding principles

Process over product, demonstrable increments, and claims that are measured rather than assumed.
Seven of them, stated as rules a plan can be checked against, are in
[the constitution](.specify/memory/constitution.md) — which is where they are enforced from, not
merely listed.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

The MIT License is a permissive open source license that allows you to use, copy, modify, and
distribute the code freely, with only the requirement to include the original copyright notice.

---

_This project is a learning journey as much as a piece of software. Expect the specs, the code, and
this README to evolve together._
