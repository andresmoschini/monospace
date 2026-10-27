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

_What a terminal writes_ calls that asymmetry a limit of the data and leaves it as the open question
about where a terminal's glyph could come from, preferring heads that follow the glyph set. This is
the answer that changes no existing cell: a table of four glyphs per style, indexed by `leaving`,
leaves the four single-arm keys `╶ ╴ ╵ ╷` untouched, so a set of heads stays a question for another
slice. Neither open slice that touches this vocabulary reaches it:
[#63](https://github.com/andresmoschini/monospace/issues/63) loads a glyph set from a file, and a
loaded set is still a table of arms, while
[#206](https://github.com/andresmoschini/monospace/issues/206) re-spells the format, and a third
terminal is better settled here than invented there.

## Scope

### In

- `monospace-core`: a `Style` type, one default style, and a third `Terminal` variant. The design
  note in `connector.rs` saying the set is closed at two says three, for the reason it gave.
- `monospace-terminal-styles`: a crate holding the other ten styles and the lookup a name goes
  through. It joins the crates the `wasm` step compiles.
- `monospace-description`: `{ "kind": "style", "style": "<name>" }` on the wire, resolved against
  the eleven.
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

**D1 — the table is indexed by `leaving`, and the glyph points opposite it.**

**Answer:** the four entries are indexed by the direction the connector leaves the endpoint in, and
each holds the glyph that points the other way, so `solid-pointer` is
`up: ▼, right: ◄, down: ▲, left: ►`. **Why not** indexing by the direction the glyph points: the
endpoint holds no such value, so every lookup would complement `leaving` first and the inversion
would live in the code rather than in the table that states it. **What it costs:** the table reads
backwards against Unicode's names, where `BLACK RIGHT-POINTING POINTER` sits under `left`.
**Answered by** the maintainer.

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

**D4 — the terminal holds the resolved table, not the name.**

**Answer:** `Terminal::Style { style: Style }` carries the four glyphs, and the description format
resolves the name once, as it reads the file. **Why not** `Style { name: String }` looked up at draw
time: the core would hold the vocabulary of names, which is the new crate's and which #63 wants to
make a file's, and every draw would need the catalog handed to it. **Why not** resolving the name to
a glyph in the reader and handing back `Terminal::Glyph`: nothing above the reader could tell a
style from a literal, so an editing layer could show neither the name nor change it. **Answered by**
practice, in the session that wrote this.

**D5 — the core owns one style and the type; a new crate owns the other ten and the lookup.**

**Answer:** `monospace-core` holds `Style`, `Style::default()` — the `solid-pointer` table — and the
third variant; `monospace-terminal-styles` holds the other ten, `by_name`, and `all()`, which
includes the core's default. It is the split `monospace-core` and `monospace-glyph-sets` already
make: the core ships one, the crate ships the rest, and the union is named once. **Why not** all
eleven in the core: every appearance the project ships would be a core table, which is what the
glyph-set split exists to avoid. **Why not** the lookup in the core beside `GlyphCatalog`: the core
never looks a style up, so that catalog would hold names nothing reads, while the one default it
needs is a value. **Answered by** practice, in the session that wrote this.

**D6 — an unknown name draws the default, in silence; a missing name is refused.**

**Answer:** `"style": "nonsense"` draws the `solid-pointer` table and reports nothing, and
`{ "kind": "style" }` without a name is refused by the message that names the missing field. **Why
not** refusing an unknown name: the list is open the way glyph sets are, so a file written against a
later catalog would stop being readable by an earlier build, where a glyph set degrades instead.
**Why not** reporting the substitution: `parse` answers a `Diagram` or a `ParseError` and has no
third channel, and adding one is a change to every caller for a misspelling. **What it costs:** a
misspelled style is invisible. **Answered by** practice, in the session that wrote this.

## Model slice

- `docs/model.md` §1 _Vocabulary_: a `Style` row — the four glyphs an endpoint may choose from.
- §7 _The initial set_: the connector paragraph says a terminal is one of three, and the sentence
  about a glyph terminal's glyph pointing opposite `leaving` extends to a style.
- §7 _What a terminal writes_: a third paragraph after **arm** and **glyph**, saying a style writes
  one glyph chosen by its table from `leaving`, covers the cell, and follows no stroke. The
  paragraph calling the asymmetry a limit of the data gains a clause: a style is the answer
  available now, and heads that follow a glyph set stay the open question below.
- §7 _The initial set_, the paragraph saying the core holds no default stroke: one clause separates
  the base stroke, which a figure always names, from the fallback style, which answers a name
  nothing claims. The core decides no appearance a figure left unsaid.
- §11 _Open questions_: the question about a terminal's glyph narrows to a set of heads.
- `docs/terminal-styles.md` (new): the eleven tables indexed by `leaving`, the inversion stated
  beside them, in the shape of `docs/glyph-sets.md`; `README.md`'s layout table names it.
- `docs/diagram-model.md` §6 _Attachment_: one clause — a style's glyph follows `leaving`, which the
  attachment does not decide. The section's rule stands.
- `docs/diagram-demo.md`: _Two terminals, and the one difference between them_ becomes three, with a
  style row in the picture already there, because that document owes one instance of each decision
  the format makes.

## Public surface

```rust
// monospace-core
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    /// What an endpoint leaving upward writes: `▼`.
    pub up: Glyph,
    pub right: Glyph,
    pub down: Glyph,
    pub left: Glyph,
}

impl Style {
    #[must_use]
    pub fn glyph(&self, leaving: Direction) -> &Glyph;
}

pub enum Terminal {
    // `Glyph` and `Arm`, unchanged
    Style { style: Style },
}

// monospace-terminal-styles
#[must_use]
pub fn by_name(name: &str) -> Option<Style>;
#[must_use]
pub fn all() -> Vec<(&'static str, Style)>; // the eleven, `solid-pointer` included
pub fn solid_triangle() -> Style; // and one per style, named after it:
// small_solid_triangle, outline_triangle, small_outline_triangle, solid_pointer, unicode_arrow,
// ascii_arrow, white_diamond, black_diamond, white_circle, black_circle
```

`Style::default()` is the `solid-pointer` table, and `monospace-description` resolves a wire name as
`monospace_terminal_styles::by_name(&name).unwrap_or_default()`. The lookup takes a `String` rather
than a closed `enum`: a closed enum would turn an unknown name into a deserialization error instead
of D6's fallback.

## Behavior

1. A style terminal writes the glyph its table holds for the endpoint's `leaving`, and that glyph
   points opposite `leaving`.
2. A style covers the cell and composes with nothing: it writes no arm and follows no stroke.
3. `solid-pointer` is `up: ▼, right: ◄, down: ▲, left: ►`, and the other ten are the table in
   _Examples_.
4. Each endpoint chooses its own style, independently of the other endpoint and of the stroke.
5. `{ "kind": "style", "style": "<name>" }` resolves against the eleven, and a name it does not hold
   draws `solid-pointer`.
6. `{ "kind": "style" }` with no `style` field is refused, naming the missing field.
7. `arm` and `glyph` draw what they drew before, and every picture that used them is unchanged.
8. `monospace-terminal-styles` compiles for `wasm32-unknown-unknown` and joins the `wasm` step.

## Examples

**The eleven**, indexed by `leaving`: the column says which way the connector leaves, not which way
the glyph points.

| name                     | leaves `up` | leaves `right` | leaves `down` | leaves `left` |
| ------------------------ | ----------- | -------------- | ------------- | ------------- |
| `solid-triangle`         | ▼           | ◀              | ▲             | ▶             |
| `small-solid-triangle`   | ▾           | ◂              | ▴             | ▸             |
| `outline-triangle`       | ▽           | ◁              | △             | ▷             |
| `small-outline-triangle` | ▿           | ◃              | ▵             | ▹             |
| `solid-pointer`          | ▼           | ◄              | ▲             | ►             |
| `unicode-arrow`          | ↓           | ←              | ↑             | →             |
| `ascii-arrow`            | v           | <              | ^             | >             |
| `white-diamond`          | ◇           | ◇              | ◇             | ◇             |
| `black-diamond`          | ◆           | ◆              | ◆             | ◆             |
| `white-circle`           | ○           | ○              | ○             | ○             |
| `black-circle`           | ●           | ●              | ●             | ●             |

`solid-triangle` and `solid-pointer` share `▲` and `▼`, because Unicode names no up- or
down-pointing pointer; they differ on left and right. The four shapes that do not point carry one
glyph in all four columns, which is not a special case: a style is a table of four, and nothing says
the four differ.

**The case that fixes the key.** An endpoint at `(0, 0)` leaving `down` and one at `(1, 1)` leaving
`left`, both with `solid-pointer`. The `▲` belongs to `leaving: "down"` and the `►` to
`leaving: "left"`:

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
name the catalog does not hold produces, and `{ "kind": "style" }` is refused for the missing field.
Neither is observed: the pictures come from the glyphs each style resolves to, and _What proves it_
holds the resolution, the fallback and the refusal.

## What proves it

Nothing exists yet: the deciding stage adds no code. The names and tables bind the building stage.

| Rule | Test                                                                                     |
| ---- | ---------------------------------------------------------------------------------------- |
| 1    | `a_style_writes_the_glyph_its_table_holds_for_the_leaving_direction`                     |
| 2    | `a_style_covers_the_cell_and_composes_with_nothing`                                      |
| 3    | `the_eleven_styles_are_the_tables_this_spec_lists`                                       |
| 4    | `the_two_endpoints_of_a_connector_choose_their_styles_independently`                     |
| 5    | `an_unknown_style_name_draws_the_default`                                                |
| 6    | `a_style_terminal_without_a_name_is_refused_by_name`                                     |
| 7    | the characterization, and every picture in a tracked document, which the gate re-renders |
| 8    | the `wasm` step, which names the crate                                                   |

The two pictures are held by their markers: `cargo xtask render` rewrites them and the gate fails if
one is not what its description draws.

## Open questions

- **Whether a misspelled style should be reported once the format can say more than a refusal.** D6
  answers silence where there is no second channel, and adding one is a change to `parse` and every
  caller.
