---
status: draft
date: 2026-10-10
---

# The window leaves the description and belongs to the caller

## Why now

`monospace-description` hands back a `Window` beside every diagram it reads, and its own rustdoc
says the type is temporary and expected to leave the description.
[#201](https://github.com/andresmoschini/monospace/pull/201) is the change that wrote that sentence,
having moved the format out of the command-line binary into a crate of its own and made it the only
crate that depends on `serde`. It settled the format as permanent — _Is a diagram serializable, and
in what?_ moved out of `docs/diagram-model.md` §11's open questions and the word _provisional_ came
out of four documents — and it left the one type in it that was never permanent still sitting in the
public API.

The model has held the other half for longer. `docs/diagram-model.md` §7 says the diagram measures
nothing and sizes nothing, that what falls outside the window is clipped exactly as a stamp outside
a buffer's window has always been, and that which part of a diagram to draw is the caller's question
in any case. **The format contradicts the model on the one field where it matters**, and the
contradiction is now visible in a public signature rather than in a private struct.

There is also a caller coming. The interactive application draws what fits the region below its
menu, at the scroll position it is at, and reads no window out of a file to do it
([#175](https://github.com/andresmoschini/monospace/issues/175)). A format that hands every caller a
window gives that application a value it must immediately ignore, and a reader of the format who is
not building a terminal has no way to tell which of its two fields are the caller's business.

**Nothing here is drawn, and that is deliberate.** Both options for where the window goes produce
the same picture, which is why §The decision argues in prose rather than with a `render` marker — a
picture cannot show the difference between two spellings of the same rule.

## Scope

### In

- `crates/monospace-description`: the `canvas` field, the `Canvas` and `Window` types, and the
  `From` conversions that built a `Window`. `parse` returns a `Diagram` and nothing else.
- `monospace-cli`: `--size` and `--origin`, a predefined window for the bare demonstration, and the
  removal of `canvas` from `assets/demo.json`.
- The grammar of a `<!-- render -->` marker: the window moves out of the JSON and onto the line that
  opens the marker. Every one of the twenty-three markers in the repository changes, and the four
  documents and four specs carrying them are redrawn by `cargo xtask render`.
- `xtask/src/render.rs`: `OPEN` stops being a constant compared for equality and becomes a prefix,
  with the attributes after it parsed by hand.
- `CONTRIBUTING.md`: the marker illustration, and the sentence naming the module that owns the
  format — which #201 left pointing at a file it deleted.

### Out

- **`deny_unknown_fields` on the description.** D5.
- **Measuring a diagram.** `docs/diagram-model.md` §11 asks how big a diagram is and answers that
  nothing measures one. Deriving a window from a bounding box would answer it, and D4 is why this
  change does not.
- **The dead paths #201 left behind**, which are their own issue: `CONTRIBUTING.md` and `specs/173`
  name `crates/monospace-cli/src/description.rs`, and `docs/diagram-demo.md` names
  `contracts/description-format.md`. Only the sentence in `CONTRIBUTING.md` about where the format
  is documented is fixed here, because this change rewrites the illustration beside it.
- **Writing a description.** The format is read and not written, and nothing here changes that.

## The decision

**D1 — the marker line carries the window, and its origin is optional.**

**Answer:** `<!-- render: 20x7` and `<!-- render: 7x5 at -3,-2`. **Why not** leaving the window in
the JSON and adding a second field beside it: that is a rename, and the field is what has to go.
**Why not** a header line inside the comment ahead of the JSON: it leaves `OPEN` an exact constant,
which is worth something — the grammar is strict and says so when it is broken — but it makes the
body of the comment something other than the description verbatim, and `render_one` writes that body
to a file as-is today. **Why not** a second HTML comment above the marker: then the attributes are
not inside the thing that has to be well-formed and `cargo xtask render --check` has nothing to
compare. **What it costs:** `OPEN` is a prefix rather than a constant, so the eleven-line grammar
check at `xtask/src/render.rs:220` becomes a parse, and the error message `expected()` prints has to
show the new form. **Answered by** the maintainer, in the session that wrote this.

**D2 — the size is required in a marker, and only the origin has a default.**

**Answer:** `<!-- render: 20x7` for the twenty markers at `(0,0)`, and the size written out in all
twenty-three. **Why not** making the size optional too, on the grounds that the application has a
default: a marker that leaves the size out would draw at whatever `monospace-cli` defaults to, and
that constant is in the binary rather than in the file. Changing it would then move generated
pictures in eight documents that never mention it, and the gate would report the difference rather
than the cause. **What it costs:** a marker is one token longer than the JSON line it replaces, and
the twenty markers at `(0,0)` could have been one word shorter. **Answered by** the maintainer, in
the session that wrote this.

**D3 — the command-line application defaults to the demonstration's window.**

**Answer:** given a path and no `--size`, it draws into `50x13` at `(0,0)`. **Why not** refusing
without a window: that is the stricter answer and it is wrong here, because the two callers are not
the same caller. `cargo xtask render` always passes a size — D2 makes every marker state one — so
the default is never what a generated picture uses, and the only person it serves is one running the
binary by hand. A default that only a human reaches for can be wrong without consequence. **Why
not** the terminal's own size: the path form prints into a Markdown fence, and a picture whose width
depends on the machine that rendered it is a picture `cargo xtask render --check` fails on for every
reader but the one who wrote it. **What it costs:** a number in the binary that no file states,
which is the cost D2 accepts for markers and this accepts for a person typing a path. **Answered
by** the maintainer, in the session that wrote this.

**D4 — the size is declared, not derived.**

**Answer:** every marker states the size it is drawn at. **Why not** a bounding box over the shapes:
twenty of the twenty-three markers choose a window deliberately larger than their figures — a `17x3`
window for shapes occupying `12x3` — and `specs/086` draws at `3x3` for the sole purpose of showing
a box half outside the window. A bounding box redraws nearly every picture in the repository and
turns one that teaches clipping into one that teaches nothing. **Why not** making it the
demonstration's window everywhere: the same pictures, at fifty columns. **What it costs:** the
author of a marker picks a size, and a size picked wrongly is a picture nobody checked. **Answered
by** the maintainer, in the session that wrote this.

**D5 — a description that still carries a `canvas` is read, and the canvas ignored.**

**Answer:** no `deny_unknown_fields`; `serde` ignores the field as it ignores any unknown one. **Why
not** refusing it by name: a stricter error is a change to the wording `ParseError` exists to pass
through unchanged, and fifteen tests assert on that wording verbatim. **Why not** making the field
optional and reading it when it is there: that keeps two sources for one window and needs a rule for
which wins, and the rule is the decision this change is removing. **What it costs:** a file written
against the old format draws at the default window instead of failing, which is a silent wrong
answer rather than a refusal. **Why that is acceptable here:** there is no writer, so nothing
outside this repository has ever produced a file, and every one of the twenty-three markers is
rewritten in the same increment. **What reopens it:** the writer. A format somebody's files depend
on owes a reader a version and a story for a field that stopped meaning something, and that is the
question `docs/diagram-model.md` §11 already defers to it. **Answered by** the maintainer, in the
session that wrote this.

**D6 — `Window` is deleted rather than moved to `monospace-core`.**

**Answer:** `parse` returns `Result<Diagram, ParseError>`, and `Window` leaves the public API
entirely. **Why not** promoting it to `monospace-core`: `Buffer::new` and `render` both take
`origin` and `size` as two arguments today, and a type that pairs them would be a convenience
nothing in the core asks for — it would be added to a crate that is never removed, for a caller that
has one. **Why not** keeping it in `monospace-description` as a type the caller fills in: a public
type in a crate about reading files, describing something no file mentions, is a name a reader will
expect to find populated. **Answered by** the maintainer, in the session that wrote this.

## Model slice

- `docs/diagram-model.md`: none of its rules change. §7 already holds that the window is the
  caller's and that the diagram sizes nothing, and this change makes the format agree with it rather
  than amending it. §11's _How big is a diagram?_ stays open, because D4 declines to answer it.
- `docs/model.md`: none.
- The format's own documentation is the crate doc of
  [`monospace-description`](../crates/monospace-description/src/lib.rs), which loses the `canvas`
  paragraph and the sentence promising the window would leave. §11's paragraph on the format is
  already correct and is not amended.

**There is no model sentence to add, and that is worth saying.** The rule this change enforces was
already written; what changes is a format contradicting it. A spec that amended the model here would
be adding a second statement of a rule that has one.

## Public surface

The deciding stage adds none of this; it is what the building stage publishes.

```rust
// monospace-description
pub fn parse(json: &str) -> Result<Diagram, ParseError>;

// monospace-cli
// monospace-cli [--size <width>x<height>] [--origin <x>,<y>] [path]
```

`monospace_core::Pos` and `monospace_core::Size` are what the flags parse into, and the private
`Pos` and `Size` that mirror them for `serde` stay: `box.at`, `box.size` and every connector
endpoint use them. Only `Canvas` and the two `From` implementations that built a `Window` go.

**There is no `Window` in either crate afterwards.** D6 is visible in the absence rather than in a
signature, and `render(&buffer, glyphs, origin, size)` keeps taking the pair as two arguments, which
is what lets a caller render a rectangle smaller than the buffer it drew into.

## Behavior

1. `parse` reads a description and hands back the diagram it holds, and nothing else.
2. A description carrying a `canvas` is read, and the canvas has no effect on what is drawn.
3. The command-line application takes a window from `--size` and `--origin`, and draws into it.
4. `--size` is required by a render marker and optional at the command line.
5. A render marker without an `at` draws from `(0,0)`.
6. A render marker with an `at` draws from the origin it names.
7. A render marker whose size or origin is not a number is reported by file and line, and no picture
   is written.
8. Given a path and no `--size`, the command-line application draws into `50x13` at `(0,0)`.
9. Given no argument at all, the demonstration draws into `50x13` at `(0,0)` and nothing on the
   command line changes that.
10. The demonstration's eight pictures before the walk-back are unchanged by this change.
11. `Buffer::owner` still answers with an `Offset` counted from the window's own corner, and the
    offsets `specs/086` cites resolve to the same cells they do today.
12. Every picture in every tracked document is the picture its marker declares, which
    `cargo xtask render --check` decides.

## Examples

**A marker with a size, and one with an origin.** Hypothetical in their new spelling — no code
produces them yet — but not in their pictures or their JSON: both markers are the ones the
repository already holds, `docs/diagram-demo.md:39` and `specs/086:259`, with the `canvas` line
moved onto the opening line. **Both pictures below were observed**, by running `monospace-cli` on
each description as it stands today.

````markdown
<!-- render: 17x3
{ "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light", "fill": "░" },
    { "kind": "line", "id": 2, "at": { "x": 6, "y": 1 }, "len": 4, "orientation": "horizontal",
      "stroke": "light" },
    { "kind": "connector", "id": 3,
      "from": { "at": { "kind": "point", "x": 12, "y": 1 }, "leaving": "right",
                "terminal": { "kind": "glyph", "glyph": "◀" } },
      "to":   { "at": { "kind": "point", "x": 16, "y": 1 }, "leaving": "left",
                "terminal": { "kind": "arm" } },
      "stroke": "light" } ] }
-->

```text
┌──┐
│░░│  ────  ◀────
└──┘
```

<!-- /render -->
````

````markdown
<!-- render: 7x5 at -3,-2
{ "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": -2, "y": -1 }, "size": { "width": 4, "height": 3 },
      "stroke": "double", "fill": "░" },
    { "kind": "box", "id": 3, "at": { "x": 6, "y": 3 }, "size": { "width": 4, "height": 3 },
      "stroke": "heavy" } ] }
