---
status: accepted
date: 2026-09-04
decision-makers: Andrés Moschini
---

# Pin the Rust toolchain to an exact version

## Context and Problem Statement

The brief requires a stable toolchain but leaves open whether the repository pins one. This phase is
building a quality gate whose steps — `rustfmt`, `clippy` — are tools whose opinions change between
releases. A lint added in a new stable release, or a formatting rule that changes its mind, turns a
green gate red without anyone having touched the code.

There is a second problem underneath it. The gate depends on components (`rustfmt`, `clippy`) and on
a compilation target (`wasm32-unknown-unknown`) that are not part of a default Rust installation. If
those are assumed rather than declared, the gate works on the machine it was written on and fails,
or worse silently skips, anywhere else.

## Decision Drivers

- A failing gate must be attributable. When it goes red, the cause should always be the change being
  made, never the date.
- The project must be workable in a clean environment without a list of manual setup steps that can
  go stale.
- The brief constrains the project to the stable channel.
- Reproducibility over currency: falling a few weeks behind the newest compiler costs nothing here,
  while an unexplained gate failure costs an afternoon.

## Considered Options

- **A** — `channel = "1.98.1"`: an exact version.
- **B** — `channel = "stable"`: track the channel, declare the components.
- **C** — No `rust-toolchain.toml` at all.

## Decision Outcome

Chosen option: **A, an exact version**, because it is the only one under which the quality gate
cannot change behaviour without a commit that says so.

The file fixes four things:

1. **`channel = "1.98.1"`.** Updating it is its own task, with its own commit and its own
   learning-log entry, rather than something that happens to the project while nobody is looking.
2. **`profile = "minimal"` plus explicit `components`.** `rustfmt`, `clippy` and `rust-src` are
   installed; `rust-docs` is not, since `cargo doc --no-deps` does not need the offline standard
   library documentation. `rust-src` is included because rust-analyzer degrades noticeably without
   it, and a clean clone should give a working editor, not merely a working build.
3. **`targets = ["wasm32-unknown-unknown"]`.** The check that enforces ADR-0001's boundary needs
   this target. Declaring it here means it installs itself rather than being a setup instruction
   someone has to find.
4. **Stable-only `rustfmt` configuration.** The formatting options worth having — `group_imports`,
   `imports_granularity` — are nightly-only, and using them would require a second toolchain in
   every clean environment and in CI. Nightly also cannot be pinned in an equivalent way: pinning a
   dated nightly produces a reference that rots, and the unstable options it enables can change
   behaviour between dates. Spending the reproducibility this ADR buys in order to sort imports is a
   bad trade, so `rustfmt.toml` stays on defaults.

### Consequences

- Good, because the gate is deterministic across machines and across time. Whatever CI does, a
  developer can reproduce exactly.
- Good, because a clean clone needs no toolchain setup instructions: running any cargo command
  installs the pinned toolchain, its components and the wasm target.
- Bad, because rustup treats `stable` and `1.98.1` as different installations even when they are the
  same version, so pinning downloads a second toolchain — measured at roughly 1.4 GB for the
  existing `stable` install, less here because of `profile = "minimal"`. Disk cost is paid once per
  machine and can be reclaimed by removing the now-unused `stable` toolchain.
- Bad, because staying current is now manual work. Nothing announces that a new release exists, so
  the pin will drift behind unless updating it is treated as a recurring task.
- Neutral, because rustup's auto-installation on a bare `rustup` subcommand is deprecated and warns
  that it may be removed. Cargo's proxies still install the pinned toolchain on demand, but the
  documented setup command should be `rustup toolchain install`, which reads this file and does not
  rely on deprecated behaviour.

### Confirmation

`rustup show active-toolchain` reports
`1.98.1-x86_64-pc-windows-msvc (overridden by .../rust-toolchain.toml)`, and
`rustup component list --installed` shows `rustfmt`, `clippy`, `rust-src` and
`rust-std-wasm32-unknown-unknown` and nothing else. Any environment that builds the project has
therefore already applied this decision — there is no way to build with a different compiler by
accident.

## Pros and Cons of the Options

### A — Exact version

- Good, because `rustfmt` and `clippy` cannot change their minds between commits.
- Good, because it matches how the rest of the project's tooling is pinned, so there is one rule
  rather than one rule per ecosystem.
- Bad, because of the duplicate toolchain on disk and the manual update cadence.

### B — `channel = "stable"` with components declared

- Good, because components and targets still install themselves, which is most of the practical
  benefit.
- Good, because the project stays current for free.
- Bad, because the gate can go red on a day when nothing was committed, and the failure looks
  identical to one caused by the change in progress. That is precisely the signal this ADR is trying
  to protect.
- Bad, because two developers, or a developer and CI, can be on different compilers while both
  believe they are on "stable".

### C — No `rust-toolchain.toml`

- Good, because nothing to maintain and nothing downloaded.
- Bad, because the components and the wasm target become undocumented assumptions about the machine,
  which is exactly the failure mode of a gate that appears to pass.

## Reversibility

Trivially reversible: the decision lives in one four-line file, and deleting or editing it takes
effect on the next cargo invocation. Nothing else in the repository encodes the version.

The only cost that is not instantly recoverable is disk space already spent on downloaded
toolchains, which is reclaimed with `rustup toolchain uninstall` whenever wanted.

The stable-only `rustfmt` half is equally reversible in mechanics, but reversing it means accepting
a second toolchain in every environment, which is a larger change than editing a config file.

## Confidence

Medium-high (~80%).

This is the most easily reversed of the phase's structural decisions, so the confidence matters less
than usual. The residual doubt is about the update cadence rather than about pinning: an exact pin
that nobody ever bumps quietly becomes an old-compiler project, which is the opposite of the brief's
goal of learning current Rust.

What would prove this wrong: the pin still reading `1.98.1` in six months. The countermeasure is
that bumping it is a normal task with a learning-log entry, not maintenance chores nobody schedules.

## More Information

- `docs/brief.md` sections 5 and 8, for the stable-channel constraint and the open question this
  answers.
- [ADR-0002](0002-no-minimum-supported-rust-version.md), which explains why pinning a development
  toolchain is not the same thing as promising a minimum supported version.
- [ADR-0001](0001-virtual-cargo-workspace-under-crates.md), whose WebAssembly check is the reason
  the target is declared here.
