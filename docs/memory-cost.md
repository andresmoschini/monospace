# What a diagram and a session cost

**Status:** Measured, 2026-10-12. Nothing here is a design decision, and nothing here changes the
model: [`docs/model.md`](model.md), [`docs/diagram-model.md`](diagram-model.md) and
[`docs/editing-model.md`](editing-model.md) still own what a diagram is and what a session does.
This document answers one question they each hold open — how much memory the current design asks for
— with numbers rather than an estimate, so that a later change can be argued about from a
measurement. **§5 goes past what was asked for and weighs the case for and against making the
diagram persistent and immutable, §6 asks whether a library should hold it, §7 asks what Rust
already has, and §8 puts the options in the order the measurements support; none of the four decides
anything.**

**Created:** 2026-10-12

**Measured before the base types narrowed.** Every number below was taken when `Pos`, `Offset`,
`Size` and `ShapeId` were still 32-bit;
[#219](https://github.com/andresmoschini/monospace/issues/219) makes them 16-bit and does not
re-measure this document, so these are the older, larger sizes.

**The question.** `docs/editing-model.md` §8 lists _How many steps a session keeps_ as deliberately
unresolved, and gives the condition that would settle it: _the first diagram big enough for the
copies to be felt rather than measured._ This is that measurement, taken on a 300-shape diagram with
1000 changes, which is a session that has been open for a while. **It is one measurement, not a
threshold:** 300 shapes and 1000 changes are figures chosen because they are round and plausible,
not because somebody found the point where the copies start to hurt. What the numbers do settle is
the order of magnitude, which is far enough from comfortable that the question is worth deciding on
other grounds than a feeling.

## 1. How it was measured

A counting global allocator wrapping `std::alloc::System`, recording live bytes and their peak, with
every size derived from the crate as it is rather than from a reading of the source. The harness is
a throwaway crate outside this repository — it path-depends on `monospace-core`, `monospace-diagram`
and `monospace-editing` and is not part of the gate — because what is being measured is the memory
this repository allocates today, and adding a benchmark to the repository would be a decision nobody
has asked for.

Three numbers are reported for every diagram: the diagram alone, what 1000 further commands add, and
the peak reached while they are carried out. The peak matters because a model can be cheap to hold
and expensive to reach, and only the peak tells you which of the two a session would actually have
to survive.

**The shape of the diagram is fixed across every figure here:** 100 boxes, 100 lines and 100
connectors, each carrying a `Stroke`, each box a fill glyph, each connector anchored by a reference
to a box. That is one part in three of each kind rather than a claim that such a diagram is typical.

## 2. What a shape weighs

| Type        | Bytes | Note                                       |
| ----------- | ----- | ------------------------------------------ |
| `Shape`     | 120   | the largest of the three arms              |
| `Endpoint`  | 48    | `Position` 16, direction 1, terminal 24    |
| `Position`  | 16    | `Absolute` or `Reference`                  |
| `Reference` | 16    | identity, anchor and offset                |
| `Stroke`    | 24    | a `String`                                 |
| `Glyph`     | 24    | a `String`                                 |
| `Command`   | 16    | the largest of the four arms               |
| `Placed`    | 128   | a `Shape` in a diagram, with its `ShapeId` |

**`Shape` is 120 bytes because `Stroke` and `Glyph` are `String`s, not because the geometry is
large.** `Connector { from, to, stroke }` is 48 + 48 + 24 and nothing else; the two positions inside
it are eight bytes each. A diagram of 300 shapes therefore holds roughly 38 KiB of positions and a
little over 35 KiB of string headers, and the second number is the larger one.

## 3. The current model

A `Diagram` is a `Vec<Placed>` and a `Step` holds a whole `Diagram`. `Session::record` clones the
diagram as it is, pushes that clone into the history, and assigns the original to the field it
keeps. So every command adds one complete copy of every shape, and nothing is shared between two
steps.

| Diagram | Alone    | +1000 commands | Per step | Peak      |
| ------- | -------- | -------------- | -------- | --------- |
| 300     | 67.7 KiB | **38.3 MiB**   | 40 122 B | 38.4 MiB  |
| 300     | 67.7 KiB | **153.1 MiB**  | 40 142 B | 153.2 MiB |

The second row is 4000 commands, and it is in the table because it is the row that answers the
question. **40 122 B and 40 142 B per step are the same figure within measurement noise: the cost is
exactly linear in the number of commands and exactly independent of anything else.** A history of 10
000 commands on this diagram is about 390 MiB, and nothing in the design would stop it.

Two consequences worth stating plainly:

- **At 1000 commands the history is 570× the diagram.** The diagram being edited is a rounding error
  in the session holding it.
- **The peak is only 110 KB above what is retained**, so nothing here is a transient allocation
  storm. The cost is entirely in what is kept, and the fix is not about allocation behavior.

### The per-shape figure is capacity rounding, not the cost of a shape

The 300-shape diagram reports 231 B per shape while `Placed` is 128, which is not a fact about
`Placed`. Building by pushing grows a `Vec` to a power of two, so a diagram of 300 shapes holds 512
slots, and two thirds of them are empty. Measured across the boundary:

| Shapes | Bytes   | Per shape |
| ------ | ------- | --------- |
| 256    | 35 237  | 138 B     |
| 300    | 69 284  | **231 B** |
| 511    | 70 474  | 138 B     |
| 512    | 70 474  | 138 B     |
| 513    | 138 075 | **269 B** |
| 1024   | 140 965 | 138 B     |

513 shapes costs twice what 512 costs and 1.95× what 511 costs, for one shape more. **The per-step
figures in the table above are not affected by this**, because each step's `Vec` is built by `clone`
and gets a capacity equal to its length rather than one rounded up: 40 122 B is 313 slots' worth of
`Placed` plus the strings, and it is the honest per-step number. The rounding shows up in the live
diagram, where it is 21 KiB of empty slots on a 300-shape diagram.

### Time

Five independent runs of 1000 commands, each building a fresh session:

| Min     | Median   | Max      |
| ------- | -------- | -------- |
| 78.7 µs | 101.4 µs | 127.0 µs |

**The spread is a third of the value, so this figure is reported as a range and nothing finer should
be read into it.** The cost is a linear scan of the order (`Diagram::find` is `iter().position`),
plus two deep clones per command — `record` clones once for the step and once for the field. In an
application that repaints on every event, ~100 µs per command is worth more attention than the
memory, because it is felt immediately and the memory is not.

## 4. Two persistent models, measured

The question that prompted this: whether a diagram should be persistent and immutable, so that
changing a shape returns a new diagram pointing at the same unchanged shapes and the new one. Two
variants were built and measured, because the naive reading of the idea and the one that actually
wins are not the same.

| Model                               | Alone    | +1000 changes | Per step | Time       |
| ----------------------------------- | -------- | ------------- | -------- | ---------- |
| Current, `Vec<Placed>` per step     | 67.7 KiB | 38.3 MiB      | 40 122 B | ~100 µs    |
| Persistent, persistent order vector | 45.6 KiB | 4.7 MiB       | 4 960 B  | —          |
| Persistent, trie index              | 57.4 KiB | **1.2 MiB**   | 1 248 B  | **0.5 µs** |

**The persistent vector is not the answer, and the reason is the shape of the data rather than the
quality of the idea.** Sharing the figures — which is the expensive part, 112 bytes each — leaves
300 pointers to be copied per change, and 300 × 16 bytes is 4 960 B. That is an 8× improvement on
the current model and still linear in the number of shapes, so a diagram of 3000 shapes pays ten
times as much per step. **The order has to be a persistent structure too, not a copied vector.**

The trie variant shares everything outside the path to what changed. Measured: the index is 33 trie
nodes and the depth is 2, and one change allocates exactly 2 trie nodes and 1 figure node — 2 ×
536 + 112 = 1 184 B, against 1 248 B measured. **That is ~32× less memory than the current model and
~200× less time**, and it is logarithmic rather than linear in the number of shapes.

The cost is paid elsewhere: a persistent diagram is a pointer graph, so it needs `Rc` or `Arc`,
which makes it `!Send` and unsuitable for holding across threads as it stands, and it changes what
`Clone` means — `Clone` becomes O(1) rather than O(n), which is exactly what `Session::record` wants
and exactly what any caller relying on `Clone` being a deep copy would notice. **None of those are
arguments against it and none of them are free either; they are the things a change would have to
decide.**

### Interning the strings is a different lever

`Shape` at 120 bytes is mostly `Stroke` and `Glyph`, and both are `String`. Replacing them with
interned `u16` handles, keeping the same three arms and the same fields, gives:

| Type       | `String` | Interned |
| ---------- | -------- | -------- |
| `Shape`    | 120 B    | 60 B     |
| `Endpoint` | 48 B     | 28 B     |
| `Placed`   | 128 B    | 68 B     |

which takes a step's 300 slots from 38 400 B to 20 400 B. **On the current model that is roughly a
halving — 38.3 MiB becomes about 19 MiB — and it is orthogonal to persistence, which means the two
can be taken separately and the cheaper one taken first.** It does not change the growth rate: the
history remains linear in the number of commands, so this reduces a constant and leaves the shape of
the curve alone. This is a layout calculation from `size_of` on equivalent fields, not a measured
allocation count, and it is the one figure here that is derived rather than measured.

## 5. What a persistent diagram would buy and cost

Memory and time are two of the reasons to make the change and neither is the deciding one. This
section is the rest of the argument, and it is weighted toward the costs because the benefits above
are already measured and the costs are the ones a reader has to weigh against a use case that does
not exist yet.

### The benefits that are not about memory

**Drawing would stop walking the order on every lookup.** `Diagram::find` is
`self.shapes.iter().position(...)`, so every lookup by identity is a linear scan of the order.
`Diagram::draw` walks all 300 shapes and each connector resolves its endpoints through the diagram,
which is one such scan per endpoint.

Measured, as the minimum over 40 draws of each size rather than a mean, because this machine's
timing noise spans more than the effect:

| Shapes | Draw, min | Ratio         |
| ------ | --------- | ------------- |
| 75     | 685 µs    | —             |
| 150    | 1309 µs   | 1.9× for 2.0× |
| 300    | 2610 µs   | 2.0× for 2.0× |
| 600    | 5685 µs   | 2.2× for 2.0× |
| 1200   | 11 650 µs | 2.0× for 2.0× |

**Drawing is linear, not quadratic, and the naive reading of the code is wrong.** The reason is that
the scan finds the shape early: identities are issued in order and a connector's box was added long
before it, so the scan terminates after a fraction of the diagram rather than after all of it. An
earlier run of this measurement on the mean rather than the minimum gave ratios scattered between
1.3× and 2.8× for the same 2× of shapes, which is the reason the minimum is used — **the effect is
real but it needed a better estimator to see, and a reader comparing their own numbers should expect
the same noise.**

So a persistent index does not turn a quadratic into a linear here; it makes the constant smaller
and removes a scan that has no reason to be there. At 1200 shapes the diagram draws in 11.6 ms, and
that is the number a caller repainting on every keystroke would feel. **Persistence helps that, and
it is not the reason the same scan is worth removing**, because an index is the fix whether or not
the diagram is persistent.

**The lookup would stop being a correctness hazard.** Nothing stops a caller from building a diagram
with 10 000 shapes today; nothing measures it. An index makes the cost of a lookup knowable in
advance rather than discovered.

### The costs

**A persistent diagram is not `Send` or `Sync`, and that is a constraint on a caller that does not
exist yet.** `Rc` is `!Send`; `Arc` fixes it and costs an atomic per clone. Nothing in this
repository uses threads — there is no `thread`, no `Arc`, no `Mutex` anywhere in `crates/` — so
**nothing here would break, and nothing here would notice.** The cost is entirely on the future: an
application that wants to draw on a background thread cannot hold a diagram across the handoff, and
that is a decision the persistence change would make by default rather than by argument.

**`Clone` would stop meaning what its documentation says.** `docs/diagram-model.md` §2 states that a
diagram is a value and copying one is not sharing anything, and `docs/editing-model.md` §7 promises
a caller that a copy taken from a session is _"changeable, free of the session, and unconnected to
it."_ Under persistence `Clone` is O(1) and the copy shares every unchanged figure. **That promise
is still true — an immutable structure cannot be changed through a shared node — but the reason it
is true changes from "nothing is shared" to "sharing is safe because nothing is mutable," and both
documents would have to say so.** A reader who takes the current wording as a guarantee that the two
copies are physically separate would be relying on an implementation detail, and the current model
happens to provide one.

**`Remove` would get more expensive, and it is the one operation where that is true.** `remove`
freezes every reference naming the figure it takes, walking the whole order and rewriting what
pointed at it. Today those are in-place writes into a `Vec` that already exists; under persistence
each rewritten figure is a **new allocation**, and the shapes that referenced it are exactly the
ones that change. Measured on the current model, where the cost is already the highest of any
command:

| Shapes | Remove     | Ratio          |
| ------ | ---------- | -------------- |
| 300    | 64–75 µs   | —              |
| 600    | 124–144 µs | ~1.9× for 2.0× |
| 1200   | 417–466 µs | ~3.3× for 2.0× |

**A removal costs more than a move by a wide margin, and it grows faster than the shape count.** A
move on a 300-shape diagram is ~100 µs dominated by two full clones; a remove is 64 µs because it
writes in place and allocates nothing. Under persistence the move becomes ~1 µs and the remove
becomes one allocation per referencing shape — so **the operation that is currently cheapest in
memory becomes the most expensive one to represent.** It does not become expensive in absolute
terms; it becomes the worst case in a model whose whole argument is that changes are cheap.

**`find` returning `None` for an identity from another diagram survives; `add_under` is a conflict
and not a cost, and §6 corrects this item's own first draft.** `add_under` permits two entries under
one identity, and a map keyed by identity cannot hold both. This item first measured the conflict
away, on the grounds that `find` is `iter().position(..)`, which stops at the first match — so `get`
on a doubled identity already returns the first and has always ignored the second — and that a map
keeping the first therefore reproduces today's behavior without being told to.

**Measured: that is half right, and the half that is wrong costs a figure.** A map does not keep the
first unless it is written to, and nothing in the crate says which way to write it —
`HashTrieMap::insert` replaces. More to the point, today _neither_ figure is dropped: both are held
and both are drawn, and a single-valued map drops one whichever way it is spelled. **A map does not
reproduce today's behavior; it turns a question the rule never answered into a silent choice.** What
the rule never answered is whether one identity may name two figures at all, and §7 takes that up as
a separate change.

**The counter that hands out identities stops being trivially copyable.** `Diagram::next` is a
`NonZeroU32` and the rustdoc is explicit that two copies answer `add` with the same ordinal, which
is what keeps two copies from handing out one identity. Under persistence that still holds — it is a
`Copy` field — so **this one is free**, and it is listed because it looks like it might not be.

**Testing gets harder to read.** The current model's tests assert on `&Diagram` and compare with
`==`, because a diagram is a value and equality is structural. Under persistence a test that checks
two diagrams agree has to compare either structurally (still possible) or by pointer identity (fast,
and true more often than it looks, which makes it a poor assertion). **The existing test module in
`crates/monospace-diagram/src/diagram.rs` runs from its line 258 to the end of a file 4 130 lines
long — about 3 900 of them — and asserts through the public API throughout, so this is mostly a
matter of assertions staying as they are rather than a rewrite** — but "mostly" is doing real work
in that sentence and the change should expect to touch tests.

**Reference cycles become representable, and a leak becomes possible.** Today a shape refers to
another by `ShapeId`, a number, so the structure cannot contain a cycle and dropping a diagram frees
everything. Under persistence, if references ever become pointers rather than numbers, a cycle
becomes constructible and `Rc` will not collect it. **Today they are numbers, so this is not a
present problem — and it is listed because it is the failure mode that a persistent data structure
introduces quietly and that shows up much later as memory that is never released.**

### What the comparison actually comes down to

Three of the costs above are cheap, one is real, and one is a genuine conflict:

| Cost                               | Verdict                                                                                                                                  |
| ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `!Send` on a diagram               | Free today, constrains a caller that does not exist yet                                                                                  |
| `Clone` becomes O(1)               | Two documents change; the guarantee they state still holds                                                                               |
| Tests comparing diagrams           | Mostly unchanged; expect to touch assertions                                                                                             |
| Cycles become representable        | Not possible today, since references are numbers                                                                                         |
| `Remove` becomes the worst case    | Real: the cheapest operation becomes the most expensive                                                                                  |
| Two figures may share one identity | **Real, and unchanged: a map holds one figure per identity, and which of the two survives is decided by the spelling, not by the model** |

And the alternative is not "do nothing": `docs/editing-model.md` §8 holds open _How many steps a
session keeps_, and **bounding the history is a one-line change that caps the linear term at
whatever the bound is**, where persistence removes the term but costs the conflicts above. At 1000
commands the current model holds 38.3 MiB and a bound of 100 would hold 3.8 MiB; at 10 000 commands
the bound holds the same 3.8 MiB and persistence holds about 12 MiB. **The bound wins whenever the
history would have been longer than the bound, and loses nothing when it would not** — which is the
whole of the argument for it, and the reason a spec should compare the two rather than assume
persistence is the only way down.

## 6. Would a library do, and which one

`CONTRIBUTING.md` _Dependencies_ says two things that bear directly on this: a dependency is not
added without asking the maintainer first, and **infrastructure concerns prefer idiomatic,
well-established crates over reinvention.** A persistent map is infrastructure in exactly that
sense, so the repo's own rule points toward a crate rather than the trie in §4. Whether it should is
a separate question from whether it is allowed, and the measurements below are the answer to the
first.

### Both candidates, measured

The hand-rolled trie from §4 rebuilt on `rpds`'s own structures, same figures, same allocator, same
1000 changes:

| Model                        | Alone    | +1000 changes | Per step  | Lookup       |
| ---------------------------- | -------- | ------------- | --------- | ------------ |
| Current model                | 67.7 KiB | 38.3 MiB      | 40 122 B  | linear scan  |
| Persistent, hand-rolled trie | 57.4 KiB | 1.2 MiB       | 1 248 B   | —            |
| **Persistent, `rpds` 1.2.1** | 78.7 KiB | **0.9 MiB**   | **949 B** | **21–35 ns** |

**`rpds` is 24% cheaper per step than the trie in §4, and that is worth understanding rather than
reporting as a win.** §4 attributed 536 B to a trie node, and the arithmetic 2 × 536 + 112 = 1 184 B
matched the measurement, so the model was right about the code as written. It was wrong about what a
trie node costs: `rpds` packs its nodes tighter than `Box<[Slot; 32]>` where every slot is a 16-byte
enum, because it stores nodes and entries in separate arrays. **The 536 B was a property of my
`Slot` enum, not of HAMTs.** Per key the whole diagram is 265–275 B across 300, 1200 and 4800
shapes, which is the per-shape cost including the figure; a library that is both faster and smaller
on the very operation it was written for is not a close call.

The base diagram is _larger_ with `rpds` — 78.7 KiB against 57.4 KiB — because the index holds two
structures where the hand-rolled one held one, and both are duplicated per version. **That is paid
once and is irrelevant beside 1000 steps**, and it is worth stating because a reader comparing the
"Alone" columns alone would draw the wrong conclusion.

### `rpds` rather than `im`, on evidence rather than preference

|           | `im` 15.1.0 | `rpds` 1.2.1 |
| --------- | ----------- | ------------ |
| Published | 2022-04-29  | 2026-05-15   |
| Edition   | 2018        | 2024         |
| License   | MPL-2.0+    | MIT          |
| Downloads | 29.1M       | 2.3M         |

`im` is the better-known name and has more downloads; `rpds` is four years newer, on the same
edition this repository uses, and MIT where `im` is MPL-2.0+. **Two things decide it here and
neither is popularity.** First, `CONTRIBUTING.md` requires that a dependency's publication date be
**at least seven days old** and reported — `rpds` 1.2.1 clears that by five months and `im` 15.1.0
clears it by four years, so both pass and neither is disqualified. Second, this repository pins with
`=` and builds for `wasm32-unknown-unknown`; `rpds` is MIT like this repository and states a 1.85.0
MSRV, which the pinned toolchain satisfies. **`im`'s last release predates edition 2024 and this
repository's own history, and its copyleft license is a thing a maintainer has to accept
deliberately** — both are arguments for `rpds`, and neither is a technical disqualification of `im`.

### What a library does not decide

**The `add_under` conflict is not smaller than §5 said, and this corrects this section's own first
draft.** §5 called it a conflict between the model and a map, on the grounds that a map cannot hold
two figures under one identity. This section first answered that the conflict was smaller: §6
measured that `Diagram::get` on an identity carried twice returns the first of the two, because
`find` is `iter().position(..)` and `position` stops at the first match, so a map built with
`entry().or_insert()` would reproduce today's behavior exactly.

**Measured: both halves of that are wrong, and the second one costs a figure.** `get` answering with
the first figure is not the same claim as a map keeping the first figure, and **nothing in the crate
decides which spelling to write: `rpds::HashTrieMap::insert` replaces, so the obvious spelling keeps
the last figure and drops the first.** Whichever of the two spellings is chosen, a single-valued map
drops one of the two figures — and today neither is dropped, because both are held and both are
drawn. **A map therefore does not reproduce today's behavior; it makes a choice explicit, and the
choice is a rule about a diagram that nobody has taken.**

What does survive is the narrower half §7 names, and it survives under every option measured: the
second figure is unreachable by identity, and already was. §5's summary table is amended accordingly
and §7 returns to where that half belongs.

**A dependency is a permanent answer to a question that has not been asked yet.** `CONTRIBUTING.md`
requires asking the maintainer, and this is the ask. A persistent diagram is a hypothetical — §5's
costs are real and the benefit is measured against a session that does not exist — so the honest
order is to settle whether the diagram becomes persistent at all, and only then to settle what holds
it.

**The 21–35 ns lookup is the figure to keep.** It is the one number here that is not about memory,
and it is what §5's `draw` measurement predicts: the scan goes away without the diagram changing its
algebra. **If the diagram gets an index at all, the index is what was wanted; persistence is a
second question riding along with it.** An `im`-style index on the current mutable `Vec` — a
`HashMap<ShapeId, usize>` beside it, rebuilt never because the order only changes on add and remove
— would deliver the same lookup for none of the costs in §5, and it is the smaller change to argue
about.

## 7. Legibility, and what Rust already has

The question this section answers is not which library is fastest — §6 measured that — but which one
leaves code that is easier to read, easier to keep correct, and harder to break by accident. **On
that question the ranking is the reverse of the memory ranking, and the middle option wins.**

### The three options, as code

The current model is 4 130 lines in `crates/monospace-diagram/src/diagram.rs`, and the whole of the
lookup story is one function:

```rust
fn find(&self, id: ShapeId) -> Option<usize> {
    self.shapes.iter().position(|placed| placed.id == id)
}
```

Adding a `HashMap<ShapeId, usize>` beside the existing `Vec<Placed>` replaces that with a lookup and
one new invariant. **Everything else in the file — `add`, `replace`, `forward`, `backward`, `draw`,
`remove` — is untouched**, because none of them depends on where a figure lives. That is the whole
change: about ten lines to build the index, about five to maintain it.

The persistent versions in §4 and §6 are a different shape of work. A persistent diagram replaces
`Vec<Placed>` with two structures that must agree with each other, and `remove` becomes a
reconstruction rather than a mutation. The trie in §4 was 519 lines including its harness; a
production version with tests would be larger than the change it replaces. **This is not an argument
against persistence — it is what persistence costs in a codebase whose diagrams are 300 shapes.**

### The invariant, and what happens without it

An index beside a vector is one line of code and one rule: **every index addresses a slot holding
that identity.** `add` keeps it. `replace` cannot break it. **`remove` breaks it immediately**,
because `Vec::remove` shifts every later slot down by one, and the fix is to walk the map and
decrement everything above the hole.

Measured, building it both ways:

|                                     | Time   | Correctness |
| ----------------------------------- | ------ | ----------- |
| Lookup, linear scan over 300        | 56 ns  | —           |
| Lookup, `HashMap` beside the vector | 10 ns  | —           |
| Remove, no index                    | 0.3 µs | —           |
| Remove, index with the fixup        | 0.6 µs | correct     |

**The invariant costs 0.3 µs per removal and buys a 5.6× faster lookup.** And omitting the fixup is
the failure mode worth writing down, because it is silent:

```text
added 1..=6, removed 2. Now asking for what is left:
  get(1) -> shape 0      correct
  get(2) -> absent      (correct)
  get(3) -> shape 3      WRONG: returns another shape
  get(4) -> shape 4      WRONG: returns another shape
  get(5) -> shape 5      WRONG: returns another shape
  get(6) -> PANIC: the index points past the end of the vector
```

**Three shapes answer with the wrong figure and nothing says so; the sixth panics.** The wrong
answers are worse than the panic, because they are what a diagram drawn to a terminal looks like — a
box where a line should be, with no error anywhere. The protection is not the type system, which
cannot see it, and not the compiler. It is one test that removes a shape and asks for the one after
it.

### What makes each option harder to break

| Option                     | Read by                              | The thing that can go wrong                           |
| -------------------------- | ------------------------------------ | ----------------------------------------------------- |
| Linear scan (today)        | Anyone                               | Nothing structural; the cost is linear                |
| `HashMap` beside the `Vec` | Anyone who knows `HashMap`           | **The index drifting from the order after a removal** |
| Hand-rolled trie           | Only its author                      | Everything, and §4's own numbers show how easily      |
| `rpds`                     | Anyone who has used a persistent map | The `add_under` half of §6, and `!Send`               |

**The hand-rolled trie is the least legible option available, and it is worth saying so plainly.** A
HAMT has collision nodes, depth, and a resize strategy, and this repository would own all of it. The
§4 prototype was written by someone who already knew what a HAMT is, and it still took two attempts
to get a per-node count right. **What §4 measured was the value of the idea, not a recommendation to
build it** — and §6 showed a library does the same job for a quarter less memory, which is the
honest answer to "should we write our own".

### So which one

**`std` first, and the library only if the diagram becomes persistent.** In order of how much each
one costs to be wrong about:

1. **Bound the history.** One line, no new structure, no invariant, and §5's comparison showed it
   caps the linear term outright. It also answers the question `docs/editing-model.md` §8 has been
   holding open.
2. **`HashMap<ShapeId, usize>` beside the existing `Vec`.** ~15 lines, one invariant, a test that
   catches its only violation, and a 5.6× faster lookup. The `remove` fixup is the whole risk and it
   is three lines.
3. **`rpds`, if and only if a persistent diagram is decided on its own merits** — a spec, an issue,
   and a `deciding` stage, for a model that must first settle what a diagram is after §5's `Clone`
   paragraph and what `!Send` means for a caller.

**Nothing above is free and the third is not cheap, but the first two are small enough to be
reversible, and neither asks the model a question it cannot answer today.** The measurement in this
document is the argument for doing something about a 40 KB-per-command history; it is not by itself
an argument for making the diagram persistent.

### One thing neither option settles

`add_under` permits two figures under one identity, and §6 measured that `get` returns the first and
ignores the second — **which means the second figure is currently unreachable by identity while
still being drawn.** That is true today, with a `Vec`, and the unreachable half is true of all four
options above. **Under any of them that holds a map, the second figure also stops being drawn,
because a map drops one of the two** — and §6 corrects its own first draft on which one.

**So the model has not answered two questions here, not one.** Whether one identity may name two
figures at all, and if it may not, what a diagram says when a caller names one twice. Neither is a
consequence of any of the four options, so neither should wait for one: both are questions about
`docs/diagram-model.md`, and together they are the one item in this document that is a finding about
the model rather than about its cost.

## 8. The decision, as the measurements leave it

Everything above is a measurement or a consideration. This section is the one that says what a
reader should do with them, and **it is a recommendation rather than a decision**: it is one
person's reading of the numbers, written down so that disagreeing with it means disagreeing with a
specific claim.

### What each option would buy, for what

The costs already measured, gathered into one table. **The order of the rows is the order the code
should be changed in**, which is not the order the sizes would suggest:

| Option                     | Buys                              | Costs                              | Asks the model |
| -------------------------- | --------------------------------- | ---------------------------------- | -------------- |
| Bound the history          | Caps the linear term at the bound | ~1 line, one number to choose      | No             |
| `HashMap` beside the `Vec` | Lookup 56 ns → 10 ns              | ~15 lines, one invariant, one test | No             |
| Intern the strings         | ~38.3 MiB → ~19 MiB at 1000 steps | A catalog and its lifecycle        | No             |
| `rpds` and persistence     | ~38.3 MiB → ~0.9 MiB, log growth  | New model questions, `!Send`       | Yes            |

**The first three are independent of each other and of the fourth.** Bounding caps what is retained;
an index makes lookups cheap; interning makes each copy smaller. Any two of them compound, and a
reader who wants only one place to start can start with the first and stop.

**The first two are the ones the code argues for, and neither needs a spec.** They are small, they
are reversible, and each has one way to be wrong that a single test catches. That is the whole
reason they come first — not because the memory they save is the largest, since interning saves
more.

### What would change this recommendation

The recommendation is a reading of measurements taken on one machine, one diagram shape and one
session length. Five things would move it, and they are worth stating so that a later reader knows
what to re-measure:

- **A reason to hold a diagram persistently that is not its memory.** This document ranks the four
  options by what they cost to be wrong about, and the last row is marked _Yes_ because persistence
  is the only one that asks the model a question. **A change may still take that row while reading
  the memory figures as the least of its reasons** — an immutable diagram is a discipline a caller
  can be held to, and a discipline is not measured in bytes. What that does not do is make the other
  three rows wrong: bounding the history, indexing the order and interning the strings remain
  independent, and interning remains the cheapest per unit of risk.

- **A diagram of real size.** 300 shapes is a guess. §5 measured drawing at 1200 shapes and found
  11.6 ms, which is the number that would start to matter, and this document did not measure the
  history at 1200 shapes. A real diagram from a real description would replace the guess.
- **A session that is actually long.** The whole case is 1000 commands. A tool where a session
  rarely exceeds fifty commands does not have this problem, and nothing here would say otherwise.
- **A caller that needs threads.** §5 records that nothing in this repository uses threads today. A
  decision to render on a background thread makes `!Send` a blocker rather than a cost, and the
  third row of the table above stops being optional.
- **A diagram the user edits by dragging.** §5 records that one command is one step is deliberately
  deferred. A drag would make the per-command cost a per-gesture cost and would change what a bound
  is measured against.

### The two things worth doing that are not on that list

**The `add_under` finding is independent of all four options and should not wait for one.** §6
measured that a second figure under a live identity is unreachable by identity while still being
drawn, and that is true today with a `Vec`; under a map it is also no longer drawn, because a map
drops one of the two. It is a question about `docs/diagram-model.md` §3 _Identity_, it is worth an
issue on its own, and every one of the four options above would have to be written against whatever
that issue decides.

**Interning is the cheapest of the four per unit of risk, and it was measured least.** Its figure is
a `size_of` calculation rather than an allocation count, and it introduces a catalog whose lifetime
nobody here owns yet — `Stroke` is compared, hashed and turned into glyph keys in `monospace-core`,
so an interned handle is a change to how strokes are named, not only to how they are stored. **It
halves the constant on the current model without changing the growth rate**, which makes it
attractive if the growth rate is going to be bounded anyway.

### What this document is for

It answers the question `docs/editing-model.md` §8 set — _the first diagram big enough for the
copies to be felt rather than measured_ — with a measurement rather than a threshold, and then
carries that measurement through the options. **It does not change the model, does not propose a
change, and does not answer _how many steps a session keeps_, which is still that document's to
answer.** The measurement exists so that the answer to that question can be an argument rather than
a guess.

## 9. What this settles and what it does not

**Settled:** the current model's memory is linear in commands × shapes, at roughly 40 KB per command
on a 300-shape diagram, with a peak barely above what is retained. Persistence with a persistent
order brings that to roughly 1 KB per command and to logarithmic growth in shapes. Interning the
strings halves the constant and is independent of both. **Of the two persistent structures measured,
`rpds` is both cheaper and smaller per change than the hand-rolled trie, so a library is the better
buy wherever persistence is decided at all** — and §6 records that this repository's own rule
prefers one anyway.

**And, from §7, the ordering the code argues for rather than the memory:** bounding the history and
then indexing the existing `Vec` together attack both measured costs — the 40 KB per command and the
linear scan — for about sixteen lines, one invariant, and one test, with no new structure and
nothing the model has to answer. **A persistent diagram is the only option on this list that asks
the model a question, and it should be answered on its own merits rather than as a way of buying the
other two.** Writing the HAMT by hand is the one option that is not worth taking on any grounds: it
is less legible than the scan it replaces and less correct than the crate that does the same job for
less memory.

**Not settled, and deliberately:**

- **Where the threshold is.** 300 shapes and 1000 commands were chosen for roundness. The linear
  coefficient is measured; the point at which a session becomes uncomfortable is a judgement about a
  machine and a user, and this document does not make it.
- **Whether to do any of it.** This is a measurement, not a proposal. `docs/editing-model.md` §8
  holds the question _How many steps a session keeps_ open, and bounding the history is a smaller
  and cheaper answer than changing the data structure; that comparison belongs in a spec, which is
  where a decision like it is argued. **§8 gives the ordering these measurements support and states
  what would change it, and that is a recommendation to weigh rather than a decision taken.**
- **Whether persistence is compatible with the model.** §5 works through what it reaches, and one of
  the six items there is a conflict rather than a cost: `add_under` holds that two figures may share
  one identity. **§6 first measured that conflict away and then took the measurement back: a map
  holds one figure per identity, which of the two survives is decided by the spelling rather than by
  the model, and today both figures are held and both are drawn.** The conflict is the size §5 said
  it was. What a persistent diagram still reaches is §5's `Clone` paragraph, the meaning of a
  diagram as a value, and `!Send` on a caller that does not exist yet. **This is a change with a
  decision to agree before any code is written, and it wants a branch of its own.**
- **Whether one identity may name two figures at all.** §7 measured that the second is unreachable
  today and under every option considered here, while still being drawn. That is a question about
  `docs/diagram-model.md` §3 _Identity_ rather than about any of these costs, and **it is the one
  finding here that does not depend on a decision nobody has taken.**
- **How much of the cost the renderer bears.** The diagram and the session are measured here, and §5
  measures drawing, but a `Buffer` and the glyph catalogue are not — and a diagram drawn at every
  event is a separate question from a diagram held.

## Reproducing

The harness is not in this repository and nothing here depends on it being restored; the figures
were taken on Windows, x86-64, Rust 1.98.1, `--release`, with a single 300-shape diagram and 1000
commands unless stated otherwise. **The numbers that will not reproduce exactly on another machine
are the times in §3, §4, §5, §6 and §7, and the allocation totals, which include the allocator's own
rounding**; the sizes in §2, the linearity in §3, the per-step breakdown in §4, the comparison in
§6, the lookup ratio in §7 and the ordering in §8 are properties of the data structures and hold
anywhere. The `rpds` figures were taken against `rpds` 1.2.1 as published on 2026-05-15; a later
version may pack its nodes differently and should be re-measured rather than assumed.
