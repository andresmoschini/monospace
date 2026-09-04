# Monospace

ASCII diagramming project. `monospace-core` is the library holding all domain logic (parsing,
layout, rendering); `monospace-cli` is a minimal non-interactive consumer that ships the `monospace`
binary.

Read `docs/brief.md` before proposing any change. It defines scope, principles and what is
deliberately out of scope.

## Language

- Code identifiers, comments and doc comments: English.
- README, `docs/`, specs and ADRs: English.
- Commit messages and PR descriptions: English.
- CLI output, error messages and help text: English.

## Commands

<!-- filled in once the validation entrypoint exists -->

## Quality gate

- Every commit that lands must leave the build and all validations green.
- Never use `git commit --no-verify`. If the hook fails, fix the cause or stop and tell me — do not
  bypass it.
- The pre-commit hook and CI run the same validation entrypoint.

## Dependencies

- Do not add a dependency without asking me first.
- Before adding or pinning any version, verify it was published at least 7 days ago, and report the
  version and its publish date.
- Domain logic (`monospace-core`): prefer the standard library.
- Infrastructure concerns: prefer idiomatic, well-established crates.

## Workflow

- Do not write code before the plan is agreed.
- One task per commit.
- Follow Kent Beck's rule: "for each desired change, make the change easy (warning: this may be
  hard), then make the easy change." Split the preparatory refactor from the behavioural change, and
  never mix them in the same commit. Prefix messages so the distinction is visible in the log.
- A structural commit must not change behaviour: the existing tests pass unchanged, and no test is
  added or modified.
- If the preparatory refactor turns out to be hard, stop and tell me before starting it. That is a
  design signal worth discussing, not something to push through.
- Large refactors go expand/contract: add alongside, migrate, then remove, each step green and
  committed separately.
- Architecture decisions go in `docs/decisions/` as an ADR, written when the decision is made.
- After each increment, append an entry to `docs/learning-log.md`.
