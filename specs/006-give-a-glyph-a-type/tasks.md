---
description: "Task list for feature 006, give a glyph a type of its own"
---

# Tasks: Give a glyph a type of its own

**Input**: Design documents from `/specs/006-give-a-glyph-a-type/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/](contracts/public-api.md),
[quickstart.md](quickstart.md)

**Tests**: Included. The spec requests them explicitly — SC-002, SC-003, SC-004 and SC-008 each name
what must be asserted, and _Testing_ in the constitution makes unit tests for core logic the minimum
any spec may ask for.

**Tests are not separate tasks, and not written first.** The template's default asks for failing
tests before implementation; that cannot hold here, because _Demonstrable increments_ requires every
commit to leave the gate green, and a commit whose tests fail is not green. So the tests for a
behavior ride in the same task and the same commit as the behavior. What the spec pins down is that
each rule has a test naming it, not the order in which the two are typed.

**Organization**: by user story, which here is also by increment. One task per commit, per
_Demonstrable increments_.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story the task belongs to
- Exact file paths are in the descriptions

## Path Conventions

`crates/monospace-core/src/` holds the domain logic; `crates/monospace-cli/` is the consumer and is
not touched. Tests live in `#[cfg(test)] mod tests` inside the module they cover, as every existing
module in the crate does — no test file is added.

---

## Phase 1: Setup

**Purpose**: capture the evidence that SC-001 is measured against, before anything changes.

- [x] T001 Capture the front end's current output to `../glyph-baseline.txt`, outside the
      repository, following _Capture the baseline first_ in [quickstart.md](quickstart.md)

T001 produces no commit: its output is evidence for the pull request body, and committing it would
create a second copy of what `crates/monospace-cli/tests/cli.rs` already asserts. It is a task
because skipping it turns SC-001 from a measurement into a claim, which is exactly what _Claims are
measured, not assumed_ forbids.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Empty, and deliberately so.** Nothing blocks both stories. Validating the fifteen Light rules
cannot happen before `Glyph` exists, so it belongs to US1; the dependency serves the widened
predicate, so it belongs to US2. A phase running before both would put the dependency ahead of the
increment whose diff is meant to measure what it bought, which is what SC-006 counts.
[plan.md](plan.md) records this under the Constitution Check.

**Checkpoint**: nothing to wait for — US1 can begin.

---

## Phase 3: User Story 1 - A validated glyph of one character (Priority: P1) 🎯 MVP

**Goal**: a caller has a type that can only hold one non-control character, and the catalog and the
renderer speak in it. The rendered output does not move.

**Independent Test**: construct a glyph from each single-character row of the table under _Examples_
in the spec and assert which are accepted; build the built-in Light catalog and ask it for every key
a light cell can produce; run `cargo run -p monospace-cli` and diff against the T001 baseline.

- [x] T002 [US1] Add `Glyph` with `new` and `as_str`, its rustdoc and its unit tests, in
      `crates/monospace-core/src/glyph.rs`, and re-export it from `crates/monospace-core/src/lib.rs`
- [x] T003 [US1] Move the catalog and the renderer onto `Glyph` in
      `crates/monospace-core/src/glyph.rs` and `crates/monospace-core/src/render.rs`, adapting the
      existing assertions through `as_str`

**T002 covers**: FR-001 to FR-006 and FR-016 — a `String` payload validated as exactly one `char`
that is not in Unicode's `Cc` category ([data-model.md](data-model.md), V1 to V3), `Option` rather
than `Result`, a private payload with `new` as the only way in, and the derives
[contracts/public-api.md](contracts/public-api.md) lists. Its tests are the P1 rows of the spec's
_Examples_ table plus the format-character boundary from SC-008. The rustdoc says what is accepted
and that neither width nor normal form is part of the invariant.

**T003 covers**: FR-007 to FR-010, and FR-016 again for the items it adds. The rules map becomes
`HashMap<GlyphKey, Glyph>`; the Light table's row type carries `&'static str` for the glyph so that
no row moves in US2 (FR-014); `glyph` answers `Option<&Glyph>`; `light()` panics through an `expect`
naming the offending rule and carries the `# Panics` section [research.md](research.md) R5 explains;
`render` collects glyph text with `" "` as the fallback. The test walking all fifteen rules lands
here (SC-003), and the existing catalog and rendering assertions are edited mechanically with no
test added or removed (SC-004).

**Checkpoint**: the gate is green, the front end's output is byte-for-byte identical to the T001
baseline, and `crates/monospace-cli/` does not appear in the diff. US1 is a complete increment and
could stop here.

---

## Phase 4: User Story 2 - The invariant widens to a grapheme cluster (Priority: P2)

**Goal**: a caller can hold `é` decomposed, a flag, or an emoji built from several code points —
anything that occupies one cell — while every control character stays refused.

