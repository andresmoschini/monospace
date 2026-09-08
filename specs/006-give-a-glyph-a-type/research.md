# Phase 0 research: Give a glyph a type of its own

**Feature**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Date**: 2026-09-08

The representation decision and its costs are not researched here. They were taken in
[ADR-0019](../../docs/decisions/0019-represent-a-glyph-as-a-grapheme-cluster.md), which owns the
options considered, the consequences and the confidence; this file links to it rather than standing
in for it. What follows is investigation local to this feature, and each item says whether it is
settled or verified at implementation time.

## R1 — What the dependency answers, and what it does not

**Decision**: `unicode-segmentation` supplies the cluster half of the invariant only. The control
half comes from the standard library's `char::is_control()`, which is exactly Unicode's `Cc`
category, and needs no dependency at all.

**Rationale**: the crate implements UAX #29 — it answers _where the boundaries are_ (grapheme
clusters, words, sentences). It carries no General_Category data, so it cannot answer "is this a
control character" or "is this a format character". In `unicode-rs` those tables live in a different
crate. This is what makes the spec's two-story split possible: P1 needs no dependency, because the
whole of a one-character invariant is `chars()` plus `is_control()`.

It is also what settled the first clarification. Refusing the `Cf` category alongside `Cc` would
have needed a second dependency _and_ would have refused the joiner that holds a multi-code-point
emoji together, contradicting FR-011.

**Alternatives considered**: adding a category crate (`unicode-general-category` or equivalent) to
refuse clusters made only of format characters. Rejected in the clarification, for the reason the
spec's _Assumptions_ records: it would enforce half of the width problem and document the other
half, and width is what ADR-0019 deliberately left outside the invariant.

**Verified at implementation**: the exact call for extended grapheme clusters is expected to be
`UnicodeSegmentation::graphemes(text, true)`. The commit that adds the dependency confirms the
signature against the pinned version's own documentation rather than against this note.

**Implementation note**: prefer taking two items from the iterator over counting it. "Exactly one
cluster" is `next().is_some() && next().is_none()`, which stops after two boundaries; `count() == 1`
walks the whole input, and the input comes from a caller.

## R2 — When the version is pinned

**Decision**: not here. The exact version is chosen in the commit that adds the dependency, and that
commit names the version and its publication date.

**Rationale**: the constitution attaches the seven-day rule to the act of adding or pinning, and
[SC-007](spec.md) attaches the report to the same commit. A version verified while the plan is
written is a fact with a shelf life — if implementation happens a week later, the number is either
re-verified or stale, and re-verifying makes the first check work that was thrown away. Nothing in
the plan or the design depends on which version it turns out to be.

**Alternatives considered**: pinning now and re-verifying at implementation. Rejected: it produces
one verification that is discarded and a number in a document that outlives its accuracy.

**Verified at implementation**: the publication date is at least seven days old, and it is reported
with the version. If every release is newer than that, the rule says stop and say so rather than
choose.

## R3 — What the type holds in P1

**Decision**: `Glyph` owns a `String` from P1 onwards. P1 validates "exactly one `char`, and not a
control character"; P2 changes that predicate to "exactly one grapheme cluster, with no control
character in it". The storage never changes.

**Rationale**: this corrects the input document, which prescribed "a `char` inside" for the first
commit. That cannot be built. The agreed public surface has `as_str(&self) -> &str`, and a `Glyph`
holding a `char` has nothing string-like to borrow from — `char::encode_utf8` needs a buffer the
caller provides, so there is no safe way to return a `&str` tied to `&self`. The type system refuses
the shape, which is the cheapest possible way to find out.

The correction makes the second increment smaller, not larger. With the storage settled in P1, P2's
diff is the predicate, the dependency, and the tests for what the widening now accepts — which is
exactly what SC-006 counts.

**Alternatives considered**:

- `Glyph([u8; 4])` with `str::from_utf8`, keeping `Copy` for one increment. Rejected: it buys a
  property P2 removes anyway, and it is an inline representation — the thing ADR-0019 explicitly
  parks until a measurement asks for it.
- Changing P1's surface to return a `char` and widening it in P2. Rejected: FR-014 forbids it, and
  it is the whole reason the input document had `new` take `&str` from the start.

## R4 — Where the type lives

**Decision**: in the existing `glyph` module, beside `GlyphKey` and `GlyphCatalog`.

**Rationale and alternatives**: in [plan.md](plan.md) under _Structure Decision_, which owns it. It
is cheap to undo, so by principle VI's own test it is learning-log material rather than an ADR.

## R5 — How the loud failure is spelled

**Decision**: `GlyphCatalog::light()` panics through an `expect` naming the offending rule, and
carries a `# Panics` section in its rustdoc.

**Rationale**: FR-009 asks for a loud failure on data this library ships, and `clippy::pedantic` is
warn-level workspace-wide while the gate runs clippy with `-D warnings`, so
`clippy::missing_panics_doc` will require the section as soon as the panic exists. The learning log
already records this same trade for `render`, where the section arrived with an `expect` and left
when the panic did — so the cost is known rather than estimated.

The message names which rule failed, because the fifteen rows are indistinguishable in a backtrace
and the panic exists to be diagnosed by whoever added a bad row.

**Alternatives considered**: returning a `Result` from `light()`. Rejected in the spec: a caller can
do nothing about a bug in shipped data, and an error type would be a value nobody branches on.
Silently dropping a bad rule was rejected for the reason FR-009 gives — it renders as a space, which
is indistinguishable from a key with no rule.
