---
status: abandoned
date: 2026-09-07
---

# 0004 — Give a glyph a type of its own

> **Abandoned, and kept as an input.** This spec was a draft when the project moved its spec home to
> Spec Kit, so it was never agreed and never implemented. It is not extended or corrected from here.
> What it asks for is still wanted: it is the raw material for the Spec Kit feature that replaces
> it, which is authored from this file rather than from scratch. See
> [the directory's note](README.md) and
> [ADR-0021](../decisions/0021-move-the-spec-home-to-spec-kit.md).
>
> **Replaced by [`specs/006-give-a-glyph-a-type/`](../../specs/006-give-a-glyph-a-type/spec.md).**
> That feature is where the agreed rules live, and it supersedes what this file prescribes wherever
> the two differ — including the shape of the increments below, which planning it found could not be
> built as written. Read this one for what was wanted; read that one for what was agreed.

## Why now

Two queued slices put glyphs where this library has never had them. A cell will hold a literal glyph
instead of deriving one from its arms, which means a glyph arriving from a caller. Glyph sets will
be loaded from a file, which means one arriving from outside the process. Today there is nothing to
receive either as: a glyph is a `char`, which admits `\n` and rejects `é` written as `e` plus a
combining acute.

Doing it before either of them means both arrive to a validated type that already exists, instead of
inventing validation while also learning to parse a file or to compose a new kind of cell. And it is
the half that changes nothing: no output moves, and the front end is not touched.

[ADR-0019](../decisions/0019-represent-a-glyph-as-a-grapheme-cluster.md) carries the decision and
its costs, including the dependency. This spec is what it looks like when implemented.

## Scope

### In

- `Glyph`, one validated grapheme cluster, with `Glyph::new` as its only constructor.
- The catalog holding and answering with `Glyph` rather than `char`.
- The fifteen built-in rules of the Light table validated when the catalog is built.
- `render` collecting glyphs rather than characters.
- The dependency ADR-0019 accepts, `unicode-segmentation`.

### Out

Every item names where it is handled instead.

- **Cells that hold a literal glyph.** The next spec, and one of the two consumers this exists for.
  Nothing in this slice puts a `Glyph` anywhere near a `Cell`.
- **Loading glyph sets from a file.** A later spec, and the other consumer. It is what turns
  `Glyph::new` returning `None` into a message someone reads.
- **Column width.** ADR-0019 leaves it out of the invariant deliberately: a wide grapheme is
  accepted and shifts the rest of its row by a column. Documented, not enforced, and not this
  slice's to revisit.
- **Anything about strokes.** `Stroke` keeps its public field and its lack of validation. ADR-0019
  says why the two look alike and are not.
- **Everything spec 0001 left out and this one does not name** — degradation, a catalog from more
  than one set, walking a region — stays out, with the destinations that spec gave.

The near miss is the rendered output. Not one character of it moves, in the front end or in any
test, because every rule in the Light table is a single character and a single character is a
grapheme cluster. That is the point: this slice widens what the library can hold, not what it holds.

## Model slice

Implements _Strokes, glyph sets and the catalog_ and _Rendering_ exactly as they already stand: what
a catalog is, how a key is answered, and the two-step lookup are untouched.

What changes is one line of the vocabulary. `Glyph` read "the character a cell renders to" and now
reads "what a cell renders to: one grapheme cluster", because a character is the thing this slice
stops being enough. The amendment is committed with ADR-0019, which takes the decision it follows.

**One rule of spec 0001 is restated.** Its rule 6 promises lines of exactly `width` **characters**.
Once a glyph may be several code points, a line is `width` **glyphs**, which is the same rectangle
and no longer the same character count. Nothing observable changes here — every built-in glyph is
one character — but the promise has to be read in cells from now on, and the slice that first stores
a multi-character glyph is the one that would otherwise have broken it silently.

## Public surface

```rust
pub struct Glyph(/* private */);

impl Glyph {
    pub fn new(text: &str) -> Option<Self>;
    pub fn as_str(&self) -> &str;
}

impl GlyphCatalog {
    pub fn glyph(&self, key: &GlyphKey) -> Option<&Glyph>;
}
```

- **The payload is private, and `new` is the only way in.** That is the whole confirmation ADR-0019
  claims: an invalid glyph cannot exist, so there is nothing to catch downstream. It is also what
  makes the inside changeable later without a caller noticing.
- **`new` returns `Option`, not `Result`.** There is one way to fail and nothing to say about it
  beyond "not a glyph"; an error type would be a value nobody branches on. The spec that loads a
  file is where a rejection becomes a message, and what that message needs — which file, which row —
  is context the loader has and `Glyph` does not.
- **`as_str` rather than a `char` accessor.** A glyph is not always one `char`, and the only thing
  the library does with one is write it into the rendered string.
- **`glyph` answers with a borrow.** The catalog owns its rules and a glyph is no longer `Copy`;
  returning by value would clone once per rendered cell. `render` collects `&str` and its space
  fallback is `" "`.
- **`Glyph` derives `Debug`, `Clone`, `PartialEq` and `Eq`.** `Debug` and the comparisons are what
  the tests assert through; `Clone` because the inside owns its storage. Not `Copy`, which is the
  cost ADR-0019 records.

Every public item carries rustdoc. `Glyph`'s says what it accepts, and says in the same breath that
width is not part of the invariant — that a wide grapheme is one glyph and will shift its row.

## Behavior

1. `Glyph::new` answers `Some` when its text is exactly one grapheme cluster and contains no control
   character, and `None` otherwise.
2. Empty text is not a glyph.
3. Text of more than one grapheme cluster is not a glyph.
4. A grapheme cluster built from several code points is one glyph: `é` as `e` followed by U+0301, a
   regional-indicator pair, an emoji with a modifier.
5. A control character is not a glyph, including when it is part of a single cluster.
6. `GlyphCatalog::light()` answers every key the Light table gives a glyph for, with that glyph, and
   answers no other key — spec 0001's rule 10, unchanged except in what the answer is made of.
7. `GlyphCatalog::light()` panics if one of its own rules is not a valid glyph. That is a bug in
   data this library ships, not a condition a caller can act on, and it is the one place where
   failing loudly beats failing as a `None` that renders as a space.
8. `render` produces, for any catalog whose glyphs are single characters, exactly what it produced
   before this slice.

Rule 7 brings a `# Panics` section with it: `clippy::missing_panics_doc` is enforced by the gate and
will require one on `light()` the moment the panic exists. The learning log already records this
trade for `render`, where the same section arrived with an `expect` and left when the panic did.

## Examples

**What is a glyph.**

| Text         | `Glyph::new` | Why                                                    |
| ------------ | ------------ | ------------------------------------------------------ |
| `"│"`        | `Some`       | one cluster, one character                             |
| `"e\u{301}"` | `Some`       | `é` decomposed: two code points, one cluster, rule 4   |
| `"🇦🇷"`       | `Some`       | a regional-indicator pair is one cluster               |
| `""`         | `None`       | rule 2                                                 |
| `"ab"`       | `None`       | two clusters, rule 3                                   |
| `"\n"`       | `None`       | rule 5                                                 |
| `"\r\n"`     | `None`       | one cluster by UAX #29, and control characters, rule 5 |

The last row is why the invariant has two halves rather than one. `"\r\n"` passes the cluster test
and has to be refused anyway, so "exactly one cluster" alone would have let the one input through
that breaks the rendered rectangle instead of merely shifting it.

**From a key to a glyph.** The example spec 0001 named "From a cell to a character", now one step
longer. The key with `light` on top and bottom and nothing on the sides answers a glyph whose
`as_str()` is `"│"`:

```rust
let catalog = GlyphCatalog::light();
let key = GlyphKey {
    top: Some(Stroke::from("light")),
    right: None,
    bottom: Some(Stroke::from("light")),
    left: None,
};

assert_eq!(catalog.glyph(&key).map(Glyph::as_str), Some("│"));
```

This is the shape every existing assertion on `glyph` takes after this slice: `Some('│')` becomes
`Some("│")` through `as_str`. The change is mechanical and it is the whole of what the first commit
does to the tests.

**Nothing moves.** `cargo run -p monospace-cli` prints the three blocks spec 0003 left — the single
box, the pair stamped `Above`, the pair stamped `Below` — byte for byte, and
`crates/monospace-cli/tests/cli.rs` is not touched. A glyph that is one character renders as that
character, and every rule this library ships is one character.

## Acceptance

- [ ] `cargo xtask check` passes.
- [ ] `cargo run -p monospace-cli` prints exactly what it printed before, and
      `crates/monospace-cli/tests/cli.rs` is unchanged. That file not appearing in the diff is what
      says this slice changed nothing observable.
- [ ] A test per behavior rule 1 to 5, named after what it asserts, covering each row of the table
      above.
- [ ] A test walks all fifteen rules of the built-in Light table and asserts each one constructs as
      a glyph. It is what rule 7's panic is protecting, and it is the reason the panic never fires.
- [ ] The existing catalog and render tests keep asserting what they assert, through `as_str`. The
      edit is mechanical and no test is added or removed in the commit that makes it.
- [ ] `unicode-segmentation` is added to `crates/monospace-core/Cargo.toml` with an exact version at
      least seven days old, and the version and its publication date are named in the commit that
      adds it.
- [ ] Every new public item has rustdoc, and `Glyph`'s says that width is not part of the invariant.

**Two commits, in this order.** The first introduces `Glyph` with the signature above and a `char`
inside, accepting exactly one non-control character, and threads it through the catalog and the
renderer. The second widens the inside to a grapheme cluster and adds the dependency. Splitting them
this way is what keeps the second commit's diff equal to what the dependency buys: because `new`
already takes `&str` and already returns `Option`, no call site and no table row moves twice.

The first commit modifies existing test assertions, which the repository's rule for a structural
commit does not allow. It is a refactor rather than a structural commit by that rule's own
definition, the edits are type-level and mechanical, and the commit message says so rather than
leaving a reader to notice.

## Open questions

**Whether a glyph should carry its storage inline.** An allocation per glyph is invisible for
fifteen rules and a nine-cell diagram, and is not invisible for a page of text. ADR-0019 accepts it
and says the answer would be an inline representation inside this same type, which no caller would
see. What would settle it: the first slice that stores text in volume, and a measurement rather than
a guess.

**Whether width comes back.** It is out of the invariant, so a wide grapheme shifts its row by a
column and the rectangle stops looking like one. Nothing in this slice can produce that, since every
built-in rule is narrow; the first one that can is the slice that lets a caller supply a glyph. What
would settle it: seeing a diagram with an emoji in it, and deciding whether the shift is a defect or
the documented price.
