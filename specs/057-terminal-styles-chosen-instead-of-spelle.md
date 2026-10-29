---
status: agreed
decided: "#256"
date: 2026-10-29
---

# Terminal styles, chosen instead of spelled out

_Rewritten against the vocabulary #247 built, superseding the eleven tables agreed in #245._

## Why now

Issue #247 gave a cell the vocabulary this slice needs: `Cell::Symbol` holds a `Symbol` and the
`SymbolStyleName` it is drawn in, and the catalog answers the pair at render. `solid` ships in the
core and six more styles in `monospace-glyph-sets`, so `--symbols` already prints every look a build
draws. A terminal that names one of them is the remaining half, and it closes the silent mistake
this issue opened with: a caller writes `▲` at one end and `►` at the other, and nothing checks that
either agrees with the direction it leaves in.

The spec agreed in #245 described that terminal as a `Style` of four glyphs and a new crate of
eleven, and #246 began to build it. #247 replaced the vocabulary: a style is a column of a symbol
rule, not a table of four, and the seven names are not the eleven. So #245's D3–D6 no longer
describe the code, and #247's _Out_ says this spec is rewritten against it before it is built. That
rewrite is this document; the spike `spikes/continue-247` is the evidence that the shape works, not
a branch that merges.

Neither open slice reaches it: #63 loads an arm table from a file, and #206 re-spells the format. A
loaded style stays that slice's question, and this wire extends the same format.

## Scope

### In

- `monospace-core`: `Terminal::Symbol { style: SymbolStyleName }`; the `pointing`/`symbol_for`
  mapping beside `arm_side`; `Endpoint::draw`, so a third terminal is one match rather than a
  rewrite of `Connector::draw`; the `Head` → `Literal` rename and the `NamedSymbol` fragment; the
  design notes in `connector.rs` and `head.rs` that say two terminals and no terminal names a
  symbol.
- `monospace-description`: `{ "kind": "symbol", "style": "<name>" }`, a missing `style` refused by
  name, and the message naming `glyph`, `symbol` and `arm`.
- `monospace-core`'s shape gallery: one connector whose endpoints name `solid`, and one
  `NamedSymbol` block; the gallery's catalog gains `solid` so a symbol cell reads as the glyph it
  draws rather than as a hole.
- Docs: `docs/model.md` §7 and §11; `docs/diagram-model.md` §6; `docs/diagram-demo.md`'s section on
  terminals; one sentence in `docs/symbols.md`.
- Tests: the orientation rule over the four directions, the terminal that draws what a spelled head
  draws, two endpoints naming two styles, the space an unknown name draws, the refusal of a missing
  field, and that `glyph` and `arm` still draw what they drew.

### Out

- **The eleven style tables and `monospace-terminal-styles`.** #247 replaced them: a style is a
  column of a symbol rule, `solid` is the core's one table and six more are the sets crate's. #245's
  D5 and the crate it named do not survive, and no crate joins the workspace.
- **A terminal that names a symbol which does not point.** A diamond, a hexagon and a circle have no
  direction to point against `leaving`, so a symbol terminal cannot reach them; they stay literals
  (`{ "kind": "glyph", "glyph": "◆" }`) and a terminal that named one would be a fourth variant.
  `◆──────►` is a glyph at the diamond end.
- **A style loaded from a file.** #63 is the same question for arm tables; a loaded symbol table
  belongs with it.
- **A style chosen once per connector.** D2.
- **A default or a fallback style.** A name no table holds is a space, as #247's cell rule already
  says; `solid` is the core's table, not a default.

## The decision

**D1 — the endpoint names the style, and the symbol is derived from `leaving`.**

