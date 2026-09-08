<!--
Maintainer note, stripped before this file reaches Claude's context.

The line below is an import, not a mention: Claude Code expands `@path` at launch, so the
constitution is in context from the first message of every session instead of being a file Claude
has to remember to open. Relative paths resolve against this file, which is why it starts at
`.specify/`.

To confirm it loaded, run `/context` and look for the constitution under "Memory files". If it is
missing, nothing errors — Claude simply works without the rules, which is the failure mode worth
checking after touching this file.
-->

# Monospace

ASCII diagramming project in Rust. What it is, what is in scope, and every rule that governs a
change to it are in the constitution, imported here so it is in context rather than waiting to be
read:

@.specify/memory/constitution.md

Everything below is what only this file owns. Where the constitution states a rule, it is not
restated here — two wordings of one rule is a rule that gets followed at random.

## Working with me

- Do not invent something new because I did not know the ecosystem well enough to ask for the usual
  thing. When it is settled practice, the conventional answer is the answer.
- Talk to me in whatever language I write in. What lands in the repository is English regardless.

## Commands

```sh
cargo xtask check          # the whole quality gate; the hook and CI run this and nothing else
cargo xtask fix            # every automatic fix the gate knows about
cargo xtask setup          # install the Node tooling the gate needs
cargo run -p monospace-cli # run the CLI; a bare `cargo run` is ambiguous
cargo test --workspace     # tests only, for a faster loop
```

`CONTRIBUTING.md` covers setup, which tool owns which file, and what to do when a check fails.
