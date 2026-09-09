---
status: accepted
date: 2026-09-08
decision-makers: Andrés Moschini
---

# Take a feature's number from its GitHub issue

## Context and Problem Statement

A feature currently gets its number from `create-new-feature.sh`. The script scans `specs/` for
directories matching a three-or-more-digit prefix, takes the highest, and adds one. Nothing else in
the toolchain has an opinion about that number, and nothing outside the clone is consulted to
produce it.

That worked while one person worked on one branch at a time. The move recorded in
[ADR-0023](0023-direction-and-backlog-in-a-github-project.md) ends that assumption: the backlog is a
GitHub Project so that more than one person can see what is being worked on, and a capability issue
is now the input to `/speckit-specify`. Two people starting two features from two clones both scan
their own `specs/`, both find the same highest prefix, and both are assigned the same number. No
warning fires, because from inside each clone the number is genuinely free. The collision surfaces
at merge, as two directories claiming one number and two branches named after it.

Renumbering after the fact is not a rename of a directory. A feature number is quoted in pull
request titles and branch names, in the learning log, and in the notices the frozen specs carry to
point at what replaced them. Whichever of the two features loses the argument has its paths
rewritten everywhere they were already cited, and the record of what was asked for stops matching
the record of when it was asked.

So the number has to come from somewhere that cannot hand it out twice, and the decision cannot wait
for the second person: the first concurrent pair is the event that costs, and by then the convention
is whatever it was.

## Decision Drivers

