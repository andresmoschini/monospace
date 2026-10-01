---
status: accepted
scope: domain
commitment: load-bearing
date: 2026-09-30
decision-makers: Andrés Moschini
---

<!-- The feature directory named under More Information is the issue title truncated at forty
     characters by `cargo xtask spec`, which landed inside a word: cspell:ignore carri -->

# Let a caller choose an identity, and say where the numbering resumes

## Context and Problem Statement

A diagram issued its shapes' identities itself, in array order: `add` handed back `#1`, `#2`, and so
on, and a reference written `"#5"` meant the **fifth entry of that list**. The identity was
therefore a position, and everything downstream inherited that: two descriptions holding the same
two boxes and the same connector, differing only in the order they listed the boxes in, drew two
different pictures, because the reference's `#2` was a different box in each.

That also put a cost in the test suite rather than in the code. `demo_without_its_first_entry`
removes the first listing from the shipped demonstration and re-reads the text, and while identities
came from array order that shifted every name after it. The helper carried a loop rewriting `"#5"`
to `"#4"` whose comment said, in as many words, that this was "the whole of what a positional
identity costs". The workaround was in the suite because the property was in the format.

## Decision Drivers

- **A reference must name a shape rather than a place**, so reordering a file's entries cannot move
  a connector by accident. That is the model, not this file format: `docs/diagram-model.md` §3.
- **The numbering a file hands on must survive being read.** A shape added to a diagram read from a
  file continues that file's numbering rather than restarting it, and the gaps between are unused.
- **The model owns the rule; the file format is the application's**
  ([ADR-0035](0035-keep-the-cli-demo-format-out-of-the-model.md)). A rule observable from outside
  `monospace-diagram` is the model's; the two fields that carry it across a wire are this crate's
  own business.

## Considered Options

**There is no picture here, and the absence is the finding rather than an omission.** Every option
pair draws the same picture: `numbered_from` shows itself only in what `add` hands back, `add_under`
not moving the counter is invisible in any rendering, and two entries under one identity draw
exactly what two entries under two draw. What is chosen between is a number nobody sees and a
promise the diagram makes. A `<!-- render: -->` marker could not help even in principle: a named
shape is not expressible in the format `xtask` reads, so a marker would draw the position-named
picture while claiming the named one — the exact confusion this record exists to end
([ADR-0064](0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md)).

- The counter keeps the last issued ordinal and `numbered_from` subtracts on the way in.
- The counter is an `Option<u32>`, and every `add` matches on it.
- The counter means "the ordinal the next `add` takes", stored as itself, with no subtraction
  anywhere.
- `add_under` hands back `Option<ShapeId>`, or checks that the name is unused, or advances the
  counter past any `#N` a caller wrote.
- `add_under` hands back nothing, checks nothing, and does not move the counter.
- The ordinal is read off the identities the file carries rather than carried as a field of its own.

## Decision Outcome

Chosen option: **the counter is the ordinal the next `add` takes, stored as itself; `add_under`
writes the caller's name, checks nothing and hands back nothing** — because a name that moves when
the order moves is not a name, and an off-by-one inside a public seeding method is the one kind of
bug nothing in a signature catches.

### Consequences

- Good, because a description's entries can be listed in any order and a reference naming a box
  still finds that box. The loop in `demo_without_its_first_entry` is gone, and with it the comment
  that explained it; the same `assert_eq!` it existed to hold in place is now a claim about the
  model.
- Good, because `numbered_from` reads the way its parameter is named, and the assertion that pins it
  — `numbered_from(3)` hands back `#3`, asked by the identity's own text — fails in exactly that one
  place if the convention is reversed and nowhere else.
- Good, because nothing outside `monospace-cli` has to change: the counter is a `u32` and the
  placement is a `Vec::push` of the same `Placed` an `add` pushes, so `monospace-core` does not move
  and the gate's `wasm` step already compiles both methods.
- **Bad, because `next_id` is trusted rather than checked.** A stale value — an entry renamed, a
  `#2` deleted — hands back an identity already in use, and the shape that arrives is one nothing
  can name. Accepted with **nothing to verify it**, and named as such in the format contract and
  here rather than described as tested.
