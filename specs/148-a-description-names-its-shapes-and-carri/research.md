<!-- cspell:ignore serde -->

# Phase 0 research: a description names its shapes, and carries the ordinal the next one takes

Six questions, all module-level, all answered here. The four that are the maintainer's are on
[decisions.md](decisions.md), not repeated. Each answer below lives inside one crate or one file and
is undone by changing the code that gives it, so none takes a record of its own.

## The picture, and why no entry on the sheet carries one

Every option pair on the sheet draws **the same picture**. That is the finding rather than an
absence: what the four entries choose between is a number nobody sees, a promise the diagram makes,
and which of two crates holds a name — and a picture beside an option pair it cannot tell apart is
decoration.

**No marker can carry this slice's evidence either**, and `spec.md` says so before this file does: a
named shape is not expressible in the format `xtask` reads, so a marker would draw the
position-named picture while claiming the named one — the exact confusion the slice exists to end
([ADR-0064](../../docs/decisions/0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md)).
The specification therefore draws B1.1's pair by hand, labels it as measured, and this file cites it
rather than repeating it.

What was run, on 2026-09-30 with `cargo run -p monospace-cli`, is three files: two boxes and one
connector over a fourteen-by-three window. The first two are B1.1's pair — the entries in both
orders, the reference naming `#2` both times — and they reproduce the specification's two pictures.
The third keeps the entries swapped and names `#1`, and draws byte for byte the first one: the place
moved the arrow in the second file, the name does not in the third. That is B1.2, measured.

Each was also run with **no** `id` on any entry, and each came out exactly as it does with one,
because a field the format does not know is dropped in silence. So no picture moves when this slice
lands — and the second file's does not move either, which is the point rather than a comfort: today
its arrow is where its **position** puts it, and a marker would have claimed otherwise.

## Q1: What the diagram's counter is, and what the new entry point looks like

**Decision**: a `u32` meaning **the ordinal the next `add` takes**, so `add` builds `#next` and then
increments. `numbered_from(next: u32) -> Diagram` seeds it, and
`add_under(&mut self, id: ShapeId, shape: Shape)` places a shape under a name the caller wrote.

**Rationale**: measured. The field is `next: u32` and `add` increments **before** use
(`crates/monospace-diagram/src/diagram.rs:44` and `:57`) — invisible while the only source is
`Diagram::new()`, and an off-by-one in a public seeding method: `numbered_from(27)` would have to
store `26` for the next `add` to hand back `#27`. Storing the ordinal itself removes the trap.

**Alternatives considered**: `Option<u32>`, which makes every `add` carry a match for a case a
seeded diagram cannot be in; keeping the increment and documenting the subtract, which is that trap
with a comment on it; a `usize`, which is what `Vec` indexes with and not what an identity is
written in.

## Q2: What `add_under` hands back, and whether it touches the counter

**Decision**: nothing, and it does not touch the counter.

**Rationale**: measured against the specification. B3.1 says two entries carrying one identity are
**both** held and the first is what every change finds, so `add_under` cannot fail and there is no
`None` to return — the reason `remove` and `replace` hand back nothing. And the sheet puts the
promise on the file, so `next_id` is where numbering resumes and nothing a caller writes moves it:
`add_under("#7")` on a diagram numbered from 3 leaves it at 3, and the next `add` is `#4`. That is
D2's accepted cost in one sentence, and what the specification's Edge cases mean by an identity the
diagram issues.

**Alternatives considered**: `-> Option<ShapeId>`, which has no absent case to report; advancing the
counter past any `#N` handed in, which is the alternative D1 declines and would put a `#N`
convention inside a crate whose `ShapeId` is any string.

## Q3: What the two new fields are called, and where they sit

