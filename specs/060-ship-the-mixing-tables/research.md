# Phase 0 Research: Ship the mixing tables

The spec's Assumptions already settle what a normal Phase 0 would raise — no new crate, no new
dependency, no ADR, the interface mirrors the single-stroke tables feature 056 shipped. Two things
are left open: how the four new functions are expressed, and how the demonstration's placements are
corrected so each pair lands on a row its table actually covers.

## The shape of the four new functions

**Decision**: Each of the four mixing tables gets its own private `const` row table
(`MIXING_LIGHT_AND_DOUBLE`, `MIXING_LIGHT_AND_HEAVY`, `MIXING_LIGHT_ROUND_AND_DOUBLE`,
`MIXING_LIGHT_ROUND_AND_HEAVY`), using the same `Row` tuple every existing table already uses, and
its own public function built through the existing `build(table, rows)` helper feature 056
extracted:

```rust
#[must_use]
pub fn mixing_light_and_double() -> GlyphCatalog {
    build("Mixing Light and Double", MIXING_LIGHT_AND_DOUBLE)
}
#[must_use]
pub fn mixing_light_and_heavy() -> GlyphCatalog {
    build("Mixing Light and Heavy", MIXING_LIGHT_AND_HEAVY)
}
#[must_use]
pub fn mixing_light_round_and_double() -> GlyphCatalog {
    build("Mixing Light Round and Double", MIXING_LIGHT_ROUND_AND_DOUBLE)
}
#[must_use]
pub fn mixing_light_round_and_heavy() -> GlyphCatalog {
    build("Mixing Light Round and Heavy", MIXING_LIGHT_ROUND_AND_HEAVY)
}
```

**Rationale**:

- `Row` already carries two independent `Option<&'static str>` stroke names on top of two more, so a
  mixing table — whose rows name two different strokes rather than one — needs no new field and no
  new type. Nothing about `GlyphKey`, `Stroke` or `GlyphCatalog::from_rules` cares whether a row's
  four sides name one stroke or two (verified by reading `monospace-core/src/glyph.rs`, not
  assumed).
- `build` already takes a table name and takes rows of any size, so it needs no change to accept
  eighteen or fifty rows instead of fifteen — it was written generically in feature 056 and this
  feature is the proof that the generality was worth it.
- Four functions of the same shape as `ascii()`, `double()`, `heavy()` and `light_round()` keeps
  FR-005 literally true — "reachable the same way Double, Heavy and Light Round already are" —
  rather than introducing a second calling convention for tables that happen to mix two strokes.

**Alternatives considered**:

- **One function per table taking no argument, but named after the two strokes in an order that does
  not match the document's section titles** (e.g. `double_light()`). Rejected: `docs/glyph-sets.md`
  already names each section _Mixing Light and Double_, _Mixing Light and Heavy_, and so on; a
  function name that mirrors the section title exactly is the one a reader can look up without
  translating, and there is no other convention in this crate to match instead.
- **A single function taking two `Stroke`s and returning whichever mixing table pairs them.**
  Rejected: it would need a runtime "which pair is this" dispatch and a fallback for pairs with no
  table (light+ascii, double+heavy, and so on), machinery this crate has never needed for its
  single-stroke tables and that FR-010 does not ask for.
- **Folding the four mixing tables into `double()`, `heavy()` and `light_round()` themselves**, so
  that building the Double table also pulls in its mixing rows. Rejected: FR-006 requires a catalog
  built from a mixing table and its two single-stroke tables to behave differently from one built
  from the single-stroke tables alone (the mixing table must be what makes the difference), which
  means the mixing rows cannot live inside the single-stroke functions without collapsing that
  distinction.

## Where the four tables' rows come from

The 18 + 50 + 18 + 50 = 136 rows are copied verbatim, in the same row order, from
`docs/glyph-sets.md`'s four _Mixing ..._ sections — the same way every table already in
`monospace-glyph-sets` was copied from its own section. `docs/glyph-sets.md` already states that
_Mixing Light Round and Double_ and _Mixing Light Round and Heavy_ are "the same combinations as
[the Light equivalent], with `light` replaced by `light-round`" — so those two tables are checked
against the Light-Round document rows directly (not derived from `MIXING_LIGHT_AND_DOUBLE` or
`MIXING_LIGHT_AND_HEAVY` in code): FR-003 and FR-004 both ask for the rows the document records, and
the document already records them explicitly rather than by cross-reference.

