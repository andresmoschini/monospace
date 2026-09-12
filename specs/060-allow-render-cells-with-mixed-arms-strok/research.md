# Phase 0 Research: Allow render cells with mixed arms' strokes

The spec's Assumptions settle the questions a normal Phase 0 would raise about scope and process: no
new dependency, no new crate, no new ADR —
[ADR-0037](../../docs/decisions/0037-give-each-arm-its-own-stroke.md) already made the one decision
this feature depends on. What is left is how that decision, and
[ADR-0009](../../docs/decisions/0009-degrade-a-cell-to-its-base-stroke.md)'s degrade rule, are
actually expressed in code — both were designed in prose before this feature and neither has a line
of implementation yet.

## How `Arm::Set` carries its stroke, and where the degrade step lives

**Decision**: `Arm::Set` becomes `Arm::Set(Stroke)`. `Arm` drops `#[derive(Copy)]` (keeps
`Clone, Debug, PartialEq, Eq`). `StrokeCell::key()` builds its `GlyphKey` from each arm's own
stroke:

```rust
pub fn key(&self) -> GlyphKey {
    let side = |arm: &Arm| match arm {
        Arm::Set(stroke) => Some(stroke.clone()),
        Arm::Closed | Arm::Unset => None,
    };
    GlyphKey {
        top: side(&self.top),
        right: side(&self.right),
        bottom: side(&self.bottom),
        left: side(&self.left),
    }
}
```

A second method, `degraded_key`, builds the key ADR-0009 calls "the uniform one": every side that
`key()` would answer `Some` for becomes `Some(self.base.clone())` instead; `Closed` and `Unset`
still answer `None`.

```rust
pub fn degraded_key(&self) -> GlyphKey {
    let side = |arm: &Arm| match arm {
        Arm::Set(_) => Some(self.base.clone()),
        Arm::Closed | Arm::Unset => None,
    };
    GlyphKey {
        top: side(&self.top),
        right: side(&self.right),
        bottom: side(&self.bottom),
        left: side(&self.left),
    }
}
```

`Cell::glyph_str` tries `key()` first and `degraded_key()` second, exactly the two lookups ADR-0009
describes:

```rust
Cell::Strokes(cell) => glyphs
    .glyph(&cell.key())
    .or_else(|| glyphs.glyph(&cell.degraded_key()))
    .map(Glyph::as_str),
```

**Rationale**:

- This is the model's own vocabulary, not a translation of it: _Rendering_ in `docs/model.md`
  already lists exactly these two lookups, in this order, with `Closed` and `Unset` reading the same
  at key-building time. Nothing here is a new rule.
- Two small, named methods read at the one call site that needs them (`Cell::glyph_str`) keep the
  "why that character?" answer to one sentence apiece, matching ADR-0009's own driver: predictable
  beats faithful, cheap to state.
- No third lookup and no loop over subsets: option A was chosen in ADR-0009, and this is its direct
  expression, not a reopening of that choice.

**Alternatives considered**:

- **One method returning both keys, or an iterator of keys to try in order.** Rejected: two lookups
  is the whole rule, and a `Vec<GlyphKey>` or a tuple return obscures that count instead of showing
  it. Two named methods read like the two sentences ADR-0009 already uses to describe itself.
- **Build the degraded key inline in `Cell::glyph_str` rather than as a `StrokeCell` method.**
  Rejected: `key()` already lives on `StrokeCell`, next to the arms and the base stroke it reads; a
  sibling method keeps both lookups next to the data they are built from and next to each other,
  where a future reader of `cell.rs` finds both without following a call into `render.rs`.
- **Give `Arm::Set` an `Option<Stroke>` defaulting to the cell's base at construction time**, so a
  caller need not repeat the stroke on every arm. Rejected: this is exactly Option B from ADR-0037,
  already rejected there for the reason that matters here too — it reintroduces two ways to express
  "this side is the base stroke," and `key()` would have to resolve the `None` case against `base`
  anyway, which is the coupling ADR-0037 exists to remove.

## Updating ~112 `Arm::Set` construction sites

**Decision**: every existing `Arm::Set` in `monospace-core` (shapes, fragments, and their tests)
becomes `Arm::Set(stroke)`, where `stroke` is the same value already given to that same cell's
`base` field. No site changes which sides are `Set`, `Closed` or `Unset` — only what each `Set` now
carries, and in every existing site that value is the one stroke the cell already draws in.

