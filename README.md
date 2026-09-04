# Monospace

> ASCII diagramming, built one honest commit at a time.

**Monospace** is a set of applications and libraries for creating diagrams using ASCII characters —
think boxes, arrows, flowcharts, and architecture sketches, all rendered in plain monospaced text.

## Why this project exists

This isn't primarily about shipping a diagramming tool. It's a deliberate exercise in learning how
to work effectively with **Spec-Driven Development** using Claude, and in practicing solid **Rust
architecture, design, and idioms** along the way. The diagrams are the vehicle; the process is the
point.

## How we're building it

- **Methodology:** Spec-Driven Development, iterating with Claude at every step.
- **Language:** Rust.
- **Cadence:** small, incremental commits. Each one aims to leave the project in a working,
  demonstrable state — no long-lived broken branches, no giant reveals.

## Roadmap

The plan is to grow outward from a solid core:

1. **Core library (Rust)** — the diagramming engine.
2. **Simple CLI application** — a minimal, non-interactive console app to exercise the library and
   produce diagrams from the terminal.
3. **Interactive TUI application** — a richer terminal UI with mouse support, editing, and
   drag-and-drop diagram manipulation.
4. **WebAssembly library** — compile the core to WASM for use in the browser.
5. **Web application** — a front-end built on top of the WASM library.

Each stage builds on a working foundation from the one before it.

## Current status

🚧 **Just getting started.** This repository currently contains only the output of `cargo init`.
There's no functionality yet — this README exists to set the intent and direction before the first
real spec is written.

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