Each row is checked against the document's row for the same key before its test is written
(mirroring feature 056's SC-004 practice), rather than transcribed once and trusted.

## Correcting the demonstration's placements (FR-009)

**What is there today**: `crates/monospace-cli/assets/demo.json` places six pairs of overlapping
boxes below the original two (added by feature 056). Reading them against the four pairs a mixing
table can now cover:

| Boxes (stroke pair)                          | One of the four mixing pairs?                         |
| -------------------------------------------- | ----------------------------------------------------- |
| `light` at (0,9) / `double` at (2,10)        | Yes — Light and Double                                |
| `double` at (8,9) / `light` at (10,10)       | Yes — Light and Double                                |
| `light` at (16,9) / `heavy` at (18,10)       | Yes — Light and Heavy                                 |
| `heavy` at (24,9) / `light` at (26,10)       | Yes — Light and Heavy                                 |
| `light` at (32,9) / `light-round` at (34,10) | **No** — Light and Light Round is not one of the four |
| `heavy` at (40,9) / `double` at (42,10)      | **No** — Heavy and Double is not one of the four      |

So today's demonstration exercises Light+Double and Light+Heavy but never Light Round+Double or
Light Round+Heavy — exactly the edge case FR-009 and the spec's Edge Cases section name. The
existing CLI test `crossings_between_the_new_tables_degrade_to_whichever_figure_is_in_front` (added
by feature 056) asserts, by name, that all six crossings degrade today; four of its six assertions
must change once this feature ships, since Light+Double, Light+Heavy and (after the correction
below) Light Round+Double and Light Round+Heavy will no longer degrade at those positions.

**Decision**: Replace the two placements that pair a stroke outside the four mixing tables — the
`light`/`light-round` pair at (32,9)/(34,10) and the `heavy`/`double` pair at (40,9)/(42,10) — with
a `light-round`/`double` pair and a `light-round`/`heavy` pair, at the same positions and with the
same sizes and fill (none), changing only the `stroke` field on each box. This mirrors the existing
`light`/`double` and `light`/`heavy` pairs exactly, just with `light-round` in place of `light`,
which is the same substitution `docs/glyph-sets.md` itself describes for those two tables.

The two pairs that already cross Light+Double and Light+Heavy are left in place. Light+Heavy is
complete (all fifty combinations), so any crossing between them is necessarily covered. Light+Double
covers only eighteen of fifty; the existing crossing's key is checked against `docs/glyph-sets.md`'s
_Mixing Light and Double_ table when the demonstration is updated (implementation-time, per
principle IV — run it, then write down what happened), and if it does not land on a covered row the
box positions are adjusted, not the stroke pair, so FR-010's "must not touch a figure unrelated to
the four mixing pairs" stays true.

**Rationale**: this is the minimal change that satisfies FR-009 — every figure already in the
demonstration keeps its position and its role in the layout; only the two stroke names that do not
belong to any of the four mixing pairs change, to the two mixing pairs that were missing. No new
figure is added and no existing one is removed, keeping FR-010's "no other" intact.

**Alternatives considered**:

- **Add four more boxes for the two missing pairs, leaving the existing six as they are.** Rejected:
  it grows the demonstration instead of correcting it, and leaves two placements (Light+Light-Round,
  Heavy+Double) that the spec's edge case explicitly says should not exist as mixing-pair
  demonstrations, since neither is one of the four tables this feature ships.
- **Renumber or reposition every box below the original two.** Rejected: it would touch figures the
  correction does not need to touch, against FR-010, for no gain over changing two `stroke` fields.

## The CLI's catalog and its existing test (FR-008)

`crates/monospace-cli/src/description.rs`'s `Description::render` already builds its catalog with
`GlyphCatalog::union([...])` over `GlyphCatalog::light()` and `monospace_glyph_sets`'s four
single-stroke tables. This feature adds the four mixing tables to that same union — a one-line
change to the array — so FR-008 is satisfied by construction: `GlyphCatalog::union` already lets a
key from any set answer independently of order (feature 054's guarantee, reused unchanged).

The existing test named above is updated in the same commit as the `demo.json` correction, since a
test that keeps asserting today's degraded characters would fail the moment the catalog and the
placements both change; Kent Beck's structural/behavioral split (principle V) does not apply to
updating a test's expected values to match a behavioral change the same commit makes, only to
splitting a refactor from a feature.

## No new dependency, no new crate, no ADR

Unchanged from features 054 and 056: `monospace-glyph-sets` needs nothing beyond `monospace-core`,
already depended on. The spec's own Assumptions record that no ADR is needed — ADR-0036 and feature
054 already settled the extension point and the crate boundary this feature reuses without change.

## Documentation left describing the pre-feature state

`docs/glyph-sets.md`'s opening note and `docs/model.md`'s _Strokes, glyph sets and the catalog_ both
currently say the mixing sets are "reference only" or "loaded from a file when someone asks for
them" — both become false once this feature ships them as data. The spec's FR-010 scopes this
feature's functional requirements to "the four mixing tables and the demonstration change ... and no
other," which governs what the feature builds, not whether a document describing what already exists
is left stating something no longer true. Principle IV ("claims are measured, not assumed") and the
precedent both feature 054 and feature 056 set (each corrected the same two documents for the tables
it added) call for the same small correction here: one sentence in each document, naming the four
mixing tables alongside the five single-stroke ones as data, with no table's rows restated and no
other sentence touched. This is called out explicitly, rather than folded in silently, since the
spec's FR list does not name it the way features 054 and 056's did.
