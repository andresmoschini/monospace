---
status: accepted
scope: domain
commitment: load-bearing
date: 2026-10-02
decision-makers: Andrés Moschini
---

<!-- The feature directory named under More Information is the issue title truncated at forty
     characters by `cargo xtask spec`, which landed inside a word: cspell:ignore referen -->

# Freeze what hung from a removed shape where it stood

## Context and Problem Statement

[ADR-0041](0041-resolve-a-position-through-a-reference.md) settles that a reference naming a shape
that is not there resolves to nothing, and that the figure holding it is not drawn. Taking a shape
out is how a reference comes to name nothing, and the record's second consequence said that was the
right answer: "an editor can delete a shape without repairing everything that referenced it".

The consequence was wrong in a way only running the code found. The repair was not a decision
anybody had taken — it was what happened to nobody. Three routes to a picture — a reference to an
identity never added, the same reference after the shape was taken out, and the same reference over
an identity held by a figure that answers no side — all drew **one** picture, byte for byte, so
nothing in a diagram could tell them apart. And the delivered result was the loudest version of it:
taking the box out took fifteen cells of a twelve-by-three window with it, arrow included. So the
question is what a removal does to a reference naming the figure it removed.

## Decision Drivers

- **It must not be possible to remove a shape without updating the references to it.** Not that the
  default is convenient: there is no opt-out, so there is no sixth row in §9's table of five and
  nothing for a caller to forget.
- **A removal reaches the references naming the removed shape and nothing else.** A reference to
  another figure is left exactly as it was, and so is one naming an identity the diagram never held:
  the removal did not create that broken reference and does not repair it.
- **[ADR-0041](0041-resolve-a-position-through-a-reference.md)'s resolution rule is not re-argued
  here.** Its chain is one link long and its cycle question is #89's. What changes is the state of a
  reference whose subject is removed.
- **A frozen end is an ordinary absolute position.** No new variant, no new name, no second meaning
  for an attachment — a term here would be a term every later reader had to be taught.

## Considered Options

**The options differ in what they draw, so they are shown rather than described.** The arrangement
is §4's own: a box four cells by three at the origin and a connector whose `from` is a reference to
that box's right side. As written, under either option:

```text
┌──┐    ┌─┐
│  ├────│ │
└──┘    └─┘
```

**Chosen — freeze.** The box is taken out and the arm stands exactly where it stood. `{3, 1}` reads
`─` rather than `├`, because the box's own border used to compose with the arm into one junction and
the arm now stands alone. **Generated**: the gallery's fourth block in
`an_endpoint_hangs_from_a_side_and_follows_it`.

```text
        ┌─┐
   ─────┤ │
        └─┘
```

**Declined — leave it unresolved**, which is what the code does today. The same removal takes the
arm with it and the row where it stood is empty. **Hypothetical**: no tracked code produces this
picture now, and what produced it was measured before the rule landed.

```text
        ┌─┐

        └─┘
```

**Declined — repair, and report.** `remove` hands back what it froze, or lists the references it
broke: a signature and an answer nobody asked for, unreachable from outside the crate in any case.

## Decision Outcome

Chosen option: **freeze the references naming the removed shape, into the absolute point each was
resolving to**, because a removal is a statement about one figure and a caller deleting a box means
the arrows drawn on it are still arrows. A frozen end is `Position::Absolute`, which already exists:
`Position`, `Reference`, `Endpoint` and `Shape` keep their variants and fields, and `remove`'s
signature, visibility and receiver are unchanged. What changed is the **behavior** of one item 081
already recorded, which is why there is no fourth contract.

### Consequences

- Good, because the demonstration now shows the rule in the shipped run: a seventh picture that is
  the sixth with the box gone and **the arrow standing**, one `remove` call and one caption line in
  the CLI's own code. `assets/demo.json` and the description format are untouched
  ([ADR-0035](0035-keep-the-cli-demo-format-out-of-the-model.md)).
- Good, because a removal is no longer silent about what it did, and the two pictures simply differ.
  How anything would tell a removal from a shape that was never there — §11's second question — is
  **dissolved rather than answered**: nothing remembers that a removal happened.
- Good, because nothing can opt out: there is no flag and no separate change, so the diagram's own
  text has nowhere to record that references were left stale.
- **Bad, because a frozen end is a point and a point has no side to name.** A shape put back under
  the removed identity — through `add_under` — comes back, and the connector **does not re-hang from
  it**. Nothing went looking for the reference the frozen end was. A displacement reaching any
  absolute position's coordinates reaches this one, so a figure put back displaced slides away from
  the arrow rather than taking it. This is a real cost of the rule and it is pinned by a test.
- **Bad, because a caller that wants the arrow to follow a replacement cannot ask for it.** Under
  the old rule nothing followed anything and there was nothing to want; now the difference is real,
  and the answer is to name the reference again in the connector.
