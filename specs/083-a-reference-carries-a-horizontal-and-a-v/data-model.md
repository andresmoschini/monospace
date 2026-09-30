# Data model: a reference carries a horizontal and a vertical offset

What this slice adds to `monospace-diagram`, what it changes in `monospace-cli`, and the one entry
it changes in `assets/demo.json`. Everything features 079 through 082 built stays as it is unless it
is named here. Why the arithmetic is where it is and why the wire grows a union is in
[research.md](research.md), and the three answers it takes are on [decisions.md](decisions.md),
named beside each; the public surface alone is in
[contracts/diagram-api.md](contracts/diagram-api.md) and the file format in
[contracts/description-format.md](contracts/description-format.md).

The slice is smaller than its three parts suggest: one public field gains a value, one method gains
a step, one file format gains a union. What is not small is who has to be re-spelled when the union
lands, and that is where the design spends most of its care.

## `Reference` — a third field

| Field    | Type      | Meaning                                                  |
| -------- | --------- | -------------------------------------------------------- |
| `id`     | `ShapeId` | The identity of the shape the position hangs from        |
| `anchor` | `Anchor`  | Which of that shape's four sides it hangs from           |
| `offset` | `Delta`   | How far from that side, along each screen axis, in cells |

Three fields, and that is the whole of the type. §1's _Vocabulary_ row has always named three — "a
`ShapeId`, an `Anchor` on it, and a horizontal and vertical offset" — and §4 has always said a
reference "is resolved by asking the referenced shape for that anchor and adding the offsets". Both
sentences were there before the code was, and this slice is what makes them true.

`offset` is a [`Delta`](../../../crates/monospace-diagram/src/delta.rs) and not two `i32` fields
(D1). `Delta` is already a horizontal and a vertical signed amount, already derives `Copy`, and
`Delta::apply` is already the crate's only arithmetic (research.md Q1, `delta.rs:41`), so reusing it
leaves one place where coordinates are added rather than two. The cost is real and it is the cost
the model pays: `Delta`'s own row reads "how far a figure _moves_ along each axis", and after this
slice the type also means how far from a side. That row is one clause wider in
`docs/diagram-model.md`, in its own `docs` commit.

The field is public, as `id` and `anchor` are, and the type still derives
`Debug, Clone, PartialEq, Eq` and not `Copy` — `Delta` is `Copy` and `ShapeId` is a `String`, so the
struct inherits the weaker of the two. Nothing changes about how a reference is built: it is a
literal, as it was.

## `Position::resolve` — the three steps

```rust
pub fn resolve(&self, diagram: &Diagram) -> Option<Pos> {
    match self {
        Self::Absolute(at) => Some(*at),
        Self::Reference(reference) => {
            let shape = diagram.get(&reference.id)?;
            let point = shape.anchor(reference.anchor)?;
            Some(reference.offset.apply(point))
        }
    }
}
```

The order is §4's, and it is the whole of research.md Q1's measurement: ask the diagram for the
figure, ask that figure for the anchor, then add the offset to the point that came back. Two `?`s
and one addition, and every one of the three is a step the model already names.

Two properties fall out of that order rather than being written:

- **The offset is added to what the anchor answers now.** A box displaced five cells right answers
  its right side centre five cells right, and the offset is added to that — which is SC-002, and it
  is the difference between an offset as a gap from the side and an offset as a point.
- **A reference that resolves to nothing is still nothing.** Both `?`s come before the addition, so
  a large offset on a reference to an absent figure adds to nothing. There is no clamping, no
  fallback and no report, and the figure holding it is not drawn however large the offset is
  (SC-004, B2.2).

`Delta::apply` saturates rather than wrapping, and it is the same function the displacement uses. A
reference to `{0, 0}` on a right side with a horizontal offset of `i32::MIN` resolves to a point
past the end of any window a `u32` width can describe, which draws nothing — the same answer, and
for the same reason, that a displacement gives.

### `Position::displaced_by` — deliberately unchanged

