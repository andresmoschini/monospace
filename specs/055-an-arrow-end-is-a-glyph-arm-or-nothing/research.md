# Phase 0 Research: An arrow's end is a glyph or an arm

ADR-0063 already moved the reasoning for what an endpoint writes into the `Design notes` of
`shape::arrow`, so that altitude is settled. What is left is type design in three crates, a rename
on the wire, and one model's vocabulary — cheap to undo, none of it an ADR.

Every picture came out of `cargo run -q -p monospace-cli -- <file>`. Where an arm terminal appears
it was produced by standing a `line` where the terminal would write: a line's first cell is an `End`
carrying one arm (`line.rs:56-63`), so a horizontal line of length 2 sets the right arm at its first
position and nothing else — the same cell decision, through the same glyph set and renderer.

## Q1 — Where the terminal writes, and the alternative that was discarded

**Decision**: at the endpoint. `at` becomes the cell the terminal hangs from rather than a cell the
endpoint owns. Measured on the spec's arrangement — two 3×3 boxes at (0,0) and (8,0), canvas 11×3,
endpoints at (2,1) leaving right and (8,1) leaving left — the proposal is:

```text
┌─┐     ┌─┐
│ ├─────┤ │
└─┘     └─┘
```

and the alternative, the terminal one step further in the leaving direction with `at` left
unwritten:

```text
┌─┐     ┌─┐
│ │─────│ │
└─┘     └─┘
```

The reason to discard the alternative is in its picture, not in an argument: the cell at (2,1) is a
border run that stops and nothing joins it, so the arrow hangs one cell short of the box whose
border it names. `at` is the only position the caller wrote down, so a terminal that does not write
there leaves a gap nobody drew — and that picture is reachable deliberately by hanging the endpoint
one cell short, which the spec's first B2 edge case already tells a caller to do.

## Q2 — Whether `derive_path` moves, and what the model adds about it

**Decision**: it does not move, and the model gains one clause rather than a rule.

`derive_path(a, da, b, db) -> Option<Vec<Pos>>` takes no glyph and never read one, and `Arrow::draw`
passes the two `at`s and the two `leaving`s — a terminal cannot reach the route by construction, so
SC-001 and SC-002 need no code change, and the five body cells are identical in the two Q1 pictures.
The model needs the one thing the code already does and the prose does not: the path writes no
endpoint cell, whatever a terminal puts there. A clause, not a rule, and not a term in `Cost`.

## Q3 — The wire shape of the renamed field, and where the grapheme rule goes

**Decision**: one internally tagged enum, field `terminal`, tagged `kind`,
`rename_all = "lowercase"`, with `Glyph { glyph: Glyph }` and `Arm`. decisions.md D2 puts the
alternative beside it.

`ShapeDescription` in the same file is already `#[serde(tag = "kind", rename_all = "lowercase")]`
(`description.rs:134`), and the one precedent in the file is followed. Measured against the external
form, two differences are real and one I expected is not:

- **The external form's obvious spelling does not compile.** `Glyph(Glyph)` needs `T: Deserialize`,
  and `Glyph` has none, so the attribute carrying FR-014 has nowhere to sit. The cheapest fix is a
  hand-written `impl Deserialize` for a one-field wrapper. Internally tagged, `deserialize_with`
  goes on a _named_ field and the derive does the rest — one attribute — and the error text is byte
  for byte the same in both: `"ab" is not exactly one grapheme cluster`.
- **Two shapes for one field's values.** Every internal value is an object. Externally a unit
  variant is a bare string (`"arm"`) and a variant with a payload is an object keyed by the variant
  name, so a third terminal may arrive either way and the JSON alone does not say which.
- **Not a difference: SC-005.** Both report
  ``unknown variant `dot`, expected `glyph` or `arm``` and both report a missing field as``missing
  field
  `terminal```, so B1.3's "presence does not decide" holds either way. The external form's "no tag name to collide" is true and small: the collision moves to the variant name rather than disappearing, and it buys consistency with the shape level, where`kind`
  already means the same thing.

The test pinning FR-014, `a_multi_grapheme_head_fails_to_deserialize` (`description.rs:249`), is
renamed with the field either way. `monospace-core` gains no dependency: `Terminal` is a plain enum
over `Glyph` and `Side`.

## Q4 — A glyph terminal and an arm terminal on one cell

**Decision**: no rule is stated, and decisions.md D3 asks the maintainer to confirm that, because
the measurement says the answer is otherwise arriving by accident.

