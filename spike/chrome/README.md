# Spike: the chrome, on ratatui and `tuirealm`

A throwaway that answers seven questions about building the application's chrome — a top menu bar
with drop-downs, a pop-up, a status bar — on `ratatui 0.30.2` with `tuirealm 4.1.0` and
`crossterm 0.29.0`. It exists because [#174](https://github.com/andresmoschini/monospace/issues/174)
decided the framework and [#189](https://github.com/andresmoschini/monospace/issues/189) is the
screen that would show whether the decision was right. Writing the screen is the only way to know.

**Nothing here is proposed for merging.** It is not a workspace member, the gate does not build it,
and no version it pins is a version the application has agreed to. Read it, run it, disagree with
it, and delete the branch.

## Two of these are applications, and they are the point

Everything else on this branch prints what it measured and exits. These two take the whole screen,
take the mouse, and answer to keys and clicks — so the framework question can be answered by using
it rather than by reading about it.

```sh
cd spike/chrome
cargo run --bin q6-tuirealm     # the SAME screen on tuirealm
cargo run --bin q6-plain-full   # the SAME screen on plain ratatui
```

Both take `F1` for the menu, `F2` for the dialog, `Esc` to close the menu, and mouse capture is on,
so `Close` in the dialog can be clicked. `q` bumps a counter in both. `Q` quits in the `tuirealm`
one.

**Try this on `q6-tuirealm`, and it is the whole finding.** Open the dialog with `F2`, then **resize
the terminal**. The dialog moves. The click target does not — the hit-test compares against two
constants in `on()` that nothing updates, and the button stops responding without a word from the
program. Then run `q6-plain-full`, resize, and click: the button follows, because that version keeps
the `Rect` the draw closure handed it. The evidence and the reasoning are in
[`reports/geometry.md`](reports/geometry.md).

Both fall back to printing the same picture when stdout is not a terminal, which is what
`q6_boilerplate` compares.

## The other six, which print and exit

```sh
cargo run --bin q3-popup     # the menu bar, the drop-down, the canvas, the pop-up, the status bar
cargo run --bin q7-mouse     # four ways a click can reach a control, and the one that fails
cargo run --bin q5-tree      # the resolved dependency tree
cargo run --bin q6-boilerplate   # points at reports/line-count.txt
cargo run --bin q1-state     # the state, read and mutated from a component
cargo run --bin q4-snapshot  # one screen through two render paths, and whether they agree
```

`q1-negative` does not run: it is the counter-example that has to fail to compile to mean anything,
so build it and read the error rather than running it.

## The questions, and what was observed

Each is a program under `src/bin/`, named for its question. Nothing is asserted here that the
programs do not print.

| Bin             | Question                                                  | Answer                                                    |
| --------------- | --------------------------------------------------------- | --------------------------------------------------------- |
| `q1-state`      | Can the state live outside the component tree?            | Yes, in a crate with zero dependencies                    |
| `q1-negative`   | Can it live outside in the same crate?                    | No — it must compile, to show it cannot run               |
| `q2-cycle`      | Does the Elm cycle break "the action is the only writer"? | It coexists, but nothing enforces it                      |
| `q3-popup`      | Does a pop-up compose over an ASCII canvas?               | Yes, and the canvas is restored afterwards                |
| `q4-snapshot`   | Can the screen be snapshotted?                            | Yes, through `TestTerminalAdapter`                        |
| `q5-tree`       | Is there one `crossterm`?                                 | One: `0.29.0`                                             |
| `q6-tuirealm`   | What does the screen cost on `tuirealm`?                  | An application — see above — and 151 lines against 103    |
| `q6-plain-full` | What does it cost on plain `ratatui`?                     | The same application, 103 lines                           |
| `q7-mouse`      | Does hit-testing work?                                    | It is ours to write, and `tuirealm` costs us the geometry |

The counts are in [`reports/line-count.txt`](reports/line-count.txt), which also says why the first
measurement taken was wrong and in which direction. The geometry is in
[`reports/geometry.md`](reports/geometry.md).

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

- **`tuirealm` does not save lines.** 151 against 103 at the same functionality, and 360 for a
  properly structured version of the same screen. The first count taken said 124 against 44 and
  looked like a saving; it was counting a draw-only `ratatui` program against one with an event
  loop, a message enum, an update function and a hit-test. `reports/line-count.txt` says so in full.
  The case for `tuirealm` is not line count in either direction.
- **A component cannot see where it was drawn.** `Component` has `view`, `attr`, `query`, `state`,
  `perform` and `on` — no `area()`, no `rect()`, nothing on the trait that returns the geometry. The
  `Rect` arrives in `view(f, area)` and `View` discards it, so a control that must answer a click is
  handed its own geometry back through `attr()`, or is given absolute coordinate ranges that stop
  matching the layout and fail silently. Plain `ratatui` gives the area in the closure and lets you
  keep it. You can see this on screen by resizing `q6-tuirealm`.
- **`q2-cycle`: four paths can write the state, not one.** A component holding `Rc<RefCell<App>>`
  can mutate `App` inside `on()`, which makes the `Msg` decorative. `tuirealm`'s own demo keeps
  state inside a component and mutates it in `perform()`. The claim that the action enum is the only
  writer holds by review, not by construction.

## Three things worth reading first

- **Run `q6-tuirealm` and resize the terminal.** That is the finding reading cannot give you, and it
  takes ten seconds.
- **`src/bin/q7-mouse.rs`** is the one to read if you only read one. It goes through all four ways a
  click can reach a control and shows which two work.
- **`src/chrome.rs`** is the real thing — a menu bar, a drop-down, a pop-up with a Close button, and
  per-component hit-testing. It is 360 lines and is the honest cost of the screen.

## License

MIT, as the repository is.