-->

```text

 ╔══╗
 ║░░╟─┐
 ╚═╤╝ │
   └──┘
```

<!-- /render -->
````

The first is what a marker says when the origin is the one twenty of them use, and its JSON is the
description with the `canvas` line gone. The second is `specs/086`'s marker with its origin written
out, and the only difference between the two is the `at` — which is rule 5 against rule 6, and the
whole of what D2 costs. The trailing blanks the CLI pads each row to the window's width with are
trimmed here exactly as `xtask`'s `trim_trailing_blanks` trims them, which is why the two fences
look narrower than the windows they are drawn at.

**Those two pictures are the acceptance list for this change.** They are what the building stage
must produce, and they are here rather than described because a spec whose examples have not been
run is making a claim nobody checked.

**What the three flags do to one drawing.** Also hypothetical, and for the same reason: no code
parses them yet. It is written as the outcome rather than as the invocations, because what the flags
decide is the window and nothing else — the JSON is the same file in all three.

```text
  monospace-cli file.json                          50x13 at (0,0)    rule 8
  monospace-cli --size 6x4 file.json                6x4 at (0,0)    rule 3
  monospace-cli --size 7x5 --origin -3,-2 file.json  7x5 at (-3,-2)  rules 3 and 6
```

The first line is the demonstration's window, which is what makes the second line mean something:
the default is a value someone chose, and passing one is how a caller chooses a different one.

