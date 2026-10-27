---
status: implemented
decided: "#245"
implemented: "#246"
date: 2026-10-27
---

# Terminal styles, chosen instead of spelled out

## Why now

An endpoint's terminal is an `arm` or a `glyph`: an arm composes with the cell and follows the
connector's stroke, and a glyph is a literal the caller spells out. A head points, and no glyph set
holds a rule that points, so a caller writes `▲` at one end and `►` at the other, and nothing checks
that the two agree with the directions they leave in.

_What a terminal writes_ calls that asymmetry a limit of the data and leaves it as the open question
about where a terminal's glyph could come from, preferring heads that follow the glyph set. This is
the answer that changes no existing cell: a table of four glyphs per style leaves the four
single-arm keys `╶ ╴ ╵ ╷` untouched, so a set of heads stays a question for another slice. Neither
open slice that touches this vocabulary reaches it:
[#63](https://github.com/andresmoschini/monospace/issues/63) loads a glyph set from a file, and a
loaded set is still a table of arms, while
[#206](https://github.com/andresmoschini/monospace/issues/206) re-spells the format, and a third
terminal is better settled here than invented there.

## Scope

### In

- `monospace-core`: a `Style` type, its `StyleCatalog`, one default style, and a third `Terminal`
  variant, which carries a style's name. The design note in `connector.rs` saying the set is closed
  at two says three, for the reason it gave.
- `monospace-terminal-styles`: a crate holding the other ten styles, one function per table
  returning a catalog, and `all()`, their union. It joins the crates the `wasm` step compiles.
- `monospace-description`: `{ "kind": "style", "style": "<name>" }` on the wire, carried as a name
  and resolved only when the diagram is drawn.
- `docs/model.md`, a new `docs/terminal-styles.md`, `docs/diagram-demo.md`'s section on terminals,
  and `README.md`'s layout table.
- Tests for the eleven tables, the orientation rule, the fallback, the refusal, and that `arm` and
  `glyph` still draw what they drew.

### Out

- **A style that composes, or an arm-like style.** A style covers the cell, so composition is the
  glyph's; a style that joined the border would be a set of arm rules, and an arm cannot point.
- **The four single-arm rules `╶ ╴ ╵ ╷`,** which would make an end derived from a glyph set. Those
  keys are claimed today, so adding them changes what existing cells render; the model leaves it as
  a slice of its own.
- **A style loaded from a file.** [#63](https://github.com/andresmoschini/monospace/issues/63) is
  the same question for glyph sets, and a caller's own table belongs to that slice.
- **A style chosen once per connector.** D2 rejects it.

## The decision

**D1 — each entry is named by the way its glyph points, and `glyph` inverts once.**

**Answer:** the four entries are named by the direction their glyph points, so `solid-pointer` is
`up: ▲, right: ►, down: ▼, left: ◄`, and `Style::glyph(leaving)` answers the entry named for the
direction opposite `leaving`. **Why not** naming each entry by the direction the connector leaves
in: `▼` would sit under `up`, which reads backwards in the struct, in the documented table and
against Unicode, where `BLACK DOWN-POINTING TRIANGLE` is `▼`. **What it costs:** the inversion has
moved into one function instead of one table, and a field name now says where its glyph points.
**Answered by** the maintainer, who reversed the draft on reading it.

**D2 — the style is chosen per endpoint.**

**Answer:** each endpoint's terminal is one of `arm`, `glyph` and `style`, so one connector may
carry two styles, or a style at one end and an arm or chosen glyph at the other. **Why not** a
`style` key on the connector with the endpoints as overrides: two ends wanting different heads is
ordinary (`◆──────►`), so the override would be the common case and a precedence rule would exist
only to save one word. **What it costs:** a symmetric arrow names the style twice. **Answered by**
the maintainer.

**D3 — a style writes a literal glyph, and it is not an arm.**

**Answer:** it writes one glyph from its table at `at`, covers the cell, and nothing composes into
it — the behavior `glyph` already has. It does not follow `stroke` and does not degrade through the
glyph catalog. **Why not** a style as a set of arm rules: an arm cannot point, which is why this
exists, and a style that composed would turn a shared cell into a corner and lose the head. **Why
not** deriving the style from the stroke, so `ascii` draws `> < ^ v`: one description would then
draw differently depending on which glyph tables a build shipped. **What it costs:** a style over a
box's border interrupts it, as a glyph does today, which is the caller's choice where the endpoint
stands. **Answered by** the maintainer.

**D4 — the terminal holds the style's name, and the drawing resolves it.**

**Answer:** `Terminal::Style { name: StyleName }` carries the name the description wrote, and the
glyph is looked up when the diagram is drawn, against the `StyleCatalog` the drawing is handed.
**Why not** the resolved table, which the draft chose first: a reader that resolves a name has to
own the tables, so the format depends on the eleven and a binary cannot read a description with a
set of its own — the opposite of how a glyph set works. Holding a name in the core widens no
vocabulary: `Stroke` is a name too, and the names that exist are the tables'. **Why not** resolving
the name to a glyph in the reader and handing back `Terminal::Glyph`: nothing above the reader could
tell a style from a literal, so an editing layer could show neither the name nor change it. **What
it costs:** one more parameter on the drawing, and the same description drawn by two builds with
different tables draws different heads — exactly what a glyph set already decides for every stroke
cell. **Answered by** the maintainer, who reversed the draft on reading the build.

**D5 — the core owns one style and the catalog type; a new crate owns the other ten and the union.**

**Answer:** `monospace-core` holds `Style`, `Style::default()` — the `solid-pointer` table — the
`StyleCatalog` type with `from_styles`, `union`, `solid_pointer` and `style`, and the third variant;
`monospace-terminal-styles` holds the other ten, one function per table returning a catalog, and
`all()`, their union, which includes the core's default. It is the split `monospace-core` and
`monospace-glyph-sets` already make: the core ships one, the crate ships the rest, and the union is
named once. **Why not** all eleven in the core: every appearance the project ships would be a core
table, which is what the glyph-set split exists to avoid. **Why not** the reader importing the data
crate and resolving names itself: which styles a build ships is then decided inside the format, and
a binary that meant to load fewer tables could not, while the one thing a reader needs — a name
answered — does not require it to own the tables. **What it costs:** one more type in the core, and
a drawing that carries a style name must be handed a catalog. **Answered by** practice, in the
session that wrote this, and amended below where the build first put the lookup in the data crate.

### Amended while building

**Amended on 2026-10-27 in #246, and the amendment is here rather than a silent rewrite so that what
was agreed and what is being built can both be read.**

**The decision that changed is D5, and the reason is the shape the glyph sets already have.**
`monospace-description` was built resolving every style through
`monospace-terminal-styles::by_name`, which made the format depend on the eleven tables and gave
`all()` a second caller. Nothing above the reader could then read a description with a different set
of styles, which is exactly what the glyph sets avoid: the drawing is handed a `GlyphCatalog`, and
the crate that reads a description knows no table. `StyleCatalog` is that same shape for names a
description writes, and the first amendment handed one to `parse`; the second carries it to the
drawing instead.

**The rules that changed follow from it.** The style lookups are the catalog's own, so `by_name` is
gone; a table is a catalog rather than a `Style`, so `all()` is a union of catalogs rather than a
`Vec` of pairs; and the CLI and the application name `monospace_terminal_styles::all()` where they
are assembled, as they already name `monospace_glyph_sets::all()`. **Nothing else moves:** the
eleven names, their four glyphs, the fallback of D6 and every picture are what they were.

**Amended again the same day, and this one reverses D4.** Resolving the name in the reader left the
two catalogs in different places: the glyph catalog is consulted where the drawing is drawn, and the
style catalog was consulted where the file is read, so the reader needed a table and the application
did not. **The reason for the reversal is the symmetry that D5 was amended for, taken one step
further:** a catalog belongs to the drawing, so the terminal keeps the name and `Diagram::draw` is
handed the styles, exactly as `render` is handed the glyphs.

**What moved with it.** `Terminal::Style` holds a `StyleName` rather than a `Style`; the fallback of
D6 happens where the name is looked up instead of where the file is read; `parse` takes the text
alone and this crate depends on no catalog at all; and the CLI and the application hand the style
catalog to the drawing, the application keeping it beside the glyph catalog for every frame. **The
cost is stated rather than hidden:** `Shape::draw` and `Diagram::draw` gain a parameter, and a
description drawn by a build with another set of tables draws other heads. **Nothing moves on
screen:** the sweep's 1856 renderings and the 42 tracked pictures are byte for byte what they were,
which is what proves the reversal changed where a table is read and not what it draws.

**D6 — an unknown name draws the default, in silence; a missing name is refused.**

**Answer:** `"style": "nonsense"` draws the `solid-pointer` table when the diagram is drawn, and
reports nothing, and `{ "kind": "style" }` without a name is refused by the message that names the
missing field. **Why not** refusing an unknown name: a catalog is open the way glyph sets are, so a
file written against a later one would stop being readable by an earlier build, where a glyph set
degrades instead. **Why not** reporting the substitution: a diagram is drawn rather than answered,
and a drawing has no channel a report could travel on — adding one is a change to every caller for a
misspelling. **What it costs:** a misspelled style is invisible. **Answered by** practice, in the
session that wrote this.

## Model slice

- `docs/model.md` §1 _Vocabulary_: a `Style` row — four glyphs, one for each way a head may point —
  a `StyleName` row, and a `StyleCatalog` row, as there is a `GlyphCatalog` one.
- §7 _The initial set_: the connector paragraph says a terminal is one of three, and the sentence
  about a glyph terminal's glyph pointing opposite `leaving` extends to a style.
- §7 _What a terminal writes_: a third paragraph after **arm** and **glyph**, saying a style writes
  the glyph its table names for the direction opposite `leaving`, covers the cell, and follows no
  stroke. The paragraph calling the asymmetry a limit of the data gains a clause: a style is the
  answer available now, and heads that follow a glyph set stay the open question below. A closing
  paragraph says a terminal keeps a style's name and the drawing resolves it against the catalog it
  is handed — the shape a glyph is looked up in.
- §7 _The initial set_, the paragraph saying the core holds no default stroke: one clause separates
  the base stroke, which a figure always names, from the fallback style, which answers a name
  nothing claims, when the drawing looks the name up. The core decides no appearance a figure left
  unsaid.
- §11 _Open questions_: the question about a terminal's glyph narrows to a set of heads.
- `docs/terminal-styles.md` (new): the eleven tables, each entry named for the way it points, with
  `glyph`'s inversion stated beside them, in the shape of `docs/glyph-sets.md`, and above them the
  catalog the drawing is handed.
- `docs/diagram-model.md` §6 _Attachment_: one clause — a style's glyph follows `leaving`, which the
  attachment does not decide — and a paragraph saying the terminal holds a name the drawing
  resolves. The section's rule stands.
- `docs/diagram-demo.md`: _Two terminals, and the one difference between them_ becomes three, with a
  style row in the picture already there, because that document owes one instance of each decision
  the format makes.

## Public surface

```rust
// monospace-core
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    /// The glyph that points up: `▲`.
    pub up: Glyph,
    pub right: Glyph,
    pub down: Glyph,
    pub left: Glyph,
}

impl Style {
    /// The entry named for the direction opposite `leaving`.
    #[must_use]
    pub fn glyph(&self, leaving: Direction) -> &Glyph;
}

/// The name a style is written under: a name, and nothing more.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StyleName(pub String);

/// Every style in play, by the name a description writes.
pub struct StyleCatalog {
    // no field is public
}

impl StyleCatalog {
    /// A catalog from an ordered group of tables; the first to claim a name keeps it.
    #[must_use]
    pub fn from_styles<S: Into<String>>(styles: impl IntoIterator<Item = (S, Style)>) -> Self;
    /// A catalog from other catalogs, in the order given; the earlier one wins a name.
    #[must_use]
    pub fn union(catalogs: impl IntoIterator<Item = Self>) -> Self;
    /// The core's own table: `solid-pointer`.
    #[must_use]
    pub fn solid_pointer() -> Self;
    /// The style `name` answers to, or `None`.
    #[must_use]
    pub fn style(&self, name: &str) -> Option<&Style>;
}

pub enum Terminal {
    // `Glyph` and `Arm`, unchanged
    Style { name: StyleName },
}

// monospace-diagram
impl Diagram {
    /// Draws every shape into `sink`, resolving every style name against `styles`.
    pub fn draw(&self, sink: &mut dyn StampSink, styles: &StyleCatalog);
}

// monospace-terminal-styles
#[must_use]
pub fn all() -> StyleCatalog; // the eleven, `solid-pointer` included
pub fn solid_triangle() -> StyleCatalog; // and one per style, named after it:
// small_solid_triangle, outline_triangle, small_outline_triangle, solid_pointer, unicode_arrow,
// ascii_arrow, white_diamond, black_diamond, white_circle, black_circle

// monospace-description
pub fn parse(json: &str) -> Result<Diagram, ParseError>;
```

`Style::default()` is the `solid-pointer` table, and the drawing resolves a terminal's name as
`styles.style(&name.0).cloned().unwrap_or_default()`. The name is a `String` inside `StyleName`
rather than a closed `enum`: a closed enum would turn an unknown name into a deserialization error
instead of D6's fallback, and it would put the vocabulary of names in the core where the tables
belong.

## Behavior

1. A style terminal writes the entry named for the direction opposite its endpoint's `leaving`, so
   the glyph points away from the route.
2. A style covers the cell and composes with nothing: it writes no arm and follows no stroke.
3. `solid-pointer` is `up: ▲, right: ►, down: ▼, left: ◄`, and the other ten are the table in
   _Examples_.
4. Each endpoint chooses its own style, independently of the other endpoint and of the stroke.
5. `{ "kind": "style", "style": "<name>" }` keeps the name, and the drawing resolves it against the
   catalog it is handed; a name that catalog does not hold draws `solid-pointer`.
6. `{ "kind": "style" }` with no `style` field is refused, naming the missing field.
7. `arm` and `glyph` draw what they drew before, and every picture that used them is unchanged.
8. `monospace-terminal-styles` compiles for `wasm32-unknown-unknown` and joins the `wasm` step.

## Examples

**The eleven**, each column the way its glyph points: the endpoint leaves the other way, and
`Style::glyph` is what bridges the two.

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

The marker spells the style with the glyph it resolves to, because the gate renders with the code
that exists; `{ "kind": "style", "style": "solid-pointer" }` at each end draws the same.

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

**The fallback.** `"style": "no-such-style"` draws rule 5's default, so the first picture is what a
name the drawing's catalog does not hold produces, and `{ "kind": "style" }` is refused for the
missing field. Neither is observed: the pictures come from the glyphs each style resolves to, and
_What proves it_ holds the resolution, the fallback and the refusal.

## What proves it

Nothing exists yet: the deciding stage adds no code. The names and tables bind the building stage.

| Rule | Test                                                                                                        |
| ---- | ----------------------------------------------------------------------------------------------------------- |
| 1    | `a_style_writes_the_entry_for_the_direction_opposite_leaving`                                               |
| 2    | `a_style_covers_the_cell_and_composes_with_nothing`                                                         |
| 3    | `the_eleven_styles_are_the_tables_this_spec_lists`                                                          |
| 4    | `the_two_endpoints_of_a_connector_choose_their_styles_independently`                                        |
| 5    | `an_unknown_style_name_draws_the_default`, in the layer that draws, since that is where a name is looked up |
| 6    | `a_style_terminal_without_a_name_is_refused_by_name`                                                        |
| 7    | the characterization, and every picture in a tracked document, which the gate re-renders                    |
| 8    | the `wasm` step, which names the crate                                                                      |

The two pictures are held by their markers: `cargo xtask render` rewrites them and the gate fails if
one is not what its description draws. **The characterization is what proves the reversal of D4
moved nothing on screen**: 1856 renderings and 42 pictures are what they were before the name
replaced the table, so the change is where a catalog is read and not what it draws.

## Open questions

- **Whether a misspelled style should be reported once a drawing can say more than what it wrote.**
  D6 answers silence where there is no second channel, and adding one is a change to the drawing and
  every caller.
