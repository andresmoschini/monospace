---
status: draft
date: 2026-10-27
---

# Terminal styles, chosen instead of spelled out

## Why now

An endpoint's terminal is an `arm` or a `glyph`: an arm composes with the cell and follows the
connector's stroke, and a glyph is a literal the caller spells out. A head points, and no glyph set
holds a rule that points, so a caller writes `▲` at one end and `►` at the other, and nothing checks
that the two agree with the directions they leave in.

[#247](https://github.com/andresmoschini/monospace/issues/247) gives a cell the kind that answers
this: it holds a style name and a glyph name, and the catalog resolves it at render. What is missing
is the vocabulary — eleven tables of four glyphs, one per direction, and a terminal that names one.
That is this change. The four single-arm rules `╶ ╴ ╵ ╷` stay untouched, so a head derived from a
glyph set remains a question for another slice, and neither open slice that touches this vocabulary
reaches it: [#63](https://github.com/andresmoschini/monospace/issues/63) loads a glyph set from a
file, and [#206](https://github.com/andresmoschini/monospace/issues/206) re-spells the format.

## Scope

### In

- `monospace-core`: a third `Terminal` variant carrying a style name, and the stamp the connector
  makes from it. The design note in `connector.rs` saying the set is closed at two says three, for
  the reason it gave; `Terminal::Glyph`'s comment saying that no set holds a rule that points is no
  longer true and goes, because a glyph terminal is still a literal the caller spelled.
- `monospace-terminal-styles`: a crate holding the other ten tables and `all()`. It joins the crates
  the `wasm` step compiles.
- `monospace-glyph-sets`: `all()` unions those tables with the arm tables, so a composition root
  still calls one function.
- `monospace-description`: `{ "kind": "style", "style": "<name>" }` on the wire, kept as written.
  Its dependency on `monospace-terminal-styles` goes away, and the doc comment and the test that say
  a `kind` outside the accepted ones is answered by naming **two** say three.
- `docs/model.md` §7 and §11; a new `docs/terminal-styles.md`; one clause in `docs/glyph-sets.md`;
  `docs/diagram-demo.md`'s section on terminals; `README.md`'s layout table.
- Tests for the eleven tables, the direction rule, the wire, and that `arm` and `glyph` still draw
  what they drew.

All of it builds against the spec [#247](https://github.com/andresmoschini/monospace/issues/247)
agreed in [#248](https://github.com/andresmoschini/monospace/pull/248), and none of it restates that
cell.

### Out

- **The named cell itself.** `Cell::Named`, `GlyphName`, `NamedKey`, the two catalog maps, the
  fallback and the `solid-pointer` table are
  [#247](https://github.com/andresmoschini/monospace/issues/247)'s, and this builds against them.
- **A style that composes, or an arm-like style.** A style covers the cell, so composition is the
  glyph's; a style that joined the border would be a set of arm rules, and an arm cannot point.
- **A head that follows a glyph set.** Those rules are claimed today, so adding them changes what
  existing cells render; the model leaves it as a slice of its own.
- **A style loaded from a file.** [#63](https://github.com/andresmoschini/monospace/issues/63) is
  the same question for glyph sets.
- **A style chosen once per connector.** D2 rejects it.

## The decision

**D1 — each entry is named by the way its glyph points, and the connector inverts once.**

**Answer:** the four entries are named by the direction their glyph points, so `solid-pointer` is
`up: ▲, right: ►, down: ▼, left: ◄`, and the connector asks for the entry named for the direction
opposite `endpoint.leaving`, as `GlyphName::from(opposite(leaving))`. **Why not** naming each entry
by the direction the connector leaves in: `▼` would sit under `up`, which reads backwards in the
table, in the documentation and against Unicode, where `BLACK DOWN-POINTING TRIANGLE` is `▼`. **Why
not** an inversion helper on `Style` too: two places that invert is how they drift, and the
connector is the only place that knows an endpoint. **What it costs:** `opposite` — already private
in `connector.rs` — is the whole distance between a direction and a glyph name. **Answered by** the
maintainer, who reversed the draft on reading it.

**D2 — the style is chosen per endpoint.**

**Answer:** each endpoint's terminal is one of `arm`, `glyph` and `style`, so one connector may
carry two styles, or a style at one end and an arm or chosen glyph at the other. **Why not** a
`style` key on the connector with the endpoints as overrides: two ends wanting different heads is
ordinary (`◆──────►`), so the override would be the common case and a precedence rule would exist
only to save one word. **What it costs:** a symmetric arrow names the style twice. **Answered by**
the maintainer.

**D3 — a style writes the named cell, and the catalog answers it at render.**

**Answer:** the terminal stamps `Cell::Named(NamedKey { style, name })` at its position, covers the
cell like a literal, and follows no stroke. **Why not** resolving the name to a glyph while reading
the file: the appearance would come from the reader's dependency rather than from the catalog the
binary was handed, and the name would not survive to anything above the reader. **Why not** a style
as a set of arm rules: an arm cannot point, which is why this exists, and a style that composed
would turn a shared cell into a corner and lose the head. **What it costs:** a style over a box's
border interrupts it, as a glyph does today, which is the caller's choice where the endpoint stands.
**Answered by** the maintainer.

**D4 — the terminal holds the name, and the reader stops knowing the styles.**

**Answer:** `Terminal::Style { style: String }` carries what the file wrote, and
`monospace-description` neither resolves nor refuses nor falls back: it keeps the name. **Why not**
carrying the resolved table: two terminals naming one style would compare unequal the moment a
build's tables differed, so nothing could match or rewrite a style by name. **Why not** refusing an
unknown name in the reader: the list is open the way glyph sets are, and a file written against a
later catalog would stop being readable by an earlier build. **What it costs:** a misspelling is
invisible, and the cell's fallback is where that is paid. **Answered by** practice, as this was
written.

**D5 — the ten live in their own crate, and one union puts them in every binary.**

**Answer:** `monospace-terminal-styles` holds the other ten as catalogs and `all()`, which includes
the core's `solid-pointer`; `monospace_glyph_sets::all()` unions the arm tables and that, so a
composition root still calls one function and no list is kept in two binaries. **Why not** the ten
in the core: every appearance the project ships would be a core table, which is what the glyph-set
split exists to avoid. **Why not** a `by_name` lookup beside `all()`: nothing resolves a name any
more, so it would be a function with no caller. **What it costs:** a binary that builds a catalog
without `all()` draws a space where a style would be. **Answered by** practice, as this was written.

## Model slice

- `docs/model.md` §7 _The initial set_ and _What a terminal writes_: a terminal is one of three, and
  the style paragraph says it writes a named cell. The paragraph calling the asymmetry a limit of
  the data gains the clause that a style answers it and heads following a set stay open.
- §11 _Open questions_: the question about a terminal's glyph narrows to a set of heads.
- `docs/terminal-styles.md` (new): the eleven tables, each entry named for the way it points, with
  the connector's inversion stated beside them, in the shape of `docs/glyph-sets.md`.
- `docs/glyph-sets.md`: one clause saying its tables are arm rules and the named ones are in
  `terminal-styles.md`, so the two do not read as one vocabulary with a hole.
- `docs/diagram-model.md` §6 _Attachment_: one clause — a style's glyph follows `leaving`, which the
  attachment does not decide. The section's rule stands.
- `docs/diagram-demo.md`: _Two terminals, and the one difference between them_ becomes three, with a
  style row in the picture already there, because that document owes one instance of each decision
  the format makes.

## Public surface

```rust
// monospace-core
pub enum Terminal {
    // `Glyph` and `Arm`, unchanged
    Style { style: String },
}

// monospace-terminal-styles: `all()` — the ten, and the core's `solid-pointer` — plus one catalog
// per style, named after it: solid_triangle, small_solid_triangle, outline_triangle,
// small_outline_triangle, unicode_arrow, ascii_arrow, white_diamond, black_diamond, white_circle,
// black_circle
```

`Style`, `NamedKey`, `GlyphName`, `Cell::Named` and the fallback are
[#247](https://github.com/andresmoschini/monospace/issues/247)'s. `Style::catalog(name)` is what
turns one of these tables into the four rules the catalog holds, so a style table is data and the
crate is its only home.

## Behavior

1. A style terminal stamps a named cell carrying its endpoint's style name and
   `GlyphName::from(opposite(leaving))`, so the glyph points away from the route.
2. A style writes no arm and follows no stroke. What the cell it lands on does to whatever was there
   is [#247](https://github.com/andresmoschini/monospace/issues/247)'s rule, and this change adds
   none.
3. Each endpoint chooses its own style, independently of the other endpoint and of the stroke.
4. `solid-pointer` is `up: ▲, right: ►, down: ▼, left: ◄`, and the other ten are the table below.
5. The wire keeps the name as written: `{ "kind": "style", "style": "<name>" }` is not resolved
   while reading, and a name no catalog holds is not a parse error.
6. `{ "kind": "style" }` with no `style` field is refused, naming the missing field, and a `kind`
   that is none of the three is refused by a message that names all three.
7. `monospace_glyph_sets::all()` answers every one of the eleven; a catalog built without
   `monospace_terminal_styles::all()` draws a space where the ten would be.
8. `arm` and `glyph` draw what they drew before, and every picture that used them is unchanged.
9. `monospace-terminal-styles` compiles for `wasm32-unknown-unknown` and joins the `wasm` step.

## Examples

**The eleven**, each column the way its glyph points: the endpoint leaves the other way, and
`GlyphName::from(opposite(leaving))` is what bridges the two.

| name                     | `up` | `right` | `down` | `left` |
| ------------------------ | ---- | ------- | ------ | ------ |
| `solid-triangle`         | ▲    | ▶       | ▼      | ◀      |
| `small-solid-triangle`   | ▴    | ▸       | ▾      | ◂      |
| `outline-triangle`       | △    | ▷       | ▽      | ◁      |
| `small-outline-triangle` | ▵    | ▹       | ▿      | ◃      |
| `solid-pointer`          | ▲    | ►       | ▼      | ◄      |
| `unicode-arrow`          | ↑    | →       | ↓      | ←      |
| `ascii-arrow`            | ^    | >       | v      | <      |
| `white-diamond`          | ◇    | ◇       | ◇      | ◇      |
| `black-diamond`          | ◆    | ◆       | ◆      | ◆      |
| `white-circle`           | ○    | ○       | ○      | ○      |
| `black-circle`           | ●    | ●       | ●      | ●      |

`solid-triangle` and `solid-pointer` share `▲` and `▼`, because Unicode names no up- or
down-pointing pointer; they differ on left and right. The four shapes that do not point carry one
glyph in all four columns, which is no special case: nothing says a style's four entries differ.

**The case that fixes the key.** An endpoint at `(0, 0)` leaving `down` and one at `(1, 1)` leaving
`left`, both with `solid-pointer`. The `▲` is the `up` entry, written by the endpoint leaving
`down`, and the `►` is the `right` entry, written by the one leaving `left`:

<!-- render: 2x2
{
  "next_id": 2,
  "shapes": [
    { "kind": "connector", "id": 1,
      "from": { "at": { "kind": "point", "x": 0, "y": 0 }, "leaving": "down",
                "terminal": { "kind": "glyph", "glyph": "▲" } },
      "to":   { "at": { "kind": "point", "x": 1, "y": 1 }, "leaving": "left",
                "terminal": { "kind": "glyph", "glyph": "►" } },
      "stroke": "light" }
  ] }
-->

```text
▲
└►
```

<!-- /render -->

The marker spells the style with the glyph it resolves to, because `cargo xtask render` runs the
code that exists and no style terminal does yet. The building stage replaces both markers with
`{ "kind": "style", "style": "solid-pointer" }` and deletes this paragraph: the pictures must not
move, which is the picture half of rule 8.

**Two styles in one connector,** which is D2's case: a diamond at the end leaving rightward and a
pointer at the end leaving leftward, `◆──────►`.

<!-- render: 8x1
{
  "next_id": 2,
  "shapes": [
    { "kind": "connector", "id": 1,
      "from": { "at": { "kind": "point", "x": 0, "y": 0 }, "leaving": "right",
                "terminal": { "kind": "glyph", "glyph": "◆" } },
      "to":   { "at": { "kind": "point", "x": 7, "y": 0 }, "leaving": "left",
                "terminal": { "kind": "glyph", "glyph": "►" } },
      "stroke": "light" }
  ] }
-->

```text
◆──────►
```

<!-- /render -->

**A style over a border.** A style covers the cell, so a connector arriving at a box's corner with
`black-diamond` writes `◆` there and the corner is gone. That is rule 2 and the caller's choice of
where the endpoint stands; it is not a composition rule waiting to be written.

None of this is observed: no style terminal exists in this stage. _What proves it_ holds the
direction rule, the wire, the refusal and the tables.

## What proves it

Nothing exists yet: the deciding stage adds no code. The names and tables bind the building stage.

| Rule | Test                                                                                                                               |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------- |
| 1    | `a_style_terminal_stamps_the_name_of_the_direction_opposite_leaving`                                                               |
| 2    | `a_style_writes_no_arm_and_follows_no_stroke`                                                                                      |
| 3    | `the_two_endpoints_of_a_connector_choose_their_styles_independently`                                                               |
| 4    | `the_eleven_styles_are_the_tables_this_spec_lists`                                                                                 |
| 5    | `a_style_name_reaches_the_cell_as_written`                                                                                         |
| 6    | `a_style_terminal_without_a_name_is_refused_by_name`, and `an_unrecognized_terminal_kind_names_it_and_the_three_that_are_accepted` |
| 7    | `all_answers_every_style_and_a_catalog_without_it_draws_a_space`                                                                   |
| 8    | the characterization, and every picture in a tracked document, which the gate re-renders                                           |
| 9    | the `wasm` step, which names the crate                                                                                             |

The two pictures are held by their markers: `cargo xtask render` rewrites them and the gate fails if
one is not what its description draws.

## Open questions

- **Whether a head should follow a glyph set after all.** The four single-arm rules would give a
  head derived from a set, and those keys are claimed today. §11 keeps it open, and this change does
  not answer it.