**Rationale**: every shape this project has today draws in one stroke per stamp, so `base` and every
`Set` arm's new payload are the same value at every site that exists — this is exactly the
"restriction" ADR-0012 described and ADR-0037 lifts: nothing already built could ever construct a
cell whose arms disagree, so no site needs a second stroke value invented for it. `Arm` losing
`Copy` means a handful of these sites move from a bare identifier to `.clone()`; the compiler finds
every one, which is why this is mechanical rather than exploratory.

**Alternatives considered**:

- **Keep `Copy` by using `Stroke` behind a cheap handle (an index or an `Rc<str>`) instead of an
  owned `String`.** Rejected here: ADR-0037 already weighed this cost (bad consequence: four
  redundant allocations per connected cell) and accepted it, deferring interning to a future
  measured need rather than this feature. Revisiting `Stroke`'s representation is out of scope for a
  feature whose spec's Assumptions describe it as applying ADR-0037, not reopening it.

## The four mixing tables

**Decision**: four new `Row` consts in `monospace-glyph-sets/src/lib.rs` — `LIGHT_DOUBLE` (18 rows),
`LIGHT_HEAVY` (50), `LIGHT_ROUND_DOUBLE` (18), `LIGHT_ROUND_HEAVY` (50) — copied verbatim, in
document order, from `docs/glyph-sets.md`'s four _Mixing …_ sections, through the existing private
`build(table: &str, rows: &[Row]) -> GlyphCatalog` helper feature 056 already extracted. Four public
functions follow the existing `double()`/`heavy()`/`light_round()` shape and naming:

```rust
#[must_use]
pub fn light_double() -> GlyphCatalog {
    build("Mixing Light and Double", LIGHT_DOUBLE)
}
// light_heavy(), light_round_double(), light_round_heavy() follow the same shape.
```

**Rationale**: this is feature 056's finding, restated because it applies unchanged — a table is a
`GlyphCatalog` built from its own rows through the one shared helper, and a fifth through eighth
table cost one `const` and one three-line function apiece. Naming after the two strokes a table
mixes, in the order `docs/glyph-sets.md` names them, keeps every table's public name readable
without needing a comment to explain it, mirroring how `light_round` already reads as "the Light
Round table" rather than an abbreviation.

**Alternatives considered**:

- **One function taking two stroke names and returning whichever mixing catalog matches, or
  `None`.** Rejected: it is a runtime failure mode (a typo'd stroke name) standing in for what four
  functions make a compile error, and it does not mirror how `double()`/`heavy()`/`light_round()`
  are already reached (FR-010's "no privilege a table written outside this project would not equally
  have" cuts the other way too — a hand-written mixing table has no such dispatcher to register
  with).
- **Fold the four mixing tables into the four single-stroke ones they extend** (e.g., one
  `light_and_friends()` returning Light plus its two mixing tables already merged). Rejected: User
  Story 3 requires picking a mixing table independently of the single-stroke tables it mixes ("wants
  Light Round to mix the way Light does"), which a pre-merged function forecloses.

## Reconciling `docs/model.md` with what ships

**Decision**: correct `docs/model.md`'s _Strokes, glyph sets and the catalog_ section — "only the
mixing sets are left loaded from a file when someone asks for them" — to say the four mixing sets
ship as data in `monospace-glyph-sets`, the same way ASCII, Double, Heavy and Light Round already
do, alongside the correction FR-012 already asks for in `docs/glyph-sets.md`.

**Rationale**: the spec's Assumptions say `docs/model.md` needs no change, reasoning from
_Rendering_ and _Worked examples_, which already describe the mixed key and are correct as written.
This one sentence, elsewhere in the same document, describes today's _implementation status_ rather
than a rule, and this feature is precisely the event that sentence was describing the absence of.
Leaving it would make the model actively wrong rather than merely silent, which is a worse state
than the one-sentence fix costs. This is a documentation-accuracy correction, not a new design
decision: the rule the model states — a set is a set, mixed or not, with no special behavior once it
is in a catalog — needs nothing added or changed.

**Alternatives considered**:

- **Leave `docs/model.md` untouched, per the spec's Assumption, and treat the stale sentence as
  future cleanup.** Rejected: the constitution gives `docs/model.md` ownership of design intent, and
  a sentence describing the four mixing tables as not-yet-loaded, in the same commit that loads
  them, is not intent — it is an error a reader would trip on immediately after this feature ships.

## No new dependency, no new crate, no new ADR

Unchanged from features 054 and 056's findings for `monospace-glyph-sets`, and confirmed for
`monospace-core`: nothing beyond the workspace's existing path dependencies is needed anywhere in
this feature. [ADR-0037](../../docs/decisions/0037-give-each-arm-its-own-stroke.md) already records
the one decision every part of this plan depends on; nothing surfaced while researching it rises to
a second one.
