# Monospace

> ASCII diagramming, built one honest commit at a time.

**Monospace** is a set of applications and libraries for creating diagrams using ASCII characters —
think boxes, connectors, flowcharts, and architecture sketches, all rendered in plain monospaced
text.

## Why this project exists

This isn't primarily about shipping a diagramming tool. It's a deliberate exercise in building a
real library and command-line tool in **plain, well-documented Rust**, and in keeping its
documentation honest as it grows. The diagrams are the vehicle; the craft is the point.

The idea, part of the domain logic, the design and the model come from a private project of the same
author that will not be published — see [the model](docs/model.md) for the full note. The Rust
architecture, the implementation and the design notes in the source are this repository's own, even
where some carry over an approach already worked out in that project.

## How we're building it

- **Language:** Rust.
- **Cadence:** small, incremental commits. Each one aims to leave the project in a working,
  demonstrable state — no long-lived broken branches, no giant reveals.
- **Criterion:** for each desired change, make the change easy, then make the easy change — Kent
  Beck's rule. The unblocking is often the hard part, and a change spent only on making the next one
  easy is a legitimate commit. "Small and incremental" describes how a commit ends, not how much it
  is allowed to contain.

## Roadmap

Core library, then a minimal CLI, then an interactive TUI, then WebAssembly, then the web app — each
stage on a working foundation from the one before it. The phases and the capabilities the work is
heading towards live on the [GitHub Project](https://github.com/users/andresmoschini/projects/2);
why the CLI comes before the TUI is a decision this project has already made.

## Current status

🚧 **It draws.** Boxes, optionally filled; lines; and connectors that route themselves between two
endpoints, however far apart those are. Where two shapes meet, their strokes compose into a single
cell rather than one overwriting the other, so a crossing becomes a junction and two boxes share a
corner — and the front-most shape's fill is what covers what is behind it:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 20, "height": 7 } },
  "next_id": 7,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light", "fill": "░" },
    { "kind": "box", "id": "#2", "at": { "x": 3, "y": 1 }, "size": { "width": 4, "height": 3 },
      "stroke": "double", "fill": "░" },
    { "kind": "line", "id": "#3", "at": { "x": 0, "y": 5 }, "len": 9, "orientation": "horizontal",
      "stroke": "light" },
    { "kind": "line", "id": "#4", "at": { "x": 5, "y": 3 }, "len": 4, "orientation": "vertical",
      "stroke": "light" },
    { "kind": "connector", "id": "#5", "from": { "at": { "kind": "point", "x": 7, "y": 1 }, "leaving": "right",
        "terminal": { "kind": "glyph", "glyph": "◄" } },
      "to": { "at": { "kind": "point", "x": 16, "y": 4 }, "leaving": "left",
        "terminal": { "kind": "arm" } },
      "stroke": "light" },
    { "kind": "box", "id": "#6", "at": { "x": 16, "y": 3 }, "size": { "width": 4, "height": 3 },
      "stroke": "heavy", "fill": "▓" } ] }
-->

```text
┌──┐
│░░╠══╗◄───┐
└──╢░░║    │
   ╚═╤╝    │    ┏━━┓
     │     └────┨▓▓┃
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
- **1856** renderings across 8 snapshots, pinning every arrangement of the connector's route — a
  range too wide to assert by hand, so a change to it is reported rather than reviewed.
- A **twelve**-step quality gate, run identically by the `pre-commit` hook and by CI — including the
  check that keeps every crate but the CLI compiling for WebAssembly, so stage 4 stays reachable
  instead of becoming a rewrite.

Still missing, and named here rather than implied: an editing surface, the TUI, and persistence.

The model behind all of this is provisional, and is meant to move. It records a partial
understanding written before the code that implements it, and it is amended whenever reality
contradicts it — see [the model moves](CONTRIBUTING.md#the-model-moves). The domain vocabulary is
ours to rename as the understanding changes, so nothing here should be read as a settled design.

## Getting started

```sh
rustup toolchain install   # reads rust-toolchain.toml
cargo xtask setup          # installs the Node tooling the gate needs

cargo run -p monospace-cli                      # a demonstration: two pictures, one shape moved
cargo run -p monospace-cli -- path/to.json      # one picture, for the description you give it
cargo xtask check                               # the whole quality gate
```

`monospace-cli` holds no domain logic of its own — it turns a description into a `Diagram` and draws
it.

[CONTRIBUTING.md](CONTRIBUTING.md) covers all of it properly, including what each check owns and
what to do when one fails.

### Layout

| Path                          | What it is                                                   |
| ----------------------------- | ------------------------------------------------------------ |
| `crates/monospace-core`       | The library. All domain logic lives here, and nothing else.  |
| `crates/monospace-diagram`    | The model: a diagram as an ordered set of shapes, drawable.  |
| `crates/monospace-glyph-sets` | The glyph tables the core does not ship as built-in data.    |
| `crates/monospace-cli`        | The command-line application. Holds no logic of its own.     |
| `xtask/`                      | Repository automation. `cargo xtask check` is the gate.      |
| `docs/model.md`               | The domain's design, its provenance, and its open questions. |

## Guiding principles

Claims are measured rather than assumed, and the two that can be are enforced on every commit: a
picture in a tracked document is either generated from a description the file carries or labelled
hypothetical, and a change to one is reported by `cargo xtask render` rather than reviewed. The rest
are habits the pull request body asks about.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

The MIT License is a permissive open source license that allows you to use, copy, modify, and
distribute the code freely, with only the requirement to include the original copyright notice.

---

_This project is a learning journey as much as a piece of software. Expect the code and this README
to evolve together._