The reference arm of `displaced_by` still comes back exactly as it went in, and the offsets do not
change that. §4 states the destination — "a displacement reaches its offsets" — and this slice does
not walk there, which is [#143](https://github.com/andresmoschini/monospace/issues/143)'s decision
and 082's D4's answer. The gap is now one field narrower than it was at 082, because there are
offsets to reach and the code still does not: B3 and SC-006 pin the no-op, and the contract lists it
under what is still not here.

The silent half is what makes this a real intermediate state rather than an oversight. A caller
displacing the figure a reference hangs from sees the connector's far end stand still while the near
end travels — which is what 082 pinned, and what the fifth picture of the shipped demonstration
shows.

## `monospace-cli` — four private mirror types

`description.rs` holds the file format, and every type in it is private to that conversion
([ADR-0035](../../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md)). The union
needs four types, none of them public, none of them in `monospace_diagram`:

| Type                | Shape                                | Becomes                                        |
| ------------------- | ------------------------------------ | ---------------------------------------------- |
| `At`                | `#[serde(tag = "kind", …)]`          | `monospace_diagram::Position`                  |
| `Pos`               | already there, unchanged             | `monospace_core::Pos`, and `At::Point`'s inner |
| `AnchorDescription` | `#[serde(rename_all = "lowercase")]` | `monospace_diagram::Anchor`                    |
| `OffsetDescription` | `{ dx: i32, dy: i32 }`, `Default`    | `monospace_diagram::Delta`                     |

`At` is the union D2 settled on, tagged exactly the way `Terminal` and `ShapeDescription` already
are in this file:

```rust
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum At {
    Point(Pos),
    Reference {
        shape: String,
        anchor: AnchorDescription,
        #[serde(default)]
        offset: OffsetDescription,
    },
}
```

`Point` is a **newtype** variant holding the file's own `Pos` rather than a struct variant with `x`
and `y` written out, and that was measured rather than assumed: against this crate's own `serde`,
both spellings read `{"kind": "point", "x": 1, "y": 1}` and refuse `{"x": 1, "y": 1}` with
`missing field \`kind\``and`{"kind": "arrows",
…}`with ``unknown variant`arrows`, expected`point`or`reference` ``, byte for byte between them. The newtype is taken because it keeps one `Pos`in the file rather than a second pair of coordinates to convert, and it carries a trap worth writing down: **an internally tagged newtype variant deserializes and does not serialize.** The file is read and never written, so nothing here is broken — but a caller that later wanted a`Serialize`
on this type would have to widen the variant, and this is where to find out why.

Two more measurements from the same probe, both of them the format's existing behavior rather than
anything this slice adds:

- `{"kind": "point", "x": 1, "y": 1, "shape": "#5"}` is **accepted**, and the `shape` is dropped in
  silence. A tag is the rule and a stray field beside it is still an unknown field, exactly as a
  description carrying `mode` is today (079's contract says so).
- `offset` is optional, and absent is zero.
  `{"kind": "reference", "shape": "#1", "anchor": "right"}` is a reference standing on the side
  itself, which is the case B4.3 and the gallery's own block both need to be able to say.

## The demonstration's tenth entry

```json
"to": {
  "at": { "kind": "reference", "shape": "#5", "anchor": "bottom",
          "offset": { "dx": 1, "dy": 1 } },
  "leaving": "down",
  "terminal": { "kind": "glyph", "glyph": "▲" }
}
```

`#5` is the fifth entry, a four-by-three box at `{20, 1}`. `side_centre` (`position.rs:141`) puts
its bottom centre at `{20 + 3 / 2, 1 + 2}` = `{21, 3}`, and `{21, 3} + (1, 1)` is `{22, 4}` — the
point that entry spells outright today (`demo.json:73`). So the file says the same place by the
other route, and **all five pictures come out byte for byte what they are** (B5.1, SC-005): the
demonstration removes `#1`, displaces `#1` and displaces `#3`, and the fifth shape is none of them.

The tenth entry's `from` stays a point — now a tagged one — and the fifth picture still rehangs it
in code, because that is the demonstration's own change to the picture and not the file's.

### The one test this moves

`demo_without_its_first_entry` (`main.rs:511`) builds the demonstration with its first entry left
out, **as text**, and reads it again. Reading it again issues the identities from scratch in array
order, so the nine remaining entries are numbered `#1` to `#9` and the same `"#5"` now names what
was the sixth entry: the box at `{18, 0}`, whose bottom centre is `{19, 2}` and whose `+ (1, 1)` is
`{20, 3}`. The fourth picture would then be a different picture from the one the test compares it
to. Measured, both ways, before this slice: the two renderings differ from row 3 down.

So the helper also renumbers the one reference it moves (`"#5"` to `"#4"`), and its doc comment says
why. That is not a workaround bolted on: it is the maintainer's D3 answered in the only place the
consequence is observable. A file names its shapes by the place they are listed, which is what makes
this slice's format work, and a test that removes a listing shifts every identity after it.

## What the tests pin

Five contract tests and no characterization, per the specification's _Testing expectations_. None of
them is a new test file; each lands beside the code it covers.

| Rule                                     | Where                     | How it is asked                                        |
| ---------------------------------------- | ------------------------- | ------------------------------------------------------ |
| The arithmetic, asked                    | `position.rs`             | `resolve` on a right side with `(2, 0)` is `{5, 1}`    |
| Zero is not a resolve that failed        | `position.rs`             | An `assert_ne!` beside it, as 082's does               |
| The same picture as the point            | `diagram.rs`              | Equal cells against the same connector from the point  |
| The offset travels with the side         | `diagram.rs`              | The box displaced, compared against the new point      |
| Non-resolution with a large offset       | `diagram.rs`              | The figure absent, every other shape's cells unchanged |
| A box replaced by a line under an offset | `diagram.rs`              | The line's own middle, not the box's old one           |
| A displacement still leaves it           | `diagram.rs`              | 082's test, unchanged, which is the claim              |
| The demonstration                        | `main.rs`, `tests/cli.rs` | Five pictures unchanged, the tenth entry a reference   |

The gallery's own label is a fourth thing this slice moves, and it does not move by itself:
research.md Q6 recorded that a `Reference` gaining a field changes the block's label, and the label
is **a string written by hand** (`gallery.rs:368`), which the gallery's own `WHAT` says twice over.
So the label is edited to `Reference(#1, Right, offset (0,0))` in the same commit that adds the
arithmetic, and the snapshot beside it moves for that one reason: a label cannot disagree with the
values it names, and there is now a third value.

## What this does not add

No ADR, no new crate, no dependency, and nothing at all in `monospace-core`. The three answers sit
inside [ADR-0040](../../../docs/decisions/0040-let-each-shape-answer-its-own-anchor-points.md) and
[ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md), whose _Decision
Outcome_ already says a position is absolute or a reference, that resolving one means asking the
referenced shape for its anchor, and that only a connector's endpoint may hold one; D1 adds a field
to a type the model's vocabulary already listed, D2's format is the one ADR-0035 keeps provisional,
and D3 is a refusal that names §11 as still open. The single row of `docs/diagram-model.md` this
slice amends is §1's, and it is a clause rather than a rule.
