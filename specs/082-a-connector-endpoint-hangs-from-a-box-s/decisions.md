# Decisions: A connector endpoint hangs from a box's side anchor

**Feature**: #82 | **Written**: 2026-09-28 | **Answered**: 2026-09-29 — all five

Five entries, all domain-level, all the maintainer's, all answered on 2026-09-29. Two questions a
reader will look for are absent on purpose, and [research.md](research.md) says where each was
answered instead: whether `monospace-core` changes is 081's D1, already answered in the future
tense, and what `remove` does to a reference is the specification's own clarification. No entry
carries a picture — each subject is a type or a sentence, which is what principle IV says not to
draw one for.

## D1 — Does a `Reference` carry offsets in this slice?

- **Proposal**: `Reference { id: ShapeId, anchor: Anchor }` — two fields; #83 adds the third.
- **Altitude**: domain — §1's row and §4's sentence are the model's. Confidence: medium-high.
- **If this is wrong**: adding two fields later is additive and no caller changes, so the expensive
  direction is amending §1 and §4 down to two.
- **The alternative**: `Reference { id, anchor, dx, dy }` now, zeros until #83. Additive, and it
  makes §4's displacement sentence expressible — but principle III's removal test says an entry that
  breaks nothing when taken out is removed, and an offset nothing can set past zero is that.
- **Yours to answer**: yes.
- **Answer**: confirmed, 2026-09-29 — two fields. §1 and §4 stand as written and #83 adds the third,
  and the offset is what the two-field form makes unreachable today: a caller who wants the endpoint
  two cells off a side has no way to say it and places both figures by hand.

## D2 — Does only an endpoint's position become a `Position`?

- **Proposal**: `Endpoint.at` becomes one; `Box.at` and `Line.at` stay `Pos`, so ADR-0041's
  restriction is the type system rather than a sentence beside it.
- **Altitude**: domain — the public shape of all three `Shape` variants. Confidence: high.
- **If this is wrong**: #89 changes two field types on a type every caller matches, rather than
  filling one already there.
- **The alternative**: all three become a `Position` and the restriction stays a rule — a type that
  on two of three fields can only ever be `Absolute`, and #89 is what carries the cycle obligation
  ADR-0041 attaches to widening.
- **Yours to answer**: yes — ADR-0041 states the restriction and not how it is expressed.
- **Answer**: confirmed, 2026-09-29 — only the endpoint. `Position` appears on one field and
  ADR-0041's restriction is the type system rather than a rule to get right at run time, so #89
  inherits the field and the cycle obligation together rather than a half-made type.

## D3 — Can a caller name an identity?

- **Proposal**: nothing new. `ShapeId::new` is already public; §3 gains one sentence saying a caller
  may spell an identity while the diagram still issues them.
- **Altitude**: domain — §3 and §11 are the model's. Confidence: high.
- **If this is wrong**: §3 keeps saying "the diagram generates one" beside a demonstration writing
  `#3`, and a reader cannot tell which it means.
- **The alternative**: `add_under(&mut self, id: ShapeId, shape: Shape)`. §11's own trigger is the
  first slice with a name worth keeping — reading a diagram from a file — and this is not it.
- **Yours to answer**: yes — the specification hands this here as the domain-level entry it is.
- **Answer**: confirmed, 2026-09-29 — nothing new. `ShapeId::new` already spells one and the
  demonstration already spells `#1`; what this slice adds is a second spelling and one sentence in
  §3 saying a caller may spell an identity while the diagram still issues them. §11's own trigger is
  the first slice reading a diagram from a file, and `add_under` would answer it early and bring a
  question nobody asked: what two callers choosing `#3` do.

## D4 — Does §4's displacement sentence get amended, now the code does something else?

- **Proposal**: amend §4 with one sentence saying that while a reference has no offsets a
  displacement adds nothing to it and the position does not move, marked as what #143 replaces.
- **Altitude**: domain — the document says at its head to amend it when reality contradicts it.
  Confidence: medium.
- **If this is wrong**: the model states a rule the code does not follow, the drift that header
  exists to prevent. D1 answered with three fields settles this the other way.
- **The alternative**: leave §4, on the argument that #143 will replace the sentence anyway — which
  is what the specification's "none of them amended" says.
- **Yours to answer**: yes — here the specification and the model disagree.
- **Answer**: confirmed, 2026-09-29 — leave §4 as written. The model states the destination and #143
  is the slice that reaches it; amending now would write an intermediate state that goes stale in
  weeks. The temporary gap is the cost, and #83 and #143 carry the resolution.

## D5 — Where a position resolves, and whether a caller can ask

- **Proposal**: on the position, and public — `Position::resolve(&self, &Diagram) -> Option<Pos>`.
  `Shape::draw` takes the `&Diagram` it needs; nothing is passed in as a behavior.
- **Altitude**: domain — public surface, and 081's D2 is the same shape of question. Confidence:
  high.
- **If this is wrong**: making it `pub(crate)` later costs one keyword and no caller, and none
  outside the crate calls it yet. The expensive direction is the reverse.
- **The alternative**: a closure as the second argument, which leaves `Position` with no behavior
  and lets any caller answer without consulting the diagram. A trait over the diagram was declined
  too: one implementor, and the stub a test resolves against tests the stub.
- **Yours to answer**: yes.
- **Answer**: confirmed, 2026-09-29 — the position resolves, and it is public. A caller knowing a
  `Position` and its `Diagram` can ask; one that does not resolve answers nothing rather than
  failing. Putting it on the position also closes what the closure form left open: the rule is the
  type's behavior, not a convention at a call site.
