# Pull request templates

Three, because the repository has three shapes of change and one body cannot serve them.

| File                                                         | For                                                     | Answers                                                      |
| ------------------------------------------------------------ | ------------------------------------------------------- | ------------------------------------------------------------ |
| [`deciding.md`](deciding.md)                                 | Stage 1 of a feature                                    | Is this what is wanted, and is this how it will be resolved? |
| [`building.md`](building.md)                                 | Stage 2 of a feature                                    | Does it work, and is it what was agreed?                     |
| [`../pull_request_template.md`](../pull_request_template.md) | A change to the repository's own tooling, docs or rules | What changes, why now, and what breaks?                      |

## Why the tooling one is not in here

GitHub picks `.github/pull_request_template.md` by default and reaches this directory only through a
`?template=` query parameter, which nobody types. The tooling body is the one a human opens by hand
— the two feature stages are opened by `cargo xtask spec` — so it lives at the path GitHub already
offers, and keeping a second copy in here would be two identical files to hold in sync for a
selector nobody uses.

```text
.github/
├── pull_request_template.md              # tooling; GitHub's default for a PR opened by hand
└── PULL_REQUEST_TEMPLATE/
    ├── deciding.md
    └── building.md
```

**Not built yet.** The two stage bodies are meant to be passed to `gh pr create --body-file` by
`cargo xtask spec`, which knows which stage it is opening. It does not do that yet, and it still
opens the three branches of the arrangement ADR-0051 replaced. Until it does, copy the right file
into the body by hand:

```sh
gh pr create --base main --title "..." --body-file .github/PULL_REQUEST_TEMPLATE/deciding.md
```

## Why every section stays

A section with nothing to say says **"None."** rather than being deleted. A missing section reads as
an oversight and costs the reviewer a question; an explicit "None." is an answer, and for three of
these sections it is the answer that should be there most of the time:

- _Records this stage wrote_ — "None." means the slice decided nothing durable, which is the normal
  and desirable case.
- _Decisions taken here_ — "None, everything was on the sheet." is what a working decision sheet
  produces, and anything else is the signal that the sheet is guessing.
- _Blast radius_ — "Nothing; it is additive." is common, and is worth stating rather than leaving to
  be inferred.

## Why these sections and not the ones in use today

Across 53 merged pull requests there is no template, and the bodies use more than twenty distinct
headings for what is largely the same handful of ideas: _What was verified, not assumed_, _Measured,
not assumed_, _Evidence_, _Verification_ and _Accepted on observation_ are five headings for
principle IV. _What is here_, _What is in it_, _Scope_ and _Summary_ are four for one.

That is the same failure the constitution already refuses elsewhere: two wordings of one thing is a
thing that gets followed at random. Fixing it is worth more than the wording of any single section,
which is why these three files are deliberately dull.

What survives from current practice, because it was already right:

- **Summary** and **Test plan**, the two headings that were already near-universal (20 and 19 of
  53).
- **Before / after** with generated pictures side by side, which PR #109 already does and which is
  the single best thing in the repository's review history. It is now mandatory rather than
  occasional.
- **Worth a reviewer's attention**, which appeared twice and is the most useful section a solo
  maintainer can write for their future self.
