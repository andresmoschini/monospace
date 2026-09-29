<!-- The maintainer's decision draft is a Spanish file outside this repository. -->
<!-- cspell:ignore decisiones -->

# Data model: a connector endpoint hangs from a box's side anchor

What this slice adds to `monospace-diagram`, and the one line it changes in `monospace-cli`.
Everything features 079, 080 and 081 built stays as it is unless it is named here. Why each type is
shaped this way is in [research.md](research.md) and the five answers it takes are on
[decisions.md](decisions.md), named beside each; the public surface alone is in
[contracts/diagram-api.md](contracts/diagram-api.md).

Three types arrive together, in one new module, and one of them is the first type in this crate to
hold a behavior. `Anchor` names a side, `Reference` names a shape and a side of it, and `Position`
is either a point or a `Reference` — and resolving one is what `Position::resolve` does, which is
why `Position` names a `Diagram` and is no longer a leaf. Only `Endpoint.at` becomes a `Position`,
which is what turns ADR-0041's restriction from a rule beside the code into a field's type (D2).

## `Anchor` — new, public

```rust
/// One of the four sides of a figure a position can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Top,
    Right,
    Bottom,
    Left,
}
```

Four variants and no fifth. §5 _Anchor points_ names the four, and the other five of issue #62's
nine are #90's, which waits for a consumer.

It is not `monospace_core::Direction` (geometry.rs:34), which names which way a figure leaves rather
than where it is. The two coincide for a side today — a figure hanging off a box's right side
usually leaves rightward — and one name would mean the wrong thing the day a corner arrives, since a
corner has two sides and no single direction out of it.

`Copy` is `Pos`'s derive and the enum holds nothing, so a caller can pass one without cloning it the
way it has to clone a `Reference`.

## `Reference` — new, public

| Field    | Type      | Meaning                                           |
| -------- | --------- | ------------------------------------------------- |
| `id`     | `ShapeId` | The identity of the shape the position hangs from |
| `anchor` | `Anchor`  | Which of that shape's four sides it hangs from    |

Two fields, and that is D1: §1's _Vocabulary_ row names three — an identity, an anchor and two
offsets — and the offsets arrive with #83, so this is the first two of the three the model
describes. What the two-field form makes unreachable today is a caller who wants the endpoint two
cells off a side: there is no way to say it, and the caller places both figures by hand.

Both fields are public, as `Delta`'s are. The type derives `Debug, Clone, PartialEq` and `Eq` and
not `Copy`, because `ShapeId` is a `String` and only `Clone` (diagram.rs:13) — nothing here needs it
to be `Copy`, and `Position` inherits the same.

## `Position` — new, public

```rust
/// Where something stands: a point, or a reference to a side of a shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Position {
    Absolute(Pos),
    Reference(Reference),
}
```

`From<Pos> for Position` is the second way in, for the caller holding a point. It is here for the
same reason `From<(i32, i32)> for Delta` is: the field this type lands on is filled from a `Pos` at
every site that builds an endpoint today, and without the conversion the twelve of them in this
crate would each name the variant. The conversion is a second way in and does nothing else.

It does not reach the command-line application, which is worth saying rather than leaving to be
found: `description.rs` holds a `Pos` of its own and only implements `From` **into** the core's, so
there is no `into()` there to keep working — see that section.

`Copy` is not derived, for the reason `Reference` gives. Two variants and no third, because §4 says
a position is either absolute or a reference and names nothing else.

## `Shape` — changed

### `Endpoint.at` — its type changes

`Pos` becomes `Position`, and the two other fields are untouched. The terminal still occupies
whatever position the endpoint stands at, and for an absolute one that is the same cell it did.

This is the only field in the crate that changes type, and it is the whole of D2. `Box.at` and
`Line.at` stay `Pos`, so the restriction is the type system: a `Position` on `Box.at` would be a
field that can only ever hold `Absolute`, and #89 is what widens it and brings the cycle obligation
ADR-0041 attaches to widening with it.

