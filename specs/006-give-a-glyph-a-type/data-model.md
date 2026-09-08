# Phase 1 data model: Give a glyph a type of its own

**Feature**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Date**: 2026-09-08

The domain vocabulary is owned by [`docs/model.md`](../../docs/model.md) and is not restated here.
This file records the shape the two increments give it in `monospace-core`, and the validation rules
that shape carries.

## Glyph

What a cell renders to. One value, immutable once built, with no state transitions.

| Field   | Type     | Visibility | Notes                                                             |
| ------- | -------- | ---------- | ----------------------------------------------------------------- |
| payload | `String` | private    | The glyph's text. Private is what leaves no way around the check. |

**Derives**: `Debug`, `Clone`, `PartialEq`, `Eq`. Not `Copy`, which is the cost ADR-0019 records.

**Construction**: `Glyph::new(text: &str) -> Option<Self>`, the only way in.

### Validation rules

| #   | Rule                                                | Increment | Spec           |
| --- | --------------------------------------------------- | --------- | -------------- |
| V1  | The text is not empty                               | P1        | FR-002         |
| V2  | The text is exactly one `char`                      | P1        | FR-003         |
| V2' | The text is exactly one extended grapheme cluster   | P2        | FR-012         |
| V3  | No `char` in the text is in Unicode's `Cc` category | P1, P2    | FR-004, FR-013 |

V2' replaces V2; V1 and V3 are unchanged by the widening. V1 needs no separate check in either
increment — empty text has zero characters and zero clusters, so it fails V2 and V2' on its own. It
is listed because it is a named behavior rule with its own test, not because it is a separate
branch.

**Not validated, deliberately**: column width, and normal form. A wide grapheme is one glyph and
shifts its row; `é` in two normal forms is two glyphs that render alike and compare unequal
(FR-018). Both are recorded in the rustdoc rather than enforced, and ADR-0019 owns why.

### Identity

Two glyphs are equal when their text is equal, byte for byte. There is no normalization and no
appearance-based comparison — see FR-018 and the clarification it came from.

## GlyphCatalog

Every rule in play. Only what this feature changes is listed; the two-step lookup, the absence of a
count and the fact that nothing tells a built-in rule from a loaded one are unchanged and belong to
_Strokes, glyph sets and the catalog_ in the model.

| Field | Type                       | Before                    |
| ----- | -------------------------- | ------------------------- |
| rules | `HashMap<GlyphKey, Glyph>` | `HashMap<GlyphKey, char>` |

**Answering a key**: `glyph(&self, key: &GlyphKey) -> Option<&Glyph>` — a borrow, because the
catalog owns its rules and a glyph is no longer `Copy`.

**Construction**: `light()` builds from the fifteen rows of the Light table and panics if one of
them is not a valid glyph (FR-009). The table's own row type carries `&'static str` for the glyph
from P1, so that no row moves when the invariant widens (FR-014).

## GlyphKey, Cell, Buffer, Stroke

Unchanged. `Stroke` keeps its public field and its lack of validation; ADR-0019 records why a stroke
and a glyph look alike and are not. Nothing in this feature puts a glyph near a `Cell` — that is the
next slice.

## Rendering

`render` collects glyph text rather than characters. A position with no cell, no answer, or outside
the window renders as a single space, which is a `&str` literal rather than a `Glyph`: the fallback
is the absence of a glyph, not a glyph that happens to be blank. That distinction is what keeps
FR-010 true without inventing a "space glyph" the catalog would then have to hold.
