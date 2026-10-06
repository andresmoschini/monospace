# Spike: the chrome, on ratatui and `tuirealm`

A throwaway that answers seven questions about building the application's chrome — a top menu bar
with drop-downs, a pop-up, a status bar — on `ratatui 0.30.2` with `tuirealm 4.1.0` and
`crossterm 0.29.0`. It exists because [#174](https://github.com/andresmoschini/monospace/issues/174)
decided the framework and [#189](https://github.com/andresmoschini/monospace/issues/189) is the
screen that would show whether the decision was right. Writing the screen is the only way to know.

**Nothing here is proposed for merging.** It is not a workspace member, the gate does not build it,
and no version it pins is a version the application has agreed to. Read it, run it, disagree with
it, and delete the branch.

## Running it

```sh
cd spike/chrome
cargo run --bin q3-popup     # the menu bar, the drop-down, the canvas, the pop-up, the status bar
cargo run --bin q7-mouse     # four ways a click can reach a control, and the one that fails
cargo run --bin q5-tree      # the resolved dependency tree
cargo run --bin q6-boilerplate   # the line counts below, recounted
```

`q3-popup` and `q7-mouse` print what they observed and then a verdict. `q4-snapshot` prints a screen
through two render paths and says whether they agree.

The pop-up needs a terminal; the rest print to stdout.

## The questions, and what was observed

Each is a program under `src/bin/`, named for its question. Nothing is asserted here that the
programs do not print.

| Bin              | Question                                                  | Answer                                                    |
| ---------------- | --------------------------------------------------------- | --------------------------------------------------------- |
| `q1-state`       | Can the state live outside the component tree?            | Yes, in a crate with zero dependencies                    |
| `q1-negative`    | Can it live outside in the same crate?                    | No — it must compile, to show it cannot run               |
| `q2-cycle`       | Does the Elm cycle break "the action is the only writer"? | It coexists, but nothing enforces it                      |
| `q3-popup`       | Does a pop-up compose over an ASCII canvas?               | Yes, and the canvas is restored afterwards                |
| `q4-snapshot`    | Can the screen be snapshotted?                            | Yes, through `TestTerminalAdapter`                        |
| `q5-tree`        | Is there one `crossterm`?                                 | One: `0.29.0`                                             |
| `q6-boilerplate` | How many lines is a menu bar and a pop-up?                | 124 with `tuirealm`, 103 without, at parity               |
| `q7-mouse`       | Does hit-testing work?                                    | It is ours to write, and `tuirealm` costs us the geometry |

### What confirmed the decision

- **`src/state.rs` compiles and runs with no dependencies at all.** `App` and `Action` import
  neither `ratatui` nor `tuirealm`, and `standalone/` is that claim as a build: a crate whose
  `[dependencies]` is empty and which `use`s the same module. `q1-state` then mutates it from a
  component's `update`.
- **The pop-up works and leaves nothing behind.** `q3-popup` reports
  `canvas bytes restored after close: true` — byte-identical, not approximately.
- **One `crossterm`.** `q5-tree` prints the resolved tree; the interesting part is that
  `ratatui-crossterm v0.1.2` is not a second `crossterm`.

### What the decision did not expect

- **`q6-boilerplate`: `tuirealm` does not save lines.** 124 against 103 at the same functionality. A
  properly structured version of the same screen is 360. The case for `tuirealm` is not line count.
- **`q7-mouse`: a component cannot see where it was drawn.** `Component` has `view`, `attr`,
  `query`, `state`, `perform` and `on` — no `area()`. The `Rect` arrives in `view(f, area)` and is
  discarded, so a control that must answer a click is handed its own geometry back through `attr()`.
  The alternative is `MouseEventClause` ranges in absolute screen coordinates, which do not follow
  the layout when it changes and fail silently when they stop matching. Plain `ratatui` gives the
  area in the closure and lets you store it; `tuirealm` makes you round-trip it.
- **`q2-cycle`: four paths can write the state, not one.** A component holding `Rc<RefCell<App>>`
  can mutate `App` inside `on()`, which makes the `Msg` decorative. `tuirealm`'s own demo keeps
  state inside a component and mutates it in `perform()`. The claim that the action enum is the only
  writer holds by review, not by construction.

## Two things worth reading first

- **`src/chrome.rs`** is the real thing — a menu bar, a drop-down, a pop-up with a Close button, and
  per-component hit-testing. It is 360 lines and is the honest cost of the screen.
- **`src/bin/q7-mouse.rs`** is the one to read if you only read one. It is where `tuirealm` is worse
  than plain `ratatui`, and it is the part of the framework's model that a mouse-driven canvas
  feels.

## License

MIT, as the repository is.
