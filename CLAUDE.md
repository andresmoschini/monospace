# Monospace

ASCII diagramming project. `monospace-core` is the library holding all domain logic (parsing,
layout, rendering); `monospace-cli` is a minimal non-interactive consumer that ships the
`monospace-cli` binary. The name `monospace` is reserved for the interactive TUI of a later phase.

Read `docs/brief.md` before proposing any change. It defines scope, principles and what is
deliberately out of scope. When a decision contradicts it, amend the brief in the same increment and
say so in the commit — never drift from it silently.

## Language

- Code identifiers, comments and doc comments: English.
- README, `docs/`, specs and ADRs: English.
- Commit messages and PR descriptions: English.
- CLI output, error messages and help text: English.

## Commands

```sh
cargo xtask check          # the whole quality gate; the hook and CI run this and nothing else
cargo xtask setup          # install the Node tooling the gate needs
cargo run -p monospace-cli # run the CLI; a bare `cargo run` is ambiguous
cargo test --workspace     # tests only, for a faster loop
```

`CONTRIBUTING.md` covers setup, which tool owns which file, and what to do when a check fails.

## Quality gate

- Every commit that lands must leave the build and all validations green.
- Never use `git commit --no-verify`. If the hook fails, fix the cause or stop and tell me — do not
  bypass it.
- The pre-commit hook and CI run the same validation entrypoint.
- A new check is added to that entrypoint, never to CI or to a hook directly. Two definitions of
  "green" is the failure this arrangement exists to prevent.
- Adding a check is two commits: first fix what it finds, then enable it. If it finds nothing, say
  so — do not manufacture a fix to fill the commit.

## Verification

- Do not assert behavior you have not observed, and least of all in an ADR or a commit message. Run
  it first, then write down what happened.
- Verify a new check by making it fail on purpose and then restoring. A green run only proves the
  command ran.
- Answer "is X faster, smaller, better?" with a measurement. Report a median over several runs and
  say when the noise is larger than the difference, rather than quoting a single run.
- Before trusting the gate, run it on a fresh clone. Files written by hand skip the transformations
  Git applies on checkout, so the working copy can be green while the repository is broken.
- Keep configuration minimal by removal: take each entry out and confirm something breaks. An entry
  that changes nothing is rot, and in a spell checker or a lint config it silently permits errors.

## Dependencies

- Do not add a dependency without asking me first.
- Before adding or pinning any version, verify it was published at least 7 days ago, and report the
  version and its publish date.
- Domain logic (`monospace-core`): prefer the standard library.
- Infrastructure concerns: prefer idiomatic, well-established crates.

## Workflow

- Do not write code before the plan is agreed.
- When a decision is mine — cost, scope, taste, or a risk I carry — give me the real options with
  their trade-offs, what is reversible and what is not, your recommendation with a confidence level,
  and what information would change it. Do not choose for me.
- When it is settled practice and I have no stake in it, choose the conventional answer, say what
  you chose and why, and carry on. Do not invent something new because I did not know the ecosystem
  well enough to ask for the usual thing.
- One task per commit.
- Follow Kent Beck's rule: "for each desired change, make the change easy (warning: this may be
  hard), then make the easy change." Split the preparatory refactor from the behavioral change, and
  never mix them in the same commit. Prefix messages so the distinction is visible in the log.
- A structural commit must not change behavior: the existing tests pass unchanged, and no test is
  added or modified.
- If the preparatory refactor turns out to be hard, stop and tell me before starting it. That is a
  design signal worth discussing, not something to push through.
- Large refactors go expand/contract: add alongside, migrate, then remove, each step green and
  committed separately.
- Architecture decisions go in `docs/decisions/` as an ADR, written when the decision is made.
- After each increment, append an entry to `docs/learning-log.md`. An increment is a slice of work
  that reaches a demonstrable state, not a commit — it usually spans several.
