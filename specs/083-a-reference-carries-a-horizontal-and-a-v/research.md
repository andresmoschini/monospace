# Phase 0 research: a reference carries a horizontal and a vertical offset

Seven questions. Four are module-level and answered here; three are the maintainer's and are on
[decisions.md](decisions.md), not repeated. Each module-level answer lives inside one crate and is
undone by changing the code that gives it, so none takes an ADR of its own.

## Q1: What carries the two amounts, and where is the arithmetic?

**Decision**: `Reference` gains the offsets and `Position::resolve` adds them — ask the diagram for
the shape, ask that shape for the anchor, add to the point it returns. What type carries them is D1;
this holds either way. On the wire an offset may be omitted, and absent is zero.

**Rationale**: measured. `Position::resolve` (`position.rs:101`) is the only site that asks a
reference anything, `Shape::anchor` (`shape.rs:90`) is `pub(crate)` and takes an `Anchor` alone, and
§4 says a reference "is resolved by asking the referenced shape for that anchor and adding the
offsets", which puts the two steps in that order and in that place. `Delta::apply` (`delta.rs:41`)
is already the crate's only arithmetic and already saturates.

**Alternatives considered**: the offset on `Endpoint`, which §1's row does not put it on; the
arithmetic inside `Shape::anchor`, which makes a side middle depend on who asked; and requiring the
offset on the wire, which makes every reference spell two zeros to say nothing.

## Q2: What does an endpoint's position become on the wire?

**Decision**: D2. All three spellings were measured in a probe against this crate's own `serde`.

**Rationale**: measured. An internally tagged `at` reports `unknown variant \`arrows\`, expected
\`point\` or \`reference\``and`invalid type: string "left", expected
i32`— and refuses every description written without a tag, which is every description in the repository: 13 markers carry a real description and 7 contain a connector, beside`assets/demo.json`and the test literals. An untagged union reads all of them, reports`data
did not match any variant of untagged enum At`for anything wrong, and reads`{"x": 1, "y": 1,
"shape": 4}`as a point, dropping the shape in silence. A sibling`reference`beside an optional`at`
refuses nothing: both present, or neither, deserializes cleanly, so "exactly one" would be a rule in
prose where the type could have held it.

**Alternatives considered**: externally tagged, rejected by 082's Q3 for a reason that still holds.

## Q3: Does a file name a shape by the identity the diagram issues?

**Decision**: D3, and §11 is what makes it the maintainer's.

**Rationale**: the format names its shapes by the place they are listed and has no name to write
down, so a reference must name one. `Diagram::add` issues `#1`, `#2`, … in array order
(`diagram.rs:56`), so `"shape": "#5"` asserts what the reader will do and nothing more — the
identity survives `forward` and `backward` because §3 says it does, and does not survive a
reordering of the file, because that is where it was issued. A file choosing its own names answers
§11, whose stated trigger is "the first slice where a caller has a name worth keeping — reading a
diagram from a file is the obvious one", and 082's D3 declined it early by saying exactly that.

**Alternatives considered**: naming by array index, the same information in a spelling that admits
it is positional.

## Q4: What two questions 082 already answered are not raised again?

**Decision**: neither is raised. A displacement still does not reach the offsets, and
`Shape::anchor` still does not become public.

**Rationale**: 082's D4 answered the first — "leave §4 as written; the model states the destination
and #143 is the slice that reaches it" — and B3 and SC-006 pin the no-op. The answer holds with more
to leave alone rather than less: the offsets exist after this slice and the displacement still does
not reach them, which is the intermediate state that answer was written to allow. 082's Q3 answered
the second, and Q1 leaves nothing new reaching past `Position::resolve`.

**Alternatives considered**: adding the displacement to the offsets now, which is #143's entry.

## Q5: What does the shipped description carry, and why do the pictures not move?

**Decision**: `assets/demo.json`'s tenth entry spells its `to` as a reference to the fifth shape's
bottom with an offset of one in each axis, and nothing else in the file moves.

**Rationale**: measured. The fifth entry is a four-by-three box at `{20, 1}` (`demo.json:33`), so
`side_centre` (`position.rs:141`) puts its bottom centre at `{21, 3}`, and `{21, 3} + (1, 1)` is
`{22, 4}` — the point that entry's connector already spells (`demo.json:73`). The demonstration
removes `#1`, displaces `#1` and displaces `#3`, and the fifth shape is none of them, so all five
pictures come out byte for byte what they are today.

**Alternatives considered**: building the reference in code as the demonstration already builds the
fifth picture's `from` (`main.rs:172`), which leaves the shipped file unable to say anything the
code can and makes the format's new power untested by the demonstration.

## Q6: What does the gallery say, and what will move?

**Decision**: correct its claim that it is the only carrier able to reach a reference, in the commit
that makes it wrong, and add no block for the offset.

**Rationale**: measured. `gallery.rs:374` grounds the claim on "a `<!-- render: -->` marker reads a
description, and the wire format holds a point" — both halves false once `at` can hold a reference.
082's Q7 set the rule: a rustdoc is code, so it is corrected where it becomes false. The block's
label comes from the shape's own `Debug`, so a `Reference` that gains a field prints it and the
snapshot moves; what it moves to is observed when the code exists, not written here. §6's picture
keeps spelling the point `{3, 1}`: it is the model's, and the specification amends none of it.

**Alternatives considered**: a third block for the offset, which B1.2 already draws; handing the
block to a marker now that one can reach it, which loses the two-block before-and-after ADR-0064
gives the gallery and keeps nothing the marker does better.

## Q7: What the model does not need to say, and one word a contract says wrong

**Decision**: §1's `Delta` row is widened by one clause, in the building stage, because D1 reuses
`Delta` for the offset; no other section of `docs/diagram-model.md` is amended; and one word of the
description contract is corrected while this slice amends it anyway.

**Rationale**: §1's `Reference` row already names three fields and §4 already says a reference
resolves by asking for the anchor and adding the offsets, so the slice is that sentence becoming
true. `Delta`'s own row reads "how far a figure moves along each axis: a horizontal and a vertical
amount", and D1 gives the type a second use, so the first clause no longer holds alone — its second
does, and the row is one clause wider. The amendment follows the answer rather than preceding it,
which is where 082's D3's one-sentence change to §3 landed (`505fd0d`, on the building branch). What
a caller may ask is 082's D5; whether an unresolved reference is reported is ADR-0041's option B;
that only an endpoint may hold a reference is 082's D2. Separately, `specs/079`'s contract spells
`"kind": "arrow"` and the wire tag became `"connector"` in `c7539bc` under ADR-0065 — measured, a
description carrying `arrow` is refused by name. 082's Q7 called a contract that contradicts the
code worse than none, and this slice amends that same file.

**Alternatives considered**: amending §4's displacement paragraph, which 082's D4 declined.
