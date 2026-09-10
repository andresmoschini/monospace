---
status: "accepted"
date: 2026-09-10
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Draw a line's end as one arm, and keep an arrow's head a chosen glyph

## Context and Problem Statement

_An end is an arm; a head is a glyph_ in [`docs/model.md`](../model.md) said, until this record,
that both an end and a head are chosen glyphs the caller supplies. The reasoning ran from a
measurement: a cell with only one arm renders the same character as a segment, so an end cannot be a
by-product of a line's arms and something has to put a character there deliberately. Feature 039's
spec took that as given and pinned `╾───╼` as the picture of a horizontal line of length 5.

Reviewing that plan surfaced two things the reasoning had not weighed.

**The glyph does not follow the style.** Two lines of the same stroke, in a diagram rendered with
the ASCII set, would carry whatever characters their callers happened to pass. Everything else about
a figure's appearance is derived — the base stroke names a style, the glyph set turns arms into
characters — and the end was the one place where the caller decided how the drawing looks. That is
the same objection that put a `stroke` field on each figure instead of a hardcoded `"light"` inside
the core.

**A chosen glyph refuses to join.** A literal composes as a cell with four `Closed` arms, so nothing
connects into one. Two lines meeting at right angles with an end at the same position therefore
cannot render a corner — and a corner is what a diagram wants there. A one-armed stroke cell, with
its other three sides left `Unset`, gives exactly that: each line contributes its own arm and the
position renders as the corner the two lines make.

The measurement itself was re-verified rather than assumed. In every single-stroke set the four
single-arm keys are already claimed: the Light table in [`docs/glyph-sets.md`](../glyph-sets.md)
answers left-only and right-only with `─` and top-only and bottom-only with `│`, and the ASCII table
answers `-` and `|`. `╴ ╵ ╶ ╷` appear in no set in the repository. So the measurement stands and
only the conclusion drawn from it is wrong: an end really is indistinguishable from a segment in the
text — and that turns out to be acceptable, because what makes an end an end is not the character it
renders but which sides it leaves undecided.

Heads are not in the same position. A head points, so it needs a directional character, and the
repository has no rule that could produce one: `▲ ► ◄ ▼` appear in no set, and `╾ ╼` are already
claimed in both mixing sets that hold them by keys meaning heavy on one side and light on the other.

## Decision Drivers

- Appearance is derived from the style, not passed in by the caller — the rule that put `stroke` on
  each figure rather than a constant in the core.
- `Unset` means "not mine to decide", per
  [ADR-0008](0008-compose-overlapping-cells-with-three-state-arms.md). A line's outward side is
  exactly a side the line has no opinion about, so leaving it `Unset` is what the model already
  prescribes for it.
- `Buffer`, `stamp`, `Cell` and the renderer are unchanged by feature 039, and `docs/glyph-sets.md`
  is untouched by it.
- _Claims are measured, not assumed_: the character a one-armed cell renders is read out of the
  tables, not predicted.

## Considered Options

- **A chosen glyph the caller supplies**, for both an end and a head. The model's answer until now.
- **One arm toward the line's interior, the other three sides `Unset`**, for an end; a chosen glyph
  for a head.
- **One arm, plus four new single-arm rules** rendering `╶ ╴ ╵ ╷`, so that an end is both derived
  and visibly an end.
- **No end at all**: a line is a plain run of segment cells.

## Decision Outcome

Chosen option: **one arm for a line's end, and a chosen glyph for an arrow's head**, because an
end's job is to say where the stroke stops and what may join it there, which arms express and a
literal cannot, while a head's job is to point, which no glyph set in this repository can express at
all.

A line is therefore described by a position, a length, an orientation and a stroke, and carries no
glyph. An arrow keeps the head glyph at each of its two endpoints.

### Consequences

- Good, because a line's end follows the diagram's style: the same line drawn with a different
  stroke or rendered against a different set gets that set's character, with no rule added anywhere.
