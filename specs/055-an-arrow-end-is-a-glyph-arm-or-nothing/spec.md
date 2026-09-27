<!-- The feature branch is named after the issue title, truncated by `cargo xtask spec`. -->

# Feature Specification: An arrow's end is a glyph or an arm

**Feature Branch**: `055-an-arrow-end-is-a-glyph-arm-or-nothing-deciding` | **Issue**:
[#55](https://github.com/andreschini/monospace/issues/55) | **Created**: 2026-09-26 | **Status**:
Draft

**Input**: Issue #55, "An arrow end is a glyph or an arm" — the wish as it was filed, read with the
amendment the session that settled it wrote into the issue itself.

## What this slice implements

- [_The initial set_](../../docs/model.md#the-initial-set) — an endpoint becomes a position, a
  leaving direction and a terminal, and `at` becomes the cell the terminal hangs from rather than a
  cell the endpoint owns. The section also states that what an endpoint may write is vocabulary this
  document owns, so that naming one more of them is a change here and in the figure and not a change
  to any record: the record covering the move states where the reasoning lives, not what the
  vocabulary holds.
- [_What a terminal writes_](../../docs/model.md#what-a-terminal-writes) (titled _An end is an arm;
  a head is a glyph_ until this slice) — the argument stands entire, because no glyph set holds a
  rule that points and a head still needs a glyph the caller chooses. What is added is that the two
  answers stop being forced by the data.
- [_The route of an arrow_](../../docs/model.md#the-route-of-an-arrow) — one clause: the path writes
  no endpoint cell, whatever the endpoint puts there.
- [§6 _Attachment_](../../docs/diagram-model.md#6-attachment) — the endpoint the diagram layer
  mirrors gains the same terminal, and the sentence there calling that definition unchanged goes.

The model changes first, in the same increment, as the constitution asks: a slice that needs a rule
the model does not have has the model grow it before the code does. One question the issue leaves
open — what becomes of [ADR-0029](../../docs/decisions/0029-draw-a-line-end-as-one-arm.md), whose
second half this slice reverses — is a decision this slice has to take, and it belongs on the sheet
`/speckit-plan` part one writes. It is not answered here.

Two assumptions carry the rest. Nothing outside this repository reads the description format — the
contract documenting it calls itself provisional — so naming the terminal differently needs no
migration and no second spelling of it. And every one of the 1856 pinned renderings is a glyph
terminal, so all of them are expected to stand still; one that moves is evidence that something was
changed beyond the terminal.

## Clarifications

### Session 2026-09-27

- Q: Two figures on one cell — which keeps it, the one in front or the one behind? → A: The one in
  front; `B2` read it the other way and `SC-004` named the wrong border cell with it.

## Behavior

Every picture below is the same arrangement: a box on each side, and an arrow between them with each
endpoint standing on the nearer box's border cell. Each group states its scenarios first and shows
the pictures underneath, because a generated picture cannot sit inside a numbered item without one
of the two formatters moving it.

### B1 — One field, a tag on it, and two things it can name

1. **Given** an endpoint whose terminal is a chosen glyph, **When** the arrow is drawn, **Then** the
   cell at the endpoint holds that glyph, and nothing composes into it.
2. **Given** an endpoint whose terminal is an arm, **When** the arrow is drawn, **Then** the cell at
   the endpoint carries the one arm the arrow arrives on, in the arrow's own stroke, leaves its
   other three sides undecided, and renders through the glyph set like every other stroke cell.
3. **Given** the field that names the terminal, **When** a description is read, **Then** it is one
   field carrying a tag, not a field whose presence or absence decides. A glyph and an arm are two
   values of the same field, and neither of them is what leaving the field out would mean — which is
   what lets a terminal the model has not named yet be added without changing the shape of a
   description.

A glyph terminal at both ends, which is what the arrow draws today and what every pinned rendering
in the repository is:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 11, "height": 3 } },
  "shapes": [
    { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 3, "height": 3 },
      "stroke": "light" },
    { "kind": "box", "at": { "x": 8, "y": 0 }, "size": { "width": 3, "height": 3 },
      "stroke": "light" },
    { "kind": "arrow",
      "from": { "at": { "x": 2, "y": 1 }, "leaving": "right",
        "terminal": { "kind": "glyph", "glyph": "◄" } },
      "to": { "at": { "x": 8, "y": 1 }, "leaving": "left",
        "terminal": { "kind": "glyph", "glyph": "►" } },
      "stroke": "light" } ] }
-->

```text
┌─┐     ┌─┐
│ ◄─────► │
└─┘     └─┘
```

<!-- /render -->

### B2 — An arm terminal composes; a glyph terminal covers

1. **Given** the same arrangement drawn with the arrow last, **When** it is drawn again between the
   two boxes, **Then** the two pictures differ, and the figure in front keeps the shared cell. A
   glyph covers the position it is given and nothing joins it, so which of the two survives is the
   order the caller wrote them in. That is _A cell can be a literal instead_ applied to a figure,
   and it is why a caller who wants a border to survive places the endpoint one cell short of it.
2. **Given** the same arrangement with an arm terminal at both ends, **When** it is drawn in those
   two orders, **Then** both pictures are the second one below, and the border cell is a junction in
   both.

