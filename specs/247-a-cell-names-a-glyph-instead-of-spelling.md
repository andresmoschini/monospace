---
status: implemented
decided: "#250"
implemented: "#251"
date: 2026-10-28
---

# A cell names a glyph instead of spelling it

## Why now

A cell holds four arms or one literal glyph. The arms resolve at render, through the catalog, so
replacing a rule changes every cell drawn from it. The literal never looks anything up: `▲` is
whatever the caller typed, and no table can say otherwise. Anything that points needs the second
answer and cannot have it — a caller writes `▲` at one end and `►` at the other, and nothing checks
that either agrees with the direction it leaves in.

A style is a table of glyphs, one per shape, and a terminal should write "the solid look, pointing
up" rather than a character the caller chose, so that the catalog decides what a build draws and a
name survives to whatever reads the diagram later.
[#57](https://github.com/andresmoschini/monospace/issues/57) is the terminal that writes one. The
vocabulary is seven rather than four because a style draws a look: a diamond or a circle is as much
a head as an arrow.

## Scope

### In

- `monospace-core`: `Symbol`, `SymbolStyleName`, `SymbolCell`, `Cell::Symbol`; a second map in
  `GlyphCatalog`, `from_symbol_rules`, `symbol` and `symbol_styles`; `union` over both; `solid`; and
  the matches a third kind breaks — `is_decided`, `glyph_str`, `merge`, `decides_a_side` and the two
  galleries, which resolve a symbol cell through the catalog they render with.
- `monospace-glyph-sets`: the other six styles, each a table whose rows name it, and `all()` widened
  to union them.
- `monospace-cli`: `--symbols`, the usage line, and the refusals beside it.
- Docs: `docs/model.md` §1, §3–§6 and §9; a new `docs/symbols.md`; one line in `docs/glyph-sets.md`;
  `README.md`'s usage and layout; and `AGENTS.md`'s sentence giving the CLI two modes.

### Out

- **Any figure that writes a symbol cell.**
  [#57](https://github.com/andresmoschini/monospace/issues/57) is the terminal that does; its spec
  is agreed, unimplemented, and rewritten against this vocabulary before it is built — the wire
  format, the style per endpoint and the orientation rule are that spec's. Its eleven style names do
  not survive: `solid` and `outline` are looks over seven symbols, not one table per shape.
- **A symbol rule loaded from a file.** [#63](https://github.com/andresmoschini/monospace/issues/63)
  is the same question for arm rules.
- **A style that composes.** A symbol cell covers the cell; a style that joined a border would be
  arm rules, and an arm cannot point.

## The decision

**D1 — a cell can hold a symbol and a style, and the catalog answers the pair at render.**

**Answer:** `Cell::Symbol(SymbolCell { symbol, style })` is the third kind, rendered by looking its
pair up in the one catalog the renderer was handed. **Why not** resolving the names into a literal
where they are written: the names die there, and a typed `▲` and "the solid look, direction up"
would be the same cell. **Why not** carrying the names and the glyph: one answer with two sources,
and a table that changed later would leave them disagreeing. **Why not** a second catalog argument
on `glyph_str`: a cell of arms and a cell of a symbol would be rendered against two catalogs with
nothing saying which a caller meant. **What it costs:** every exhaustive match over cells grows, and
a symbol cell is never equal to the literal drawing the same character. **Answered by** the
maintainer.

**D2 — the vocabulary is seven closed symbols and one open style name.**

**Answer:** `Symbol` is a closed enum — `DirectionUp`, `DirectionRight`, `DirectionDown`,
`DirectionLeft`, `Diamond`, `Hexagon`, `Circle` — and `SymbolStyleName` is an open newtype over a
`String`, as `Stroke` is. The core's matches decide an eighth symbol; the catalog decides which
styles exist. **Why not** `pointing: Direction`: a direction is one family of symbol, and a diamond,
a hexagon or a circle does not point, so the key would change the day the vocabulary widened — it
has widened here. **Why not** `name: String`: a misspelled symbol and a missing pair would answer
alike, and the core would hold a vocabulary it does not decide. **Why not** `#[non_exhaustive]`: an
eighth symbol should break every match that decides it, not be swallowed by a wildcard. **What it
costs:** adding a symbol breaks every exhaustive match outside the crate.

**D3 — a symbol cell is its own key; there is no second type.**

**Answer:** a symbol rule is filed under the `SymbolCell` itself, and `symbol(&SymbolCell)` is the
lookup. **Why not** a `SymbolKey` carrying the same fields: a second spelling of one value, a
conversion at every write and lookup, and a reader who must know which of the two a catalog holds.
**Why not** keying on the symbol alone and holding a style inside the value: a catalog holds one
glyph per key, so two styles for one symbol would need a second map inside it. **What it costs:**
the cell and the key are one rectangle, so a symbol cell that grows a field changes the key.

**D4 — the catalog holds two maps, and a pair it lacks renders as a space.**

**Answer:** `GlyphCatalog` keeps `arms` — the `GlyphKey` map, renamed from `rules` — and gains
`symbols`, a `SymbolCell` map; `union` merges both, first claim wins within each. `glyph_str` tries
the cell's exact pair and nothing else; a pair it does not hold answers nothing, which is a space.
**Why not** degrading a missing pair to another style as a stroke cell degrades to its base stroke:
the arm fallback stays inside the cell's own strokes, while a style fallback replaces the look
whole, and a cell holding `outline`/`diamond` named no second style. **Why not** the draft's default
style: a misspelled style would draw as `solid-pointer`, a typo would be invisible twice over, and
the default would be a second copy of a table. **Why not** one map over one key enum: different
keys, different answers to a miss, and folding them churns every `GlyphKey` use and makes §5's "a
rule maps four arms" untrue. **What it costs:** a misspelled style and a missing shape are both a
space, with nothing said.

**D5 — a symbol cell composes as a literal does, and stays a different value.**

**Answer:** in the buffer it is opaque: on top it replaces what it meets, and under a stamp of arms
it is replaced whole with that stamp's `Unset` sides closed. Two symbol cells compose as two
literals do, and the top one wins. Equality is by the pair, so a symbol cell and the literal drawing
the same character are two cells. **Why not** normalizing one into the other on the first stamp: the
names are what an editing layer could rewrite, and the first figure to land would throw them away.

**D6 — a style is a column of a symbol rule, and a table may hold more than one.**

**Answer:** a symbol rule is a `(SymbolCell, Glyph)`, so the style travels in the key and a catalog
answers only the look each cell names. The tables are data: `solid` is seven rows in the core —
`▲▶▼◀◆⬢●` — and `outline` (`△▷▽◁◇⬡○`), `solid-pointer` (`▲►▼◄`), `small-solid` (`▴▸▾◂`),
`small-outline` (`▵▹▿◃`) and `arrow` (`↑→↓←`) are the sets crate's, with `ascii` (`^>v<`) adding `o`
for the circle. `solid` is the core's, as Light is; the rest and `all()` are the sets crate's, and
nothing pairs a name with a table. **Why not** the style in the value and the symbol alone as the
key: a catalog could hold one look per symbol, so `all()` could not hold solid's `▲` beside ascii's
`^`. **Why not** a table per style, named by the function that builds it: the name would then live
where the tables are listed rather than in the table — `solid`'s in the sets crate while its rows
sit in the core — and a theme that wanted two looks in one table could not say so. **Why not** a
`TerminalStyle` of four glyphs: a style over seven symbols is not four fields — `ascii` answers five
of the seven — and the catalog already holds a pair-to-glyph rule. **Why not** requiring every style
to answer every symbol: a pointer has no diamond, and repeating its triangle would draw a shape
nobody chose. **Why not** one name per glyph, as the draft's eleven had: a style is a look, and `◆`
and `◇` are its diamond, not styles. **What it costs:** `solid-pointer`/`Circle` draws a blank, and
a style name repeats on every row that carries it, as a stroke name does. **Answered by** the
maintainer, who asked for one name per look.

**D7 — the sheet is a mode of the command line, and it takes nothing else.**

**Answer:** `monospace-cli --symbols` prints one row per style the catalog reports, in the order it
reports them, the name and one cell per symbol in `Symbol::ALL` order, each cell naming that style,
rendered through `render` with the one catalog — `all()`. A pair the style lacks is the space it
renders to. A path, `--size` or `--origin` beside it is refused by name, because the sheet is the
catalog's own. **Why not** a catalog per row: the cells already name their style, and a narrowed
copy of the same data would be the second source D1 refused. **Why not** only `docs/symbols.md`: a
document drifts, and the sheet is the same data rendered by the code. **Why not** a `render` marker:
the marker machinery renders a description, and no figure writes a symbol cell yet. **Why not**
accepting a path or window: each names a diagram the sheet has nowhere to put. **What it costs:** a
third mode in a binary whose comment says two, a usage line with three forms, and a test that is the
only check the sheet gets.

## Model slice

- §1 _Vocabulary_: `Cell` gains a third kind; `Symbol`, `SymbolStyleName`, `SymbolCell` and a symbol
  rule join the table; `GlyphCatalog` holds both maps.
- §3 _The cell_: a subsection after _A cell can be a literal instead_ — a symbol cell, its two
  names, that it covers the cell and composes as a literal, and that a missing pair is a space.
- §4 _Stamping_: a symbol cell sits wherever a literal sits; the two opaque kinds compose alike.
- §5 _Strokes, glyph sets and the catalog_: a symbol rule maps a `SymbolCell` to one glyph and never
  degrades, so a style is a column of the rule; a glyph set stays a table of arms, and a table may
  hold rules of more than one style; `union`'s first claim covers each map.
- §6 _Rendering_ and §9 _Properties worth testing_: the third branch — one lookup of the pair, a
  miss is no glyph — and the property that a complete style answers every symbol.

## Public surface

```rust
// monospace-core
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol { DirectionUp, DirectionRight, DirectionDown, DirectionLeft, Diamond, Hexagon, Circle }
impl Symbol { pub const ALL: [Symbol; 7]; }
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolStyleName(pub String); // From<&str>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolCell { pub symbol: Symbol, pub style: SymbolStyleName }
pub enum Cell { Strokes(StrokeCell), Literal(Glyph), Symbol(SymbolCell) }
impl GlyphCatalog {
    pub fn from_symbol_rules(rules: impl IntoIterator<Item = (SymbolCell, Glyph)>) -> Self;
    pub fn symbol(&self, key: &SymbolCell) -> Option<&Glyph>;
    pub fn symbol_styles(&self) -> Vec<&SymbolStyleName>;
    pub fn solid() -> Self;
}
// monospace-glyph-sets
pub fn outline() -> GlyphCatalog; // and one per style
pub fn all() -> GlyphCatalog; // the nine arm tables and the seven styles
```

`solid` is the core's, as `light` is, and `symbol_styles()` reads the names off the rules rather
than off a list, so the sheet asks a catalog which styles it holds. `Cell::glyph_str` keeps its
single `&GlyphCatalog`; a pair the catalog does not hold is `None`.

## Behavior

1. A symbol cell renders as the glyph filed under its pair; a pair the catalog does not hold is
   nothing, which is a space. No second lookup, no degradation.
2. A symbol cell is decided on all four sides: it writes no arm and follows no stroke.
3. In the buffer it composes as a literal: above, it replaces what it meets; below, a stroke cell
   replaces it whole with the stamp's `Unset` sides closed. Two symbol cells compose as two
   literals.
4. A symbol cell and the literal drawing the same character render alike and stay different cells:
   equality is by the pair.
5. `union` merges both maps by the same first-claim rule within each, and neither answers for the
   other. Two styles of one symbol are two pairs, so a catalog holds both and `union` keeps both.
6. `solid` and `outline` answer all seven; the four pointer styles answer the four directions and
   `ascii` adds the circle; `Diamond` and `Hexagon` render as spaces under `ascii`, and so does
   `Circle` under the four.
7. `symbol_styles()` reports each style the symbol rules name, once, in name order; `all()` reports
   the seven — `arrow`, `ascii`, `outline`, `small-outline`, `small-solid`, `solid`,
   `solid-pointer`.
8. `--symbols` prints one row per style `all()` reports, in that order, the name padded to the
   longest and one space, then one cell per symbol in `Symbol::ALL` order, each naming the row's
   style, through `render` and `all()` itself. A miss is a space and the row keeps it.
9. `--symbols` with a path, `--size`, `--origin`, or twice, is refused by name; the usage line lists
   the flag as a form of its own. A bare run, a path run, and every tracked picture are unchanged.

## Examples

**The sheet**, columns the four directions, diamond, hexagon and circle; blank cells are missing
pairs, which the style renders as a space. Rule 8 prints it:

```text
arrow         ↑→↓←
ascii         ^>v<  o
outline       △▷▽◁◇⬡○
small-outline ▵▹▿◃
small-solid   ▴▸▾◂
solid         ▲▶▼◀◆⬢●
solid-pointer ▲►▼◄
```

The four pointer styles end with three spaces, one per symbol each lacks; `ascii` keeps its two gaps
before the `o`. The repository trims trailing whitespace from this fence, which is exactly why a
miss at the end of a row is easy to mistake for a full one.

**A pair the catalog lacks is a space.** A catalog holding only `solid`/`diamond` answers `◆`; asked
for `outline`/`diamond` or `nonsense`/`diamond` it answers nothing, as it does for every pair when
it holds no symbol rule. No fallback and no second lookup: a misspelled style is indistinguishable
from a style that does not draw the shape.

**A name is not the character.** These two values draw the same `▲` and are not equal, which rule 4
keeps apart in a buffer:

```rust
Cell::Symbol(SymbolCell { symbol: Symbol::DirectionUp, style: SymbolStyleName::from("solid") })
Cell::Literal(Glyph::new("▲").expect("▲ is one glyph"))
```

## What proves it

The names bind the building stage, and the run below is what was observed. The cell's render, its
composition and the two maps are `monospace-core`'s; the styles and `all()` are
`monospace-glyph-sets`'s; the sheet and its refusals are `monospace-cli`'s subprocess tests.

| Rule | Test                                                                                      | Where                        |
| ---- | ----------------------------------------------------------------------------------------- | ---------------------------- |
| 1    | `a_symbol_cell_answers_its_exact_pair_and_a_missing_pair_is_nothing`                      | `cell.rs`                    |
| 2    | `a_symbol_cell_writes_no_arm_and_is_decided_on_every_side`                                | `cell.rs`                    |
| 3    | `a_symbol_cell_composes_as_a_literal_does`                                                | `buffer.rs`                  |
| 4    | `a_symbol_cell_and_the_literal_that_draws_it_render_alike_and_differ`                     | `cell.rs`                    |
| 5    | `union_merges_both_maps_and_holds_two_styles_of_one_symbol`                               | `glyph.rs`                   |
| 6    | `solid_answers_all_seven_symbols`                                                         | `glyph.rs`                   |
| 6    | `outline_is_complete_the_four_pointer_styles_answer_directions_and_ascii_adds_the_circle` | `glyph-sets/src/lib.rs`      |
| 7    | `symbol_styles_reports_each_style_once_in_name_order`                                     | `glyph.rs`                   |
| 7    | `all_holds_every_style_and_reports_the_seven_in_name_order`                               | `glyph-sets/src/lib.rs`      |
| 8    | `symbols_prints_a_row_per_style_in_name_order_with_a_space_for_a_miss`                    | `monospace-cli/tests/cli.rs` |
| 9    | `symbols_refuses_a_path_and_a_window_and_the_usage_line_names_it`                         | `monospace-cli/tests/cli.rs` |
| 9    | the characterization, and every picture a tracked document carries                        | the gate                     |

## Open questions

- **Whether a misspelled style should be reported once the format can say more than a refusal.** D4
  answers silence, and adding a channel is a change to every caller.
- **Whether an incomplete style should refuse the write instead of drawing a space.**
- **Whether #57's wire names a style and a symbol as two fields or one cell string**, and whether a
  loaded symbol table spells a rule the same way. Both are the next specs' questions.
