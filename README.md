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
stage on a working foundation from the one before it. The phases and the capabilities the work is
heading towards live on the [GitHub Project](https://github.com/users/andresmoschini/projects/2);
why the CLI comes before the TUI is
[ADR-0022](docs/decisions/0022-non-interactive-cli-before-the-tui.md).

## Current status

🚧 **It draws.** Boxes, optionally filled; lines; and arrows that route themselves between two
endpoints, however far apart those are. Where two shapes meet, their strokes compose into a single
cell rather than one overwriting the other, so a crossing becomes a junction and two boxes share a
corner — and the front-most shape's fill is what covers what is behind it:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 20, "height": 7 } },
  "shapes": [
    { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light", "fill": "░" },
    { "kind": "box", "at": { "x": 3, "y": 1 }, "size": { "width": 4, "height": 3 },
      "stroke": "double", "fill": "░" },
    { "kind": "line", "at": { "x": 0, "y": 5 }, "len": 9, "orientation": "horizontal",
      "stroke": "light" },
    { "kind": "line", "at": { "x": 5, "y": 3 }, "len": 4, "orientation": "vertical",
      "stroke": "light" },
    { "kind": "arrow", "from": { "at": { "x": 8, "y": 1 }, "leaving": "right", "head": "►" },
      "to": { "at": { "x": 15, "y": 4 }, "leaving": "up", "head": "▲" },
      "stroke": "light" },
    { "kind": "box", "at": { "x": 16, "y": 3 }, "size": { "width": 4, "height": 3 },
      "stroke": "heavy", "fill": "▓" } ] }
-->

```text
┌──┐
│░░╠══╗ ►──────┐
└──╢░░║        │
   ╚═╤╝        │┏━━┓
     │         ▲┃▓▓┃
─────┼───       ┗━━┛
     │
```

<!-- /render -->

A diagram is an ordered set of shapes, each carrying the identity the order is addressed by, and it
draws front to back into a window the caller gives it — which is what makes moving one shape forward
change the picture. Nine glyph tables are in reach: the core's own Light and eight more held beside
it, so the same box draws in ASCII, Double, Heavy and their combinations without a second code path.

Behind that, and all of it enforced rather than merely intended:

- **110** tests in `monospace-core`, **217** across the workspace.
- **1856** renderings across 8 snapshots, pinning every arrangement of the arrow's route — a range
  too wide to assert by hand, so a change to it is
  [reported](docs/decisions/0053-report-a-characterization-instead-of-reviewing-it.md) rather than
  reviewed.
- An **eleven**-step quality gate the pre-commit hook and CI run identically, including the check
  that keeps every crate but the CLI compiling for WebAssembly, so stage 4 stays reachable instead
  of becoming a rewrite.

Still missing, and named here rather than implied: an editing surface, the TUI, and persistence.

## Getting started

```sh
rustup toolchain install   # reads rust-toolchain.toml
cargo xtask setup          # installs the Node tooling the gate needs

cargo run -p monospace-cli                      # a demonstration: two pictures, one shape moved
cargo run -p monospace-cli -- path/to.json      # one picture, for the description you give it
cargo xtask check                               # the whole quality gate
```

A description is JSON: a canvas and a list of shapes, documented in
[the format's contract](specs/079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md).
`monospace-cli` holds no domain logic of its own — it turns a description into a `Diagram` and draws
it.

[CONTRIBUTING.md](CONTRIBUTING.md) covers all of it properly, including what each check owns and
what to do when one fails.

### Layout

| Path                          | What it is                                                      |
| ----------------------------- | --------------------------------------------------------------- |
| `crates/monospace-core`       | The library. All domain logic lives here, and nothing else.     |
| `crates/monospace-diagram`    | The model: a diagram as an ordered set of shapes, drawable.     |
| `crates/monospace-glyph-sets` | The glyph tables the core does not ship as built-in data.       |
| `crates/monospace-cli`        | The command-line application. Holds no logic of its own.        |
| `xtask/`                      | Repository automation. `cargo xtask check` is the gate.         |
| `docs/model.md`               | The domain's design, its provenance, and its open questions.    |
| `docs/decisions/`             | Why things are the way they are, recorded as they were decided. |

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