- Good, because two lines whose ends meet at one position compose into a corner. Checked against the
  table in _Stamping_ for both modes: `Below` writes only the arms the target left `Unset`, and
  `Above` writes every arm the stamp names and leaves the stamp's `Unset` sides alone, so either
  order ends with both arms `Set`.
- Good, because no glyph set gains a rule and `docs/glyph-sets.md` stays untouched.
- Good, because `Line` loses two fields and the caller loses two decisions it had no basis for
  taking.
- Bad, because **a line renders identically to a run of segment cells**. A horizontal line of length
  5 is `─────`, so feature 039's requirement that a line end "in something distinguishable from a
  segment" is deleted rather than satisfied, and every pinned picture in its second user story
  changes. What is left to observe is the join, which is a better test than the picture was.
- Bad, because it leaves an asymmetry in the middle of one figure's description: an arrow's route
  and its ends follow the style, and its two heads do not. That is a limit of the glyph sets rather
  than a design preference, and it is written into _An end is an arm; a head is a glyph_ so that the
  next reader finds a reason there instead of an inconsistency.
- Neutral, because the option that would make an end visible — adding the four single-arm rules —
  stays available and stays a feature rather than a tweak: those four keys are claimed today, so
  changing them changes what existing cells render, which is a behavioral change against a guarantee
  feature 039 makes.

### Confirmation

A test that draws a horizontal line and a vertical line whose ends share one position and asserts
the rendered character is the corner, not a segment, a T or a cross. It is the test that fails if an
end is ever made a literal again, and feature 039's spec carries it as an acceptance scenario.

## Pros and Cons of the Options

### A chosen glyph the caller supplies, for both

- Good, because an end is then visibly an end, and `╾───╼` reads as a line with two ends.
- Good, because it needs nothing from any glyph set.
- Bad, because the character does not follow the style, which is the one thing a style is for.
- Bad, because nothing joins a literal, so two lines meeting at a shared end cannot make a corner.

### One arm for an end, a chosen glyph for a head

- Good, because both halves are derived as far as the repository's data allows, and the half that is
  not says why.
- Bad, because a line and a run of segments are the same text.

### One arm, plus four new single-arm rules

- Good, because it is the only option that is both derived and visible.
- Bad, because it rewrites four rows that already answer, so cells that render `─` today would
  render `╶`. That is a change to the renderer's output, which belongs to a feature of its own with
  its own before-and-after.

### No end at all

- Good, because it is the least there is.
- Bad, because the outermost cell would then carry an arm pointing outward at nothing, so anything
  arriving from outside would join it as a through-line rather than at a corner. The join, which is
  the whole benefit, is the part it gets wrong.

## Reversibility

Cheap. `End` is one crate-private fragment and `Line`'s field list is public but new: restoring
caller-supplied end glyphs is adding two fields back and swapping one fragment for another. It gets
expensive only once a diagram description outside the process names end glyphs, which is a layer
that does not exist.

No ADR is superseded by this. The conclusion it reverses was prose in `docs/model.md`, which is
design intent and is amended in the same increment as this record.

## Confidence

High (85%) for a line's end.

Medium (65%) for keeping a head caller-supplied. What would change it: a feature that gives glyph
sets rules for heads, at which point the caller's glyph becomes an override rather than the only
source. The observation that would prove the current split wrong is a diagram where the heads look
foreign against the set that drew everything else — which is precisely what this decision fixes for
ends and knowingly leaves in place for heads.

## More Information

- _An end is an arm; a head is a glyph_ in [`docs/model.md`](../model.md), amended alongside this
  record, and _Open questions_ there, where heads that follow the glyph set are the remaining half.
- [ADR-0008](0008-compose-overlapping-cells-with-three-state-arms.md) for what `Unset` means, and
  [ADR-0026](0026-represent-a-cell-as-a-sum-of-strokes-and-a-literal.md) for why a literal has four
  `Closed` arms.
- [ADR-0028](0028-give-each-fragment-its-own-cell-rule.md), which makes `End` and `Head` two
  fragments rather than one parametrized by a cell.
- The Light and ASCII tables in [`docs/glyph-sets.md`](../glyph-sets.md) hold the measurement.
