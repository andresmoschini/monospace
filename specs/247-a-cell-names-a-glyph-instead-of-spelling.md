---
status: draft
date: 2026-10-27
---

# A cell names a glyph instead of spelling it

## Why now

A cell holds four arms or one glyph. The arms are resolved when the cell renders, through the
catalog, so a rule can be replaced and every cell drawn from it changes. The glyph is spelled by the
caller and never looked up: `▲` is whatever the caller typed, and no table in reach can say
otherwise.

Anything that points needs the second answer and cannot have it. A style is a table of four glyphs,
one per direction, and a terminal that uses one should write "the solid pointer, pointing up" rather
than a character the caller chose — so that the catalog decides what a build draws, as it does for
every stroke, and so that a name survives to whatever reads the diagram later. This spec adds the
kind of cell that carries such a name; [#57](https://github.com/andresmoschini/monospace/issues/57)
is the terminal that writes one.

## Scope

### In

- `monospace-core`: `GlyphName`, `NamedKey` and `Cell::Named`; a second map in `GlyphCatalog` beside
  the arms one, and `from_named_rules`/`named` to build and read it; the one fallback that map
  needs; `Style` and the `solid-pointer` table, so the fallback has something to answer with.
- `docs/model.md` §3, §4, §5 and §6: the third kind of cell, the second kind of rule, how each
  composes, and the branch the renderer takes for one.
- Tests for the fallback, the two maps, the composition and the equality a named cell keeps.

### Out

- **Any figure that writes a named cell.** The terminal that does is
  [#57](https://github.com/andresmoschini/monospace/issues/57)'s, with the wire format and the other
  ten tables. This ships the kind, and one table to draw it with.
- **A name that is not a direction.** `GlyphName` holds the four the arrow styles use. A name for a
  shape that does not point — a cap, a crossing — is a later addition, and the enum is where it
  lands.
- **A named rule loaded from a file.** [#63](https://github.com/andresmoschini/monospace/issues/63)
  is the same question for glyph sets, and how a loaded rule would spell its two names is that
  slice's question.
- **Reporting a misspelled style.** D3 answers silence, and _Open questions_ says why.

## The decision

**D1 — a cell can hold a name, and the catalog answers it at render.**

**Answer:** `Cell::Named(NamedKey)` is the third kind, beside arms and a literal. It renders by
looking its key up in the catalog, so the character is decided at render and a catalog handed to the
renderer is the one that decides. **Why not** resolving the name where it is written, into a
literal: the name dies there, and a caller holding a catalog could not change what the figure draws;
a figure that wrote `▲` and one that wrote "the solid pointer, up" would be the same cell. **Why
not** a kind that carries both the name and the glyph it resolved to: one answer would have two
sources, and a table that changed afterwards would leave the two disagreeing with nothing to say
which wins. **What it costs:** every exhaustive match over cells grows, and a named cell is never
equal to the literal that draws the same character. **Answered by** the maintainer, who asked for
the resolution to follow the glyph sets.

**D2 — the key is a style name and a closed glyph name.**

**Answer:** `NamedKey { style: String, name: GlyphName }`, where `GlyphName` is a closed enum in the
core — `Up`, `Right`, `Down`, `Left` — and `impl From<Direction> for GlyphName` is the one-way
bridge. The style is open data because a catalog decides which styles exist; the name is closed
because the core decides what a style may say. **Why not** `pointing: Direction` in the key: a
direction is one family of name, and the next one — a cap, a crossing — is not a direction, so the
key would change the day the vocabulary widens. **Why not** `name: String`: a misspelled name and a
style that does not hold it would fall back alike, so neither could ever be reported. **What it
costs:** a `From` impl at the call site, and a core enum that grows with each name added. **Answered
by** practice, as this was written.

**D3 — the catalog holds two maps, and the one fallback is the cell's.**

**Answer:** `GlyphCatalog` keeps `arms` — the `GlyphKey` map, renamed from `rules` — and gains
`named`, a `NamedKey` map; `union` merges both, first claim wins. `Cell::glyph_str` tries the exact
named key, then the same name under `DEFAULT_STYLE_NAME` (`"solid-pointer"`), then answers nothing.
**Why not** a `default_named` map or a `default_style` field in the catalog: the catalog stays pure
data, and a default table inside it would answer keys nobody wrote, which is the thing the arm
fallback is careful not to be. **Why not** one map over one key enum: the two are different kinds of
rule with different fallbacks — four arms degrade to the base stroke, a name degrades to another
name — and folding them would churn every `GlyphKey` use and make §5's "a rule maps four arms"
untrue. **What it costs:** a catalog without the default table has no fallback and draws a space for
every style it does not hold. **Answered by** practice, as this was written.

**D4 — a named cell composes as a literal does, and stays a different value.**

**Answer:** in the buffer it is opaque, exactly as a literal is: on top it replaces what it meets,
and under a stamp of arms it is replaced whole with that stamp's `Unset` sides closed. Equality is
by key, so a named cell and the literal drawing the same character are two cells. **Why not**
normalizing one into the other on the first stamp: the name is the only part an editing layer could
rewrite, and the first figure to land would throw it away. **Why not** letting two named cells merge
key by key: they have no arms to merge, and the top one is the answer for the same reason it is for
literals. **What it costs:** two diagrams that look alike are not equal, and a buffer says which
figures named their glyphs and which spelled them. **Answered by** practice, as this was written.

## Model slice

- `docs/model.md` §3 _The cell_: a third kind beside arms and a literal. The opening "and not a
  character" gains the clause that a named cell is not a character either — it is a name that
  becomes one at render.
- §4 _Stamping_: the table gains named cells wherever a literal sits, because the two compose alike.
- §5 _Strokes, glyph sets and the catalog_: a second kind of rule. A named rule maps a style name
  and a glyph name to one glyph, is looked up by name rather than by arms, and does not degrade to a
  stroke. The catalog holds both maps, and `union`'s first-claim rule covers both.
- §6 _Rendering_: the numbered list gains its third branch — a named cell is one lookup, then the
  same lookup under the default style name, then nothing.
- §9 _Properties worth testing_: the arm fallback's property gains a sibling — a catalog holding the
  default table answers every name under it.

## Public surface

```rust
// monospace-core
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlyphName { Up, Right, Down, Left } // one per direction, today

impl From<Direction> for GlyphName { fn from(direction: Direction) -> Self; }

/// A style name and the name of the glyph that style draws there.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedKey { pub style: String, pub name: GlyphName }

pub enum Cell { Strokes(StrokeCell), Literal(Glyph), Named(NamedKey) }

/// The style a cell falls back to when the catalog does not hold the one it asks for.
pub const DEFAULT_STYLE_NAME: &str = "solid-pointer";

impl GlyphCatalog {
    // `from_rules`, `union`, `light` and `glyph` as they are; `union` merges both maps
    pub fn from_named_rules(rules: impl IntoIterator<Item = (NamedKey, Glyph)>) -> Self;
    pub fn named(&self, key: &NamedKey) -> Option<&Glyph>;
    pub fn solid_pointer() -> Self;
}

/// Four glyphs, each named for the direction it points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style { pub up: Glyph, pub right: Glyph, pub down: Glyph, pub left: Glyph }

impl Style {
    /// The four named rules this table makes, all under `name`.
    pub fn catalog(&self, name: &str) -> GlyphCatalog;
}
```

`Style` is the shape a table is written in and nothing more: four glyphs, turned into rules under a
name. It has no method that answers by a direction the connector leaves in — that inversion belongs
to the figure, and [#57](https://github.com/andresmoschini/monospace/issues/57) is the figure that
needs one.

## Behavior

1. A named cell renders as the exact key the catalog holds; failing that, as the same name under
   `solid-pointer`; failing that, as nothing, which is a space.
2. A named cell covers the cell and composes with nothing: it writes no arm and follows no stroke.
3. A named cell composes as a literal does — on top it replaces what it meets, and under a stamp of
   arms it is replaced whole with that stamp's `Unset` sides closed.
4. A named cell and the literal drawing the same character are different cells: equal keys are equal
   cells, and nothing compares what two cells render to.
5. `union` merges the arm map and the named map by the same first-claim rule, and the two never
   answer for each other.
6. `solid-pointer` is `up: ▲, right: ►, down: ▼, left: ◄`, and `Style::catalog("solid-pointer")`
   makes exactly those four rules.
7. A catalog without the default table draws a space for every name it does not hold, and a catalog
   holding it answers a style it does not know with that table.
8. A cell of arms, a literal, and every picture drawn from them render as they did before.

## Examples

**The fallback, as four catalogs.** The cell asks for `solid-pointer`/`up` unless the row says
otherwise:

| the catalog holds                        | the cell asks for    | renders |
| ---------------------------------------- | -------------------- | ------- |
| `solid-pointer`/`up` and nothing else    | `solid-pointer`/`up` | `▲`     |
| `outline-triangle`/`up` and nothing else | `solid-pointer`/`up` | space   |
| `solid-pointer`/`up` and nothing else    | `nonsense`/`up`      | `▲`     |
| no named rule at all                     | `solid-pointer`/`up` | space   |

The second row is D3's cost: the default is a _name_, so a catalog shipping no table under it has no
fallback to give. The third is rule 1's second lookup, where the style asked for is not the one
held.

**A name is not the character.** These two values draw the same `▲` and are not equal, which is what
rule 4 keeps apart in a buffer:

```rust
Cell::Named(NamedKey { style: "solid-pointer".to_owned(), name: GlyphName::Up })
Cell::Literal(Glyph::new("▲").expect("▲ is one glyph"))
```

**Composing, both orders.** A named cell stamped above a stroke cell replaces it; stamped below one,
the stroke cell takes the position and closes the sides it left `Unset`. The same two stamps with a
literal in place of the named cell produce the same buffer, except that the cells are not equal.

No picture carries a named cell yet, because no figure writes one until
[#57](https://github.com/andresmoschini/monospace/issues/57) lands: none of this is observed.

## What proves it

Nothing exists yet: the deciding stage adds no code.

| Rule | Test                                                                                     |
| ---- | ---------------------------------------------------------------------------------------- |
| 1    | `a_named_cell_tries_the_exact_key_then_the_default_style_then_nothing`                   |
| 2    | `a_named_cell_covers_the_cell_and_composes_with_nothing`                                 |
| 3    | `a_named_cell_composes_as_a_literal_does`                                                |
| 4    | `a_named_cell_is_not_the_literal_that_draws_it`                                          |
| 5    | `union_merges_both_maps_and_the_two_never_answer_for_each_other`                         |
| 6    | `solid_pointer_is_the_table_the_spec_lists`                                              |
| 7    | `a_catalog_without_the_default_table_answers_no_name_it_does_not_hold`                   |
| 8    | the characterization, and every picture in a tracked document, which the gate re-renders |

## Open questions

- **Whether a misspelled style should be reported once the format can say more than a refusal.** D3
  answers silence where there is no second channel, and adding one is a change to `parse` and every
  caller.
- **Whether a named cell and the literal drawing the same character should ever merge.** Rule 4
  keeps them apart because the name is what a later editing layer would want; normalizing one into
  the other would lose it.
