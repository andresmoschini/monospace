# Decisions: Displacing a figure that holds a reference moves it

**Feature**: #143 | **Written**: 2026-10-01 | **Answered**: 2026-10-01 — all three, two as proposed
and D2 wider than its proposal

Three entries, all domain-level, all the maintainer's. **None carries a picture, and that is a
finding rather than an omission**: every option pair below draws the same picture, because each
entry is about _where_ a picture or a record lives rather than about what one looks like. The four
pictures that do differ are measured in [research.md](research.md) Q2, and the rule they belong to
was answered in the specification's 2026-10-01 clarification, so it is not asked again here.

**D1 and D3 are confirmed as written; D2 is answered wider than the proposal it carried.** The sixth
picture stays and the gallery gains a third block, so the specification amends **B3** and **SC-004**
beside the amendment D3 already asks for.

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
- **Answer**: confirmed — one record for this rule alone, beside and citing
  [ADR-0041](../../docs/decisions/0041-resolve-a-position-through-a-reference.md), on the building
  branch rather than here. A record is sized to its subject, and the other three rules were never
  weighed: 082 and 083 took them as the model restating, so a record over all four carries options
  considered and pros and cons invented after the fact. Revising ADR-0041 is declined — its subject
  is resolution, a displacement resolves nothing, and a 178-line record would only grow.

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
- **Answer**: **both**, which is wider than the proposal. The demonstration's sixth picture stays
  and is the evidence the defect shipped: today's sixth block is the fifth byte for byte
  ([research.md](research.md) Q1), and its test can name the cells that changed, the only claim that
  shows both boxes stood still. The gallery gains a third `block()` beside the two it holds, because
  it is the one carrier whose picture is **drawn by the code this slice changes**: a marker cannot
  displace anything
  ([ADR-0035](../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md),
  [ADR-0064](../../docs/decisions/0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md))
  and a hand-drawn block goes stale silently, so a broken rule drops a snapshot there and nothing in
  a document. One asks whether the bug was real, the other what the rule draws. The snapshot grows a
  five-row picture and sixteen surface rows, and **B3**, **SC-004** and **Testing expectations** are
  amended here beside D3's.

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
- **Answer**: confirmed — `spec.md`'s **Testing expectations** names `shape.rs:179-183` beside the
  two `position.rs` paragraphs and the test, on this branch. It is the one worth naming twice over:
  `Shape::displaced_by` is `pub` where `Position::displaced_by` is `pub(crate)`, so it is the
  paragraph a caller reads first, and it is the only one of the four that asks the question rather
  than answering it.