- **Bad, because a repeated identity is not refused.** Two entries may carry one; both are held, the
  first is what every change and every reference finds, and the second is a shape nobody can name
  until the first is removed. Pinned as a behavior by two tests, one at each level, so a later slice
  that decides to report it has to say so rather than discover it.
- Bad, because "unique within that diagram" had to stop being a promise. §3 is amended in the commit
  that follows the code, and the rustdoc on `ShapeId` says the same thing: uniqueness is now a claim
  about what the diagram **issues**.

### Confirmation

Four contract tests, each asked against the model rather than against the other side of a
comparison: a chosen identity is found by that identity and by no other one; the ordinal is asked
rather than drawn; a repeated identity is held and `get` returns the first; a missing identity is
refused by name. `cargo xtask check` compiles `monospace-diagram` for `wasm32-unknown-unknown`,
which covers both new methods. Only review enforces that no picture was added here.

## Pros and Cons of the Options

Each option above is one the sheet carried; the six below are what each of them would have cost, and
the two that were close enough to argue are given their Good half as well.

### Store the last issued ordinal, and subtract on the way in

- Good, because `add` is untouched and one line in a new method is a small diff.
- Bad, because `numbered_from(27)` has to store `26` for the next `add` to hand back `#27`: an
  off-by-one in a public method whose signature cannot catch it, with the parameter's name lying
  about what it means. Rejected.

### Advance the counter past any `#N` a caller wrote

- Good, because an issued identity could then never collide with a written one.
- Bad, because it puts a `#N` convention inside a crate whose `ShapeId` is any string, and it
  contradicts the field a file carries: a file says where **its** numbering resumes, and nothing a
  caller writes should move that. Rejected.

### `Option<u32>`, `add_under` reporting a collision, or deriving the ordinal

The three left, each rejected for one reason: a seeded diagram can never be in the absent case, so
`Option<u32>` makes every `add` carry a match for an unreachable arm; there is no collision to
report the caller has not already been told about, so a `Result` with no error case promises a check
nobody asked for; and identities are free text, so there is nothing to count and a file naming `#1`,
`#7` and `#9` could not say what comes next. Each would have given something up front — a `Option`
says outright that a diagram may have no opinion about numbering, a collision report is more useful
than absorbing one, and a derived ordinal means a file cannot hold a counter that goes stale. None
of the three is worth a cost like the two above.

## Reversibility

Undoing the counter's meaning is cheap today and gets expensive at the first file that depends on
it: the convention lives in 43 descriptions and in one method's public contract, so changing it
means re-spelling all of them and every assertion that asks for `#3` from `numbered_from(3)`.

Undoing `add_under` is **permanent from the moment it lands**: the 114 identities already written
into the repository's descriptions would have to leave again, and a reader that had started
honouring them would go back to naming places. That is the constitution's own test for
`load-bearing` and not a judgement call — something outside the module depends on it, and reversing
it means migrating data. The two accepted costs above are not part of that: each is one line, and
each is pinned so that the slice which repairs one says so rather than discovers it.

## Confidence

High (90%).

Nothing here is reasoned rather than observed: both methods are exercised by the shipped
demonstration on every run. What would change it is a consumer that wants to ask a diagram which
identities it holds — a listing, which this slice declines, and which would turn `add_under` from a
way to name a shape into a second reader. What would prove the stale-counter cost wrong is a file
whose author deletes an entry and does not notice.

## More Information

- The model this amends: `docs/diagram-model.md` §3 _Identity_, §9 _Changing a diagram_ and §11
  _Open questions_, all three amended in the commit that follows the code.
- The file format it reaches, and the whole of what has to be re-spelled when its two fields stop
  being optional:
  [148's description-format contract](../../specs/148-a-description-names-its-shapes-and-carri/contracts/description-format.md).
  It supersedes 083's and is the application's own interface rather than the model's
  ([ADR-0035](0035-keep-the-cli-demo-format-out-of-the-model.md)).
- The silent hole a name arrives at, unchanged from a number:
  [ADR-0041](0041-resolve-a-position-through-a-reference.md), with
  [#88](https://github.com/andresmoschini/monospace/issues/88) as where it is answered.
- Why no marker could carry this record's evidence:
  [ADR-0064](0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md).