**Answer:** `Terminal::Symbol { style: SymbolStyleName }` writes
`SymbolCell { symbol: pointing(opposite(leaving)), style }`, all four directions, through a private
`symbol_for` beside `arm_side`. The caller names the look and cannot name the direction. **Why not**
letting the terminal carry its own `Symbol`: a caller could write `▲` at an endpoint leaving `down`
and nothing would refuse it, which is the silent mistake this slice exists to end. **Why not** the
`Style` of four glyphs and `Style::glyph(leaving)` that #245 agreed: #247 replaced a table per style
with one pair-to-glyph catalog, so a style is a column rather than four fields, and a table keyed by
direction cannot hold a diamond. **What it costs:** one private mapping, and a terminal that cannot
draw a non-pointing head. **Answered by** the maintainer, on reading the spike.

**D2 — each endpoint names its own style.**

**Answer:** the terminal is a field of `Endpoint`, so one connector may carry two styles (`◀────→`),
or a style at one end and a glyph or an arm at the other. **Why not** a style on the connector with
the endpoints as overrides: two ends wanting different heads is ordinary, so the override would be
the common case and a precedence rule would exist only to save one word. **What it costs:** a
symmetric arrow names the style twice. **Answered by** the maintainer, in #245.

**D3 — a symbol terminal writes a symbol cell: it covers the cell and follows no stroke.**

**Answer:** it writes `Cell::Symbol`, decides all four sides, composes as a literal does — above it
replaces what it meets, below it is replaced whole — and it writes no arm and follows no stroke.
**Why not** an arm: an arm cannot point, which is why this terminal exists. **Why not** deriving the
style from the connector's stroke: one description would then draw differently depending on which
tables a build shipped. **What it costs:** a style over a box's border interrupts it, as a literal
does, which is the caller's choice where the endpoint stands. **Answered by** #247's cell rule and
the maintainer.

**D4 — the terminal holds the name, and the render's catalog answers it.**

**Answer:** the terminal carries `SymbolStyleName`, exactly as `Cell::Symbol` does, and the pair is
written as named; the catalog handed to `render` answers it. **Why not** #245's resolved four-glyph
table: the core would hold the vocabulary #63 wants to make a file's, and a table swapped under the
drawing would not redraw the terminals that named it. **Why not** resolving the name to a literal in
the reader: nothing above the reader could tell a named head from a typed one, so an editing layer
could neither show nor change it. **What it costs:** a style the rendering catalog does not hold is
a space, and the terminal does not decide it.

**D5 — the core ships `solid`, and `monospace-glyph-sets` ships the other six.**

**Answer:** `GlyphCatalog::solid()` is the core's table, as `light()` is; `outline`,
`solid-pointer`, `small-solid`, `small-outline`, `arrow` and `ascii` are the sets crate's, and
`all()` unions every style. **Why not** #245's `monospace-terminal-styles`: #247 already split the
styles between the core and the sets crate, so a second crate would hold a second copy of the names
and a second `all()`, and the `wasm` step already names the crate that has them. **Why not** all
seven in the core: every appearance the project ships would be a core table, which is the split the
glyph sets exist to avoid. **What it costs:** `solid` is no longer a default; it is a table a caller
names like any other. **Answered by** practice, in #247.

**D6 — an unknown style is a space; a missing `style` is refused by name.**

**Answer:** `{ "kind": "symbol", "style": "nope" }` draws a space, because #247's miss is final and
every shipped style answers the four directions, so only a misspelling reaches it;
`{ "kind": "symbol" }` is refused by the message naming `style`, which also names `glyph`, `symbol`
and `arm`. **Why not** #245's fallback to `solid-pointer`: a typo would be invisible twice over, the
fallback would be a second copy of a table, and a style that lacked the direction would draw a shape
nobody chose. **Why not** reporting the substitution: `parse` answers a `Diagram` or a `ParseError`
and has no third channel, and adding one is a change to every caller for a misspelling. **What it
costs:** a misspelled style is a space. **Answered by** #247's D4 and the maintainer.

**D7 — the wire tag is `symbol`, and the field is `style`.**