- **Bad, because three pieces of the crate's own prose declared the opposite** — two sentences of
  `remove`'s rustdoc and a doc comment on a test that was false twice over. All three are corrected
  in the commit that makes them false, and §9 _Changing a diagram_ **loses** "Nothing is rewritten
  and nothing cascades" rather than gaining a sentence beside it. Two shapes under one identity are
  the undecided case: `add_under` allows it and `find` answers the first match, which the freeze
  inherits.

### Confirmation

Six contract tests in `monospace-diagram` and one in `monospace-cli`, plus one existing test
rewritten rather than deleted — it was the one test the freeze makes red, and the specification
named none.

`a_removal_freezes_what_hung_from_the_removed_shape` asserts the rule asked **and then drawn**: the
endpoint is `Absolute` at the point its reference resolved to, with an `assert_ne!` that it is not
still a reference; the cells the removed box held are **discovered**, and the cells the arrow writes
are discovered against a no-connector baseline. Nothing is claimed by a number written into a test,
because the count **10** is not `15 − 6` — `{3, 1}` changes glyph rather than going blank. The
method's own contract is pinned over **all three kinds**, holding the `Box` and `Line` arms to
answering `None` and not a copy, so #89 widening a box's `at` is two lines rather than the method.

The rule is verified by being **broken on purpose**: dropping the guard on which figure a reference
names turns `a_removal_touches_nothing_else` red on the other-figure's case and leaves
`a_removal_freezes_what_hung_from_the_removed_shape` green. Measured: 2 red, 76 green. The gallery's
fourth block is **drawn by `remove` itself**, so a freeze that stopped holding would drop a
snapshot.

**Accepted with nothing to verify it**, named rather than described as tested: that a removal leaves
**every other** figure byte for byte — a claim about a whole diagram, which the tests check only for
the figures they name — and that two shapes under one identity freeze through the first, which is
pre-existing and inherited rather than decided.

## Pros and Cons of the Options

### Freeze the references naming the removed shape

- Good, because it makes the two visible outcomes distinguishable for the first time, and because it
  repairs exactly the wrong consequence of 0041 while leaving the seven beside it alone.
- Good, because it is one method on `Shape` and one body that calls it: no flag, no signature, no
  variant, and nothing outside the crate to migrate.

### Leave the reference unresolved

- Good, because it is the rule already recorded and the code already implements, so taking it is
  free and nothing existing has to move.
- **Bad, because it is a silent no-op of the kind the constitution's principle IV exists to end.**
  Three routes draw one picture, so a caller cannot tell which happened, and the demonstration's own
  fourth step removes a figure and shows nothing at all.

### Repair, and report

- Good, because the caller is told rather than surprised.
- **Bad, because there is no fault to report** and it adds a signature for an answer nobody asked
  for. `remove` hands back nothing today, and a caller cannot reach the frozen positions from
  outside the crate to ask the question itself.

## Reversibility

**Permanent from the moment it lands**, on the constitution's own test rather than a judgement call:
`monospace-cli` calls `remove` and its seventh picture is this rule, so something outside the module
depends on it, and reversing the rule leaves every consumer's connector unwritten where it stood
this morning. Undoing it is not a migration but it is not free either: six tests and the gallery
block would have to say the opposite, and the old claim is gone from the crate's prose rather than
sitting in it waiting to be restored.

## Confidence

High (85%).

The rule is measured rather than derived: the counts, the two pictures and the one test that went
red were all taken by running the code, and two of the specification's numbers were **corrected**
rather than confirmed. What would change it is a consumer that wants a removal to re-route rather
than freeze — the arrow disappearing entirely is defensible, and the seventh picture is where that
would be argued. What would prove it wrong is a diagram whose whole purpose is to show which figures
were removed, where "nothing remembers a removal" becomes the missing feature.

## Revisions

- 2026-10-02 — recorded. Two things the design got wrong came back from running it, and they are
  corrected here rather than left to be found later. **The commitment is `load-bearing`, not
  `working`**, as this plan's first run proposed: `monospace-cli` calls `remove` and its seventh
  picture exists because of the freeze. That correction costs the full template — the options drawn
  side by side, Pros and Cons of each, and a percentage — none of which fits under a working
  record's sixty lines. **And the design sheet's one named mechanical consequence was false**: it
  said the rewrite is forced to be two passes because `resolve` takes `&Diagram` where `remove`
  holds `&mut self`. Measured, one pass compiles; what cannot be done is hold a `&mut Shape` across
  the call, so the loop indexes.

## More Information

- The record whose one consequence this contradicts, and the one it completes rather than restates —
  a displacement grows a reference's offsets, and a frozen end has none to grow:
  [ADR-0041](0041-resolve-a-position-through-a-reference.md) and
  [ADR-0067](0067-displace-a-figure-holding-a-reference-by-growing-its-offsets.md).
- Why the evidence is a gallery block and not a `<!-- render: -->` marker, and why **no marker in
  the repository can hold it**: a marker reads a description and a description carries no field that
  takes a shape out
  ([ADR-0064](0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md),
  [ADR-0035](0035-keep-the-cli-demo-format-out-of-the-model.md)).
- What a reference may reach besides a connector's endpoint, and the cycle obligation that widens:
  [#89](https://github.com/andresmoschini/monospace/issues/89).