Measured in both orders, a filled 5×5 box's interior literal standing in for the glyph and a
horizontal line of length 2 for the arm. The **front-most** shape decides: `Diagram::draw` iterates
`.iter().rev()` with `StampMode::Below` (`diagram.rs:92-93`), so the last entry in a description's
`shapes` array is the front.

```text
Glyph in front:        Arm in front:
┌───┐                 ┌───┐
│xxx│                 │xxx│
│xxx│                 │──x│
│xxx│                 │xxx│
└───┘                 └───┘
```

A literal in front erases the arm whole — the arm's two cells read `x` like their neighbors and
nothing marks where it was, because a literal is decided on every side and `stamp` returns early on
a decided target (`buffer.rs:79`). An arm in front erases the glyph at the two cells it wrote and
closes the three sides it left `Unset`, because `merge` over a literal keeps the stroke cell and
closes what it declined (`buffer.rs:133`); both then render `─`, the same character a segment does,
per ADR-0029's measurement. So the cell's meaning changes with the order — one arm and three
undecided sides becomes a through-line refusing every junction. That is "should they join" answered
by accident, which is why it is a question and not a finding.

## Q5 — The mirrored `Endpoint` in `monospace-diagram`

**Decision**: rename the field, keep the mirror, do not merge the two types.

It exists on purpose and its rustdoc says why — "so that a later change to how an endpoint is
anchored stays inside this crate (research.md Q3)", 079's Q3 (`shape.rs:11-13`). This slice is not
that change: it changes what a terminal writes, not how an endpoint is anchored, and merging the two
types is the one edit that would put anchoring inside the core.

## Q6 — What ADR-0029 and ADR-0063 need

**Decision**: nothing. No record is added, revised or reopened.

ADR-0029 already carries `status: accepted; superseded in part by ADR-0063` and
`commitment: load-bearing`, and ADR-0063 names the condition under which it reopens: "It reopens
only if the model's account of what an endpoint may write is replaced rather than extended. Naming
one more thing an endpoint can write is not that." This slice extends the account, so the condition
is not met, and the rule the record points at goes into the `Design notes` of `shape::arrow`. The
spec puts "what becomes of ADR-0029" on the sheet; research answers it — ADR-0063, dated 2026-09-27,
took that decision already, and an entry with no alternative left is not an entry.

## Q7 — The rename's footprint, and what the `render` step will not see

**Decision**: all 20 tracked occurrences of the JSON key move in one commit, and no
`<!-- render -->` block is written with the new spelling until the code lands.

Five of the 20 sit inside render markers — `README.md:56-57`, `CONTRIBUTING.md:235-236`,
`docs/model.md:343-344`, `spec.md:78-79,110-111` — and the `CONTRIBUTING.md` pair is inside the
four-backtick fence, so it illustrates the marker rather than instantiating it. The other 15 are the
CLI's `assets/demo.json` and two test JSON strings, the 045 and 079 contracts, and four Rust sites
naming the field. The `render` step walks `git ls-files '*.md'` (`xtask/src/render.rs:101`), so a
marker edited but not `git add`ed is invisible to it and the step reports nothing — `git add` first,
or the pictures go unchecked. And the current binary cannot read a `terminal` key, so the three
documents carrying a marker keep the old spelling through the deciding stage and rename it in the
same increment as the code.

045's contract is **not** renamed and that is the one inventory item with a question attached: it
already documents a `mode` field the binary stopped reading, which is the precedent — a shipped
feature's contract is the record of what that feature shipped, not a second copy of the current one.
079's contract is the one that carries the field.

## Q8 — The third terminal

**Decision**: it stays out of scope, and the tag is what keeps it out.

The enum is closed at two variants and the wire form is internally tagged, so a third terminal is a
new variant, a new tag, and one line in the model's vocabulary — no change to the shape of a
description and no change to any record (Q6). Q4 shows the cell such a terminal would land in is
already order-sensitive, and that is the part the later slice has to answer.

## Q9 — Where the model's vocabulary sentence sits

**Decision**: in _The initial set_, one sentence, and nowhere else.

That section defines the endpoint's parts, so it is where a statement about them belongs. _An end is
an arm; a head is a glyph_ argues the two members and its title names them, so a third terminal
makes that heading wrong — which is the same reason the model's own rule holds: the vocabulary is a
list the model owns, and the reasoning for each member is each figure's own (Q6). The sentence
states the rule and its consequence — naming one more is a change here and in the figure, not a
change to any record — and the consequence is what saves the next reader from opening an ADR.

## Dependencies

None added. No crate from crates.io enters, so there is no publication date to verify.
