# Data model: what a removal leaves behind, by value

The slice adds no type, no field, no variant and no signature, and it takes no rule, so **there is
no entity table here**: the arrangement is three values the specification already spells, and
[`docs/diagram-model.md`](../../docs/diagram-model.md) §4 owns what a reference resolves to when the
diagram does not hold it. What follows is that arrangement as code, the two measurements the
specification asked for and this branch took, and the list of what is deliberately not touched —
which is most of the slice.

The four answers it acts on are on [decisions.md](decisions.md), named beside each; the six
questions that were not the maintainer's are in [research.md](research.md); the commands that take
the measurements below are in [quickstart.md](quickstart.md). Every number here was measured on this
branch, and Q4's method applies — a scratch test added, run, and deleted, leaving no trace.

## The arrangement, as values

The specification draws it once and every scenario is about it: a box four cells by three at the
origin, a box three cells by three at `{8, 0}`, and a connector whose `from` is a reference to the
first box's right side carrying an offset of nothing.

| Identity | Kind        | Holds                                                                                                                                        |
| -------- | ----------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `#1`     | `Box`       | `at {0, 0}`, `size 4 × 3`, `stroke light`                                                                                                    |
| `#2`     | `Box`       | `at {8, 0}`, `size 3 × 3`, `stroke light`                                                                                                    |
| `#3`     | `Connector` | `from` → `Reference(#1, Right, offset (0, 0))` leaving `Right`, `Terminal::Arm`; `to` → `Pos { x: 8, y: 1 }` leaving `Left`, `Terminal::Arm` |

The second box is what makes the removal legible: **`#2` is the only figure in the arrangement that
survives it.** Nothing in the crate names this arrangement today — the demonstration's is a
different one, and the gallery's has no second box — so it is a test fixture and nothing more. It is
not a type to add, and the file that would hold it does not exist.

### B1.1's picture, and what `remove` costs it

```text
       ┌─┐
       │ │
       └─┘
```

**Fifteen cells of a twelve-by-three window change, and fourteen of the fifteen turn blank.** They
are three territories, and naming them is the point of the count:

| Territory                               | The cells                                                | Count  |
| --------------------------------------- | -------------------------------------------------------- | ------ |
| the box's **drawn** cells               | `{0, 0}`–`{3, 0}`, `{0, 1}`, `{3, 1}`, `{0, 2}`–`{3, 2}` | **10** |
| the arrow's written cells               | `{4, 1}`–`{7, 1}`                                        | **4**  |
| the one cell that is **not** a blanking | `{8, 1}`, `┤` → `│`                                      | **1**  |

Three things fall out of that table rather than being stated beside it:

- **The box is twelve cells and contributes ten**, because `{1, 1}` and `{2, 1}` are its interior
  and were already blank. A removal that "takes out twelve cells" is a claim about the rectangle,
  not about the picture.
- **The fifteenth cell turns `┤` into `│`**, which is the far box's own left border becoming visible
  now that the arrow is no longer welded to it. So **not every cell the removal touches becomes
  blank**, and a test that asserted "the removed cells are blank" would fail on the one cell that is
  a survivor's border appearing.
- **The end that resolves on its own goes with it.** `to` is a `Pos` and answers whatever it is
  asked, but a connector is drawn whole or not at all, so the plain point cannot outlive the
  reference on the other end.

**Fifteen and six are two measurements of overlapping territory, and adding them gives twenty-one.**
B1.1's count is over the removal and B3.4's is over the connector, and they share `{3, 1}` and
`{8, 1}`. Both are right and neither is the other — which is the whole of §11's second question in
two numbers.

### B3.4's six cells, measured as a difference

The arrow's own footprint is **six** cells, and it is read as the difference between the arrangement
and the same arrangement with **no connector in it at all** rather than off either picture:

| The cell          | With the connector   | Without it |
| ----------------- | -------------------- | ---------- |
| `{3, 1}`          | `│`                  | `├`        |
| `{4, 1}`–`{7, 1}` | written — four cells | blank      |
| `{8, 1}`          | `┤`                  | `│`        |

Four written into blank ones and two borders turned. **The ten B3.4 named is measured and wrong**,
and D4's answer amends the specification to six and names this measurement. The ten is the
demonstration's own arrow in B2.1, where it is correct — two arrangements sharing a count is how a
number reached the other.

