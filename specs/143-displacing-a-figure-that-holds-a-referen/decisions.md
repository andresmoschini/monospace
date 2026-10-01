# Decisions: Displacing a figure that holds a reference moves it

**Feature**: #143 | **Written**: 2026-10-01 | **Answered**: _pending_

Three entries, all domain-level, all the maintainer's. **None carries a picture, and that is a
finding rather than an omission**: every option pair below draws the same picture, because each
entry is about _where_ a picture or a record lives rather than about what one looks like. The four
pictures that do differ are measured in [research.md](research.md) Q2, and the rule they belong to
was answered in the specification's 2026-10-01 clarification, so it is not asked again here.

## D1 — What the record's subject is

- **Proposal**: one ADR for the reference case alone, beside and citing
  [ADR-0041](../../docs/decisions/0041-resolve-a-position-through-a-reference.md).
- **Altitude**: domain — the altitude test is `Shape::displaced_by`, which `monospace-cli` calls, so
  the rule is observable outside the crate whatever it says. Confidence: high on the altitude, low
  on the scope.
- **If this is wrong**: a record that says "a displacement reaches a reference's offsets" and no
  more, leaving the rules 082 actually established — every position a figure holds, a connector
  through both its endpoints, a displacement that builds a value and changes nothing in a diagram —
  still unrecorded and still only in §9 and in rustdoc.
- **The alternative**: one ADR for the whole displacement subject, which would name 082's rules and
  this one together; or none, ruled out by the altitude above.
- **Yours to answer**: yes — **no ADR in this repository mentions displacement at all**, so there is
  no record to revise and the subject's edges are being drawn by this slice whether it means to or
  not.
- **Answer**: _pending_

## D2 — Which carrier holds the rule's own picture

- **Proposal**: the crate's gallery. `an_endpoint_hangs_from_a_side_and_follows_it` already holds
  this exact arrangement and its box-displaced block, so the rule's block is a third `block()` call
  in a test that exists, beside the direction that already works.
- **Altitude**: domain — it decides whether a second file gains blocks and whether a snapshot grows,
  and the gallery is the crate's own account of what it can draw. Confidence: medium-high.
- **If this is wrong**: the rule's picture exists only in the demonstration's sixth block and in
  hand-drawn blocks inside two specification artifacts, so nothing in the code shows it and the
  crate's gallery keeps showing one direction of two.
- **The alternative**: the demonstration's sixth picture alone, which is the specification's plan
  and is the better evidence — the no-op reproduces there — but leaves the rule's own picture in
  prose; or a second gallery test rather than a third block, which costs a file and a snapshot for
  one picture.
- **Yours to answer**: yes — the specification committed to the sixth picture and not to this, so
  taking it further is a decision rather than an implementation detail.
- **Answer**: _pending_

## D3 — Whether the specification names the fourth declaration

- **Proposal**: amend `spec.md`'s **Testing expectations** to name `shape.rs:179-183` beside the two
  `position.rs` paragraphs and the test, on this branch.
- **Altitude**: domain — it decides whether the deciding branch merges a corrected specification or
  a known-incomplete one, and the maintainer owns what that branch claims. Confidence: high.
- **If this is wrong**: the building stage rewrites four paragraphs while the specification names
  three, and the gap is discovered by a reader comparing the diff to the spec rather than by the
  plan.
- **The alternative**: leave the spec and correct it in the building stage, which is one line in a
  commit that is already rewriting the paragraph — at the price of the deciding branch merging a
  specification that understates the slice by one file.
- **Yours to answer**: yes — it is a change to a record this branch merges.
- **Answer**: _pending_
