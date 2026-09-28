# Decisions: A connector endpoint hangs from a box's side anchor

**Feature**: #82 | **Written**: 2026-09-28 | **Answered**: _pending_

Four entries, all domain-level, all the maintainer's. Two questions a reader may expect to find here
are absent on purpose and [research.md](research.md) says where each was answered instead: whether
`monospace-core` changes is 081's D1, already answered in the future tense, and what `remove` does
to a reference is the spec's own clarification.

## D1 — Does a `Reference` carry offsets in this slice?

- **Proposal**: `Reference { id: ShapeId, anchor: Anchor }` — two fields. §1 and §4 stand as written
  and #83 adds the third.
- **Altitude**: domain — the row in §1 and the sentence in §4 are the model's, and the field is
  public surface. Confidence: medium-high; the removal test argues for it and #83 is close.
- **If this is wrong**: adding two fields later is additive and no caller has to change, so the
  expensive direction is amending §1 and §4 down to two rather than up from them.
- **The alternative**: `Reference { id, anchor, dx, dy }` now, holding zeros until #83. Additive,
  and it makes §4's "a displacement reaches its offsets instead" expressible — but principle III's
  removal test says an entry that breaks nothing when taken out is removed, and an offset no caller
  can set to anything but zero is that.
- **Yours to answer**: yes — the model's row and the model's sentence. No picture: both forms draw
  the same picture, which is the whole of the entry.

## D2 — Does only an endpoint's position become a `Position`?

- **Proposal**: `Endpoint.at` becomes a `Position`; `Box.at` and `Line.at` stay `Pos`, so ADR-0041's
  restriction is the type system rather than a sentence beside it.
- **Altitude**: domain — the public shape of all three `Shape` variants, and `description.rs` in the
  command-line application changes with it. Confidence: high.
- **If this is wrong**: #89 changes two field types on a type every caller matches, rather than
  filling one that is already there.
- **The alternative**: all three become a `Position` and the restriction stays a rule. Forward-
  compatible with #89, and it ships a type that on two of three fields can only ever be `Absolute` —
  and #89 is what carries the cycle obligation ADR-0041 attaches to widening.
- **Yours to answer**: yes — ADR-0041 states the restriction and does not say how it is expressed.
  No picture: the subject is a field's type.

## D3 — Can a caller name an identity?

- **Proposal**: nothing new. `ShapeId::new` is already public and the demonstration already spells
  `#1`; §3 gains one sentence saying a caller may spell an identity while the diagram still issues
  them.
- **Altitude**: domain — §3 and §11 are the model's. Confidence: high.
- **If this is wrong**: §3 keeps saying "the diagram generates one" beside a demonstration that
  writes `#3`, and a reader cannot tell which of the two it means.
- **The alternative**: `add_under(&mut self, id: ShapeId, shape: Shape)`, the first way a caller
  chooses one. §11's own trigger is "the first slice where a caller has a name worth keeping —
  reading a diagram from a file", and this is not that slice.
- **Yours to answer**: yes — the specification hands this to the sheet as the domain-level entry it
  is. No picture: the subject is who may issue an identity, not what it draws.

## D4 — Does §4's displacement sentence get amended, now that the code does something else?

- **Proposal**: amend §4 with one sentence saying that while a reference has no offsets a
  displacement adds nothing to it and the position does not move, marked as what #143 replaces once
  #83 lands.
- **Altitude**: domain — the document says at its own head to amend it when reality contradicts it,
  and §4's own sentence is what B4 declines to implement. Confidence: medium.
- **If this is wrong**: the model states a rule the code does not follow, which is the drift that
  header exists to prevent. Answering D1 with the three-field form settles this entry the other way,
  because the sentence becomes implementable and needs no amendment.
- **The alternative**: leave §4 as written, on the argument that the model is design intent and #143
  will replace the sentence anyway — which is what the specification's "none of them amended" says.
- **Yours to answer**: yes — the model's prose is the maintainer's, and here the specification and
  the model disagree. No picture: B4's pair already shows the difference, and the entry is about
  which sentence stands.
