# Phase 1 data model: Hold a literal glyph in a cell

**Feature**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Date**: 2026-09-09

The rules are in _The cell_, _Stamping_ and _Rendering_ of [`docs/model.md`](../../docs/model.md).
This file says what carries them in code, and traces each row of the model's table to what produces
it. It does not restate the rule.

## Cell

A sum of two kinds, which is the shape [R1](research.md) sends to ADR-0026:

| Kind      | Holds                                       |
| --------- | ------------------------------------------- |
| `Strokes` | one `StrokeCell` — a base stroke, four arms |
| `Literal` | one `Glyph` — the character it renders to   |

- **Exclusive by construction.** No cell is both and none is neither, which is FR-001 discharged by
  the type rather than by a check.
- **Not `Copy`, and it never was.** A base stroke owns a `String` and so does a glyph. `Debug`,
  `Clone`, `PartialEq` and `Eq` are derived, as `Cell` derives them today; tests compare cells read
  back through `Buffer::cell`, per
  [ADR-0011](../../docs/decisions/0011-expose-cell-for-testing-stamping.md).
- **Equality is structural.** Two literals are equal when their glyphs are, which is text equality
  without normalization — the trap feature 006 documented and did not close.
- **A literal has no arms in memory.** Its four `Closed` sides exist where composing happens; see
  [R3](research.md).

### is_decided

Widens rather than moves. For a stroke cell it is the four arms, unchanged from
[ADR-0017](../../docs/decisions/0017-ask-the-cell-whether-it-is-decided.md); for a literal it is
`true`, because a literal decides all four of its sides by definition. That one line is what makes
three of the five composition rows fall out of branches that already exist — see [R2](research.md).

## StrokeCell

Today's `Cell`, renamed and otherwise untouched: `base: Stroke`, and `top`, `right`, `bottom`,
`left` as `Arm`. Public fields stay public; nothing about it is validated, and nothing about that
changes here.

`From<StrokeCell> for Cell` exists so a caller that builds nine stroke cells for a box does not
double the width of every line. Nothing converts the other way: it would have to fail.

## Glyph

Unchanged, from feature 006. One grapheme cluster, never a control character, checked on
construction, so a cell holding one has nothing left to verify (FR-012). Width is outside its
invariant, per [ADR-0019](../../docs/decisions/0019-represent-a-glyph-as-a-grapheme-cluster.md), and
this feature puts nothing wide on the terminal.

## Buffer

Unchanged in what it is: each position holds a cell or nothing, and a cell may now be either kind.
`stamp` and `cell` keep their signatures — the type inside them is the one that widened.

## Where each composition row is produced

Every row is _Stamping_'s, cited by its target and stamp rather than restated. "Shortcut" means a
branch that exists in `Buffer::stamp` today and needs no change.

| Target        | Stamp     | Mode    | What produces it                                                           |
| ------------- | --------- | ------- | -------------------------------------------------------------------------- |
| undefined     | either    | either  | The insert branch, unchanged                                               |
| a stroke cell | arms      | either  | `merge`, unchanged                                                         |
| a stroke cell | a literal | `Above` | Shortcut: a literal is decided, so the stamp wins outright ([ADR-0018])    |
| a stroke cell | a literal | `Below` | `merge`, new case: the literal is the bottom and contributes four `Closed` |
| a literal     | arms      | `Above` | `merge`, new case: same one, reached from the other side                   |
| a literal     | arms      | `Below` | Shortcut: a decided target is left alone ([ADR-0017])                      |
| a literal     | a literal | `Above` | Shortcut: the stamp is decided ([ADR-0018])                                |
| a literal     | a literal | `Below` | Shortcut: the target is decided ([ADR-0017])                               |

[ADR-0017]: ../../docs/decisions/0017-ask-the-cell-whether-it-is-decided.md
[ADR-0018]: ../../docs/decisions/0018-mirror-the-decided-skip-in-above.md

The two `merge` rows are one case: a stroke cell being written where a literal is, whichever order
the two arrived in. That is what makes the equivalence of the two stamp orders survive a literal
(FR-008), and it is the only new arithmetic in the feature.

## Rendering

One branch. A literal answers with its own glyph's text; a stroke cell goes on building a key and
looking it up exactly as before, degradation included. The key builder narrows to take a stroke
cell, since a literal never reaches it. The borrow the renderer returns can now come from the buffer
as well as from the catalog, which is a lifetime change and not an allocation — see
[R4](research.md).

## Not representable, on purpose

- A literal with an arm. There is no field to put one in, so FR-003 needs no check.
- A cell that is both kinds, or neither.
- A glyph that is empty, several clusters, or a control character — refused by `Glyph`, one layer
  down.

## Nothing changes

`Arm`, `Stroke`, `GlyphKey`, `GlyphCatalog`, `Pos`, `Size`, `StampMode` and the glyph tables. No
rule is added to the catalog and no key is built differently.