### `Shape::anchor(&self, anchor: Anchor) -> Option<Pos>` — new, crate-private

The one search of a figure's four side centers, one arm per variant, and the only consumer is
`Position::resolve`. It stays crate-private for the reason research.md Q3 gives: the anchors exist
to be resolved _through_, drawing is the only consumer so far, and a caller holding a `Position` can
already reach the number through `resolve`. Nothing outside the crate asks a shape for an anchor,
and the specification's **What this slice does not decide** hands the direct query to the first
consumer that needs an anchor without drawing.

| Variant     | What it answers                                 |
| ----------- | ----------------------------------------------- |
| `Box`       | all four, from its own `at` and `size`          |
| `Line`      | all four, as a flat box of one cell's thickness |
| `Connector` | nothing, for all four                           |

`None` is the ordinary answer and not a failure (§5): a connector answers no anchor point at all, so
a reference to one resolves to nothing, and that is what keeps a chain one link long and makes a
cycle impossible to build.

#### The arithmetic

Two crate-private functions in `position.rs`, beside `Anchor`, because `Anchor` is the only thing
that names a side and one of them is the only reason the other exists:

- `side_center(at: Pos, size: Size, anchor: Anchor) -> Pos` — the middle of one side of a figure
  that occupies `at` through `at + (size.width - 1, size.height - 1)`.
- `flat_size(len: u32, orientation: Orientation) -> Size` — a line's extent as a box: `(len, 1)`
  horizontal, `(1, len)` vertical.

| `anchor` | The point                                     |
| -------- | --------------------------------------------- |
| `Top`    | `at.x + (width - 1) / 2`, `at.y`              |
| `Right`  | `at.x + width - 1`, `at.y + (height - 1) / 2` |
| `Bottom` | `at.x + (width - 1) / 2`, `at.y + height - 1` |
| `Left`   | `at.x`, `at.y + (height - 1) / 2`             |

Two rules rather than four, and the table is those two read in four directions. The division is a
floor, so a box four cells wide has its top and bottom centers on cells 1 and 2 rather than on a
half-cell that does not exist; a horizontal line of any length has one cell for a row, so its top
and bottom centers are its middle and are the same point asked for twice (B1.2).

`width - 1` is `saturating_sub` and each addition is `saturating_add`, which is what makes a box one
cell wide or one cell tall fall out of the general rule rather than an exception: at width 1 the
left and right centers are the same cell and the top and bottom are its two ends. It costs nothing
to be right at width 0, where the figure draws nothing at all and both answers are its own `at`.

The whole table is checkable by hand against one number the specification's own B2.1 depends on: the
shipped demonstration's third entry is a box of four by three at `{9, 2}`
(`crates/monospace-cli/assets/demo.json:18-24`), so its right side center is `{12, 3}` — which is
exactly the point the connector's `from` already holds (line 68). That is why the fifth picture
replaces a connector that already stands where its reference would resolve to.

### `Shape::draw` — its signature changes

```rust
pub(crate) fn draw(&self, surface: &mut impl Surface, diagram: &Diagram)
```

It takes the `&Diagram` it needs rather than an answer handed to it, and the `Box` and `Line` arms
ignore the argument — ADR-0041's restriction stated as a signature rather than enforced at run time.
`Diagram::draw` is already the only door, it already takes `&self`, and it already holds the search
(`get`, diagram.rs:109), so an argument reaches what is needed without inventing a second way in.

The `Connector` arm resolves both endpoints before it writes anything:

```rust
Self::Connector { from, to, stroke } => {
    let (Some(from_at), Some(to_at)) = (from.at.resolve(diagram), to.at.resolve(diagram)) else {
        return;
    };
    Connector {
        from: monospace_core::Endpoint {
            at: from_at,
            leaving: from.leaving,
            terminal: from.terminal.clone(),
        },
        to: monospace_core::Endpoint {
            at: to_at,
            leaving: to.leaving,
            terminal: to.terminal.clone(),
        },
        stroke: stroke.clone(),
    }
    .draw(surface)
}
```

