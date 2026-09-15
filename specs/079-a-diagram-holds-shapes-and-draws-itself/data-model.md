# Phase 1 Data Model: A diagram holds shapes and draws itself

The entities the spec names, as the types `monospace-diagram` introduces, and the field-for-field
mapping onto the `monospace-core` shapes they construct. The vocabulary is
[`docs/diagram-model.md`](../../docs/diagram-model.md); this file adds no rule it does not have.

## `Diagram`

| Field    | Type         | Meaning                                                     |
| -------- | ------------ | ----------------------------------------------------------- |
| `shapes` | `Vec<Shape>` | The order. Private. Its **last** element is the front (Q2). |

That is the whole of it: no buffer, no glyph catalog, no window, no rendered picture (FR-005).

**Changes**: one in this slice, `add` (FR-006). It pushes, so the new shape is at the front of the
order, in front of everything already there. It cannot fail, it returns nothing, and it gives no
identity — identity is [issue 80](https://github.com/andresmoschini/monospace/issues/80), and until
it exists nothing can name a shape to remove, replace or reorder it.

**Reads**: none. A diagram answers no question about its shapes in this slice; the only thing it
does with them is draw them.

## `Shape`

A closed enum of three kinds (FR-008), each holding the parameters and the positions of the core
shape it constructs (FR-007). Every position is absolute here — a reference is
[issue 85](https://github.com/andresmoschini/monospace/issues/85) — and no kind holds a stamp mode,
a layer, an identity or anything about where it sits in the order.

### `Shape::Box` → `monospace_core::BoxShape`

| Field    | Type            | Core field | Conversion |
| -------- | --------------- | ---------- | ---------- |
| `at`     | `Pos`           | `at`       | identity   |
| `size`   | `Size`          | `size`     | identity   |
| `stroke` | `Stroke`        | `stroke`   | clone      |
| `fill`   | `Option<Glyph>` | `fill`     | clone      |

`at` is the box's top-left corner, which is the only position a box has.

### `Shape::Line` → `monospace_core::Line`

| Field         | Type          | Core field    | Conversion |
| ------------- | ------------- | ------------- | ---------- |
| `at`          | `Pos`         | `at`          | identity   |
| `len`         | `u32`         | `len`         | identity   |
| `orientation` | `Orientation` | `orientation` | identity   |
| `stroke`      | `Stroke`      | `stroke`      | clone      |

`at` is the line's first cell, which is the only position a line has.

### `Shape::Arrow` → `monospace_core::Arrow`

| Field    | Type       | Core field | Conversion         |
| -------- | ---------- | ---------- | ------------------ |
| `from`   | `Endpoint` | `from`     | `Endpoint` mapping |
| `to`     | `Endpoint` | `to`       | `Endpoint` mapping |
| `stroke` | `Stroke`   | `stroke`   | clone              |

An arrow carries no position of its own: both of its positions belong to its endpoints, and the
route between them is derived by the core from those two and their leaving directions.

## `Endpoint`

The diagram's own, mirroring `monospace_core::Endpoint` (Q3).

| Field     | Type        | Core field | Conversion |
| --------- | ----------- | ---------- | ---------- |
| `at`      | `Pos`       | `at`       | identity   |
| `leaving` | `Direction` | `leaving`  | identity   |
| `head`    | `Glyph`     | `head`     | clone      |

## No parameter is dropped

FR-009 says a parameter the core shape takes and the diagram's kind does not carry is a parameter
the diagram cannot express, and that there must be none. The three tables above are that claim
written out: every public field of `BoxShape`, `Line`, `Arrow` and the core's `Endpoint` appears
exactly once on the left. TE-005 is what keeps it true as either side changes.

## Validation rules

None, in either direction. Every kind accepts whatever the core shape accepts, and the core already
decides what a degenerate figure draws: a box below 2 in either dimension draws nothing, a line of
length 0 draws nothing and of length 1 draws one end, an arrow whose endpoints share neither
coordinate draws its two heads and no route. The diagram adds no rejection of its own, so there is
no error type and `add` cannot fail.

## Drawing

Not a change to the model's state — it is a read of the diagram and a write to something else.

| Step | What happens                                                                          |
| ---- | ------------------------------------------------------------------------------------- |
| 1    | The caller's `Buffer` is bound to `StampMode::Below` as one `Layer` (FR-012)          |
| 2    | The shapes are visited from the front of the order to the back (FR-011)               |
| 3    | Each converts to its core shape and draws through that layer                          |
| 4    | Cells outside the buffer's window are dropped by the buffer itself, silently (FR-014) |
| 5    | Nothing is rendered: the cells stay cells, and the glyph catalog stays the caller's   |

The diagram is untouched throughout (`&self`), so drawing is repeatable and two drawings into equal
windows produce equal buffers (FR-015).

## State transitions

A diagram has one state — the sequence of shapes it holds — and one transition into it, `add`. There
is nothing else: no lifecycle, no validity to lose, and no state that drawing can move it out of.
The four remaining changes of _Changing a diagram_ (remove, replace, forward, backward) arrive with
their own issues, listed in the spec's **Out of scope**.