- The collision must be impossible by construction rather than by discipline. This is the same
  reasoning
  [one definition of green](../../.specify/memory/constitution.md#iii-one-definition-of-green-non-negotiable)
  applies to the quality gate: a rule enforced by everyone remembering is a rule that holds until
  someone is busy.
- [Claims are measured, not assumed](../../.specify/memory/constitution.md#iv-claims-are-measured-not-assumed):
  what the current scripts do was read before this record was written, not inferred from Spec Kit's
  documentation. One of the four things assumed about them turned out to be false — see
  Confirmation.
- The board is now where a feature starts. An identifier the board already owns costs nothing to
  adopt; a second identifier alongside it is a mapping someone has to maintain.
- [Process over product](../../.specify/memory/constitution.md#i-process-over-product-non-negotiable):
  the point of the team rehearsal is to find out what breaks when more than one person works this
  way. Numbering by hand would postpone exactly the thing being rehearsed.

## Considered Options

- **A** — Sequential, as today. `create-new-feature.sh` keeps scanning `specs/` and assigning
  highest + 1.
- **B** — Timestamp prefixes. Set `feature_numbering` to `timestamp` in
  `.specify/init-options.json`, giving `20260908-143022-slug`.
- **C** — The issue number. The feature takes the number GitHub assigned to the issue that
  represents it, passed to `create-new-feature.sh` as `--number`.
- **D** — Reserve the number by hand. Whoever starts a feature announces the number they are taking
  before creating anything.

## Decision Outcome

Chosen option: **C**, the issue number, because GitHub assigns it and therefore no two features can
be assigned the same one, and because the identifier already exists at the moment the feature
starts.

The directory, the branch, the milestone and the issue are meant to share one identifier from that
moment. That is the intended arrangement, not an observed one: nothing enforces it, and the history
already shows it is not automatic — PR #15 and PR #16 both delivered spec 006, from two branches
named `006-give-a-glyph-a-type` and `006-implement-give-a-glyph-a-type`. A shared number makes the
correspondence possible to state; keeping it is review, as everything below is.

The cost accepted with it is a numbering that skips. Issues are opened for things that are not
features — [issue #13](https://github.com/andresmoschini/monospace/issues/13) is a tooling item — so
the sequence under `specs/` will read `023`, `031`, and never explain the gaps from inside itself.
What it buys is that the gap is meaningless while the number is not: given `031`, the issue is
`#31`, and the issue is where the wish, the discussion and the milestone are.

**Features 001 through 006 keep their numbers.** Renaming `specs/006-give-a-glyph-a-type/` would
break PR #15, PR #16, the learning log entry for spec 006 and the notice in
[`docs/specs/0004`](../specs/0004-give-a-glyph-a-type-of-its-own.md) that points at it. The new
scheme applies to the first feature created after this record, and nothing before it is touched. The
discontinuity is visible and explained here, which is better than a renumbering that would make the
history read as though this had been decided earlier than it was.

One detail of the mechanism is worth stating because it is a trap rather than a feature: passing
`--number` is a preference, not an instruction. When the number's prefix is already used under
`specs/`, the script does not fail and does not take the next free number above the one requested —
it restarts from the highest existing prefix and increments from there, warning on stderr that it
used a different number instead. So a mistyped `--number 31` in a repository whose highest prefix is
`006` silently yields `007`, and the directory no longer names its issue. The warning is the only
signal, and it is one line among the command's output.

### Consequences

- Good, because the collision is gone by construction. Two people cannot be handed the same number,
  and neither of them has to check anything to be sure of it.
- Good, because the number stops being a local fact. Today it means "the seventh directory in this
  clone"; afterwards it means "issue 31", which is the same thing in every clone and on the board.
- Good, because it costs one argument at creation time. `--number` already exists and already
  accepts an arbitrary value, so nothing vendored is patched and `specify update` cannot undo this.
- Bad, because **the identity of a spec is coupled to the tracker.** A directory name becomes a
  pointer into GitHub, and it is only resolvable while the project is on GitHub. This is the cost
  with no mitigation, and it is the second time in this increment that a decision has moved
  something out of the clone.
- Bad, because **the sequence stops being readable as a sequence.** `023` followed by `031` does not
  say how many features there were, or that none is missing. `ls specs/` no longer answers "what has
  been specified" without the board open beside it.
- Bad, because **the discontinuity at 006 is permanent.** From here on the tree holds numbers
  produced by two different rules, with nothing in the tree saying which is which. Only this record
  says it.
- Neutral, because the mistyped-`--number` trap described above is not introduced by this decision —
  it is in the script today. What changes is how often it can matter: with numbers assigned
  externally, `--number` is passed on every run rather than never.
- Neutral, because [ADR-0021](0021-move-the-spec-home-to-spec-kit.md) has to be superseded in the
  part where it settled that the sequence continues at 006, and only in that part. Its conclusion —
  `specs/` is the spec home and `docs/specs/` is frozen — stands untouched and is not reopened here.
- Neutral, because that partial supersession needs a status form the ADR template does not list. It
  reads `accepted; superseded in part by ADR-0024`, which keeps `accepted` accurate — the decision
  is still in force — while making the partial reversal findable in the front matter rather than
  only in prose.

### Confirmation

Enforced by review, and less than that: **enforced by whoever types the command.** Nothing checks
that a directory's number matches an issue, and nothing can. The ten steps of `cargo xtask check`
are `fmt`, `prettier`, `markdownlint`, `editorconfig`, `cspell`, `clippy`, `build`, `wasm`, `test`
and `doc`; `xtask/src/main.rs` holds no reference to `specs/` and no notion of a feature. A
directory named `042-anything` with no issue behind it passes the gate.

Four claims about the current behavior were checked by reading the scripts before this record was
written. Three held and one did not.

- **The number comes from scanning the local `specs/`.** True. `SPECS_DIR` is `$REPO_ROOT/specs`,
  hardcoded, and `get_highest_from_specs` iterates its subdirectories, keeps those matching
  `^[0-9]{3,}-` while excluding timestamp directories, extracts the prefix and returns the maximum;
  the assigned number is that plus one. No network, no git.
- **The auto-correction for an explicit `--number` also sees only the local tree.** True.
  `spec_prefix_exists` globs the requested prefix under `$SPECS_DIR`, and on a conflict the retry
  loop restarts from `get_highest_from_specs` and increments until no local prefix collides. Its
  warning says as much: "conflicts with an existing spec directory".
- **Nothing parses the `NNN-` prefix.** True, apart from the two matches above, which belong to the
  allocator itself. Downstream, `FEATURE_NUM` is formatted with `printf "%03d"` and reported in the
  command's JSON and stdout, and nothing reads it back: `check-prerequisites.sh`, `setup-plan.sh`
  and `setup-tasks.sh` take `FEATURE_DIR` whole and append file names to it. The prefix is a label
  whose only reader is the code that assigns it.
- **Resolution in `common.sh` falls back to the branch name.** _False_, and it was worth checking,
  because a branch-name fallback would have made the branch part of the identity rather than a
  convention beside it. `get_feature_paths` resolves the feature directory from
  `SPECIFY_FEATURE_DIRECTORY`, then from `feature_directory` in `.specify/feature.json`, then fails
  with `ERROR: Feature directory not found` — there is no third source. `get_current_branch` returns
  `SPECIFY_FEATURE` or the empty string and never invokes git; there is no `git rev-parse`,
  `symbolic-ref` or `git branch` anywhere under `.specify/scripts/`. The dependency runs the other
  way: when no feature identifier is set, `CURRENT_BRANCH` is filled in from the feature directory's
  basename. The branch is an output of the resolution, not an input to it.

Two things follow from that last point, and they shape what this decision can promise.
`create-new-feature.sh` neither creates nor switches a branch — that belongs to a git extension
invoked through `hooks.before_specify` in `.specify/extensions.yml`, and neither the extension nor
the file is present, so every branch in this repository so far was created by hand. And the feature
actually in scope is held in `.specify/feature.json`, which `.specify/.gitignore` excludes as
machine-local per-checkout state. So the identifier shared by the directory, the branch, the
milestone and the issue is shared because someone types it four times consistently.

What has not been observed: no feature has yet been created under this scheme, and no two clones
have ever raced for a number. The collision described in Context and Problem Statement is a reading
of the allocator, not an incident.

## Pros and Cons of the Options

### A — Sequential, as today

- Good, because it is what the tool does with no arguments, and the sequence reads as a sequence:
  `006` is the sixth feature.
- Good, because the number means something with no network and no tracker.
- Bad, because the collision is silent in the only case that matters. Concurrent work in two clones
  produces two features with one number, and each is locally valid until they meet.
- Bad, because the repair is renumbering, and renumbering rewrites paths already cited in pull
  requests, issues and the learning log — the cost this record exists to avoid paying later.

### B — Timestamp prefixes

- Good, because collision is impossible by construction, and it is one line in
  `.specify/init-options.json`: the script already supports `--timestamp`, and `/speckit-specify`
  reads the `feature_numbering` value.
- Good, because it needs nothing outside the clone — no issue, no board, no network.
- Bad, because `20260908-143022-slug` is not something anyone says out loud, types into a branch
  name, or matches to an issue by eye.
- Bad, because the readable sequence is gone and nothing replaces it. Sort order is the only
  information the prefix carries, where an issue number at least resolves to a page holding the
  wish.

### C — The issue number

- Good, because GitHub assigns it, so two features cannot receive the same one.
- Good, because the identifier already exists when the feature starts, and it is the same one the
  board, the milestone and the pull request use.
- Bad, because the numbering skips, and the tree cannot explain its own gaps.
- Bad, because it couples the identity of a spec to the tracker, which is the one cost here that
  cannot be bought back.

### D — Reserve the number by hand

- Good, because it keeps the readable sequence and needs no tooling at all.
- Bad, because it depends on people announcing an intention before acting, which is the class of
  rule this project already refuses to rely on for the quality gate.
- Bad, because it fails precisely under the conditions that motivate it: more people, working at the
  same time, not all of them watching the same channel.
- Bad, because it postpones the failure the team rehearsal is meant to surface, and would let the
  rehearsal report success for the wrong reason.

## Reversibility

Cheap in one direction, and asymmetric in the same way ADR-0023 is.

Going back to sequential numbering costs nothing for the features already created: they keep their
names, exactly as 001 through 006 keep theirs under this decision. A second discontinuity is added
to the tree, and the third rule has to be written down somewhere. Nothing has to be renamed, because
the whole point of the arrangement is that a number is never reused.

Switching to timestamps later has the same shape and the same price: one line of configuration, one
more discontinuity.

What is permanent from the moment it lands is the gap. Once `specs/` holds `023` and `031`, no later
decision can make the tree read as a sequence again without renaming directories that pull requests
and the learning log already cite — which is the cost this record refuses to pay for 006, and would
have no better reason to pay then.

## Confidence

Medium-high (~75%).

What would change it: wanting to read `specs/` as a chronology. An issue number is not one — issues
are opened in an order that has little to do with when their features get built — and timestamp
numbering is better at exactly that, at the cost of one line in `.specify/init-options.json`.

What would prove it wrong: leaving GitHub. Every directory name would become a pointer into a
tracker that no longer exists, and unlike a broken link there would be nothing to repair it to.

What would not change it: the ugliness of the gaps. That was weighed and accepted, and it is the
visible half of a trade whose other half — a collision that cannot happen — is invisible precisely
when it is working.

## More Information

- [ADR-0021](0021-move-the-spec-home-to-spec-kit.md), which settled that the sequence continues at
  006 rather than restarting. This record supersedes that part of it and no other.
- [ADR-0023](0023-direction-and-backlog-in-a-github-project.md), which put the backlog on a board
  and made a capability issue the input to `/speckit-specify`. That is what gives a feature an issue
  number before it has a directory, and what makes concurrent creation something to plan for.
- [The constitution](../../.specify/memory/constitution.md), "Development Workflow", whose
  `specs/NNN-slug/` is amended in the same increment as this record to say where `NNN` comes from.
- `.specify/scripts/bash/create-new-feature.sh` and `.specify/scripts/bash/common.sh`, the two files
  read for the Confirmation section above. Both are vendored and rewritten by `specify update`, so
  what is recorded here is their behavior at spec-kit v1.0.4.