The first scenario, drawn between the two boxes:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 11, "height": 3 } },
  "shapes": [
    { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 3, "height": 3 },
      "stroke": "light" },
    { "kind": "arrow",
      "from": { "at": { "x": 2, "y": 1 }, "leaving": "right",
        "terminal": { "kind": "glyph", "glyph": "◄" } },
      "to": { "at": { "x": 8, "y": 1 }, "leaving": "left",
        "terminal": { "kind": "glyph", "glyph": "►" } },
      "stroke": "light" },
    { "kind": "box", "at": { "x": 8, "y": 0 }, "size": { "width": 3, "height": 3 },
      "stroke": "light" } ] }
-->

```text
┌─┐     ┌─┐
│ ◄─────│ │
└─┘     └─┘
```

<!-- /render -->

The second scenario, which both stamp orders draw alike:

```text
┌─┐     ┌─┐
│ ├─────┤ │
└─┘     └─┘
```

Hypothetical — hand-drawn, not generated. The second scenario of B2.

### B3 — Nothing about the body moves

1. **Given** one arrow and each of the two terminals in turn, **When** it is drawn, **Then** the
   body is the same five cells between the same two positions in both.
2. **Given** any terminal, **When** the arrow is drawn, **Then** the path writes no endpoint cell at
   all, so the terminal is the only thing an endpoint contributes to the cell it names.
3. **Given** the 1856 pinned renderings of the arrow sweep, every one of them a glyph terminal,
   **When** each is redrawn, **Then** none of them changes.

## Edge cases

- **A glyph terminal standing on a box's border cell.** The border is interrupted, and which of the
  two survives depends on which is in front — the measurement the first scenario of B2 pins. A
  degenerate arrangement draws what the general rule gives it and gains no exception.
- **Two boxes side by side with an arrow hung on the border they share.** The second box's border is
  destroyed and nothing reports it, because a surface does not read what is already there.
- **An arm terminal where another figure has already written a literal.** The order decides, as it
  does for any two figures sharing a cell.
- **Two endpoints at one position, so both terminals land on one cell.** Which of them the cell
  holds follows the same order that decides today for two heads. Whether a glyph and an arm standing
  on one cell need a stated rule of their own is a question for the decision sheet, not for here.
- **A glyph terminal naming something that is not one glyph.** The description is refused by name,
  as it is today.

## What this slice does not decide

- **What an endpoint writes when its terminal is neither a glyph nor an arm** — a terminal that
  neither points nor joins is outside this slice, and the cell such a terminal leaves behind is not
  settled here. The tag is what lets the question be answered later without a second spelling of the
  data. What would settle it: a slice that says what a terminal does when something else reaches the
  same cell and the two should not join, which a `Line`'s end wants as much as an arrow's does.
- **Whether a terminal's glyph may come from a glyph set rather than from the caller** — the tag
  leaves the shape of the data open to it, and the model already carries the question. What would
  settle it: the slice that gives a set its own heads
  ([#57](https://github.com/andresmoschini/monospace/issues/57)), which also has to say whether the
  caller's glyph becomes an override or goes away.
- **What a diagram derives for the direction an arrow leaves an attachment in** — this slice changes
  what an endpoint carries and derives nothing from where it hangs. What would settle it: the first
  slice where the caller's direction and the anchor's side are routinely the same.
- **An endpoint hanging from a shape's side anchor**
  ([#82](https://github.com/andresmoschini/monospace/issues/82)) — this slice is what makes such a
  cell survivable, and the attachment is that slice's work. What would settle it: that slice.
- **Whether the shipped demonstration shows the new capability.** A matter of taste, and not left
  for later: `decisions.md` D5 answers that the demonstration's one arrow keeps its glyph `to`
  terminal and takes an arm terminal at `from`, which changes one cell of the first picture. Every
  other cell of both pictures is what it renders today.

## Testing expectations

- **Contract** — one named test per terminal, for what it writes: a glyph literal that nothing
  composes into; one arm and three undecided sides, in the arrow's own stroke.
- **Contract** — the distinction B2 draws, asserted in both orders rather than described: one test
  where the two pictures of a glyph terminal differ, one where the two pictures of an arm terminal
  are the same.
- **Contract** — that the two terminals draw the same body, and that the path writes no endpoint
  cell whatever the terminal is.
- **Characterization** — the sweep of every arrow arrangement, 1856 renderings across eight snapshot
  files, all of them glyph terminals today. What makes it too wide to assert by hand is the count:
  it is a range, and its file says at its head that it records what the code does. It MUST not move,
  and the report of how many cases moved, in which families, with three before-and-after examples is
  what a move requires.
- Unit tests in the core are the minimum; a rule above with no test named against it is unfinished.

## Success criteria

- **SC-001**: An arrow with a glyph terminal at both ends draws the picture it draws today, and none
  of the 1856 pinned renderings of the sweep changes.
- **SC-002**: The same arrow with each of the two terminals draws the same body — the same five
  cells between the same two positions, in the arrangement above.
- **SC-003**: The arrangement above with arm terminals at both ends draws one identical picture in
  both stamp orders, with the border cell a junction in each.
- **SC-004**: The same arrangement with glyph terminals at both ends draws two different pictures,
  and the one drawn between the boxes has lost the left box's right-hand border cell.
- **SC-005**: One description file names both terminals, and a file naming anything else is refused
  by name rather than silently read as one of the two.
- **SC-006**: Every picture pinned by feature 039 and the model document's own rendered examples is
  unchanged, and the shipped demonstration changes only at the one terminal `decisions.md` D5 names.
