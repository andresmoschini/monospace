# Phase 0 Research: fix shapes without filling closing their inner arms

No `NEEDS CLARIFICATION` markers remain in the Technical Context — this is a bug fix confined to
`monospace-core`, with no new dependency, no new external interface and no new technology choice to
survey. The one open question was _what, exactly, produces the bug and what is the smallest correct
fix_ — answered below by reading the code and then measuring it, per Constitution principle IV
(claims are measured, not assumed).

## Where the bug lives

Traced through `crates/monospace-core/src/shape/box_shape.rs`, `fragment/corner.rs`,
`fragment/border.rs` and `buffer.rs`:

- `Corner` (`fragment/corner.rs`) sets `Arm::Set` toward the two sides it opens and `Arm::Unset` on
  the other two, **regardless of fill**. `Unset` never overrides anything (see `merge_arm` in
  `buffer.rs`), so a corner cell never closes a crossing. Corners are not part of the bug and need
  no change.
- `Border` (`fragment/border.rs`) sets `Arm::Closed` on the side facing the shape's interior,
  unconditionally — `BoxShape::draw` never tells it whether the box has a fill. This is the entire
  bug: a `Closed` arm on a stamp always wins over whatever is behind it (`merge_arm` only falls
  through on `Unset`), so an unfilled box's border still refuses a stroke that crosses into its
  interior, exactly as a filled one does.

## Decision: give `Border` the one bit it is missing

**Decision**: `Border` gains a `closes_interior: bool` field. `BoxShape::draw` passes
`self.fill.is_some()` to every `Border` it places. When `closes_interior` is `false`, the side
facing the interior is stamped `Arm::Unset` instead of `Arm::Closed` — "not mine to decide", per
_The cell_ in `docs/model.md` — so whatever crosses into it decides that side instead. `Corner` is
left untouched.

**Rationale**:

- It is the smallest change that removes the bug: one new field, one new call-site value, no new
  fragment, no new pass over the buffer.
- It matches the existing pattern fragments already use for shape-specific facts they cannot derive
  themselves (`side` already tells `Border` its orientation and which side it closes;
  `closes_interior` is one more fact of the same kind), per
  [ADR-0028](../../docs/decisions/0028-give-each-fragment-its-own-cell-rule.md): "the figure placing
  one names which side of itself it is and nothing more, and the rule lives with the piece."
- Measured, not assumed: a throwaway probe test (two 6×4 unfilled boxes at `(0,0)` and `(4,2)` on a
  9×5 canvas, both stamped `Above`) was built against a patched `Border`/`BoxShape` and then
  reverted once it had answered the question — nothing from it is committed. It confirmed:
  - **Before the fix**, the two overlapping border cells render `┴` and `┤` — the reported bug,
    reproduced.
  - **After the fix**, with both boxes unfilled, the same two cells render `┼` — a crossing, per
    FR-001 and acceptance scenario 1.
  - **After the fix**, with the second (top) box given a fill, the same two cells render `┴` and `┤`
    again — the fill keeps closing its interior side exactly as before, per FR-002 and acceptance
    scenario 2. Draw order matters here for which shape's own arm decides at the shared cell (the
    one stamped later has first say, per _Stamping_), which is why the _second_ box's fill was the
    one tested — a plan-time observation the tasks phase should turn into an explicit test case
    rather than leave implicit.
  - The existing 77 tests in `monospace-core` all still pass unchanged, confirming FR-003 / SC-002:
    a single unfilled shape produces byte-for-byte the same output as before, because
    `StrokeCell::key` already treats `Closed` and `Unset` identically for any side nothing else
    contests (see `cell.rs`).
  - See `specs/049-bug-with-shapes-without-filling/quickstart-example.json` and `quickstart.md` for
    a runnable version of this example through `monospace-cli`.

**Alternatives considered**:

1. **A post-hoc pass that reopens the interior arms of unfilled shapes after every shape has
   drawn.** Rejected: it would need to know which cells belong to which shape's interior after the
   fact — exactly the "a shape reports what it covers" step
   [ADR-0030](../../docs/decisions/0030-drop-extent-until-a-caller-needs-it.md) deliberately dropped
   — and it would make the result depend on draw order in a new way, breaking _The two orders are
   equivalent_ property `buffer.rs` already tests.
2. **A second fragment that only closes the interior when there is a fill**, leaving `Border` always
   `Unset` on that side. Rejected: it would duplicate `Border`'s own geometry (`from`, `len`,
   `side`) in a second fragment for one bit of information `Border` can just as well take as a
   parameter.
3. **A dedicated two-value type (e.g. `Interior::Closed` / `Interior::Open`) instead of a plain
   `bool`.** Considered for the domain-naming reason the project already favors — a bare `bool` can
   read as "boolean blindness" at a call site with more than one flag. Set aside here because
   `Border` has exactly one such flag and the call site (`closes_interior: self.fill.is_some()`)
   already reads as what it means; left as a judgment call for whoever writes the code, not a
   plan-level decision worth freezing.

## Whether this needs an ADR

No. [ADR-0028](../../docs/decisions/0028-give-each-fragment-its-own-cell-rule.md) already recorded
the decision this fix is an instance of: a fragment's arms follow from a fact the figure placing it
hands in. This changes _which_ fact `Border` is handed, not the pattern. What it does need — because
`docs/model.md` currently states the box's border rule two ways that disagree (see next section) —
is a correction to the model itself, which is not a new decision to record but a description to fix.

## `docs/model.md` disagrees with itself

- _The cell_ (section 3) already gets this right: "The top border of **a filled shape** is stamped
  with its inner side `Closed` on purpose... " (line 100) — conditional on the fill, as this fix
  makes the code.
- _The initial set_ (section 7), describing the box, does not carry the condition: "`Set` along the
  run, `Closed` on the side facing its own interior, `Unset` outward" (line 288) — stated as if
  every box closes its interior side, fill or not.

Per the constitution's "the model owns the design" and "if a slice needs a rule the model does not
have, the model changes first": the model already has the rule (section 3 states it correctly), so
this is not a new design decision to take — it is `docs/model.md`'s own box description falling out
of step with its own cell rule, and a design task, not an ADR, is the correct place to fix it.
Carried into Phase 1 as a data-model.md entity note and left for the tasks phase to schedule as one
of the first tasks, since code that depends on the corrected rule should not land before the
document it follows agrees with itself.