**The description that still carries a canvas.** Also hypothetical, and the only one here whose
output is a claim about behavior rather than about a window: with D5 this is read and the canvas
ignored, so the drawing is the second line above and the `13x9` in the file has no say in it.

```text
  { "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 13, "height": 9 } },
    "next_id": 2,
    "shapes": [ ... ] }

  monospace-cli file.json    →   6x4, the default, not 13x9
```

Rule 2 is the whole of that block, and it is the block a reader should be least comfortable with: a
file that says it is `13x9` and is drawn at `6x4` is a wrong answer given without an error. D5
accepts it because there is no writer, and names the writer as what reopens it.

## What proves it

None of these tests exists yet; the deciding stage adds no code. The names are what the building
stage is held to.

| Rule | Test                                                               |
| ---- | ------------------------------------------------------------------ |
| 1    | `parse_hands_back_the_diagram_and_nothing_else`                    |
| 2    | `a_description_carrying_a_canvas_is_read_and_the_canvas_ignored`   |
| 3    | `the_window_comes_from_the_flags_and_nowhere_else`                 |
| 4    | `a_marker_states_its_size_and_the_command_line_may_default`        |
| 5    | `a_marker_without_an_at_draws_from_the_origin`                     |
| 6    | `a_marker_with_an_at_draws_from_the_origin_it_names`               |
| 7    | `a_marker_whose_size_is_not_a_number_is_reported_by_file_and_line` |
| 8    | `a_path_and_no_size_draws_into_the_demonstrations_window`          |
| 9    | `the_bare_run_draws_into_the_demonstrations_window`                |
| 10   | `the_demonstrations_eight_pictures_are_unchanged`                  |
| 11   | `specs_offsets_resolve_to_the_cells_they_do_today`                 |
| 12   | `cargo xtask render --check`                                       |

Rule 4 is a pair of facts about two different callers, and it is the one a later change is most
likely to break by making the marker's size optional for symmetry with the flag's.

Rule 12 is the gate's own `render` step rather than a test, and it is the only line here that covers
all twenty-three markers at once.

**Measured rather than tested, and reported in the building pull request.** That the twenty-three
pictures are unchanged: every marker rewritten, `cargo xtask render` run, and the number of pictures
that moved reported — which is the honest answer to whether removing `canvas` from the format
changed anything a reader sees, and it should be `0`. That `specs/086`'s two offset tables still
resolve as written, which is the one place where the origin is load-bearing and a mistake in D1's
parsing would show up as a table that no longer matches its picture.

## Open questions

None. What surfaced while writing this became another issue rather than a section here: the dead
paths that #201 left in `CONTRIBUTING.md`, `specs/173` and `docs/diagram-demo.md`, which is prose
about where the format lives rather than about the window.
