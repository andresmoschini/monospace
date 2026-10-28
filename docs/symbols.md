# Symbols and styles

A cell can hold **a symbol and the style it is drawn in** instead of a character, and the catalog
answers the pair at render. See _A cell can name a symbol instead_ in [`model.md`](model.md) §3 for
why a cell has that third kind; this page is the vocabulary and the tables.

A **symbol** is one of seven, in this order:

| Symbol           | What it is                  |
| ---------------- | --------------------------- |
| `DirectionUp`    | An arrow pointing up        |
| `DirectionRight` | An arrow pointing right     |
| `DirectionDown`  | An arrow pointing down      |
| `DirectionLeft`  | An arrow pointing left      |
| `Diamond`        | A diamond, pointing nowhere |
| `Hexagon`        | A hexagon, pointing nowhere |
| `Circle`         | A circle, pointing nowhere  |

The first four are the directions a figure can leave in; the last three are shapes a head can be
drawn as. They are one closed vocabulary rather than a direction plus a shape, because a style draws
a look, and a diamond is as much a head as an arrow.

A **style** is the name of a look — `solid`, `outline`, `arrow`. It is open, as a stroke is: the
catalog decides which styles exist, so a style someone loads is a style like any other. A style is
not a table. It is the name a table's rules are filed under, and a table may hold rules of more than
one style, though none does today.

**A rule maps a pair to one glyph, and a pair the catalog lacks is a space.** There is no fallback
to another style: `solid-pointer` has no diamond, and asking it for one draws nothing rather than a
triangle nobody chose. That is different from arms, where a cell degrades to its base stroke —
[`glyph-sets.md`](glyph-sets.md) is that rule.

## The styles

`solid` is `monospace-core`'s, as `light` is. `outline`, `solid-pointer`, `small-solid`,
`small-outline` and `arrow` are `monospace-glyph-sets`'s, and `ascii` carries its five symbol rules
beside its arm table. `all()` unions every arm table and every style.

| Symbol           | solid | outline | solid-pointer | small-solid | small-outline | arrow | ascii |
| ---------------- | ----- | ------- | ------------- | ----------- | ------------- | ----- | ----- |
| `DirectionUp`    | ▲     | △       | ▲             | ▴           | ▵             | ↑     | ^     |
| `DirectionRight` | ▶     | ▷       | ►             | ▸           | ▹             | →     | >     |
| `DirectionDown`  | ▼     | ▽       | ▼             | ▾           | ▿             | ↓     | v     |
| `DirectionLeft`  | ◀     | ◁       | ◄             | ◂           | ◃             | ←     | <     |
| `Diamond`        | ◆     | ◇       |               |             |               |       |       |
| `Hexagon`        | ⬢     | ⬡       |               |             |               |       |       |
| `Circle`         | ●     | ○       |               |             |               |       | o     |

`solid` and `outline` are complete. The four pointer styles answer the four directions and nothing
else, because a pointer points; `ascii` adds the circle to the four arrow characters it draws, and
leaves the diamond and the hexagon to the space.

**A blank cell in that table is the space the picture draws.** The row of a style keeps its gap, so
an incomplete style reads as incomplete rather than as a shorter row.

## The sheet

`monospace-cli --symbols` prints one row per style the catalog reports, in name order, the name
padded to the longest and one space, then one cell per symbol in the order above:

```text
arrow         ↑→↓←
ascii         ^>v<  o
outline       △▷▽◁◇⬡○
small-outline ▵▹▿◃
small-solid   ▴▸▾◂
solid         ▲▶▼◀◆⬢●
solid-pointer ▲►▼◄
```

The trailing spaces the four pointer styles draw are trimmed from the fence above; a real run keeps
them, because the row is the rendering and not a summary of it. The sheet goes through `render` with
`all()`, so it cannot disagree with what a picture would draw, and a style added to a table appears
without a second list to change.

## Naming a cell

```rust
Cell::Symbol(SymbolCell {
    symbol: Symbol::DirectionUp,
    style: SymbolStyleName::from("solid"),
})
```

That value renders `▲` and is not equal to
`Cell::Literal(Glyph::new("▲").expect("▲ is one glyph"))`, which renders the same character. The
names are what an editing layer can rewrite; a literal has none.
