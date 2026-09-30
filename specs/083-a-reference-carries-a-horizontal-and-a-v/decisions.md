<!-- cspell:ignore serde -->

# Decisions: A reference carries a horizontal and a vertical offset

**Feature**: #83 | **Written**: 2026-09-29 | **Answered**: 2026-09-29 — all three, the maintainer's

Three entries, all domain-level, all answered on 2026-09-29 with the recommendation adopted. None
carries a picture: each subject is a type, a wire spelling or a sentence, which is what principle IV
says not to draw one for. research.md Q2 holds D2's measurements, Q4 the two absent questions.

## D1 — What carries the two amounts

- **Proposal**: `Reference { id: ShapeId, anchor: Anchor, offset: Delta }`, reusing `Delta` and the
  saturating `apply` it already has, which then stays the crate's only arithmetic.
- **Altitude**: domain — §1's `Reference` row names three fields, and `Delta`'s own row says how far
  a figure _moves_, so one type meaning both a movement and a gap from a side is a sentence in the
  model's vocabulary. Confidence: medium-high.
- **If this is wrong**: two `i32` fields on `Reference` instead, which is additive, and `resolve`
  grows a second saturating addition beside `Delta::apply`.
- **The alternative**: `dx` and `dy` as fields of `Reference`, which leaves §1 as written, states
  the saturation rule twice, and hands #143 two fields to add into.
- **Yours to answer**: yes — whether one type means both is the model's to say.
- **Answer**: confirmed — `offset: Delta`. `Delta::apply` stays the crate's only arithmetic, which
  is what research.md Q1 measured. The cost named above is real and lands in
  `docs/diagram-model.md`: §1's `Delta` row reads "how far a figure moves along each axis: a
  horizontal and a vertical amount", and its second clause is the one that still holds. That
  amendment follows the answer, as 082's D3's change to §3 did (`505fd0d`).

## D2 — What an endpoint's position becomes in a description

- **Proposal**: `at` becomes internally tagged — `{"kind": "point", "x": …, "y": …}` or
  `{"kind": "reference", "shape": …, "anchor": …, "offset": {…}}` — so a file may name a reference
  and the type refuses both spellings at once.
- **Altitude**: domain — ADR-0035 keeps the format provisional, but 13 generated descriptions and
  `assets/demo.json` observe it, and its throwaway premise is what a reversal would touch.
  Confidence: medium.
- **If this is wrong**: an untagged union, which reads every existing file and answers anything
  wrong with `data did not match any variant of untagged enum At`, and reads a point written beside
  a stray `shape` as a point — or a sibling `reference` next to an optional `at`, which reads
  everything and refuses nothing, putting "exactly one of the two" in prose where the type held it.
- **The alternative**: `at` left a bare point — the only option no file has to change.
- **Yours to answer**: yes — this one either rewrites every description in the repository or admits
  a rule in prose, and every picture it touches comes out identical.
- **Answer**: confirmed — internally tagged. It is the idiom the format already uses twice, and the
  only one of the three where the rule is the type rather than a sentence: `ShapeDescription` is
  already `#[serde(tag = "kind", rename_all = "lowercase")]`. The price is the 13 descriptions
  research.md Q2 counted, each a one-line edit, and every picture among them comes out byte for byte
  what it is — `c7539bc` measured that when the tag moved from `arrow` to `connector`.

## D3 — Does a file name a shape by the identity the diagram issues?

- **Proposal**: `"shape": "#5"` — the identity the reader will issue to the fifth entry, asserted
  and nothing more. `Diagram` gains nothing and §11's open question stays open.
- **Altitude**: domain — §3 says the diagram issues identities and §11 holds the question of a
  caller choosing one, whose stated trigger is "the first slice where a caller has a name worth
  keeping — reading a diagram from a file is the obvious one". Confidence: medium.
- **If this is wrong**: nothing observable breaks, and what is lost is that a file naming `#5`
  silently comes to mean a different shape once one is inserted above it — ADR-0041's silent hole,
  in the other direction.
- **The alternative**: letting the file choose — `"id": "box-above"` on an entry,
  `"shape": "box-above"` in the reference — which answers §11 with yes, amends §3 and §11, and needs
  a public way to add a shape under a chosen identity: 082's D3's `add_under`, declined then for
  being early.
- **Yours to answer**: yes — this is the slice §11's own trigger names.
- **Answer**: confirmed — `"shape": "#5"`, asserted and nothing more. `Diagram` gains nothing, §3
  stands and §11 stays open. That was weighed against the specification's own clarification, which
  pointed the other way by saying this slice settles §11, and declined because the alternative is
  additive and this is not. §11's trigger has fired and been declined; what remains is a caller with
  a name worth keeping.
