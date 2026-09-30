<!-- The measured files below were kept under a Windows temporary directory, whose variable name is
     not a word: cspell:ignore LOCALAPPDATA -->

# Specification Quality Checklist: A description names its shapes, and carries the ordinal the next one takes

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-09-30 **Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Success criteria are technology-agnostic (no implementation details)
- [x] All acceptance scenarios are defined
- [x] Edge cases are identified
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
- [x] User scenarios cover primary flows
- [x] Feature meets measurable outcomes defined in Success Criteria
- [x] No implementation details leak into specification

## Notes

Three questions were put to the maintainer on 2026-09-30 rather than left as markers. All three are
recorded under **Clarifications** and all three changed the spec; the markers are gone.

- **P1 — is the identity required on every entry? → Required.** An entry without one is refused and
  named, like `canvas`, `shapes`, `leaving`, `terminal` and `at`'s `kind`. The cost was measured
  before it was asked about: **24 `<!-- render: -->` markers carrying a real description, 63 shapes
  between them, in nine tracked files** — `README.md`, `CONTRIBUTING.md`, `docs/diagram-demo.md`,
  `docs/diagram-model.md`, `docs/model.md`, `specs/055…/spec.md`, `specs/081…/data-model.md` and
  `specs/081…/decisions.md` — plus `assets/demo.json`'s 26 entries, for 89 lines in ten files. Every
  picture among them comes back byte for byte, and 23 of the 24 have a gate step re-drawing it,
  which is the cheapest check on a mechanical change of this size; 083's `c7539bc` is the precedent
  that moving a tag across the same markers left them untouched. The one marker with no step is
  `CONTRIBUTING.md`'s, which sits inside a ````markdown`fence` — re-spelled by hand, checked by eye.
  The alternative was refused because two ways of naming the same shape is exactly the ambiguity
  083's `at` tag exists to remove, measured in its `research.md` Q2.
- **P2 — two entries carrying the same identity → Accepted.** Both are held, the first is what every
  change and every reference finds, nothing errors. Refusing it was preferred only **if `serde`
  could refuse it in one line**, and it cannot. That was measured on this crate's own `serde_json`
  before the answer was written down: two entries carrying one identity are read without an error,
  exactly as one key written twice inside one object is — `serde` looks at nothing but the shape of
  the data, and nothing in it compares a value in one array element against a value in another. So a
  refusal would have to be written by hand, and the maintainer's reason for not wanting it is the
  governing one: nobody hand-edits these files, an editor is coming, and whether a file is well
  formed is not this binary's question. **The consequence is stated rather than absorbed**: §3's
  "unique within that diagram" is amended to be a sentence about the identities the diagram _issues_
  rather than a promise it keeps on a caller's behalf, which is a second amendment to the same
  section and part of what the ADR this slice needs has to record.
- **P3 — may an identity be free text or is it an ordinal? → Free text, and every file in this
  repository keeps writing `#1`, `#2`, …** §11's own trigger is "the first slice where a caller has
  a _name worth keeping_", so the capability is free text; leaving the shipped files ordinal-spelled
  is what keeps the demonstrable change the one 083 made — evidence in the file, pictures not
  moving. Re-spelling them with names is a later commit on the same format and not a change to it,
  so P1's re-spelling does not have to be done twice.

The three answers are not independent of one another and the spec says so where it matters: P1 makes
every identity in the repository kept by hand, and P2 declines to check the one mistake a hand-kept
identity can make. That asymmetry — a **missing** identity is refused, a **repeated** one is not —
is named under **What this slice does not decide** rather than left for a reviewer to notice.

Three things sit close to the line between requirement and design, and each is named in the spec
rather than smuggled in as a requirement:

- **Where the ordinal lives.** Issue #148 defers it to this stage explicitly, and B2 states only the
  observable half — the ordinal survives being read, and it is never one the diagram has handed out
  before. A field of the file and a counter read off the identities it carries are the sheet's, with
  the alternative and its cost.
- **How a caller reaches a chosen identity.** B1.3 says what the caller can do afterwards — find it,
  replace it, remove it, move it — and not what the diagram's surface is called. 082's D3 declined a
  public way to add under a chosen identity for being early; this is the slice it waited for, so the
  spelling is on the sheet with 082's declined option named.
- **Whether the identity is a string or something narrower.** §1 settles what a `ShapeId` is, and P3
  settles what the file may write into it. Which is a design question the two together hand to the
  plan.

Every picture in `spec.md` is measured rather than generated, and each block says so on the spot:
B1.1 is what `cargo run -p monospace-cli` prints for the two files it names, run on 2026-09-30 and
kept at `%LOCALAPPDATA%\opencode\148\`. `cargo xtask render` must not be asked to fill them — a
named shape is not expressible in the format `xtask` reads, and a marker carrying one would draw the
position-named picture while claiming the named one, which is the exact confusion this slice exists
to end. The same reason applies to the gate: a `<!-- render: -->` marker is the cheapest check on a
mechanical change, and it is unavailable to this one until the feature lands
([ADR-0064](../../docs/decisions/0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md)).

Two spec numbers are stated rather than estimated, and both were measured today: `assets/demo.json`
holds 26 entries and the 24 real markers hold 63 shapes.

**One item is not satisfiable as written: `spec.md` exceeds the 120-line ceiling in principle
VIII**, at 187 attributable lines against a ceiling of 120 — 225 in the file, less the four fence
markers, the thirty-three blank lines and the one comment, counted the way 083's own checklist
counted its 199. Fourteen of those lines are the two picture blocks in B1.1, which is the measured
pair the slice's claim rests on. That is a fact about the size of the slice rather than a formatting
slip, and the constitution asks for a split first. The figure is the one after `cargo xtask fix`, so
it is prettier's number rather than the one this file was drafted at.

The split available is the crate boundary, and it is the same one 083 declined: B1, B2 and B3.1–B3.3
are `monospace-diagram` and the refusal in B3.2 — a caller may place a shape under an identity it
chose, the ordinal survives being read, and a repeated identity is accepted — while the format half
is B3.2's message, B4 and the 89 lines P1 costs: the field exists, and every file in the repository
grows it. The second needs the first and the order has no cycle, so the first half carries no
contract revision and no demonstration change.

It does not reach the ceiling either way, so it is offered rather than recommended: the arithmetic
leaves the model half near 110 attributable lines and the format half near 95, which is the same
kind of overage 081, 082 and 083 all carried and which the maintainer declined to split for the same
reason — the split buys two thinner slices at the price of a second deciding cycle with its own
issue, branch, spec, decision sheet and pull request. The number and the reason go to `plan.md`'s
Complexity Tracking, which is where the constitution puts a ceiling exceeded without a split.

Deleting prose would bring this under 120 and would be the wrong move: principle VIII says an
artifact MUST NOT restate what another already says, and the paragraphs above the ceiling each carry
something that is nowhere else — the measured cost of P1, the `serde` measurement behind P2, the
asymmetry P1 and P2 create together. The three answers have fixed this number; it will not move
again.

This template has no **Assumptions** section and none is missing: the assumptions are the three
model sections named under **What this slice implements** — two of them amended — and the five
entries under **What this slice does not decide**, each with what would force an answer. The
dependencies are the two issues this spec names, 62 and 83, with 88 and the three ADRs it cites.

The `serde` measurement behind P2 was taken by adding a test to
`crates/monospace-cli/src/description.rs`, running it, and reverting it: the working tree carries no
trace of it and `git status` shows this directory as the only change. The measurement is
reproducible by anyone who wants to check the claim rather than take it — two entries carrying one
identity, and one key written twice inside one object, and both read without an error.
