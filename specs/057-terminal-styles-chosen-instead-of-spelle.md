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
the answer that leaves every existing cell rendering as it did: a table of four glyphs per style
leaves the four single-arm keys `╶ ╴ ╵ ╷` untouched, so a set of heads stays a question for another
slice. Neither open slice that touches this vocabulary reaches it:
[#63](https://github.com/andresmoschini/monospace/issues/63) loads a glyph set from a file, and a
loaded set is still a table of arms, while
[#206](https://github.com/andresmoschini/monospace/issues/206) re-spells the format, and a third
terminal is better settled here than invented there.

## Scope

### In

- `monospace-core`: a `Style` type, its `StyleCatalog`, one default style, a third `Terminal`
  variant carrying a style's name, a third kind of `Cell` carrying a name and a direction, and a
  second catalog on `render`. The design note in `connector.rs` saying the set is closed at two says
  three, for the reason it gave.
- `monospace-terminal-styles`: a crate holding the other ten styles, one function per table
  returning a catalog, and `all()`, their union. It joins the crates the `wasm` step compiles.
- `monospace-description`: `{ "kind": "style", "style": "<name>" }` on the wire, carried as a name
  and resolved only when the cells are rendered.
- `docs/model.md`, a new `docs/terminal-styles.md`, `docs/diagram-demo.md`'s section on terminals,
  and `README.md`'s layout table.
- Tests for the eleven tables, the orientation rule, the fallback, the refusal, and that `arm` and
  `glyph` still draw what they drew.

### Out

- **A style that composes, or an arm-like style.** A head covers the cell, so composition is the
  head's; a style that joined the border would be a set of arm rules, and an arm cannot point.
- **The four single-arm rules `╶ ╴ ╵ ╷`,** which would make an end derived from a glyph set. Those
  keys are claimed today, so adding them changes what existing cells render; the model leaves it as
  a slice of its own.
- **A style loaded from a file.** [#63](https://github.com/andresmoschini/monospace/issues/63) is
  the same question for glyph sets, and a caller's own table belongs to that slice.
- **A style chosen once per connector.** D2 rejects it.

## The decision

**D1 — each entry is named by the way its glyph points, and a head names that direction.**

**Answer:** the four entries are named by the direction their glyph points, so `solid-pointer` is
`up: ▲, right: ►, down: ▼, left: ◄`, `Style::glyph(toward)` answers the entry for a head pointing
`toward`, and the connector derives that direction as the opposite of `leaving`. **Why not** naming
each entry by the direction the connector leaves in: `▼` would sit under `up`, which reads backwards
in the struct, in the documented table and against Unicode, where `BLACK DOWN-POINTING TRIANGLE` is
`▼`. **What it costs:** one reversal, in the figure that knows both directions, instead of a field
name that reads backwards. **Answered by** the maintainer, who reversed the draft on reading it, and
amended again below when the cell took the direction over.

**D2 — the style is chosen per endpoint.**

**Answer:** each endpoint's terminal is one of `arm`, `glyph` and `style`, so one connector may
carry two styles, or a style at one end and an arm or chosen glyph at the other. **Why not** a
`style` key on the connector with the endpoints as overrides: two ends wanting different heads is
ordinary (`◆──────►`), so the override would be the common case and a precedence rule would exist
only to save one word. **What it costs:** a symmetric arrow names the style twice. **Answered by**
the maintainer.

**D3 — a style writes a head, and it is neither an arm nor a literal.**

**Answer:** it writes a **head** at `at`: a third kind of cell, which says which way it points and
which style it is written in and holds no glyph. It covers the cell and nothing composes into it,
and it does not follow `stroke` or degrade through the glyph catalog. **Why not** a style as a set
of arm rules: an arm cannot point, which is why this exists, and a style that composed would turn a
shared cell into a corner and lose the head. **Why not** a literal glyph, which the draft chose: a
literal is chosen where a head is derived, and a chosen one would make the endpoint the place where
the appearance is decided, so two builds could not draw one description differently. **Why not**
deriving the style from the stroke, so `ascii` draws `> < ^ v`: one description would then draw
differently depending on which glyph tables a build shipped. **What it costs:** a third kind of
cell, and a style over a box's border interrupts it, as a glyph does today — which is the caller's
choice where the endpoint stands. **Answered by** the maintainer.

**D4 — the terminal holds the style's name, the cell it writes carries it, and the rendering
resolves it.**

**Answer:** `Terminal::Style { name: StyleName }` carries the name the description wrote; the
connector writes a head, which is a cell holding that name and the direction it points; and `render`
— the function that already turns cells into text — looks the name up in the `StyleCatalog` it is
handed, exactly as it looks a cell of arms up in the `GlyphCatalog`. **Why not** the resolved table,
which the draft chose first: a reader that resolves a name has to own the tables, so the format
depends on the eleven and a binary cannot read a description with a set of its own — the opposite of
how a glyph set works. **Why not** a catalog on the drawing, which the build tried second: every
`Shape::draw` and every `Diagram::draw` would take a parameter for a glyph no shape decides, so a
caller drawing into a buffer would pass a table that could not change one cell, while a cell of arms
already waits for its glyph with no catalog in sight. **Why not** resolving the name to a glyph in
the reader and handing back `Terminal::Glyph`: nothing above the reader could tell a style from a
literal, so an editing layer could show neither the name nor change it. **What it costs:** a third
kind of cell, and a rendering that draws a head must be handed a style catalog. **Answered by** the
maintainer, who carried the decision down to the cell on reading the build.

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
further:** a catalog belongs above the buffer, so the terminal keeps the name and `Diagram::draw` is
handed the styles, exactly as `render` is handed the glyphs.

**What moved with it.** `Terminal::Style` holds a `StyleName` rather than a `Style`; the fallback of
D6 happens where the name is looked up instead of where the file is read; `parse` takes the text
alone and this crate depends on no catalog at all; and the CLI and the application hand the style
catalog to the drawing, the application keeping it beside the glyph catalog for every frame. **The
cost is stated rather than hidden:** `Shape::draw` and `Diagram::draw` gain a parameter, and a
description drawn by a build with another set of tables draws other heads. **Nothing moves on
screen:** the sweep's 1856 renderings and the 42 tracked pictures are byte for byte what they were,
which is what proves the reversal changed where a table is read and not what it draws.

**Amended once more, and this one reverses the reversal: the lookup belongs in the cell.** Putting
the catalog on the drawing made every caller pass one, and it made `Shape::draw` take a parameter
for a glyph no shape decides — a box, a line and a fragment have nothing a table could name, and a
caller that draws into a buffer passed a table that cannot change one cell. **The reason is the cell
itself, which is where a stroke cell already waits**: a cell of arms holds four stroke names and the
rendering resolves them against the glyph sets, so a head is the same case with one name and one
direction. The connector therefore writes a **head** and draws with no catalog, `Diagram::draw`
takes the sink alone again, and `render` takes the style catalog beside the glyph one — the one
place in the stack where a cell's character is decided.

**What moved with it, and what did not.** `Cell` gains a third kind; `Style::glyph` takes the
direction the head points rather than the one the endpoint leaves in, since the cell names the first
and the connector derives it; D6's fallback happens where the head is rendered; the name still
travels through `Terminal::Style` and the reader still takes the text alone. **The cost is stated
rather than hidden:** a third kind of cell, and every rendering now needs a second catalog.
**Nothing moves on screen:** the sweep's 1856 renderings and the 42 tracked pictures are byte for
byte what they were, and the move is a change of where a name is read and not what it draws.

**D6 — an unknown name draws the default, in silence; a missing name is refused.**

**Answer:** `"style": "nonsense"` renders as the `solid-pointer` table when the cells are rendered,
and reports nothing, and `{ "kind": "style" }` without a name is refused by the message that names
the missing field. **Why not** refusing an unknown name: a catalog is open the way glyph sets are,
so a file written against a later one would stop being readable by an earlier build, where a glyph
set degrades instead. **Why not** reporting the substitution: a rendering answers text, and it has
no channel a report could travel on — adding one is a change to `render` and every caller for a
misspelling. **What it costs:** a misspelled style is invisible. **Answered by** practice, in the
session that wrote this, and answered again where the rendering took the lookup over.

## Model slice

- `docs/model.md` §1 _Vocabulary_: the `Cell` row names three kinds; a `Head` row — a cell that
  points, with the direction it points and the style it is written in; a `Style` row — four glyphs,
  one for each way a head may point — a `StyleName` row, and a `StyleCatalog` row, as there is a
  `GlyphCatalog` one.
- §3 _The cell_: a subsection after _A cell can be a literal instead_ naming the third kind, saying
  it holds a name and a direction rather than a character, and that it composes as a literal does.
- §4 _Stamping_: one paragraph saying every row naming a literal is a row naming a head as well, and
  why. The table itself stands.
- §6 _Rendering_: the intro says two catalogs, and a paragraph after the literal's says a head is
  rendered through the style catalog, the entry for the direction it points, and that a name nothing
  holds renders as `solid-pointer`.
- §7 _The initial set_: the connector paragraph says a terminal is one of three, and the sentence
  about a glyph terminal's glyph pointing opposite `leaving` extends to a style's head.
- §7 _What a terminal writes_: a third paragraph after **arm** and **glyph**, saying a style writes
  a head that points away from `leaving`, that the cell keeps the name and the rendering resolves
  it, and that it follows no stroke. The paragraph calling the asymmetry a limit of the data gains a
  clause: a style is the answer available now, and heads that follow a glyph set stay the open
  question below.
- §7 _The initial set_, the paragraph saying the core holds no default stroke: one clause separates
  the base stroke, which a figure always names, from the fallback style, which answers a name
  nothing claims, when the rendering looks the name up. The core decides no appearance a figure left
  unsaid.
- §9 _Properties worth testing_: a head in the middle of the stack is the literal's case, and one
  buffer rendered with two style catalogs draws two texts and changes no cell.
- §11 _Open questions_: the question about a terminal's glyph narrows to a set of heads, and the
  answer it names costs a third kind of cell.
- `docs/terminal-styles.md` (new): the eleven tables, each entry named for the way it points, in the
  shape of `docs/glyph-sets.md`, and above them the catalog the rendering is handed.
- `docs/diagram-model.md` §6 _Attachment_: one clause — a head points away from `leaving`, which the
  attachment does not decide — and a paragraph saying the terminal's head keeps a name the rendering
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
    /// The entry for a head pointing `toward`.
    #[must_use]
    pub fn glyph(&self, toward: Direction) -> &Glyph;
}

pub enum Cell {
    // `Strokes` and `Literal`, unchanged
    /// A head: the direction it points, and the style it is written in.
    Head { toward: Direction, style: StyleName },
}

impl Cell {
    /// The text this cell renders to, resolving a head against `styles`.
    #[must_use]
    pub fn glyph_str<'a>(
        &'a self,
        glyphs: &'a GlyphCatalog,
        styles: &'a StyleCatalog,
    ) -> Option<&'a str>;
}

/// Renders `window` from `source`, resolving heads against `styles`.
#[must_use]
pub fn render(
    source: &impl CellReader,
    glyphs: &GlyphCatalog,
    styles: &StyleCatalog,
    window: Window,
) -> String;

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
    /// Draws every shape into `sink`. The catalog a head answers to is not here: nothing is
    /// resolved until the cells are rendered.
    pub fn draw(&self, sink: &mut dyn StampSink);
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

`Style::default()` is the `solid-pointer` table, and the rendering resolves a head's name as
`styles.style(&name.0).cloned().unwrap_or_default().glyph(toward)`. The name is a `String` inside
`StyleName` rather than a closed `enum`: a closed enum would turn an unknown name into a
deserialization error instead of D6's fallback, and it would put the vocabulary of names in the core
where the tables belong. `Cell::Head` carries the name rather than the table for the same reason,
and because a buffer is the thing two builds may render differently.

## Behavior

1. A style terminal writes a head pointing opposite its endpoint's `leaving`, so the glyph points
   away from the route.
2. A head is a cell holding a style name and a direction and no glyph; it covers the cell and
   composes with nothing, writing no arm and following no stroke.
3. A head renders through the style catalog the rendering is handed: the entry named for the
   direction the head points.
4. A name the rendering's catalog does not hold renders as `solid-pointer`, in silence.
5. `solid-pointer` is `up: ▲, right: ►, down: ▼, left: ◄`, and the other ten are the table in
   _Examples_.
6. Each endpoint chooses its own style, independently of the other endpoint and of the stroke.
7. `{ "kind": "style", "style": "<name>" }` keeps the name: drawing it needs no catalog, and
   rendering it resolves the name against the catalog the rendering is handed.
8. `{ "kind": "style" }` with no `style` field is refused, naming the missing field.
9. `arm` and `glyph` draw what they drew before, and every picture that used them is unchanged.
10. `monospace-terminal-styles` compiles for `wasm32-unknown-unknown` and joins the `wasm` step.

## Examples

**The eleven**, each column the way its glyph points: the endpoint leaves the other way, a head
carries that direction, and `Style::glyph` answers it.

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

**The fallback.** `"style": "no-such-style"` renders rule 4's default, so the first picture is what
a name the rendering's catalog does not hold produces, and `{ "kind": "style" }` is refused for the
missing field. Neither is observed: the pictures come from the glyphs each style resolves to, and
_What proves it_ holds the resolution, the fallback and the refusal.

## What proves it

The names bind the building stage, which is held to them literally.

| Rule | Test                                                                                                       |
| ---- | ---------------------------------------------------------------------------------------------------------- |
| 1    | `a_style_writes_the_entry_for_the_direction_opposite_leaving`                                              |
| 2    | `a_head_holds_a_name_and_a_direction_and_no_glyph` and `a_style_covers_the_cell_and_composes_with_nothing` |
| 3    | `a_head_renders_through_the_style_catalog_it_is_handed`                                                    |
| 4    | `an_unknown_style_name_draws_the_default`, in the rendering, since that is where a name is looked up       |
| 5    | `the_eleven_styles_are_the_tables_this_spec_lists`                                                         |
| 6    | `the_two_endpoints_of_a_connector_choose_their_styles_independently`                                       |
| 7    | `a_style_name_reaches_the_rendering_as_written`                                                            |
| 8    | `a_style_terminal_without_a_name_is_refused_by_name`                                                       |
| 9    | the characterization, and every picture in a tracked document, which the gate re-renders                   |
| 10   | the `wasm` step, which names the crate                                                                     |

The two pictures are held by their markers: `cargo xtask render` rewrites them and the gate fails if
one is not what its description draws. **The characterization is what proves the two reversals of D4
moved nothing on screen**: 1856 renderings and 42 pictures are what they were before the name
replaced the table and before the cell took the name over, so the change is where a catalog is read
and not what it draws.

## Open questions

- **Whether a misspelled style should be reported once a rendering can say more than text.** D6
  answers silence where there is no second channel, and adding one is a change to `render` and every
  caller.
