---
status: accepted
date: 2026-09-04
decision-makers: Andrés Moschini
---

# Declare no minimum supported Rust version, for now

## Context and Problem Statement

The brief lists MSRV as an open tooling question: what minimum Rust version does the project target,
and how is it enforced. The edition is already fixed at 2024, which puts a hard floor at Rust 1.85 —
below that the crates do not compile at all. The question is whether to promise anything at or above
that floor.

An MSRV is a compatibility promise made to consumers: "this crate builds on version X". Its cost is
paid by the author, who may not use standard-library APIs newer than X. Its benefit is collected by
whoever depends on the crate while pinned to an older toolchain.

Today the project has no consumers. Publishing to crates.io is intended eventually, but nothing
depends on these crates and nothing outside this repository builds them.

## Decision Drivers

- The brief's purpose is building proficiency in current, idiomatic Rust. An MSRV is a standing ban
  on the newest parts of the language and standard library.
- Enforcement is not free. A declared MSRV that nobody verifies is worse than none, because it is a
  promise that will be broken silently.
- The toolchain is pinned exactly (ADR-0003), so "which version does this build with" already has a
  precise answer for development purposes. MSRV answers a different question: which versions it
  builds with for someone else.

## Considered Options

- **A** — No MSRV: omit `rust-version` from the manifests.
- **B** — MSRV at 1.85, the edition floor, verified by a dedicated CI job.
- **C** — Floating MSRV, some number of releases behind stable, revised by hand.

## Decision Outcome

Chosen option: **A, no MSRV**, because the promise has no audience yet, and making it would forbid
the newest standard library while the whole point of the project is to learn current Rust.

This is explicitly provisional. The decision is revisited when either of these happens:

- Anything is published to crates.io. At that moment MSRV stops being a style question: someone can
  pin a version of these crates, and a release that silently raises the required compiler breaks
  their build without a major version bump to warn them.
- Anything outside this repository depends on the crates.

### Consequences

- Good, because any stabilized standard-library API can be used the day it lands, which is what the
  brief asks for.
- Good, because CI stays a single job. A verified MSRV needs a second toolchain and a second job; an
  unverified one is a lie waiting to be found.
- Bad, because the moment an MSRV is adopted, the code may already depend on recent APIs, and paying
  that debt is more work than never having incurred it. This is accepted knowingly: the debt is only
  owed if the project reaches an audience, and reaching one is not certain.
- Neutral, because omitting `rust-version` also gives up a small free benefit — Clippy reads that
  field and stops suggesting APIs newer than the declared minimum. That check only exists once there
  is a value to check against.

### Confirmation

Absence is the confirmation: no `rust-version` key in `[workspace.package]` or in any member
manifest. Nothing enforces the revisit triggers above; they are a note to the future, and the
learning-log entry for the release phase is where they should be re-read.

## Pros and Cons of the Options

### A — No MSRV

- Good, because it is honest. No promise is made, so no promise can be broken.
- Good, because it costs nothing to adopt and nothing to maintain.
- Bad, because "we support whatever is current" is unhelpful to a consumer evaluating the crate.

### B — MSRV at 1.85, the edition floor, verified by a dedicated CI job

- Good, because it is the widest promise the edition allows, and it is verified rather than claimed.
- Bad, because it bans roughly a year of standard-library additions in exchange for compatibility
  nobody is asking for.
- Bad, because the verification job interacts badly with an exactly pinned toolchain: `cargo +1.85`
  is silently overridden by `rust-toolchain.toml`, so the job has to set `RUSTUP_TOOLCHAIN`
  explicitly or it will pass while testing the wrong compiler. A gate that reports success without
  checking anything is the worst possible outcome, and this is an easy way to build one.
- Bad, because it requires a second toolchain download in CI on every run.

### C — Floating MSRV, some number of releases behind stable

- Good, because it is a real promise at a cost that stays bounded.
- Bad, because "revised by hand" means it drifts. Either the revision is automated, which is more
  machinery than this project justifies, or it is forgotten.
- Bad, because it inherits the same `RUSTUP_TOOLCHAIN` trap as option B.

## Reversibility

Fully reversible, and the cost is asymmetric in a way worth naming.

Adding an MSRV later is cheap in mechanics — a `rust-version` key and a CI job — but potentially
expensive in consequences, because by then the code may use APIs newer than the version being
adopted, and each one has to be found and replaced. That cost grows with the amount of code written,
not with time.

Removing an MSRV later is trivial mechanically but is a breaking change to a published promise, so
it is not really available once anything is published.

## Confidence

High (~85%) for now; the confidence is in the timing, not in MSRV being wrong in general.

MSRV is a genuinely good practice for a published library, and the recommendation to use one is
sound advice given without knowing that this repository has no consumers. What would change this
decision is exactly what the revisit triggers describe: an audience. What would prove it wrong is
discovering an external consumer that already exists, or deciding to publish earlier than planned
with a body of code that has drifted onto very recent APIs.

## More Information

- `docs/brief.md` section 8, which raised MSRV as an open tooling question.
- [ADR-0003](0003-pin-the-toolchain-exactly.md), which pins the development toolchain and is the
  reason "which compiler builds this" already has an answer without an MSRV.
