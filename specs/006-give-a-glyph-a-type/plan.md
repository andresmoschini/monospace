# Implementation Plan: Give a glyph a type of its own

**Branch**: `006-give-a-glyph-a-type` | **Date**: 2026-09-08 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/006-give-a-glyph-a-type/spec.md`

## Summary

Replace `char` with a validated `Glyph` across the glyph catalog and the renderer, in the two
increments the spec's stories name: first the type with a one-character invariant, threaded all the
way through so it is in use rather than declared; then the invariant widened to one grapheme
cluster, which is what the dependency buys. The rendered output does not move in either.

Two findings from Phase 0 change the shape the input document
([spec 0004](../../docs/specs/0004-give-a-glyph-a-type-of-its-own.md)) prescribed, and both make the
second increment smaller rather than larger:

- **The storage is a `String` from P1, not a `char`.** The agreed surface has
  `as_str(&self) -> &str`, and a `char` cannot honor it — there is nothing string-like to borrow
  from. So P1 stores a `String` and validates "exactly one character, not a control character", and
  P2 changes only the predicate. See [research.md](research.md), R3.
- **P1 is a `feat`, not a refactor that bends a rule.** The input document called it "a refactor
  rather than a structural commit" and apologized for editing existing assertions. It is neither: it
  adds a public type with new behavior, and adapting the call sites that its own signature change
  forces is part of that change, not a separate refactor riding along. Nothing goes in Complexity
  Tracking. See the Constitution Check below.

## Technical Context

**Language/Version**: Rust 1.98.1, edition 2024, pinned exactly in `rust-toolchain.toml`
([ADR-0003](../../docs/decisions/0003-pin-the-toolchain-exactly.md)). No nightly features.

**Primary Dependencies**: none for P1 — the one-character invariant is `char::is_control()` from the
standard library. P2 adds `unicode-segmentation`, the dependency
[ADR-0019](../../docs/decisions/0019-represent-a-glyph-as-a-grapheme-cluster.md) accepts. Its exact
version is deliberately **not** pinned here; see [research.md](research.md), R2.

**Storage**: N/A. A glyph owns its text and nothing is persisted.

**Testing**: `cargo test --workspace`, run by `cargo xtask check`. Unit tests live in
`#[cfg(test)] mod tests` inside the module they cover, as every existing module in the crate does.

**Target Platform**: platform-independent library. The gate also compiles `monospace-core` for
`wasm32-unknown-unknown`, which is what enforces
[The core stays portable](../../.specify/memory/constitution.md).

**Project Type**: Rust library (`monospace-core`) with a thin non-interactive consumer
(`monospace-cli`). The consumer is not touched by this feature.

**Performance Goals**: none for this slice. ADR-0019 accepts one allocation per glyph and records
what would reopen the question; the spec's _Out of scope_ names the trigger.

**Constraints**: the core's public API carries no CLI, TUI or terminal assumption. The rendered
output must not change, measured after each increment (SC-001).

**Scale/Scope**: fifteen built-in rules, two new public items (`Glyph::new`, `Glyph::as_str`), one
changed public signature (`GlyphCatalog::glyph`), one changed private table type.

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._ Re-checked after Phase 1;
both passes reached the same result.