### The three routes, and why two of them are one picture

A and B are **the same picture, byte for byte**: a reference naming an identity `get` does not find,
reached from the two sides. Measured. C is that picture **plus the replacement's own two cells** —
where the figure now standing at `#1` is a connector, which writes its two cells and answers no
anchor — and `get(#1)` still answers `Some`.

So §4's two-row table has three arrivals and two answers, and **nothing in the diagram records which
route happened.** That is §11's second question stated as code rather than as prose, and it is why
B3.4 compares the arrow's footprint across the three rather than the whole buffer: C is not the same
buffer, and a whole-buffer comparison would fail it for the wrong reason.

### B1.2 and B1.3, by value

- **B1.2** — `remove(&#1)` then `add_under(ShapeId::new("#1"), the_box)` reproduces the pre-removal
  picture **exactly**. Two of the three claims in the rustdoc on
  `a_figure_put_back_under_the_referenced_identity_draws_the_connector_again` are therefore false,
  and D3's answer corrects both paragraphs.
- **B1.3** — `add` after the removal hands back **`#4`**, never `#1`; `get(&#1)` answers `None`; the
  new shape draws where the removed one stood and **the arrow is still not drawn**. This falls out
  of `Diagram`'s own arithmetic rather than being a rule: `remove` does not touch `next`, so the
  counter never reissues, and `add_under` is the only way to spell an identity. Nothing repairs the
  hole, and a caller who re-adds without naming the identity sees a diagram that looks the same and
  hangs from nothing.

## The demonstration's seventh picture, cell by cell

Measured against the shipped description by adding the step, running it and differencing the seventh
block against the sixth. **Twenty-two cells, and every one of the twenty-two turns to blank** — not
twenty-two cells that differ, which a rule that drew something in their place would also produce.

| Whose | The cells                                                                      | Count |
| ----- | ------------------------------------------------------------------------------ | ----- |
| `#3`  | `(13, 2)`–`(16, 2)`, `(13, 3)`–`(16, 3)`, `(13, 4)`–`(16, 4)`                  | 12    |
| `#10` | `(16, 5)`–`(19, 5)` — the arrow's top row                                      | 4     |
| `#10` | `(19, 6)` and `(22, 6)` — its two verticals, the second being the `▲` terminal | 2     |
| `#10` | `(19, 7)`–`(22, 7)` — the arrow's bottom row                                   | 4     |

Two of these are worth naming rather than leaving to a reader's eye:

- **`(16, 3)` is inside `#3`'s own footprint and is the attachment cell.** It is `#3`'s right side
  centre, and it is in the box's twelve because the box's border _is_ its rightmost column. The
  existing test `the_sixth_picture_moves_only_the_arrow` already reasons about this one cell by name
  (`THE_ATTACHMENT`); here it changes as part of the box rather than as the arrow leaving it, which
  is why the seventh's bound is **exact and whole** rather than one-sided — all twelve of the
  footprint go and nothing else inside it does.
- **The arrow's ten are not contiguous and not a rectangle.** Its top row is four cells at
  `x 16..19` and its bottom row four at `x 19..22`, so reading the count off the picture is where
  the ten can go wrong. The count is asserted against the sixth block rather than quoted.

The caption is **`With the box the arrow hangs from taken out:`**, which names the figure the way
the fifth caption does and says only what changed. No caption's wording is pinned, and none is here.

## The gallery's fourth block measures to an empty picture

D2's answer is a fourth `block()` in `an_endpoint_hangs_from_a_side_and_follows_it` with
`change: "the box taken out"`. Measured on this branch, **it draws nothing:
`wrote 0 of 24 positions`.** All three candidate diagrams give it, because the gallery's arrangement
is **one box and one connector**:

| Reached from                                        | Window         | Positions written |
| --------------------------------------------------- | -------------- | ----------------- |
| the arrangement as written, a fresh `Diagram`       | `window(8, 3)` | **0 of 24**       |
| `from_as_written`, after its connector is displaced | `window(8, 4)` | **0 of 32**       |
| `diagram`, after its box is displaced four right    | `window(8, 3)` | **0 of 24**       |