One that does not resolve takes the **whole connector** out, and the two are asked together so that
one unresolvable end is never drawn as an arm with a missing head. That rule is the connector's
rather than the position's: a connector composes two positions and belongs to neither, and §4's rule
— a shape whose position does not resolve is not drawn — reaches an observable case for the first
time (B2.4).

### `From<Endpoint> for monospace_core::Endpoint` — removed

It cannot survive, and this is measured rather than predicted: the core's `Endpoint.at` is a `Pos`
(connector.rs:106-108) and a `Position` has no `Pos` to give it without a diagram. The conversion
becomes the four lines above, built where it is used, which is how every other arm already builds
its core shape. The one `match` over `Shape` stays in `shape.rs` and the `kind_of` match in
`gallery.rs` stays beside it, so the closed set is spelled in the two places that must name it.

The test that pinned the conversion goes with it, and its claim comes back in the form the code now
has: two connectors, one with a glyph terminal and one with an arm, each drawn through a diagram,
produce exactly the buffer the same core `Connector` drawn directly produces. That is a stronger pin
than the conversion was, because it goes through the four lines rather than naming them.

### `displaced_by` — a reference does not move

The `Connector` arm's two endpoints stop being `by.apply(from.at)` and become
`from.at.displaced_by(by)`, and the method it calls is new and crate-private:

```rust
impl Position {
    pub(crate) fn displaced_by(&self, by: Delta) -> Self {
        match self {
            Self::Absolute(at) => Self::Absolute(by.apply(*at)),
            reference @ Self::Reference(_) => reference.clone(),
        }
    }
}
```