**Answer:** `{ "kind": "symbol", "style": "<name>" }`; the description's enum mirrors
`monospace_core::Terminal`, so the variant's name is the tag's. **Why not** #245's
`{ "kind": "style", ... }`: `glyph` and `arm` name what the terminal writes, and this one writes a
symbol cell; the style is the attribute that fills it, exactly as it is the `style` field of
`SymbolCell`. Naming the kind `style` would make the reader translate a variant the core calls
`Symbol`. **Why not** a `symbol` field the caller spells: D1. **What it costs:** the JSON in #57's
wish and #245's wire are superseded. **Answered by** the maintainer.

## Model slice

- `docs/model.md` §7 _The initial set_: the sentence about a glyph terminal gains a symbol terminal,
  and §7 _What a terminal writes_ says a terminal is a glyph, a symbol or an arm. The `symbol`
  paragraph follows the glyph one: the symbol is derived from `leaving`, points against it, covers
  the cell, and a pair the catalog lacks is a space.
- §7's asymmetry paragraph is rewritten rather than extended: a symbol rule can point, a terminal
  that names a style follows it, so the remaining question is the arm-rule head `╶ ╴ ╵ ╷`, not the
  terminal's. The paragraph in _The initial set_ saying the core holds no default stroke gains a
  clause: the one table it ships, `solid`, is named by whoever wants it and is not a default.
- §11 _Open questions_: the terminal question is answered and replaced by its remainder — where an
  _arm_ terminal's head comes from, once a glyph set can hold one — because this slice gives the
  symbol terminal its head and leaves the arm terminal's to that slice.
- `docs/diagram-model.md` §6 _Attachment_: one clause — a symbol terminal's symbol follows
  `leaving`, and the attachment decides neither. The section's rule stands.
- `docs/diagram-demo.md`: _Two terminals, and the one difference between them_ becomes three, with a
  `symbol` row in the picture already there. The row names `solid`, which draws the `◀`/`▶` the
  literal row already draws, so the section shows the same head arrived at two ways.
- `docs/symbols.md`: one sentence under _Naming a cell_ — a terminal names a style and writes the
  symbol pointing against `leaving` — so the document that owns the vocabulary says where a terminal
  reaches it.

## Public surface

```rust
// monospace-core
pub enum Terminal {
    // `Glyph` and `Arm`, unchanged
    /// One named symbol, in a style the caller names. The symbol points against `leaving`.
    Symbol { style: SymbolStyleName },
}

// monospace-description, on the wire:
{ "kind": "symbol", "style": "<name>" }
```

The mapping from `leaving` to the symbol stays private, beside `arm_side`, so no caller can name
one. `Style`, `Style::glyph`, `StyleCatalog`, `monospace-terminal-styles`, `by_name`, `all()` and
the eleven tables of #245 do not exist. `parse` gains no parameter and no catalog: the style is not
resolved where the file is read, and the render's catalog decides it.

## Behavior

1. A symbol terminal writes `Cell::Symbol(SymbolCell { symbol, style })`, where `symbol` is the one
   pointing opposite the endpoint's `leaving`: `up` writes `DirectionDown`, `right` writes
   `DirectionLeft`, and so on. The caller names the style and no symbol.
2. It writes no arm, follows no stroke, and composes as a literal does: above it replaces what it
   meets, and below a stroke cell it is replaced whole.
3. The pair is written as named. The catalog handed to `render` answers it; a pair it does not hold
   is a space, and the terminal neither falls back nor reports.
4. Each endpoint names its own style. One connector may carry two styles, or a style at one end and
   a glyph or an arm at the other.
5. Every shipped style answers the four directions: `solid` `▲▶▼◀`, `outline` `△▷▽◁`,
   `solid-pointer` `▲►▼◄`, `small-solid` `▴▸▾◂`, `small-outline` `▵▹▿◃`, `arrow` `↑→↓←`, `ascii`
   `^>v<` — the `Direction*` columns of `docs/symbols.md`. No style answers a diamond, hexagon or
   circle from a terminal, because a terminal has no direction to derive one from.
