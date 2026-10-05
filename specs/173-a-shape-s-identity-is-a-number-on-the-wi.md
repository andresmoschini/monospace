---
status: agreed
decided: "#182"
date: 2026-10-05
---

# A shape's identity is a number, on the wire as well as in the type

## Why now

A shape's identity is one number spelled two ways in the same object. `next_id` is `"next_id": 3`
and the twenty-six `id`s beside it are `"#1"` through `"#26"`, all in
`crates/monospace-cli/assets/demo.json` — and the type agrees with the strings, not with the
counter. That is the reason for touching the wire format at all: dropping the `#` from `ShapeId`
while the file still says `"#1"` leaves the reader parsing a string into a number, which is worse
than either spelling, so the format cannot be left alone.

The type is also what [#171](https://github.com/andresmoschini/monospace/pull/171) made expensive.
`ShapeId(String)` is 24 bytes and a heap allocation per copy, and the ownership record stores one
per stamped position — `owners: HashMap<(i32, i32), ShapeId>` at
`crates/monospace-core/src/buffer.rs:38`. **Representation removes
[#172](https://github.com/andresmoschini/monospace/issues/172) rather than answering it**: that
issue was going to make the buffer borrow its identities, a lifetime reaching every caller, and the
spike showed it does not compile against the demonstration. Four bytes that are `Copy` are why the
question does not reach here.

**The code and a document disagree about who owns the rules, and it is worth saying which way before
either moves.** `crates/monospace-core/src/identity.rs:6` argues that "the type is a string and
holds no rules" — who issues an identity, whether two may share one, and what may be done with it
are questions about a diagram. D1 puts one rule inside the type instead: `0` is not an identity.
That rule is about the _value_, not about any diagram — no diagram ever issued a `0`, and
`Diagram::new` starting at 1 says so — so it is the kind of thing the type can hold while the
issuing and uniqueness questions stay in `docs/diagram-model.md` §3. The argument is narrower than
the module doc's, and the module doc is amended with it rather than reconciled after the fact.
`docs/diagram-model.md:35` and `docs/model.md:57` are amended in the same increment, for the same
reason.

This is deliberately after #171 so the two are not mixed, and it is a change of two stages: every
representation choice here is defensible both ways, and the wire break is irreversible without a
version field to negotiate it.

**This spec runs past the 250-line ceiling and cannot be split to fit.** The type and the wire
format move in one increment because the middle state is the one the issue rules out — an integer
type beside a `"#1"` in the same file is a reader parsing a string into a number. Six decisions is
under the seven the format allows, so the length is not the cut being coarse about what to decide;
it is the evidence, and one paragraph of it is the cost of not repeating the issue. What would
shorten it is a smaller claim, which is the trade the ceiling is meant to refuse.

## Scope

### In

- **`ShapeId` becomes a newtype over `NonZeroU32`** in `monospace-core`, and `Display` writes the
  bare number. Both are D1 and D2.
- **The API takes the identity by value**: `Option<ShapeId>` on `Buffer::stamp`, `Option<ShapeId>`
  back from `Buffer::owner`, `ShapeId` on `add_under`, `find`, `get`, `remove`, `replace`,
  `forward`, `backward`. `Layer<'a, 'b>` loses its second lifetime. That is D3, and it is the whole
  of #172's question.
- **The description file carries a number**: `"id": 1` and `"shape": 1`, with `next_id` and every
  existing description file rewritten. `ShapeDescription`'s three `id: String` fields and `At`'s
  `shape: String` become integers, and the `id()` accessor with them.
- **The counter is seeded and exhausted honestly**: `numbered_from` takes a `NonZeroU32`, and `add`
  refuses to wrap past `u32::MAX` by name rather than by overflow. That is D4.
- **All 22 render markers are rewritten**, `specs/` and `README.md` included. That is D5.
- **The 59 literal identities in tests become numbers**, and the three that were deliberately not
  numeric — `"absent"`, `"other"`, `"chosen"` — become numbers too and read a little worse. That is
  the measured cost of the issue and the whole of it.

### Out

- **Where a friendly name lives**, which is the mapping layer this change makes room for and a new
  spec of its own. Two shapes are already told apart only by the strings `"chosen"` and `"other"` in
  `crates/monospace-diagram/src/diagram.rs`, so the need is visible — but nothing today says whether
  a name sits on the entry beside its number or in a table beside the diagram, and a reference
  writable either way is its own decision.
- **Checking a caller-supplied identity**, which is D6 and stays where §3 put it.
- **A version field on the description file.** The break is total and unnegotiable, and a
  provisional format that has never shipped outside this repository is the cheapest thing to break.
  The issue names this as a cost, not as a decision to take; a field would be one, and it is worth a
  spec when there is a first consumer who would have to be told.
- **Editing an identity after the fact**, which §11 already holds open. This change moves the type;
  it does not move that question.

## The decision

**D1 — the ordinal is a `NonZeroU32`, and the constructor is what says so.**

**Answer:** `ShapeId(NonZeroU32)`, built by `ShapeId::new(NonZeroU32)`. Zero is unrepresentable
rather than merely unused. **Why not** a plain `u32`: `numbered_from(0)` is legal today and `add` on
it would have to emit something, and a rule that lives in an `if` inside `add` is a rule the file
does not carry — the only way to reach `0` would be to seed the counter at it, which nothing does.
**Why not** `u32` plus a check in `numbered_from`: it makes 0 impossible on one path and leaves the
type willing to carry it on every other, which is the same defect this change exists to remove. The
cost is the import at each construction site and one more type named in a signature that already
names `ShapeId`. **Answered by** the maintainer, in the session that wrote this.

**D2 — `Display` writes the number and nothing else.**

**Answer:** `1`, not `#1`. **Why not** keeping the `#`: the type no longer carries it, so a
`Display` that adds it is a second place the convention lives, and it is the place every message
would read it from. **Why not** dropping `Display` altogether: `Debug` is derived and shows the
newtype wrapper, and a gallery label reading `Reference(ShapeId(1), Right, …)` is a worse string
than `1`. What D2 costs is checked rather than assumed — `Display for ShapeId` reaches no shipped
message today. Both captions that could have collided with a bare number interpolate an `Offset` and
not an identity (`crates/monospace-cli/src/main.rs:188`, `:365`), and the only identity strings a
reader sees are literals written by hand in `crates/monospace-diagram/src/gallery.rs:298`, `:301`
and `:408`, which are labels rather than rendering and are left as they are. **Answered by** the
maintainer, in the session that wrote this.

**D3 — the identity travels by value, everywhere it currently travels by reference.**

**Answer:** `Option<ShapeId>` in and out of the buffer, `ShapeId` by value into the diagram, and
`Layer<'a>` with one lifetime. **Why not** keeping the references and only shrinking what they point
at: a `Copy` four-byte value behind `&` is a second way to say the same thing, and `find` is the one
search every named method goes through precisely so that there is a single place a caller is
compared — two spellings of a comparison is where that goes wrong. **Why not** by value in the core
and by reference at the diagram's public boundary: that is the same split with no reason behind it,
since a `Copy` value costs nothing to pass and nothing to return. **Answered by** the maintainer, in
the session that wrote this.

**D4 — the counter takes a nonzero ordinal, and `add` says so when it runs out.**

**Answer:** `numbered_from(next: NonZeroU32)`, so a file saying `"next_id": 0` is refused by the
reader rather than seeded with an identity nothing can carry; and `add` advances with `checked_add`
and panics naming the exhaustion. **Why not** keeping `numbered_from(u32)` and skipping a zero
inside `add`: it moves D1's rule out of the type into a branch, and it turns `"next_id": 0` into a
file that silently means 1. **Why not** saturating at `u32::MAX`: two consecutive `add`s would then
hand back one identity, which §3 allows for a caller-supplied identity and does not allow for a
diagram-issued one — the diagram's own numbering is the one uniqueness claim the model makes, and
saturating breaks it quietly rather than loudly. The overflow is unguarded today
(`crates/monospace-diagram/src/diagram.rs:70`) and has two behaviors: a debug panic and a release
wrap to 0, which under D1 would be an identity nothing can carry. **Why not** leaving the `+= 1`: a
release binary would then be able to break the one invariant D1 just bought. **Answered by** the
maintainer, in the session that wrote this.

**D5 — the 22 render markers are rewritten, and a rewritten spec marker is still a marker.**

**Answer:** every marker's `"id"` becomes a number and `cargo xtask render` refills the fence, in
`docs/diagram-demo.md`, `docs/diagram-model.md`, `docs/model.md`, `README.md` and `specs/` alike.
**Why not** accepting `"#1"` in the reader so the old markers keep parsing: that is the same two
spellings in one object this change exists to end, kept forever on the reading side. **Why not**
exempting `specs/` from `xtask render`: a marker is a description of a picture the gate redraws, and
a spec that cannot be redrawn has lost the only mechanical check its pictures had. What is not
rewritten is the prose around a marker — a spec that was implemented stays what it said, and only
the artifact beside it moves. **Answered by** the maintainer, in the session that wrote this.

**D6 — `add_under` stays unchecked, and a typo'd ordinal is no more findable than a typo'd name.**

**Answer:** unchanged. Two shapes may carry one identity, both are held, and `get` answers for the
first — `two_shapes_carrying_one_identity_both_record_it_as_get_cannot_tell_them_apart`. **Why not**
checking uniqueness now that uniqueness is cheaper to check: it was not free to check before and is
not free now, and §3 puts it with the diagram for a reason D1 does not change. **Why not** rejecting
`0` at `add_under` with a bespoke message: D1 makes it unrepresentable, so there is no such call to
reject. The honest cost is the one the issue measured — an ordinal typo is a digit rather than a
character, and nothing reports either. **Answered by** the maintainer, in the session that wrote
this.

## Model slice

- `docs/diagram-model.md` §1 _Vocabulary_ (`docs/diagram-model.md:35`): "A shape's identity: a
  string, unique within its diagram" becomes a number, and the row stops claiming a spelling.
- `docs/diagram-model.md` §3 _Identity_ (`docs/diagram-model.md:88-106`): "it is a string" and the
  `#1`, `#2` examples throughout go; the paragraph about a name surviving being **listed** elsewhere
  needs its wording checked against a number, since "a name does not move when the order it is
  written in does" was said about a name; and the last paragraph gains D1's rule, which is the one
  rule the type now holds.
- `docs/model.md` §1 _Vocabulary_ (`docs/model.md:57`): the `ShapeId` row says "The name a figure is
  drawn under" — the word "name" is the claim to amend, and the issue's own count of three places
  saying "string" is two plus a word that needed amending anyway.
- `crates/monospace-core/src/identity.rs:1-20`: the module's "The type is a string and holds no
  rules" becomes "holds no rule about a diagram", which is what D1 leaves true. The second design
  note, "A whole string rather than a smaller token a diagram maps", is answered by D1 and goes.

All four land in the **building** stage, beside the code they describe. The deciding stage changes
no model, and a rule with one reader for a whole stage is not a rule.

## Public surface

```rust
// monospace-core — the identity itself
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShapeId(NonZeroU32);

impl ShapeId {
    pub fn new(ordinal: NonZeroU32) -> Self;
}

impl std::fmt::Display for ShapeId {
    /* writes the ordinal: 1 */
}

// monospace-core — the record and the query, by value
impl Buffer {
    pub fn stamp(&mut self, at: Pos, cell: Cell, mode: StampMode, owner: Option<ShapeId>);

    #[must_use]
    pub fn owner(&self, at: Offset) -> Option<ShapeId>;
}

pub struct Layer<'a> { /* … */ }
impl<'a> Layer<'a> {
    #[must_use]
    pub fn new(buffer: &'a mut Buffer, mode: StampMode) -> Self;

    #[must_use]
    pub fn stamped_by(buffer: &'a mut Buffer, mode: StampMode, owner: ShapeId) -> Self;
}

// monospace-diagram
impl Diagram {
    #[must_use]
    pub fn numbered_from(next: NonZeroU32) -> Self;

    pub fn add(&mut self, shape: Shape) -> ShapeId;

    // All of these take the identity by value now.
    pub fn add_under(&mut self, id: ShapeId, shape: Shape);
    pub fn forward(&mut self, id: ShapeId);
    pub fn backward(&mut self, id: ShapeId);
    pub fn get(&self, id: ShapeId) -> Option<&Shape>;
    pub fn remove(&mut self, id: ShapeId);
    pub fn replace(&mut self, id: ShapeId, shape: Shape);
}
```

`Buffer::owner` returning `Option<ShapeId>` by value is what #172 wanted and could not have: the
demonstration resolves its three identities by asking the picture, and it does that without a borrow
that reaches the buffer.

## Behavior

1. `ShapeId` carries a nonzero ordinal. There is no way to write `0`, from Rust or from a file.
2. `Display` writes the ordinal alone.
3. A diagram-issued identity is the counter's value and the counter advances by one; a shape added
   under a caller's identity does not move it.
4. A diagram is seeded with a nonzero ordinal and issues that ordinal first.
5. A diagram seeded at `u32::MAX` issues `u32::MAX` and then has no ordinal left, which `add`
   reports by name rather than wrapping to 0.
6. An identity is compared by its ordinal. One from another diagram is a well-formed ordinal that
   matches nothing here — the same answer a reference to an absent shape already gets.
7. A caller-supplied identity is not checked: two shapes may carry one ordinal, both are held, and
   the second is a shape no ordinal names until the first is removed.
8. The description file's `id` and `shape` are integers. `"id": "#1"` is refused, as a data error
   naming the field.
9. `next_id` is a nonzero integer. `"next_id": 0` is refused the same way, because the counter D4
   takes cannot hold it.
10. A stamp by nobody records nothing and takes nothing, unchanged by this change: `None` is still
    not an owner the buffer holds.
11. `owner` answers for an offset into the window as #171 settled it, and its answer is a value now
    rather than a borrow of one.

## Examples

**The wire format, before and after.** The same two shapes, in the same window, with the identity
spelled as it is today:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": -1 }, "size": { "width": 2, "height": 6 } },
  "next_id": 2,
  "shapes": [ { "kind": "connector", "id": "#1",
    "from": { "at": { "kind": "point", "x": 0, "y": 0 }, "leaving": "up",
      "terminal": { "kind": "glyph", "glyph": "▼" } },
    "to": { "at": { "kind": "point", "x": 0, "y": 3 }, "leaving": "down",
      "terminal": { "kind": "glyph", "glyph": "▲" } },
    "stroke": "light" } ] }
-->

```text
┌┐
▼│
 │
 │
▲│
└┘
```

<!-- /render -->

and with `"id": 1` and `next_id: 2` beside it, the picture is **byte for byte the same**. That is
the argument for D1 and D2 together, and it is the one thing a picture can say that two paragraphs
cannot: the identity is not drawn, so nothing about the output moves — which is exactly why the
change can be made now, before a front end exists to read an identity off a screen.

**That second picture is not in a marker, and cannot be in one on this branch.** The reader parses
`"id"` as a string today, so a marker carrying a number does not render yet; hand-writing the fence
would make it hypothetical, and `cargo xtask render` would overwrite it the moment the building
stage lands. It is drawn as the identical picture above rather than asserted. **What settles it is
the building stage**: this marker's own `id` becomes `1`, `cargo xtask render` refills the fence,
and `cargo xtask render --check` holding is the proof that rule 8 is what the shipped reader does.

**A file the reader refuses.** `"id": "#1"` and `"next_id": 0` are both refused as data errors, in
the vocabulary `deserialize_glyph` already uses at `crates/monospace-cli/src/description.rs:106` and
that `malformed_json_locates_the_problem_on_stderr_and_fails` already asserts for other failures:
stderr names the problem, stdout is empty, and the exit code is 1. The exact wording is not written
here because it has not been observed, and it is the acceptance list's job to pin it.

**Three identities that were not ordinals.** `crates/monospace-diagram/src/diagram.rs` tells two
shapes apart by the strings `"chosen"` and `"other"`, and asks a third diagram for an identity
nothing here holds — `a_foreign_identity()`, which its own doc at `:1425-1428` says takes the
_third_ identity precisely because `ShapeId` was a string and `#1` would collide. All three become
ordinals, and that helper's reasoning inverts: a foreign identity is now an ordinal this diagram
never issued, which is a statement about the counter rather than about the spelling. The tests keep
their names and lose their wording;
`a_chosen_identity_is_found_by_that_identity_and_by_no_other_one` is still about an identity that
was chosen by a caller, not one the diagram issued.

## What proves it

None of these tests exists yet; the deciding stage adds no code. The names are what the building
stage is held to.

| Rule | Test                                                                                         |
| ---- | -------------------------------------------------------------------------------------------- |
| 1    | `an_identity_cannot_be_zero_and_nothing_in_the_reader_or_the_type_allows_it_to_be_asked_for` |
| 2    | `an_identity_writes_its_ordinal_and_nothing_else`                                            |
| 3    | `a_diagram_issues_the_counter_and_a_shape_added_under_a_callers_identity_does_not_move_it`   |
| 4    | `a_seeded_diagram_hands_back_the_ordinal_it_was_seeded_with_and_then_a_different_one`        |
| 5    | `a_diagram_that_has_run_out_of_ordinals_says_so_rather_than_issuing_zero`                    |
| 6    | `an_identity_from_another_diagram_changes_nothing_and_does_not_panic`                        |
| 7    | `two_shapes_carrying_one_identity_both_record_it_as_get_cannot_tell_them_apart`              |
| 8    | `an_identity_written_as_a_string_is_refused_by_name_on_stderr_and_fails`                     |
| 9    | `a_next_id_of_zero_is_refused_by_name_on_stderr_and_fails`                                   |
| 10   | `a_shape_drawn_by_nobody_owns_nothing_even_where_it_writes`                                  |
| 11   | `owner_answers_for_an_offset_into_the_window_and_none_for_one_it_does_not_hold`              |

Rules 1 and 2 are held by a test on the type itself and, for the reader, by rule 8's negative case.
Rule 5 is held at the boundary with `NonZeroU32::new(u32::MAX)` and one further `add`. Rules 8 and 9
are spawned against the real binary the way `an_unrecognized_kind_names_it_on_stderr_and_fails`
(`crates/monospace-cli/tests/cli.rs:501`) is, and neither may disturb
`two_entries_carrying_one_identity_are_both_read_and_both_drawn` (`cli.rs:60`), which asserts
success and empty stderr for a file with duplicate identities — the case D6 leaves alone.

The 22 markers are held by themselves: `cargo xtask render --check` redraws every one, and a picture
that moved without a change here is the failure. Two snapshots carry an ordinal as a hand-written
label and are held by `cargo insta review`: `an_endpoint_hangs_from_a_side_and_follows_it` and
`the_order_decides_a_shared_cell`.

## Open questions

- **What an identity from another diagram means.** With a string it was well-formed and simply
  absent here, and `a_foreign_identity()` leaned on that. With an ordinal it is well-formed and
  simply absent here too — but nothing distinguishes "another diagram's 1" from "this diagram's 1,
  removed". What would settle it: a caller that actually holds two diagrams and cross-references
  them, which nothing does and §2's "a shape belongs to a diagram directly" discourages.
- **Whether a shape's ordinal should be stable when it is removed.** §3 holds an identity to its
  shape across removal and reordering, and
  `a_seeded_diagram_never_hands_out_an_identity_it_issued_before` at
  `crates/monospace-diagram/src/diagram.rs:716` is what pins it. A number invites reuse in a way a
  name does not — `#1` reads as a slot, `1` reads as a count. Nothing in the model forbids reuse and
  the test forbids it, so the answer is today the test. What would settle it: an editor that deletes
  a shape and expects the next `add` to land where the deleted one was.
- **Where a friendly name lives**, which the issue puts out of scope and which this change makes
  room for without answering. Two shapes are distinguished only by strings in the diagram's own
  tests, so the need is visible today and the shape of the answer is not.
- **The dangling `Q1`** at `crates/monospace-diagram/src/diagram.rs:33`, on the `next` field this
  change rewrites. Nothing in the repository is labelled `Q1`; the spec that asked the question it
  points at is not in `specs/`. What would settle it: nothing in this change, which should keep the
  comment's actual claim — that seeding is a plain assignment and no public method carries a
  subtraction — and drop the marker.
