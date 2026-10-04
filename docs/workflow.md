# Working a change, end to end

> **The long form.** [`CONTRIBUTING.md`](../CONTRIBUTING.md) is the rules; this is the procedure,
> walked through one issue with a person and an agent each named at every step. It repeats what the
> rules say, and deliberately: this flow replaced a heavier one, and the reasoning for how it works
> is worth more in one place than scattered across four files that each hold a piece.

Everything here starts from a GitHub issue carrying a phrase. The example is
[#90](https://github.com/andresmoschini/monospace/issues/90), "A shape offers its corner and center
anchor points", labelled `capability` and `wish` — which is what a first idea looks like before
anybody has written anything down.

## The one question

Before anything is opened:

> **Does this change force a choice between more than one defensible answer?**

| If                                                    | Rung       | Branches                              | Pull request                 | Spec                |
| ----------------------------------------------------- | ---------- | ------------------------------------- | ---------------------------- | ------------------- |
| No — the model says what to do and the code disagrees | **Fix**    | `NNN-slug`                            | the default body             | none                |
| No — the model is silent or already agrees            | **Slice**  | `NNN-slug-building`                   | `building.md`                | none                |
| **Yes**                                               | **Decide** | `NNN-slug-deciding`, then `-building` | `deciding.md`, `building.md` | `specs/NNN-slug.md` |

Two of the three add nothing to the ordinary shape of a pull request. That is the design: a stage is
worth having only if it is cheap enough that you never skip it.

**A Fix or a Slice has no spec, and that is the point.** The spec exists to hold a decision that has
not been made yet. On those two rungs there is no decision, so there is nothing to record beyond
what the code and the model already say.

### Worked: what #90 answers

`docs/diagram-model.md` §5 opens:

> A shape may offer four named points: the center of its top side, of its right side, of its bottom
> side and of its left side.

And further down:

> Four, rather than the nine issue #62 lists. What the corners and the center are waiting for is a
> consumer, and issue #90 is where they arrive.

And in §1, the vocabulary table:

> | `Anchor` | One of four named points a shape may offer: the center of each of its sides |

Three sentences say four. And `Anchor` is a `pub enum` in a library crate that `monospace-cli`
mirrors by hand. **More than one answer is defensible** → **Decide**.

## The steps

### 0 — Before anything

**Person.** Once per machine: `cargo xtask setup`, then `cargo xtask check` green.

**Agent.** Nothing. If the gate is not green before the work starts, nothing measured afterwards
means much.

**Check that.** `cargo xtask check` prints that all checks passed.

### 1 — Open the wish

**Person.** Write the issue: a title and two or three sentences. No acceptance criteria, no
requirements, no examples of behavior — that is what the next steps are for.

**Agent.** Nothing yet.

**Check that.** The issue says what someone wants, not what has to be built. Evidence is the
exception and it is worth a lot: [#104](https://github.com/andresmoschini/monospace/issues/104)
carries the smallest arrangement that shows the problem and says "16 of the 1856 renderings, in 8
arrangements taken from both ends". That is a measurement, and it beats a paragraph of
specification.

### 2 — Read the model and settle the rung

This step decides everything after it. It is reading, not writing.

**Person.** Read the section of the model the topic belongs to and answer the question. If you are
unsure, stop and read: this is the moment for it.

**Agent.** Find the sentence. Something like:

> Read §5 _Anchor points_ and the `Anchor` row of §1 _Vocabulary_ in `docs/diagram-model.md. Tell me
> which sentences would be false if #90 were implemented whole, quoting them.

**Check that.** You can quote at least one sentence. Concluding "the model says nothing about
corners" means you did not read — a model that enumerates four anchors says four.

### 3 — Draw the case

The step that removes the most rework, and the cheapest one. Twenty lines.

**Person.** Say which case you want to look at. You do not need to know what you expect to see.

**Agent.** Write the fixture, run it, and put the result where it belongs. Four carriers already
exist:

| To see                                     | Use                                                           |
| ------------------------------------------ | ------------------------------------------------------------- |
| A picture in a tracked document            | A `<!-- render: -->` marker, then `cargo xtask render`        |
| A picture that exists, to pin it           | A gallery block, which moves on its own if the picture moves  |
| A picture in a spec or a pull request body | The same marker; specs and bodies both take one               |
| A quick case, for you only                 | A test that prints, or a JSON description run through the CLI |

Then answer the only question that matters: **what does the drawing say that you did not say
before?**

**Worked: here it cannot be drawn.** A connector attached to a box's _corner_ has no path today —
`Anchor` has no such variant, so no carrier reaches it. That is not a failed step, it is the result,
and it is a useful one. The spec says "this cannot be drawn yet" in one line, which is worth more
than a paragraph describing how it would probably look, and it draws the nearest thing that does
exist: a reference with an offset. That shows why the named form matters — an offset gets you _near_
a corner, not _on_ one, and cannot tell a corner from any point of a side.

The 142 decided whether a removal froze or dropped a connector **without drawing either case**,
which is why its cell counts were wrong three times and the decision sheet had to be reopened twice.

### 4 — The deciding pull request

Only on the **Decide** rung.

**Person.** Open the branch, write the spec, fill the body, review it, open it, merge it.

```sh
cargo xtask feature new 90 --rung decide
```

The rung comes from the issue's labels unless you say otherwise — `bug` is a fix, a capability or a
wish is a slice — and it is always printed with its reason, so a default taken on faith reads as
one. **Decide is never a default**: it is the only rung where a wrong answer costs something, and
reading the model is what puts a change there.

**Agent.** Before anything is opened, the decisions with their rejected alternatives, in the shape
the template asks for, and the spec at `specs/090-a-shape-offers-its-corners.md` copied from
[`.github/spec-template.md`](../.github/spec-template.md).

**Check that.**

- `cargo xtask pr open` refused nothing, which means the tree was clean and no section is empty. A
  section with nothing to say says **"None."** — a missing section reads as an oversight.
- The body says `Refs #90`, not `Closes`.
- The spec's status line names a pull request, and the `specs` step of the gate is happy.

**What the spec carries, and why it is longer than the body.** It is **more precise**, because it
outlives the conversation. Name things — `Anchor`, not "the anchor type". Say what happens, with the
values. Carry the picture. Record counts measured rather than estimated. The body is what a reviewer
reads today; the spec is what someone reads in a year.

Two of its five sections are the ones most often left out, and both matter. **Public surface**,
because this is a library and a caller who is not in the conversation cannot be asked afterwards —
for #90 it names `Anchor` gaining five variants, a caller's exhaustive `match` needing an arm, and
`monospace-cli`'s hand-written mirror in `AnchorDescription`. **What proves it**, because a proposal
nobody can observe is a wish with better formatting, and finding that out during the deciding stage
costs one conversation instead of a merged pull request.

**The merge**, and then the second branch. The command refuses until the deciding pull request has
merged, and it asks GitHub rather than reading a file:

```sh
cargo xtask feature build 90
```

Its three refusals are distinct on purpose — "pull request #N is open, not merged", "no deciding
pull request was ever opened", and whatever `gh` itself said. A message that only says "not merged"
sends you looking; one that names the pull request does not.

### 5 — Build

**Person.** The `pre-commit` hook runs the gate, so a commit that breaks it does not happen. Run it
yourself when you want the answer before committing rather than after:

```sh
cargo xtask fix     # when you are about to touch several files
cargo xtask check
```

You decide where a commit ends. One rule: **the commit leaves the tree green.** The hook checks the
working tree, not the staged snapshot, so with unstaged changes present it verifies files that are
not exactly the ones being committed — which is why running it by hand is still worth it.

**Agent.** Most of the work, in this order:

1. **Write the test first where you can.** It is the cheap way to find out whether the rule is
   understood. A test naming something that is not there does not fail, it does not compile.
2. **The tests are the spec.** Every rule in _What proves it_ becomes an assertion. A rule with
   nothing against it is named as such in the pull request rather than counted as covered.
3. **When the code shows the model was wrong, amend the model in this pull request.** Not optional,
   and not an escape — see _The model moves_ in `CONTRIBUTING.md`. For #90 that is the three places
   in §1 and §5.
4. **Measure rather than assume.** If a characterization sweep moved, the count goes in the body.
5. **Do not quietly build something else.** If building showed a decision was wrong, say so in the
   body and in the spec's last section.

**Check that.** `cargo xtask check` green — and one manual check worth doing once per change:
**break the rule on purpose.** Remove the guard that makes the behavior work, run the tests, and
count what went red. The 142 did this and measured 2 red, 76 green, which is what says the tests are
wired to the rule rather than to the method. If nothing goes red, the tests are not testing the
rule.

### 6 — Open the pull request

**Person.**

```sh
cargo xtask pr body      # writes target/pr-body.md with the keyword already appended
                         # ... fill every section; one with nothing to say says "None."
cargo xtask pr open      # pushes and opens
```

**Agent.** Most of the body, especially the two parts that cost the most to write by hand: _Before /
after_ with generated pictures, and _The sweep_ with the real count rather than an estimate.

**Check that.** `cargo xtask pr open` did not refuse, and the pull request says `Closes #90` — a
deciding pull request says `Refs`, the building one closes.

### 7 — Merge and close

**Person.** Yours, not delegated:

```sh
gh pr view N --json closingIssuesReferences   # confirm GitHub parsed the Closes
gh pr merge --merge --delete-branch
```

Read the first line. An empty `closingIssuesReferences` means GitHub did not understand the keyword
and the issue is still open, which is the check most often skipped.

**Agent.** After the merge, and only if the pull request asks: accept the snapshots that moved, with
`INSTA_UPDATE=always cargo test --workspace`. **A characterization snapshot is not self-approving.**
Each carries a description demanding a report of how many cases moved, in which families, and
examples with before and after. Reading the diff and saying "that looks right" is the exact failure
those files exist to prevent.

**Check that.** The issue is closed, the branch is gone, and — on a **Decide** rung — the spec's
status line says whether the code still matches the decisions.

## When the deciding stage is skipped

A **Fix** or a **Slice** goes straight from step 2 to step 5. No spec, no deciding pull request, one
branch. The suffix still tells `cargo xtask pr` which body to write, and `cargo xtask feature build`
on either of them says so rather than opening a third branch for one change.

The deciding stage used to be the default for every feature. It is not any more, because it was
routinely merged and then reopened by the building stage that followed it. A proposal written at
that size is not read closely enough to be agreed with.

## What a person does and an agent does

Not a division of labor — two different kinds of attention.

|                                      | A person                    | An agent                                            |
| ------------------------------------ | --------------------------- | --------------------------------------------------- |
| Reads the model and settles the rung | decides                     | finds and quotes the sentence                       |
| Draws the case                       | says which case             | writes the fixture and reads the picture            |
| Writes the decisions                 | decides, and is accountable | drafts the table and the rejected alternatives      |
| Writes the spec                      | reviews                     | drafts it                                           |
| Builds                               | decides where a commit ends | writes code, tests and the model amendment together |
| Opens the pull request               | reviews it                  | writes the body                                     |
| Merges and closes                    | does it                     | does not                                            |

The line is **judgement**. A person decides what is true, what is wanted, and whether the change is
finished. An agent does the parts where being wrong is cheap and being slow is expensive: reading,
drafting, measuring, and writing down what was measured.

## Three questions afterwards

Worth answering, because the next decision — whether this stays or gets heavy again — depends on
them.

1. **How much did you write that was neither code nor a pull request body?** Zero is the good
   answer.
2. **Was there a moment when you wanted to open a plan?** If so, the rung was misapplied in step 2,
   or something is missing from this flow.
3. **Did step 3 teach you anything?** If drawing the case was uninformative, the step is costing
   time. If it was not, it was the cheapest step in the flow.

## What this flow does not fix

Stated so it is not oversold.

- **It does not replace having someone look.** The deciding stage earns its place because a second
  reader exists. With one person and one agent it is still the moment for the attentive read — not a
  ritual that can be skipped for free.
- **It does not check the specs against the code.** The `specs` gate step checks shape: a status
  line, five sections, and a revision that recorded itself. Whether the code does what the spec says
  is a reader's job, running it. Nothing mechanical gets there, which is why step 3 exists.
- **It does not measure itself.** If question 3 above answers zero and question 1 answers zero, the
  flow is costing ceremony, and the way back is the question in step 2.
- **It does not fix broken links.** Nothing in the gate checks them. See
  [Cross-references](../CONTRIBUTING.md#cross-references).
