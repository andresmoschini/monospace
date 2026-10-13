---
status: draft
date: 2026-10-12
---

# Hold the diagram as a persistent value

## Why now

`docs/memory-cost.md` §8 measures four ways to answer what a session costs, ranks them by what they
cost to be wrong about, and marks the persistent one _Yes_ — because it is the only one that asks
the model a question. Its fifth bullet under _What would change this recommendation_ now names the
case that changed it: **a reason to hold a diagram persistently that is not its memory.** An
immutable diagram is a discipline a caller can be held to, and the application coming in
[#174](https://github.com/andresmoschini/monospace/issues/174) through
[#181](https://github.com/andresmoschini/monospace/issues/181) is a caller that would rather be held
to it than hold it.

**The prototype already exists and it is green.** `spike/rpds` holds the diagram in `rpds`'s
`HashTrieMap<ShapeId, Shape>` and `Vector<ShapeId>`, changes every operation to take `&self` and
hand back a new `Diagram`, and passes 419 tests with not one snapshot moved. The cost of the rewrite
is known and it is a day's mechanical work.

**And it reverses a rule the model has, without anyone having decided to.** `docs/diagram-model.md`
§3 says an identity a caller supplies is not checked: two shapes may carry one, both are held, and
the second is a shape no identity names until the first is removed. `HashTrieMap::insert` replaces,
so under the prototype the second figure is not held at all. The prototype carries three `TODO:`
markers on the three tests that found it, and **they are what this spec answers.**

There is a second collision the same change makes worse, and no marker records it. `next_id` is
trusted rather than checked — `crates/monospace-description/src/lib.rs:529` says so, and calls the
repair _one line in `monospace-diagram`_. Today an ordinal that collides with a caller's identity
leaves the issued figure unreachable by identity and the caller's figure drawn. Under the prototype
the issued figure replaces the caller's, so the caller's figure is **gone**, with no error, no
report and no way to ask for it back.

**Nothing else here moves a picture**, and that is deliberate rather than an absence: D1 and D3
below change which figure answers an identity, and a picture cannot name a figure. §Examples draws
the one case where a picture does move and says plainly why the other two are not drawn.

## Scope

### In

- `crates/monospace-diagram/src/diagram.rs`: `Vec<Placed>` becomes a `HashTrieMap<ShapeId, Shape>`
  of figures and a `Vector<ShapeId>` of the order; `Placed` is deleted; `next` is renamed `next_id`;
  the six operations take `&self`; `add` steps over an ordinal a figure already sits under, and
  `add_under` on an identity already held is an upsert.
- `crates/monospace-diagram/src/utilities.rs`: new, and private. `swapped` and `without` are the two
  rewrites `rpds::Vector` does not have, and the module is where they are written once.
- `crates/monospace-diagram/Cargo.toml`: `rpds`, pinned.
- `crates/monospace-editing`: `Command::apply_to` returns a diagram rather than writing into one;
  `Command::perform`, `Session::undo` and `Session::redo` take `&self` and hand back a new session.
- `crates/monospace-description`: `into_diagram` rebinds instead of writing into a diagram it holds.
- Rustdoc that describes a diagram as a thing a caller edits: `monospace-editing/src/lib.rs`,
  `session.rs`, `gallery.rs`, and one paragraph of `monospace-description/src/lib.rs`.
- `docs/diagram-model.md` §2, §3 and §9; `docs/editing-model.md` §4, §7 and §8.

### Out

- **Bounding the history.** `docs/memory-cost.md` §8 ranks it first, and this change does not do it.
  D4 buys the constant and leaves the growth where it is, so the bound is still the answer to a
  different question — when a history is too long to be worth keeping at any cost.
  `docs/editing-model.md` §8's _How many steps a session keeps_ stays open, with the memory argument
  it rests on restated rather than closed.
- **Interning `Stroke` and `Glyph`.** Independent of D4 and orthogonal to it, as §8 says, and it
  changes how strokes are named rather than how a diagram is held. Its own change.
- **A `HashMap<ShapeId, usize>` beside the `Vec`.** D4 replaces that index rather than adding to it:
  the lookup arrives with the persistence, and a hand-kept index is the drift §7 measured at 0.3 µs
  per removal and one test to hold.
- **Threads.** D4 accepts `!Send` and names it. Nothing here makes a diagram sendable, and nothing
  here needs one to be.
- **`monospace-core`.** Nothing in it changes: `Buffer` stays mutable, `GlyphCatalog` stays a
  `HashMap`, and the gate's `wasm` step is untouched.
- **`Shape` and `Position`.** No public signature changes. `Shape::with_frozen_references` keeps
  `(&self, ShapeId, &Diagram) -> Option<Self>` and is the only place a figure is rewritten.

## The decision

**D1 — one identity names one figure and one place in the order; `add_under` is an upsert.**

**Answer:** a second `add_under` carrying an identity already held replaces the figure and leaves
the order alone, so the identity is drawn once. **Why not** keeping §3's rule that two figures may
share one: holding them needs an identity for the _occurrence_, which the model does not have and
nobody asked for, and it turns the order into a sequence of occurrences rather than of identities —
a larger change to the model than the problem. **Why not** refusing the second `add_under` in
silence: this crate reports nothing anywhere, so a refusal is a figure the caller asked for and did
not get, with no way to tell it from a figure that was never added. **What it costs:**
`docs/diagram-model.md` §3 is amended, and one picture moves — §Examples draws it.

**D2 — every operation takes `&self` and hands back a new diagram; `add` hands back a pair.**

**Answer:** `add(&self, Shape) -> (Self, ShapeId)`, and `add_under`, `forward`, `backward`, `remove`
and `replace` each `-> Self`. All six are `#[must_use]`. **Why not** `add` returning `Self` alone:
the caller needs the identity it was given, and deriving it from the counter publishes a private
field. **Why not** `add` returning `Self` and the caller looking the identity up: a lookup by an
identity the diagram has just issued is a hit by construction, not a guarantee, and it would put a
`get` between a caller and the number it was promised. **Why not** `self -> Self` throughout: the
consuming caller is the TUI, which holds a session and derives the next from it, and `&self -> Self`
is the shape that composition needs.

**D3 — an ordinal `add` issues is one no figure is already under, so `add` steps over.**

**Answer:** `add` issues the first ordinal at or after its counter that names no figure, and
advances the counter past the one it issued. **Why not** replacing, which is what the prototype
does: today the issued figure is unreachable while the caller's is drawn, and replacing is the first
change in this repository that makes a caller's figure disappear with no report and no way back.
`docs/diagram-model.md` §3 holds that the identities **the diagram issues** are unique, and
[`specs/173`](173-a-shape-s-identity-is-a-number-on-the-wi.md)'s D2 says the diagram's own numbering
is the one uniqueness claim the model makes — a diagram that hands out an ordinal a figure already
sits under has not kept it. **Why not** refusing the `add` instead: the caller asked for a figure,
and refusing means the one shape of the crate's whole vocabulary that says _I could not do that_
would have to be invented here. **What it costs:** `add` is no longer constant-time where collisions
exist, and `numbered_from`'s _the next `add` takes the ordinal `next`_ becomes _the first ordinal at
or after `next` that names no figure_. **What it reopens it:** a caller that wants the collision
reported rather than stepped over is asking for a report this crate has nowhere to put;
[#88](https://github.com/andresmoschini/monospace/issues/88) is where a diagram says which shapes it
could not draw. **What it settles elsewhere:** it is the repair
`crates/monospace-description/src/lib.rs:531` already names and leaves to this crate.

**D4 — `rpds`, pinned, in its non-sync flavour; `!Send` accepted and named.**

**Answer:** `rpds = "=1.2.1"` in `crates/monospace-diagram/Cargo.toml`. **Why not** the hand-rolled
trie `docs/memory-cost.md` §4 measured: §7 measured it as less legible than the scan it replaces,
and §6 measured `rpds` doing the same job for 24% less memory per step — 949 B against 1 248 B —
which is the honest answer to _should we write our own_. **Why not** `im`: its last release predates
edition 2024, and MPL-2.0+ is a license a maintainer has to accept deliberately rather than by
default. **Why not** the sync flavour: `Arc` instead of `Rc` costs on every node and buys a caller
this repository does not have — nothing in `crates/` uses a thread, and a terminal application is
one event loop. **What it costs:** a diagram is `!Send` and `!Sync`, permanently unless a later
change says otherwise, and `Clone` becomes O(1) rather than a deep copy — which two documents state
today and both are amended. **Reported as the rule requires:** `rpds` 1.2.1, published 2026-05-15,
MIT, edition 2024, MSRV 1.85.0, which the pinned toolchain satisfies; it builds for
`wasm32-unknown-unknown` and the gate's `wasm` step covers it.

**D5 — the session is a value too: `undo`, `redo` and `perform` take `&self` and hand back a new
one.**

**Answer:** `Session::undo(&self) -> Session`, `Session::redo(&self) -> Session`, and
`Command::perform(&self, session: &Session) -> Session`. **Why not** only `perform`: the session's
central claim today is that nothing changes a session's diagram except a command, and `undo` and
`redo` are `&mut self` — so the session is currently the one mutable value in a crate built to have
none, and D5 with `perform` left alone would teach exactly the wrong lesson to the caller it is for.
**Why not** the whole crate: `Diagram` is already a value on this change, and a caller that can copy
a session cheaply and then mutate it in place is the hole this closes. **What it costs:** the
`compile_fail` example in `session.rs` stops being load-bearing, because there is nothing left to
expose — it stays anyway, and says so.

**D6 — `Session::steps` stays a `Vec`.**

**Answer:** unchanged. **Why not** a persistent vector: the history is a line with a `truncate` and
a `push`, not a structure two snapshots share, and the expensive part of a step — the diagram — is
already shared by D1 and D2. **Why not** two stacks of steps rather than one line: `position` is an
offset into one line precisely so it cannot disagree with either end, and a pair of stacks is the
tree this model declined.

## Model slice

- **`docs/diagram-model.md` §2 _The diagram_** gains the half it has never had: that the figures are
  held apart from the order that draws them, that a diagram is a value, and that changing one
  produces another rather than editing one. The rendered picture in that section is unchanged.
- **§3 _Identity_** is amended by D1 and D3. The sentence _an identity a caller supplies is not
  checked: two shapes may carry one, both are held_ is replaced by D1's rule, and the paragraph
  below it — which says _who issues an identity and whether two may share one are the questions
  above, and they stay here_ — loses one of the two questions and keeps the other.
- **§9 _Changing a diagram_** is rewritten from the imperative. It currently reads as five commands
  (_**Takes** a shape **out of** the diagram_, _**Moves** a shape one place toward the front_);
  under D2 each of the five is a function producing a diagram, and the section has to say so in the
  voice the other sections use.
- **§8 _Ownership_ is not touched, and deliberately.** Despite the name it is about which figure
  decides a cell at drawing time, not about who owns a value. `docs/memory-cost.md` §7 and §8
  pointed the `add_under` question at it in two places; PR #211 corrected both to §3, which is where
  D1 and D3 land.
- **`docs/editing-model.md` §4** loses the _one field and one absence_ argument, which D5 makes
  vacuous, and gains the stronger one: nothing in the crate changes a session but its own methods.
  The paragraph survives in a shorter form because the `compile_fail` example beside it survives.
- **§7 _What a session hands out_** keeps _a reference to the diagram_ and _a copy of the diagram_,
  and the copy stops being a warning: it shares everything with the original.
- **§8 _How many steps a session keeps_** keeps the question open and restates its reason. The
  sentence _a diagram is a value rather than a handful of bytes, so an unbounded history is an
  unbounded amount of memory_ is true and is why; the figure behind it is 1000 steps at 949 B rather
  than 40 122 B, and the growth is still linear, which is why the question is still open.

## Public surface

The deciding stage adds none of this; it is what the building stage publishes. Only the signatures
that change are listed, with what they are today beside them, because what does not change is the
larger half of the answer.

```rust
// monospace-diagram, Diagram
pub fn add(&self, shape: Shape) -> (Self, ShapeId);          // was (&mut self, Shape) -> ShapeId
pub fn add_under(&self, id: ShapeId, shape: Shape) -> Self; // was (&mut self, ..) -> ()
pub fn forward(&self, id: ShapeId) -> Self;                 // was (&mut self, ShapeId) -> ()
pub fn backward(&self, id: ShapeId) -> Self;                // was (&mut self, ShapeId) -> ()
pub fn remove(&self, id: ShapeId) -> Self;                  // was (&mut self, ShapeId) -> ()
pub fn replace(&self, id: ShapeId, shape: Shape) -> Self;   // was (&mut self, ..) -> ()

// monospace-editing
impl Session {
    pub fn undo(&self) -> Session;    // was &mut self -> ()
    pub fn redo(&self) -> Session;    // was &mut self -> ()
}
impl Command {
    pub fn perform(&self, session: &Session) -> Session;  // was (&self, &mut Session) -> ()
}
```

**`&mut` leaves the public API of two crates in one change, and that is the change's whole point.**
`numbered_from`'s parameter is renamed `next_id` and its meaning narrows by D3; `new`, `get` and
`draw` are untouched, as are `Session::new`, `diagram`, `can_undo` and `can_redo`. `Shape`,
`Position`, `Reference`, `Anchor`, `Delta` and `Endpoint` are untouched. `monospace-cli` needs no
edit at all: it drives everything through `Command::perform` and reads through `Session::diagram`.

## Behavior

1. Every operation on a diagram takes `&self` and hands back a diagram; none changes the one it was
   given, and a caller that drops the result changes nothing.
2. `add` hands back the new diagram and the identity it gave the shape, in that order.
3. `add` issues the first ordinal at or after its counter that names no figure, and advances the
   counter past the one it issued.
4. Two `add_under` calls carrying one identity leave one figure under it, drawn once, in the place
   the identity already had in the order.
5. An identity no figure is under names nothing, and every operation naming it hands back a diagram
   equal to the one it was given.
6. `forward` and `backward` are the only operations that search, and they search the order; `get`,
   `replace` and `remove` are lookups.
7. `remove` takes the figure out and takes its identity out of the order at every place it stood, so
   the order is left closed rather than gapped.
8. What hung from a removed figure is frozen exactly as `docs/diagram-model.md` §4 says, and nothing
   goes looking for the reference afterwards.
9. A diagram that has no ordinal left to issue says so, rather than handing back one it has already
   handed out — including when the last ordinal is the one a figure already sits under.
10. `undo`, `redo` and `perform` take `&self` and hand back a session; the one given is unchanged.
11. `perform` records the step before it carries the command out, discards what was ahead of the
    position, and adopts the state at the position — as it does today.
12. A copy of a diagram or of a session shares every figure with the original, so a copy is not a
    second copy of the shapes and dropping it frees nothing the original holds.
13. `Session::steps` is a `Vec`, and nothing else about the session's storage changes.
14. The four pictures in `gallery.rs`, the eight of the demonstration and every `render` marker in
    the repository are unchanged, except the one §Examples draws.
15. `Cargo.toml` gains one dependency, pinned, and `xtask` gains none.

## Examples

**One identity added twice, which is the only picture in this change that moves.** Hypothetical: no
code produces this yet, and none of it has been observed. Two boxes 2x1 at `(0,0)` and `(4,0)` in a
`6x1` window, both added under identity `1`.

```text
  add_under(1, box at 0,0)   add_under(1, box at 4,0)

  today                    under D1
  ┌┐  ┌┐                      ┌┐
```

**The left box goes, and that is D1's whole visible content.** Under D1 the second `add_under`
replaces the figure rather than standing beside it, so the diagram draws one box where it drew two.
`get(1)` answers with the box at `4,0` and `remove(1)` takes it; today `get(1)` answers with the box
at `0,0` and `remove(1)` takes that one, leaving the other reachable. A caller that never names one
identity twice sees no change at all, which is why this is one block rather than a rule about
drawing.

**The other half of D1 and all of D3 cannot be drawn, and that is why neither is a marker.** An
issued ordinal that collides with a caller's is about which figure answers an identity:
`numbered_from(1)`, `add_under(1, box at 0,0)`, `add(line at 0,0, len 1)` draws the same picture
today and under D3, and the whole of the difference is that the line is named `2` rather than
nothing at all. `docs/diagram-model.md` §3's picture is unaffected by any of the six decisions.

## What proves it

None of these tests exists yet; the deciding stage adds no code. The names are what the building
stage is held to.

| Rule | Test                                                                      |
| ---- | ------------------------------------------------------------------------- |
| 1    | `no_operation_changes_the_diagram_it_was_given`                           |
| 2    | `add_hands_back_the_diagram_and_the_identity_in_that_order`               |
| 3    | `add_steps_over_an_ordinal_a_figure_already_sits_under`                   |
| 4    | `a_second_shape_under_one_identity_replaces_it_and_is_drawn_once`         |
| 5    | `an_identity_naming_no_figure_changes_no_cell_of_the_picture`             |
| 6    | `forward_and_backward_are_the_only_operations_that_search`                |
| 7    | `remove_leaves_the_order_closed_rather_than_gapped`                       |
| 8    | `what_hung_from_a_removed_figure_is_frozen_and_not_looked_for_afterwards` |
| 9    | `a_diagram_with_no_ordinal_left_says_so_rather_than_repeating_one`        |
| 10   | `no_session_method_changes_the_session_it_was_given`                      |
| 11   | `a_step_is_recorded_before_the_command_it_belongs_to_is_carried_out`      |
| 12   | `a_copy_of_a_diagram_is_not_a_second_copy_of_its_shapes`                  |
| 13   | `the_history_is_a_line_a_position_cannot_disagree_with`                   |
| 14   | `the_demonstrations_eight_pictures_are_unchanged`                         |
| 15   | `cargo xtask check`                                                       |

Rule 3 is the one the prototype gets wrong in the worst way, and rule 4 is the one the model has to
be amended for; between them they are D1 and D3. Rule 12 is measured rather than asserted — a copy
sharing structure is a claim about allocations, and the honest form is the counting allocator
`docs/memory-cost.md` §1 built. That no snapshot moved is the number the building stage reports, and
it should be `0` for all twenty-four markers and all twelve `insta` snapshots.

## Open questions

- **What a session keeps is still `docs/editing-model.md` §8's to answer**, and this change restates
  its reason rather than closing it. A bound remains the cheapest answer and remains compatible with
  everything decided here.
- **Whether `Diagram` should carry `Send`** is not settled by D4, which accepts this crate's answer.
  It reopens the day a caller rendering on a background thread exists, and nothing in `crates/` is
  one.
- **What a caller should be told when `add` steps over an ordinal.** D3 issues the figure and says
  nothing. Whether that is worth reporting belongs with
  [#88](https://github.com/andresmoschini/monospace/issues/88), and is its own issue rather than a
  section here.