6. `{ "kind": "symbol", "style": "<name>" }` is the wire form. An unknown name draws a space;
   `{ "kind": "symbol" }` is refused by the message naming the missing `style`, and the three kinds
   are `glyph`, `symbol` and `arm`.
7. `glyph` and `arm` draw what they drew. Every tracked picture but the demonstration document's
   terminal section is unchanged; that section gains a row, and the build may switch its marker to
   `kind: symbol`.

## Examples

**The orientation case.** An endpoint at `(0, 0)` leaving `down` and one at `(1, 1)` leaving `left`,
both naming `solid`. The `▲` is `DirectionUp`, written by the endpoint leaving `down`; the `▶` is
`DirectionRight`, written by the one leaving `left`:

<!-- render: 2x2
{
  "next_id": 2,
  "shapes": [
    { "kind": "connector", "id": 1,
      "from": { "at": { "kind": "point", "x": 0, "y": 0 }, "leaving": "down",
                "terminal": { "kind": "glyph", "glyph": "▲" } },
      "to":   { "at": { "kind": "point", "x": 1, "y": 1 }, "leaving": "left",
                "terminal": { "kind": "glyph", "glyph": "▶" } },
      "stroke": "light" }
  ] }
-->

```text
▲
└▶
```

<!-- /render -->

The marker spells the glyphs, because the gate renders with the code that exists;
`{ "kind": "symbol", "style": "solid" }` at each end draws the same picture.

**Two styles in one connector,** which is D2's case: `solid` at the end leaving rightward and
`arrow` at the end leaving leftward draw `◀──────→`.

<!-- render: 8x1
{
  "next_id": 2,
  "shapes": [
    { "kind": "connector", "id": 1,
      "from": { "at": { "kind": "point", "x": 0, "y": 0 }, "leaving": "right",
                "terminal": { "kind": "glyph", "glyph": "◀" } },
      "to":   { "at": { "kind": "point", "x": 7, "y": 0 }, "leaving": "left",
                "terminal": { "kind": "glyph", "glyph": "→" } },
      "stroke": "light" }
  ] }
-->

```text
◀──────→
```

<!-- /render -->

The marker again spells the heads; the build may name them.

**A name the catalog does not hold** draws a space, and `{ "kind": "symbol" }` is refused for the
missing field. Neither is observed here: the pictures come from the glyphs each style resolves to,
and _What proves it_ holds the miss and the refusal.

## What proves it

The terminal does not exist yet: the deciding stage adds no code. The names bind the building stage.

| Rule | Test                                                                                          | Where                       |
| ---- | --------------------------------------------------------------------------------------------- | --------------------------- |
| 1    | `a_symbol_terminal_writes_the_symbol_pointing_against_leaving`                                | `connector.rs`              |
| 2    | #247's `a_symbol_cell_composes_as_a_literal_does` and the path test                           | `buffer.rs`, `connector.rs` |
| 3    | `a_symbol_terminal_draws_the_head_a_glyph_terminal_spells_out`                                | `connector.rs`              |
| 4    | `the_two_endpoints_of_a_connector_name_their_own_styles`                                      | `connector.rs`              |
| 5    | #247's `all_holds_every_style_and_reports_the_seven_in_name_order`                            | `glyph-sets/src/lib.rs`     |
| 6    | `an_unknown_style_name_draws_a_space`, `a_symbol_terminal_without_a_style_is_refused_by_name` | `monospace-description`     |
| 7    | the characterization, and every picture a tracked document carries                            | the gate                    |

The two pictures are held by their markers: `cargo xtask render` rewrites them and the gate fails if
one is not what its description draws.

## Open questions

- **Whether a terminal should name a non-pointing symbol**, so a diamond or a circle head could be
  derived from a style. D1 leaves it out because a terminal always has a `leaving` to point against;
  a fourth variant, or a `symbol` field with a rule that a pointing symbol must agree, would answer
  it.
- **Whether a misspelled style should be reported once the format can say more than a refusal.** D6
  answers silence where there is no second channel, and adding one is a change to `parse` and every
  caller.