The gallery has no `#2`, so there is **no survivor**: the connector's `from` names the only other
figure, and removing it takes the connector with it. The block is B1.1's picture without the box
that survives it, which is why it is blank where B1.1's hand-drawn one is not.

**The block is reached from the arrangement as written, through a third `Diagram` in the same test**
— the second one already needed for block three — because the two diagrams the test holds are each
already mutated, and a removal on either is a no-op on what remains. So the cost is the one block
three set the precedent for: six lines and a reason in a comment.

**This block is still worth having, and the reason is what it would catch.** It is a snapshot, so a
removal that ever started drawing the route, or freezing it where it resolved, would write cells
into this window and move it. It is the arrangement's honest picture of the rule, and the empty
surface row beside it says `wrote 0 of 24 positions` — the number, not the impression. **The one
thing that would change it is widening the arrangement to hold a second box**, so the block could
carry B1.1's picture rather than its degenerate case; that would move all three blocks the snapshot
already holds, and it is **not taken here** — it is not D2's answer, and it is named in _Design_ as
the thing to raise if the blank block is not what the gallery is for.

## What does not change

The list is the shape of the slice:

- **`Diagram`'s whole surface** — `add`, `add_under`, `get`, `replace`, `remove`, `forward`,
  `backward`, `draw`, `numbered_from`. Every signature, every visibility, every `#[must_use]`.
- **`Position`, `Reference`, `Anchor`, `Delta`, `Shape`, `ShapeId`, `Endpoint`** — no field added,
  no field removed, no derive changed. §4's two-row table is what already answers the question.
- **`Diagram::remove`'s three paragraphs of rustdoc**, `diagram.rs:146-160`. It already says what
  this slice is about — "An identity this diagram does not hold changes nothing, with no error, no
  report and no panic" — and **that sentence is why the seventh step needs no `if let`**, unlike the
  fifth and sixth steps beside it. Measured: a description holding no `#3` gives a seventh equal to
  the sixth, and so does the empty one and the one-box one.
- **`assets/demo.json` and the description format** — the seventh is a step in `monospace-cli`'s own
  code, beside the six it already makes, so no field is added and no `<!-- render: -->` marker is
  regenerated (B2.3, SC-001).
- **`monospace-core`** — `remove`, `displaced_by` and `reference` appear **zero** times under
  `crates/monospace-core` (research.md Q6), so nothing here reaches it and the `wasm` step compiles
  what it compiles today (SC-004).
- **Every characterization snapshot** — 1916 renderings across 16 files, none of which can express a
  removal: the connector sweep builds its descriptions from shape lists and holds no reference to
  remove, and `monospace-cli`'s sweep reads no shipped file. So no ADR-0053 report is owed, and the
  gallery's snapshot is the **only** picture a removal can move — which is what made D2 a question
  about it.
- **`xtask`, and the gate** — no step is added, so principle III's two-commit rule for a new check
  does not apply.

## The two paragraphs `add_under` falsified

Named here rather than left to the specification's _Testing expectations_, because D3's answer is
that both are corrected and a reader meeting the correction wants the reason beside it. `add_under`
arrived with [#148](https://github.com/andresmoschini/monospace/issues/148) and is `pub`, and B1.2
measures what that makes possible.

| Where                                                                                                  | The false claim                                                                                                                                                      | What replaced it                                                                                                            |
| ------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `diagram.rs:2352-2353`, on `a_figure_added_under_a_spelled_identity_is_what_a_hanging_endpoint_finds`  | "a diagram offers no way to name a shape into existence, so a spelled identity can only ever be the one an `add` is about to issue"                                  | `add_under` **is** that way, and it is `pub` — which is what the case it heads is built to reach                            |
| `diagram.rs:2546-2553`, on `a_figure_put_back_under_the_referenced_identity_draws_the_connector_again` | "`remove` frees an identity permanently — `add` never hands one out twice, and there is no `add_under`" — a removal followed by an addition cannot put anything back | The first clause is true and the second is false: `add_under` puts it back and the picture returns **byte for byte** (B1.2) |

**The first half of the second paragraph survives and the second half does not.** The case goes
through `replace` because the identity is found and the kind answers the anchor, which is a reason
about resolution rather than about removals — so the paragraph's reason is rewritten rather than the
paragraph deleted, and the test itself changes nothing.