The second arm is the whole of B4, and it is a silent no-op: a displacement adds coordinates to an
absolute position and a reference has none to add to until #83. So a connector with one endpoint
absolute and one hanging is displaced through its absolute end alone, and the picture shows one end
moved and one standing still (B4.2). §4 says a displacement reaches a reference's offsets, and there
are none; D4 is the maintainer's answer that §4 is not amended to say so in the meantime, and
[#143](https://github.com/andresmoschini/monospace/issues/143) carries the real rule.

`Box` and `Line` are unchanged, because their `at` is still a `Pos` and `by.apply` is still the only
arithmetic.

## `Position::resolve` — new, public

```rust
/// Where this position stands now, or `None` when it does not resolve.
#[must_use]
pub fn resolve(&self, diagram: &Diagram) -> Option<Pos>
```

Three steps, and the third is the whole decision:

1. `Absolute(at)` is `Some(*at)`. A diagram holding nothing still answers it, which is what makes an
   endpoint that never moved survive a diagram it was taken out of and put back into.
2. `Reference(reference)` asks `diagram.get(&reference.id)` for the figure, and a figure the diagram
   does not hold is `None` — the same answer `get` gives, reached through the same search (B2.4,
   B3.1).
3. Then it asks that figure for the anchor, and an anchor the kind does not answer is `None` again.
   Both are the one answer §4 gives for two cases, and neither is an error, a report or a panic
   (B2.4).

It is public and `#[must_use]` (D5). Public, because a caller that knows a `Position` and its
`Diagram` can then ask where it stands rather than redraw the picture to find out — which is the
consumer §11's anchor question is waiting for, one level up. `#[must_use]`, because it answers and
changes nothing, the same reason `get` and `displaced_by` carry it.

**There is no offset arithmetic here, and that is D1 rather than an omission.** research.md Q1
describes resolution as looking the identity up, asking for the anchor and adding the offsets; the
first two are all there is to do until #83, and an addition of two zeros is an entry the removal
test takes out. The rustdoc says the third field arrives with the offsets, so the absence reads as a
step not yet taken rather than as a rule.

## The `monospace-cli` side

### `description.rs` — one line, in the first commit

The wire format does not change and no field is added to it: `at` is still a `{"x", "y"}` on the
wire, and the conversion wraps it.

```rust
at: Position::Absolute(endpoint.at.into()),
```

The `into()` is there today and stays, but it is doing less work than it looks. An `into()` names
its target by inference and its source by the value given, so today it resolves
`description::Pos → monospace_core::Pos` and tomorrow the same call has to resolve
`description::Pos → Position` — and there is no such `From`, because the two-step conversion Rust
does not do. The explicit variant is what makes it compile, and it is why this slice's second way in
does not reach this file.

**This line lands in the structural commit, not with the feature.** It is the only change outside
`monospace-diagram` that the field's new type forces, and a workspace that does not build is not a
commit the constitution accepts (principle II) — so it is not a question of which commit prefers it.

What it costs is that a reference is in no description format, which is why the fifth picture below
is built in the demonstration's own code and why `cargo xtask render` cannot fill it either way
(research.md Q4, and the specification's Clarifications for 2026-09-28).

`assets/demo.json` is unchanged, and so is every field of it: the displacement's amount and the
identity the arrow hangs from are the demonstration's own constants, beside the fixed delta picture
three already carries.

### `demonstrate` — a fifth picture

One diagram, mutated in place between the pictures, as pictures three and four already are. Two
identities are written out by hand beside the one for `#1`, and the reason is the specification's
Clarifications rather than a preference: a description names its shapes by position, so the
demonstration already knows which entry it means and has nothing to read back.

```text
window(), into_diagram()                    -> origin, size, diagram
caption 1, draw, render                             as written
caption 2, forward(#1), draw, render                the order changed
caption 3, get(#1) -> displaced_by(by),
          replace(#1, moved), draw, render          the position changed
caption 4, remove(#1), draw, render                 the figure is gone
caption 5, get(#10) -> rebuilt with `from` hanging
          from #3's right side, replace(#10, ..),
          get(#3) -> displaced_by(Delta { dx: 4, dy: 0 }),
          replace(#3, moved), draw, render          the arrow followed
```

`#3` is the box the arrow already hangs from, measured above: its right side center is the point the
connector's `from` already holds. `#10` is the connector, the tenth entry of `assets/demo.json`, and
naming it is the same trade naming `#3` is — a figure added before either would make the written
value name the wrong shape, and the pinned fifth picture is what catches it.

The rebuilt connector **reads the old one back** rather than restating it:

```rust
if let Some(arrow) = diagram.get(&the_arrow).cloned()
    && let Shape::Connector { from: _, to, stroke } = arrow
{
    diagram.replace(
        &the_arrow,
        Shape::Connector { from: hanging, to, stroke },
    );
}
```

`to`, its direction, its terminal and the stroke are then the description's, by construction, and
the only thing the fifth picture changes about the arrow is where it starts. That is what holds the
slice's claim rather than asserting it: a restated connector would be four values the demonstration
agreed with the description about, and a disagreement would change the picture instead of failing.

Two things in the rebuilt endpoint are the caller's rather than the anchor's, and staying that way
is a decision this slice does not take: `leaving: Direction::Right` and `terminal: Terminal::Arm`
are written out beside `Anchor::Right` and are not derived from it. §6 says the direction and the
terminal are the caller's, and deriving the direction from the side is the open question §11 keeps
([#89](https://github.com/andresmoschini/monospace/issues/89)).

The amount is `Delta { dx: 4, dy: 0 }`, the specification's own B2.2 four cells to the right, beside
the fixed delta picture three carries. It is worth saying that the destination was measured rather
than chosen by eye: the box lands at `{13, 2}` occupying `x 13..16, y 2..4`, and reading the four
rows of the shipped demonstration, the only cells in that rectangle anything else writes are the
connector's own arm at `(13, 3)` through `(15, 3)` — the arm the fifth picture replaces. So the box
lands on cells the picture's own change has cleared.

Five pictures, and the fifth is the only one about a figure the first four do not touch. The caption
is one line, because the test finds each picture by splitting the output on the blank line before it
and the newline after the caption.

### The tests that read it

`demonstrated_pictures` in `main.rs` splits three times and returns a five-tuple. Every test using
it is a mechanical update, and two of them keep their meaning unchanged: they compare the first
pictures, and the new one is appended below rather than inserted, so the coordinates they read do
not move.

- `two_overlapping_boxes_demonstrate_in_opposite_orders` and
  `a_bare_run_prints_four_captioned_pictures_...` take the first picture or the first two, and the
  count in the second's name goes to five.
- `the_third_picture_moves_one_figure_and_the_fourth_takes_that_figure_out` binds four; it takes
  five and keeps comparing what it is about.
- `an_empty_description_demonstrates_as_four_identical_pictures` and
  `one_shape_demonstrates_as_two_copies_of_itself_and_then_an_empty_window` compare pictures
  pairwise and take the fifth into account: both are all-identical claims, and both now include the
  change the fifth makes. An empty description still demonstrates five identical pictures and fails
  nothing, because every call is a no-op on an identity the diagram does not hold.
- The new test pins the fifth against the fourth: the cells that differ are `#3`'s old cells, `#3`'s
  new cells, and the connector's route, and nothing else. It is the same shape of claim picture
  three's own test makes, which is what makes a box that moved a claim rather than a caption.

`first_demonstrated_picture` in `tests/cli.rs` needs no change at all, for the reason 081 recorded
and the reason still holds: it takes the **first** block. What does change is the comment above it,
which names a count of four.

## The gallery — a fourth block

A `Diagram` holds its shapes in a private `Vec<Placed>` and offers one query by an identity, so
`gallery.rs` is the only carrier in the crate that can reach a reference at all (research.md Q4, and
[ADR-0064](../../../docs/decisions/0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md)
for why a marker cannot). The fourth block is a small box with no fill, a connector whose `from` is
a reference to that box's right side, and the box displaced four cells right — two blocks in one
snapshot, the before and the after, which is the shape `the_order_decides_a_shared_cell` already
has.

The first block is [§6 _Attachment_](../../../docs/diagram-model.md#6-attachment)'s picture, reached
two ways: the model shows it with `from` spelled as the point, and the gallery draws the same
picture with `from` a reference. B2.1 is the claim that the two are the same, and having the model
show the first and the snapshot hold the second is what lets a reader see it rather than read it.

Two things change in `gallery.rs` before the block is added, and both are one word:

- `block()`'s second line is named `change:` rather than `order:`, and `WHAT` says so beside it. The
  line is what was done between two drawings of one diagram, which 080 could only make a change to
  the order of; a displacement is a change to a figure. Leaving the name would put a shape's
  displacement in a column named for the order.
- That rename moves the three committed snapshots, because `WHAT` is their description and the label
  is in their text, so `cargo insta review` follows. It lands in the `test` commit that declares the
  module rather than in the `feat` that adds the block, which is the only place it can: the block
  cannot be added before the anchors exist, and a `refactor` may not carry a change to a snapshot.

`WHAT` is a `concat!` of seven string fragments, and the learning log's first entry in this slice's
predecessor records what a hand re-wrap of it costs: a fragment loses a word, the code compiles, the
snapshots match and only `git diff --word-diff` on the rendered sentence shows it. That is the check
to run on the rename.

## What this slice does not decide

Four rules of the model that this slice's code cannot reach, named here because the specification
names them and because a reader of this file will look for them:

- A shape whose own position does not resolve offers no anchor point, so a reference to it resolves
  to nothing in turn. No position but an endpoint's can hold a reference yet — that is
  [#89](https://github.com/andresmoschini/monospace/issues/89) — so the rule is unreachable from a
  test today. It is in the model because that document is design intent, and it can be asserted the
  day a position may hold one.
- The offsets, and everything they make expressible: #83.
- A caller naming an identity. `ShapeId::new` already spells one and this slice's demonstration
  spells a second, which is a one-sentence answer in §3 (D3) rather than a new verb on the diagram.
- The corners and the center, which are #90's and which a `Direction` rather than a `Side` would
  have to name.
