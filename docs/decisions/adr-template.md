---
status: "proposed | accepted | rejected | superseded by ADR-NNNN"
date: YYYY-MM-DD
decision-makers: "{who took the decision}"
---

# {Short title, naming the problem and the chosen solution}

## Context and Problem Statement

{Two or three paragraphs. What situation forces a decision now? What breaks or stays ambiguous if no
decision is taken? Write it so that someone who was not there can follow it.}

## Decision Drivers

- {The constraint or goal that actually shaped the choice, ideally traceable to `docs/brief.md`}
- {...}

## Considered Options

- {Option 1}
- {Option 2}
- {Option 3}

## Decision Outcome

Chosen option: **{Option N}**, because {the one-sentence reason that would survive being quoted out
of context}.

### Consequences

- Good, because {positive outcome}
- Bad, because {cost accepted knowingly — do not omit these; an ADR with no costs listed is a sales
  pitch, not a decision record}
- Neutral, because {consequence that is neither, but worth knowing}

### Confirmation

{How anyone can tell whether the decision is actually being followed: a check in the quality gate, a
test, a lint, or an explicit note that it is only enforced by review.}

## Pros and Cons of the Options

### {Option 1}

{One line describing it, if the title is not self-explanatory.}

- Good, because {...}
- Bad, because {...}

### {Option 2}

- Good, because {...}
- Bad, because {...}

## Reversibility

{What undoing this costs today, and what makes that cost grow. Distinguish what is cheap now but
expensive later from what is permanent from the moment it lands.}

## Confidence

{Low | Medium | High} ({rough percentage}).

{What information is missing that would change this decision, and what observation would later prove
it wrong. If the answer is "nothing would change it", say so — that is a strong claim worth writing
down.}

## More Information

- {Links, prior art, related ADRs, or the conversation that produced this}