**Independent Test**: construct a glyph from each multi-code-point row of the spec's _Examples_
table and assert it is accepted; assert `"\r\n"` is still refused; diff the front end's output
against the T001 baseline again.

- [x] T004 [US2] Widen the invariant to one grapheme cluster in
      `crates/monospace-core/src/glyph.rs`, adding `unicode-segmentation` to
      `crates/monospace-core/Cargo.toml`, correcting `render`'s rustdoc in
      `crates/monospace-core/src/render.rs`, and adding the tests for what the widening now accepts

**T004 covers**: FR-011 to FR-014 and FR-017 to FR-018. The predicate becomes "exactly one extended
grapheme cluster, with no `char` in Unicode's `Cc` category"; the storage does not change, which is
why no call site and no table row moves. Its tests are the P2 rows of the spec's _Examples_ table
plus the two normal forms of `é` comparing unequal (SC-008).

FR-017 is the one part of this task that is not in `glyph.rs`. `render`'s rustdoc promises lines
"exactly `size.width` characters wide", and this is the increment that makes the sentence false,
since a glyph may now be several characters. It becomes a promise in glyphs — the same rectangle,
and no longer the same character count.

**The dependency and this task are one commit on purpose.** Splitting them would leave a commit
carrying a dependency nothing uses, and would break the only claim this feature makes about its own
process: that the second increment's diff equals what the dependency bought (SC-006).

**Before pinning**: verify the chosen version's publication date is at least seven days old, and
name the version and that date in the commit message (SC-007). If every release is newer than that,
stop and say so rather than choosing. [research.md](research.md) R2 records why this happens here
and not at plan time.

**Checkpoint**: the gate is green, the output still matches the baseline, and `git show --stat` on
this commit touches no public signature and no call site T002 or T003 introduced.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [x] T005 Observe FR-009's panic on purpose by breaking one row of the Light table in
      `crates/monospace-core/src/glyph.rs`, then restoring it, following _Verifying the loud
      failure_ in [quickstart.md](quickstart.md)
- [x] T006 Append the increment's entry to `docs/learning-log.md`

T005 produces no commit either, and no test: a test that asserts the panic would have to ship a
broken table to fire it. What it produces is an observation for the pull request body, and it is
there because _Claims are measured, not assumed_ asks a new failure path to be made to fail on
purpose rather than assumed to work.

T006 is what ends the increment, per _Demonstrable increments_. The entry covers what was learned
about Rust design and idiom and about working this way, and each lesson names what was tried and
what it turned out to be.

**Not a task here**: pointing the frozen `docs/specs/0004-give-a-glyph-a-type-of-its-own.md` at
`specs/006-give-a-glyph-a-type/`, which ADR-0021 asks for. That is a documentation change to the
spec this feature replaces, and it belongs to the pull request that lands the spec rather than the
one that lands the code.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies, and it must happen before T002 or the baseline is worthless
- **Foundational (Phase 2)**: empty, blocks nothing
- **US1 (Phase 3)**: after T001
- **US2 (Phase 4)**: after US1, because it widens what US1 built
- **Polish (Phase 5)**: after US2

### Within and between the stories

T001 → T002 → T003 → T004 → T005 → T006, strictly. Every arrow is a real dependency:

- T003 needs the type T002 introduces.
- T004 changes the predicate T002 wrote and relies on the table shape T003 settled.
- T005 needs the panic T003 introduces.

### Parallel Opportunities

**None, and that is worth stating rather than inventing some.** No task carries `[P]`: every one of
them edits `crates/monospace-core/src/glyph.rs`, and each depends on the one before it. Two people
could not take US1 and US2 at the same time either, because US2 widens what US1 built — which is why
this feature is one branch and one pull request rather than one per story. The next feature, whose
two stories serve different audiences, is where branch-per-story starts paying.

---

## Implementation Strategy

### MVP first (US1 only)

1. T001, to have something to compare against.
2. T002, then T003, each with `cargo xtask check` green and each its own commit.
3. **Stop and validate**: diff the output against the baseline, and confirm `crates/monospace-cli/`
   is absent from the diff.
4. This is a complete increment. If the dependency were refused or unavailable, the feature would
   still have shipped something worth having.

### Incremental delivery

1. US1 → the type exists, is in use, and nothing visible moved.
2. US2 → the type holds what a reader sees, and nothing visible moved again.
3. Polish → the panic observed once, the learning log appended, the pull request opened with the two
   diffs as its evidence.

### One task, one commit

Every task above is one commit that leaves `cargo xtask check` green. T002, T003 and T004 are all
`feat`: each adds behavior, and each adapts the call sites its own change forces. None of them is a
structural commit, so none of them owes _Structural and behavioral change never share a commit_ an
exception — [plan.md](plan.md) works through why the input document thought otherwise.