| Principle                                       | Verdict | Why                                                                                                                                                                                                                                                        |
| ----------------------------------------------- | ------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| I. Process over product                         | PASS    | The slower route is the one taken: two increments where one would compile, so that what the dependency buys is visible as a diff. SC-006 is the measurement of whether it paid, rather than an assertion that it did.                                      |
| II. Demonstrable increments                     | PASS    | Each story ends with the workspace green and `cargo run -p monospace-cli` producing output; the spec's stories are the increments. One learning-log entry per pull request, per the agreed two-PR flow.                                                    |
| III. One definition of green                    | PASS    | No check is added, moved or configured. `cargo xtask check` is untouched by this feature.                                                                                                                                                                  |
| IV. Claims are measured                         | PASS    | "The output does not change" is verified by diffing real output after each story, not asserted (SC-001). The dependency's publication date is verified where the constitution attaches it — before adding or pinning, in the commit that does it (SC-007). |
| V. Structural and behavioral never share commit | PASS    | Every commit in this feature is behavioral: P1 adds a validating type and rewires the catalog and the renderer; P2 widens the invariant. Both are `feat`. See the note below.                                                                              |
| VI. Decisions recorded when taken               | PASS    | ADR-0019 already carries the representation decision. Phase 0 took no decision that outlives this feature; what it settled — module placement, storage type — is cheap to undo and belongs in the learning log by that principle's own test.               |
| VII. The core stays portable                    | PASS    | Everything lands in `monospace-core`. `Glyph` names no terminal concept, and the gate's `wasm` step compiles it for `wasm32-unknown-unknown` like the rest of the crate.                                                                                   |
| Constraints — language of the artifacts         | PASS    | Every artifact of this feature is in English; the conversation that produced it was not.                                                                                                                                                                   |
| Constraints — dependencies                      | PASS    | The maintainer was asked and ADR-0019 records the answer. The seven-day rule is honored at the commit that adds it, which is where the constitution puts it.                                                                                               |
| Constraints — testing                           | PASS    | Every behavior rule in the spec has a success criterion naming a test: FR-001 to FR-005 and FR-011 to FR-013 through SC-002, FR-009 through SC-003, FR-018 and the two clarified boundaries through SC-008.                                                |

**On principle V, since the input document said otherwise.** Spec 0004 described its first commit as
"a refactor rather than a structural commit" and explained why editing existing test assertions was
acceptable anyway. That framing was wrong in a way worth naming: a commit that introduces a public
type which refuses `\n` is not structural at all — it adds behavior, so it is a `feat`, and a `feat`
may touch tests. The mechanical `Some('│')` → `Some("│")` edits are not a refactor riding along;
they are the call sites the new signature forces, and splitting them out would produce a commit that
does not compile. The principle is satisfied by classifying the commit correctly rather than by
justifying an exception to it.

Inside P1 the work still goes expand then migrate, as the principle asks of anything larger: add
`Glyph` with its own tests first, then move the catalog and the renderer onto it. Both steps are
`feat`, both leave the gate green, and each is one task and one commit.

**There is no foundational work.** Validating the fifteen Light rules cannot happen before `Glyph`
exists, so it belongs to P1; the dependency serves the widened predicate and belongs to P2. A phase
that ran before both would put the dependency ahead of the increment whose diff is supposed to
measure what it bought, which is the one thing SC-006 is there to count.

## Project Structure

### Documentation (this feature)

```text
specs/006-give-a-glyph-a-type/
├── spec.md              # /speckit-specify + /speckit-clarify output
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── public-api.md    # Phase 1 output
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # /speckit-tasks output — not created here
```

### Source Code (repository root)

```text
crates/monospace-core/
├── Cargo.toml           # P2 adds the dependency here
└── src/
    ├── lib.rs           # Re-exports Glyph alongside GlyphCatalog and GlyphKey
    ├── glyph.rs         # Glyph, GlyphKey, GlyphCatalog, and the Light table
    ├── render.rs        # Collects glyph text instead of characters
    ├── buffer.rs        # Untouched
    ├── cell.rs          # Untouched
    ├── geometry.rs      # Untouched
    └── stroke.rs        # Untouched

crates/monospace-cli/
├── src/main.rs          # Untouched
└── tests/cli.rs         # Untouched, and its absence from the diff is the proof (SC-001)
```

**Structure Decision**: `Glyph` goes into the existing `glyph` module rather than a module of its
own. That module's own documentation already says it holds "glyph rules and the catalog they are
looked up in", and the type those rules map to belongs with them; adding it costs one line of module
documentation and no structural churn. The alternative — a `glyph.rs` for the type and a
`catalog.rs` for the rest, mirroring how `stroke.rs` holds `Stroke` alone — would mean moving the
catalog first, which is a structural commit this feature does not need. It stays available:
splitting later is a pure `refactor` with no caller affected, since `lib.rs` re-exports every name
from the crate root.

Tests stay in `#[cfg(test)] mod tests` next to what they cover, as `glyph.rs`, `render.rs` and
`buffer.rs` already do. No test file is added.

## Complexity Tracking

Empty, deliberately. The Constitution Check has no violations to justify: the one that looked like a
violation — a commit that edits existing test assertions — turned out to be a misclassification in
the input document rather than a departure from the principle. Recording a justification here for a
violation that does not exist would leave a reader looking for a problem that was never there.
