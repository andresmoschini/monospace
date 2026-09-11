# Phase 1 Data Model: fix shapes without filling closing their inner arms

This feature adds no new domain entity — it corrects one fact one existing fragment is missing. The
entities below are the ones this fix touches, not a new set.

## `Border` (fragment, `crates/monospace-core/src/shape/fragment/border.rs`)

| Field             | Type     | Change                                                                                                          |
| ----------------- | -------- | --------------------------------------------------------------------------------------------------------------- |
| `from`            | `Pos`    | unchanged                                                                                                       |
| `len`             | `u32`    | unchanged                                                                                                       |
| `side`            | `Side`   | unchanged                                                                                                       |
| `stroke`          | `Stroke` | unchanged                                                                                                       |
| `closes_interior` | `bool`   | **new** — whether the side facing the figure's interior stamps `Arm::Closed` (`true`) or `Arm::Unset` (`false`) |

Validation rule: the side named by `side` itself always stamps `Arm::Unset` (outward-facing, per the
existing rule) and the two perpendicular sides always stamp `Arm::Set` (along the run) — neither of
those changes. Only the side opposite `side` (the interior-facing one) now branches on
`closes_interior` instead of being hard-coded to `Arm::Closed`.

`Border` is `pub(crate)`: this is not a public API change, and no external contract is affected.

## `BoxShape` (`crates/monospace-core/src/shape/box_shape.rs`)

No field changes — `at`, `size`, `stroke` and `fill` are unchanged, and `fill: Option<Glyph>`
already carries the fact this fix needs. `BoxShape::draw` changes only in what it _passes_ to each
of its four `Border` placements: `closes_interior: self.fill.is_some()`, computed once and shared
across all four calls (a box's interior is either fully closed to crossings or fully open to them;
there is no per-side notion of "half filled").

`Corner` is unchanged and takes no new field — see research.md for why.

## `docs/model.md` — _The initial set_ (box description)

Not a code entity, but part of this fix's design surface per the constitution ("the model owns the
design"): the box's description in section 7 must state the same condition section 3 already does,
replacing:

> `Set` along the run, `Closed` on the side facing its own interior, `Unset` outward

with wording that names the condition — the interior side is `Closed` only when the box has a fill,
and `Unset` otherwise, exactly as an unfilled box's own crossing-facing side is described in section
3 already. This is a documentation correction, scheduled as an early task rather than an
architectural decision (see research.md, "Whether this needs an ADR").

## State transitions

None. A `Border`'s `closes_interior` is fixed for the lifetime of one `draw` call, derived once from
the owning `BoxShape`'s `fill` at construction time — there is no mutation and no lifecycle to
model.

## Relationships

```text
BoxShape { fill: Option<Glyph>, .. }
  --draws-->  Corner   (unchanged: Unset on both non-open sides, independent of fill)
  --draws-->  Border { closes_interior: fill.is_some(), .. }
  --draws-->  Fill     (unchanged: only placed when fill.is_some())
```
