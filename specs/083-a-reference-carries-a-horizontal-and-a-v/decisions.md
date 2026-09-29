<!-- cspell:ignore serde -->

# Decisions: A reference carries a horizontal and a vertical offset

**Feature**: #83 | **Written**: 2026-09-29 | **Answered**: _pending_

Three entries, all domain-level and all the maintainer's. No entry carries a picture: each subject
is a type, a wire spelling or a sentence, which is what principle IV says not to draw one for. The
measurements behind D2 are in [research.md](research.md) Q2, and the two questions a reader will
look for are absent on purpose — whether a displacement reaches the offsets, and whether a shape
answers its anchors publicly — with research.md Q4 saying where each was answered instead.

## D1 — What carries the two amounts

- **Proposal**: `Reference { id: ShapeId, anchor: Anchor, offset: Delta }`, reusing `Delta` and the
  saturating `apply` it already has, which then stays the crate's only arithmetic.
- **Altitude**: domain — §1's `Reference` row names three fields, and `Delta`'s own row says how far
  a figure _moves_, so one type meaning both a movement and a gap from a side is a sentence in the
  model's vocabulary. Confidence: medium-high.
- **If this is wrong**: two `i32` fields on `Reference` instead, which is additive, and `resolve`
  grows a second saturating addition beside `Delta::apply`.
- **The alternative**: the two loose fields, which leaves §1 exactly as written and keeps `Delta`
  meaning one thing — at the price of stating the saturation rule twice, and of handing #143 two
  fields to add into where one would do.
- **Yours to answer**: yes — whether one type means both is the model's to say.
- **Answer**: _pending_

## D2 — What an endpoint's position becomes in a description

- **Proposal**: `at` becomes internally tagged — `{"kind": "point", "x": …, "y": …}` or
  `{"kind": "reference", "shape": …, "anchor": …, "offset": {…}}` — so a file may name a reference
  and the type refuses both spellings at once.
- **Altitude**: domain — ADR-0035 keeps the format provisional, but 13 generated descriptions and
  `assets/demo.json` observe it, and that record's throwaway premise is what a reversal would touch.
  Confidence: medium.
- **If this is wrong**: an untagged union, which reads every existing file and answers anything
  wrong with `data did not match any variant of untagged enum At`, and reads a point written beside
  a stray `shape` as a point — or a sibling `reference` next to an optional `at`, which reads
  everything, refuses nothing, and puts "exactly one of the two" in prose where the type held it.
- **The alternative**: leaving `at` a bare point and adding the reference beside it — the only
  option no file has to change, and the only one whose rule cannot be a type.
- **Yours to answer**: yes — this one either rewrites every description in the repository or admits
  a rule in prose, and every picture it touches comes out identical.
- **Answer**: _pending_

## D3 — Does a file name a shape by the identity the diagram issues?

- **Proposal**: `"shape": "#5"` — the identity the reader will issue to the fifth entry, asserted
  and nothing more. `Diagram` gains nothing and §11's open question stays open.
- **Altitude**: domain — §3 says the diagram issues identities and §11 holds the question of a
  caller choosing one, whose own stated trigger is "the first slice where a caller has a name worth
  keeping — reading a diagram from a file is the obvious one". Confidence: medium.
- **If this is wrong**: nothing observable breaks, and what is lost is that a file naming `#5`
  silently comes to mean a different shape once one is inserted above it — ADR-0041's silent hole,
  in the other direction.
- **The alternative**: letting the file choose — `"id": "box-above"` on an entry,
  `"shape": "box-above"` in the reference — which answers §11 with yes, amends §3 and §11, and needs
  a public way to add a shape under a chosen identity. That is 082's D3's `add_under`, declined then
  for being early, on the ground that this slice was the trigger.
- **Yours to answer**: yes — this is the slice §11's own trigger names.
- **Answer**: _pending_
