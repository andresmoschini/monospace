# Pull request templates

Three, because the repository has three shapes of change and one body cannot serve them.

| File                                                         | For                                                     | Answers                                                      |
| ------------------------------------------------------------ | ------------------------------------------------------- | ------------------------------------------------------------ |
| [`deciding.md`](deciding.md)                                 | Stage 1 of a feature                                    | Is this what is wanted, and is this how it will be resolved? |
| [`building.md`](building.md)                                 | Stage 2 of a feature                                    | Does it work, and is it what was agreed?                     |
| [`../pull_request_template.md`](../pull_request_template.md) | A change to the repository's own tooling, docs or rules | What changes, why now, and what breaks?                      |

## How the right one is reached

`cargo xtask pr body` picks it, from the branch the working copy is on: `-deciding` and `-building`
take the two files here, and anything else is a tooling change and takes the default. It writes the
file to `target/pr-body.md` with the keyword line already appended — `Refs` for a deciding pull
request, which does not finish the issue, and `Closes` for the other two. Fill it in, then
`cargo xtask pr open`, which refuses a body whose sections are still empty.

```sh
cargo xtask pr body        # on a feature branch; the issue number comes from the branch
cargo xtask pr body 34     # on a tooling branch, which carries none
cargo xtask pr open        # pushes, then opens it with the body and a derived title
```

## Why the tooling one is not in here

GitHub picks `.github/pull_request_template.md` by default and reaches this directory only through a
`?template=` query parameter, which nobody types. Since `cargo xtask pr` passes `--body-file`, the
selector is never used at all, and the default is what a pull request opened in the browser gets —
which is the tooling case. Keeping a second copy in here would be two identical files to hold in
sync for nothing.

```text
.github/
├── pull_request_template.md              # tooling; GitHub's default, and what `pr body` copies
└── PULL_REQUEST_TEMPLATE/
    ├── deciding.md
    └── building.md
```

## When it is opened, which is not automatic

`cargo xtask pr` is typed, never triggered. Neither pull request is due when a Spec Kit command
finishes — the deciding one waits for the sheet to be answered, which no command does, and the
building one waits for a green `cargo xtask check`. Both moments are the maintainer's; what the tool
removes is the part that was never a judgement, which is which body and which keyword.

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
