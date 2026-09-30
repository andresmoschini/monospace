# Decisions: A description names its shapes, and carries the ordinal the next one takes

**Feature**: #148 | **Written**: 2026-09-30 | **Answered**: 2026-09-30 — all four

Four entries, all domain-level, all answered with the proposal adopted. None carries a picture, and
that is a finding rather than an omission: **every option pair below draws the same picture**.
[research.md](research.md) carries the three measured files showing it.

## D1 — Where the ordinal the next shape takes is kept

- **Proposal**: `"next_id": 27` beside `canvas` and `shapes` — the ordinal the next `add` takes,
  required and refused by name when absent.
- **Altitude**: domain — the format is provisional per
  [ADR-0035](../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md) but 25 markers
  observe it. Confidence: high.
- **If this is wrong**: the reader parses each `id` and resumes above what it finds, which is
  additive but derives nothing from identities like `right` and `arrow`.
- **The alternative**: read off the identities, `add_under` advancing past any `#N` it is handed —
  nothing to maintain, but a `#N` convention inside a crate whose `ShapeId` is any string.
- **Yours to answer**: yes — the specification's _What this slice does not decide_ hands it here.
- **Answer**: confirmed. The JSON is not meant to be hand-written; we write it by hand only because
  the editor is not built. That is also why a counter which can go stale is accepted rather than
  engineered away: it is a cost of the missing editor, not of the format.

## D2 — What stops `add` handing back an identity the diagram already holds

- **Proposal**: the file's `next_id`, trusted. The diagram does not check what it hands out, and a
  caller may write the same name on two entries.
- **Altitude**: domain — §3 says an identity is "unique within that diagram", and this decides
  whether that stays a promise or narrows to the identities the diagram issues. Confidence:
  medium-high.
- **If this is wrong**: a stale counter — an entry renamed, a `#2` deleted — makes `add` hand back a
  name in use, and the shape that arrives is the one nobody can reach. The repair is one line.
- **The alternative**: `add` advances past every identity the diagram holds, making the guarantee
  unconditional and the field a starting point; or the reader refuses a file whose `next_id` sits
  below an `id` it carries, which proves the file consistent at the cost of the reader's `#N`.
- **Yours to answer**: yes — B2.1 requires the outcome and deliberately declines to say who owes it.
- **Answer**: confirmed. The file is the one place a description says where its numbering resumes,
  and §3's sentence is amended to say what the code keeps rather than to promise more than it does.

## D3 — Whether the caller chooses the identity on the diagram, or the reader translates it

- **Proposal**: on the diagram. `numbered_from(next)` seeds the counter, `add_under(id, shape)`
  places a shape under the name the file wrote, and `get`, `remove`, `replace`, `forward` and
  `backward` all answer to that name.
- **Altitude**: domain — §11's open question is _Can a caller choose an identity?_, and
  `monospace-diagram`'s public API gains two methods. Confidence: high.
- **If this is wrong**: the reader keeps a `name → ShapeId` map and the crate gains nothing, no
  model section is amended and no record is needed. SC-002 then holds for drawing and not for the
  changes.
- **The alternative**: that translation in `monospace-cli` — about ten lines, and the same pictures.
- **Yours to answer**: yes — this is §11's own question, whose trigger the specification closes.
- **Answer**: confirmed. A reference has to reach the diagram by name or it does not resolve at all,
  so the two designs differ only in whether the crate that answers _what does a change mean_ also
  holds the name.

## D4 — How far the counter is required

- **Proposal**: in every description, the 25 tracked markers included.
- **Altitude**: domain — the format D1 and D3 change, and `cargo xtask render` re-draws all 25 on
  every run, so a wrong one is a red gate rather than a quiet difference. Confidence: high.
- **If this is wrong**: the field is a second spelling of a number the diagram can do without, and
  the fix is 25 deletions plus one `#[serde(default)]`.
- **The alternative**: optional, absent meaning the diagram starts at `#1` — a rule in prose where
  `canvas` and `shapes` are required in the type; or required only where a consumer grows a diagram,
  which here is every description and so is not a distinction.
- **Yours to answer**: yes — the scope is a cost borne by documents this repository tracks.
- **Answer**: confirmed. `canvas` and `shapes` are already written in every one of them, and a
  number sometimes written down and sometimes not is a rule a reader has to know rather than one the
  type holds.
