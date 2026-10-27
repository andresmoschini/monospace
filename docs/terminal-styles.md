# Terminal styles

A style is four glyphs, one for each way a head may point. An endpoint's terminal names one of the
eleven below, and the cell it writes is a **head**: the direction it points, away from the one the
endpoint leaves in, and the style it is written in. The glyph is the entry named for that direction
— the table's `down` holds `▲`, so the endpoint leaving down is the one that points up — and it is
chosen when the cells are rendered rather than when the figure is drawn. Naming each entry by the
way its glyph points rather than by the direction the connector leaves in is what keeps the table
readable beside Unicode, where `BLACK DOWN-POINTING TRIANGLE` is `▼`.

`solid-pointer` is carried as data by `monospace-core`, and the other ten by
`monospace-terminal-styles`. `solid-triangle` and `solid-pointer` share `▲` and `▼`, because Unicode
names no up- or down-pointing pointer; they differ on left and right. The four shapes that do not
point carry one glyph in all four columns, which is no special case: nothing says a style's four
entries differ.

The tables are a **catalog**, not a list a figure consults: `monospace-terminal-styles::all()` is
their union, and it is what a binary hands to the **rendering**, which resolves each head's style
name against it — the same place, and the same way, that a cell of arms is resolved against a glyph
set. The choice of which styles a build ships is made there, in the composition root, as the glyph
sets are — so a name a loaded catalog does not hold renders as `solid-pointer`, and a style added
here reaches every caller without a second list to change.

| Name                     | `up` | `right` | `down` | `left` |
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

A head covers the cell and composes with nothing: one standing on a box's border interrupts it,
exactly as a chosen glyph does. It follows no stroke and no glyph set either, which is what makes
one description draw the same heads whatever tables a build shipped, and it is what makes a third
kind of cell worth its price: a head carries a name and a direction, so the appearance is decided
where every other appearance is. The rule is _A cell can be a head instead_ and _What a terminal
writes_ in [`model.md`](model.md).