**Decision**: `"id"` on every entry, `"next_id"` beside `canvas` and `shapes`. Both required, so a
missing one is refused by name the way `canvas`, `shapes`, `leaving`, `terminal` and `at`'s `kind`
already are. `id` sits immediately after `kind`, where all 25 existing descriptions and
`assets/demo.json` already put their first field, so the edit is an insertion on a familiar line.
**Rationale**: measured. An unknown field is dropped in silence and a required one is a
`missing field` error, so a **misspelled `id` is caught** and a misspelled extra key is not — the
format's existing asymmetry, not something this slice adds. `id` rather than `shape` because `shape`
is already what a `reference` calls the shape it names, and one word meaning two things inside one
format is the ambiguity `kind` exists to remove. `next_id` rather than `counter` because the field
is an identity and not a count, and a name that says counter invites `shapes.len()` at the reading
end, which is the derived answer D1 declines.

**Alternatives considered**: `"name"`, which says nothing about the thing being an identity;
`"ids": { "next": 27 }`, a container holding one number; the ordinal on every entry rather than
once, which puts a piece of the diagram's bookkeeping inside a figure the model says it does not
belong to.

## Q4: What the format change reaches in the tracked tree

Over `git ls-files '*.md'` with `xtask`'s own rule — a marker inside a fence illustrates the grammar
and is not an instance of one (`xtask/src/render.rs:211`) — there are **25 markers carrying a real
description, 65 shapes across them**, in eight files, plus `CONTRIBUTING.md`, whose one marker sits
inside a ` ```markdown ` fence. `cargo xtask render` re-draws all 25 and reports each one, and the
specification says 23 of 24. Recorded rather than quietly adopted, per principle IV: the difference
is one marker and two shapes, and `docs/diagram-demo.md` — 12 markers, 38 shapes — was added in
`4c9cf9f`, after 083 measured 13. Beside those, `assets/demo.json` holds 26 entries and four Rust
files hold 22 `"kind"` occurrences between them.

The characterization is cheap: `sweep.rs` builds its cases as text from three shape templates and
one crossing pair (`crates/monospace-cli/src/sweep.rs:65` and `:131`), so 1856 renderings cost four
edits and none of the eight snapshots moves — a snapshot pins a rendering, not a description.

## Q5: What the demonstration does, and which comments become false

**Decision**: nothing changes in `demonstrate`. `#1`, `#3` and `#10` stay written out by hand, and
still name the same three figures because the shipped file names its entries that way.

**Rationale**: B2.3 asks for that, and it is the cheaper half. Reading the names back needs a
listing a diagram deliberately does not offer — `get`'s own rustdoc says "no listing of a diagram's
identities, no count, no order" — which this slice also declines to add.

Two comments become false and are corrected in the commit that makes them so, per 082's Q7 and 083's
Q6: `main.rs:104` and `main.rs:121` both say a description names its shapes by position and has
nothing to read. And the workaround at `main.rs:586` — `demo_without_its_first_entry` renumbering
`"#5"` to `"#4"` — goes with the comment explaining why, being the cost this slice removes.

## Q6: What the model says, and what the record says

**Decision**: §3 amended twice in the same paragraph — a caller may choose the identity a shape is
added under, and "unique within that diagram" becomes a sentence about the identities **the diagram
issues** rather than a promise it keeps on a caller's behalf. §9's `add` row names both ways in.
§11's first question comes out and its surviving half is rewritten in place rather than left naming
a trigger that has already fired. One new ADR, at `load-bearing`.

**Rationale**: D2 is why "unique" cannot stay a promise — the counter is trusted rather than
checked, and B3.1 accepts a repeated name — so the model's sentence has to say which of the two it
is about. `load-bearing` by the constitution's own test: something outside the module depends on it,
since `monospace-cli` calls both methods, and reversing means 91 identities leaving 26 files. Sized
to its subject, so a short record inside a 150-line ceiling.

**Alternatives considered**: no ADR, on the argument that 082's D3 and 083's D3 answered the same
question in a sheet — which is what principle VI forbids for a domain-level decision.
