---
status: accepted
date: 2026-09-04
decision-makers: Andrés Moschini
---

# Use a Node toolchain for the checks Rust cannot perform

## Context and Problem Statement

The quality gate needs four things the Rust toolchain does not provide: spell checking, Markdown and
JSON formatting, Markdown linting, and verification that commit messages follow Conventional
Commits. A fifth, checking files against `.editorconfig`, was also wanted.

The tools chosen for those jobs — cspell, prettier, markdownlint-cli2, commitlint,
editorconfig-checker — are all published on npm and have no equivalent in the Rust ecosystem that
covers the same ground. cspell in particular was chosen deliberately, because it is already in use
in the editor and its dictionaries can be committed to the repository.

So a Rust project acquires a second toolchain. The question is not really whether to use these tools;
it is how they are installed, pinned and invoked, given the constraint that the project must be
workable in a clean environment without a list of manual setup steps.

## Decision Drivers

- Nothing may be assumed about the machine. A fresh clone plus one documented command has to produce
  a working gate.
- Versions must be pinned as precisely as the Rust toolchain is, or the reasoning behind pinning the
  compiler is undermined by tools that float.
- The gate runs on every commit through the pre-commit hook, so its cost per run matters.
- The project's dependency policy is to keep dependencies few and justified.

## Considered Options

- **A** — npm with `package.json` and a committed `package-lock.json`, installed into the repository.
- **B** — A Rust-native toolset instead: `typos` for spelling, `dprint` for Markdown and JSON.
- **C** — The same npm tools, installed globally with `npm install -g`.

## Decision Outcome

Chosen option: **A, npm with a committed lockfile**, because it is the only option that pins exact
versions of the chosen tools and installs them without assuming anything about the machine.

Three details are fixed along with it, each of them measured rather than assumed:

1. **npm rather than pnpm, yarn or bun.** Every alternative has to be installed before it can install
   anything, which is a bootstrap before the bootstrap; npm ships with Node. Corepack could
   provision one from a `packageManager` field, but its place in the Node distribution has been
   unstable and it adds a step. The one thing npm is worst at — speed — is removed from the hot path
   by point 2.
2. **The gate never installs; it verifies.** A no-op `npm install` was measured at 1.8–2.7 s, which
   is more than every Rust check put together (1.3–1.5 s). Instead the gate compares the timestamp
   of `package-lock.json` against `node_modules/.package-lock.json`, the record npm writes when it
   installs. That is two `stat` calls and costs nothing measurable. When it finds the tree missing or
   stale it refuses to run anything and says `cargo xtask setup`, which runs `npm ci`.
3. **Tools are invoked as `node_modules/.bin/<tool>`, never through `npx`.** npx adds roughly a
   second per invocation — `prettier --version` takes 1.3 s through npx and 0.27 s directly. Across
   four tools that would be about four seconds of pure overhead, tripling the gate while checking
   nothing extra.

### Consequences

- Good, because every tool version is exact in `package.json` and the whole transitive tree is
  pinned by `package-lock.json`, which is the same standard the Rust toolchain is held to.
- Good, because setup is one command that needs no instructions beyond its own name.
- Good, because the gate refuses to run a partial pass. Reporting the Rust half as green while the
  Node tools are missing would be the exact failure this project keeps designing against.
- Bad, because a Rust project now requires Node to be installed to run its own quality gate. That is
  a real barrier for a Rust contributor, and it cannot be waved away by pointing out that the tools
  are good.
- Bad, because 44 MB and 9,425 files appear in `node_modules` for six direct dependencies, which pull
  in 282 entries in the lockfile. Six dependencies were approved; roughly 280 arrived. This is normal
  for npm and is exactly what the project's dependency policy exists to be uncomfortable about.
- Neutral, because the first install takes about 27 seconds. It is paid once per clone and once per
  lockfile change, never during a normal gate run.

### Confirmation

Enforced by the entry point itself: `cargo xtask check` will not run a single step while the Node
tree is missing or older than the lockfile, and says what to run instead. There is no way to get a
green gate without the tools actually being installed at the pinned versions.

## Pros and Cons of the Options

### A — npm with a committed lockfile

- Good, because `npm ci` installs strictly from the lockfile and deletes anything that does not
  belong, so two machines end up with identical trees.
- Good, because the tools live inside the repository and cannot be shadowed by whatever happens to
  be installed globally.
- Bad, because it introduces a second package ecosystem, with its own lockfile, its own update
  cadence and its own supply-chain surface.

### B — A Rust-native toolset: typos and dprint

- Good, because it avoids Node entirely: `cargo install` for both, one ecosystem, one lockfile.
- Good, because typos produces very few false positives without configuration, being
  correction-based rather than dictionary-based.
- Bad, because it does not cover the chosen ground. cspell was picked on purpose for its dictionaries
  and its existing use in the editor, and typos catches a different and smaller class of error: it
  does not flag a correctly spelled wrong word.
- Bad, because commitlint has no Rust equivalent of comparable standing, so Conventional Commit
  verification would have to be hand-written or dropped. Node would likely arrive anyway, later and
  less deliberately.

### C — The same tools installed globally

- Good, because there is no `node_modules` in the repository and nothing to commit.
- Bad, because versions are whatever each machine happens to have, so the gate can pass locally and
  fail in CI for reasons no file in the repository explains. This defeats the point of pinning.
- Bad, because it turns setup back into a list of manual instructions that will go stale.

## Reversibility

Reversible per tool, much less so as a whole.

Any single tool can be swapped for another, in npm or outside it, by editing one gate step and one
dependency. That stays cheap indefinitely.

Removing Node altogether is a different matter. It is easy today, with four tools depending on it and
no configuration written yet, and it gets harder with every dictionary, ignore file and rule
exception accumulated in their configs. Those files are the real lock-in, not the dependency entries.

The lockfile itself is disposable: it can be regenerated at any time, at the cost of picking up newer
transitive versions.

## Confidence

Medium-high (~75%).

The mechanics — npm, a committed lockfile, verify-don't-install, no npx — are measured and I am
confident in them. The residual doubt is about the premise: whether four Node tools earn their keep
in a Rust project whose stated policy is to keep dependencies few and justified. Roughly 280
transitive packages for six approved direct ones is a large answer to a small question.

What would change this: finding that the Node tools are rarely the ones catching problems. If a
year's worth of gate failures are overwhelmingly clippy and tests, this toolchain is 44 MB of
ceremony and the Rust-native option deserves a second look. What would confirm it: cspell and the
Markdown formatter catching things in documentation that no Rust check could ever see.

## More Information

- [ADR-0003](0003-pin-the-toolchain-exactly.md), which sets the pinning standard this decision
  applies to a second ecosystem.
- The measurements quoted here were taken on one Windows 11 development machine with a warm disk
  cache. They are orders of magnitude, not benchmarks, and the conclusions rest on the ratios rather
  than the absolute numbers.
