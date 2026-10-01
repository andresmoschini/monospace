# Data model: a description names its shapes, and carries the ordinal the next one takes

What this slice adds to `monospace-diagram`, what it changes in `monospace-cli`, and what it changes
in the 44 descriptions the repository tracks. Everything features 079 through 083 built stays as it
is unless it is named here. Why the counter is kept where it is and why the two fields are called
what they are is in [research.md](research.md); the four answers it takes are on
[decisions.md](decisions.md), named beside each. The public surface alone is in
[contracts/diagram-api.md](contracts/diagram-api.md) and the file format in
[contracts/description-format.md](contracts/description-format.md).

The slice is two methods and two required fields. What is not small is what has to be re-spelled
when the fields stop being optional, and that is where the design spends its care.

## `Diagram` — the counter, and two ways in

The counter is `next: u32` and it is **the ordinal the next `add` takes**. That is a rename of what
it means rather than a change of what it holds: `add` already increments before use
(`diagram.rs:57`), so a fresh diagram's `0` becomes `#1` and a diagram that has issued `#3` holds a
`3` that would become `#4`. Naming it for what it is is what lets a caller seed it (Q1).

### `numbered_from(next: u32) -> Diagram` — new, public

```rust
/// An empty diagram whose next `add` takes the ordinal `next`.
#[must_use]
pub fn numbered_from(next: u32) -> Self
```

One field, no shapes. **Storing the ordinal itself is the whole design of this method**: a counter
holding the last issued ordinal would need `numbered_from(27)` to store `26`, and an off-by-one in a
public seeding method is the kind of trap no rustdoc fixes afterwards (Q1). Measured on the existing
code rather than argued: the increment-before-use is what makes the two agree, which is also why
`new()` is unchanged and is `numbered_from(1)` in every respect that is observable — `add` on either
hands back `#1`.

`Default` still derives, and `new()` still delegates to it. A caller wanting a diagram numbered from
something other than `#1` seeds it here; nothing else in the crate reads `next`.

### `add_under(&mut self, id: ShapeId, shape: Shape)` — new, public

```rust
/// Puts `shape` at the front of the order, under the identity `id`, and returns nothing.
pub fn add_under(&mut self, id: ShapeId, shape: Shape)
```

Identical to `add` in what it does to the order — a `Vec::push` of one more `Placed` at the back,
and the back is the front of the order — and different in two things:

- **The name is the caller's**, and the diagram does not check it. B3.1 accepts two entries carrying
  one identity, both held, the first the one every change finds, the second reachable by no identity
  at all until the first is removed. So the method cannot fail, and a `-> Option<ShapeId>` would
  have no absent case to report (Q2).
- **It hands back nothing and does not touch `next`.** Nothing a caller writes moves the counter: on
  a diagram numbered from 3, `add_under("#7", …)` leaves it at 3 and the next `add` is `#4`. That is
  D2's accepted cost in one sentence, and what the specification's _Edge cases_ mean by an identity
  the diagram issues as against one a caller wrote.

Not advancing the counter is also the reason the two methods compose: a file carrying `#1`, `#7` and
`#9` with `next_id: 27` hands back `#27` from the next `add`, and the gaps are simply unused rather
than filled.

### Unchanged

`new`, `add`, `find`, `get`, `remove`, `replace`, `forward`, `backward` and `draw` keep their
signatures and their bodies, and every test 079 through 083 pinned keeps passing. `get` is still the
only reader: there is no listing of a diagram's identities, no count and no order, and `add_under`
does not become one — it adds a shape under a name the caller already holds (the specification's
_What this slice does not decide_).

### `ShapeId` — same type, wider meaning

No field, no derive and no signature change. What changes is the sentence its rustdoc carries, and
it is the sentence the model carried: "unique within that diagram" becomes a statement about the
identities **the diagram issues**, because a caller-supplied one is not checked (D2). `ShapeId::new`
already built an identity from text, so a caller could always spell one — what it could not do was
put a shape under it, and that is what `add_under` adds.

Identities compare by value and nothing normalizes them: two differing only in case or by a trailing
space are two identities, and a file may name identities with gaps.

## `monospace-cli` — two required fields and one rewritten conversion

Everything in `description.rs` is private to this conversion
([ADR-0035](../../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md)). Two types
change, and neither changes shape:

| Type               | Change                                             |
| ------------------ | -------------------------------------------------- |
| `Description`      | gains `next_id: u32`, beside `canvas` and `shapes` |
| `ShapeDescription` | each of its three variants gains `id: String`      |

`id` sits immediately after `kind` in the wire order, which is where all 44 descriptions already put
their first field — so the edit is an insertion on a familiar line rather than a reshuffle (Q3).
`next_id` is a number rather than a list or a nested object, because it is one number
(`"ids": { "next": 27 }` was measured and declined).

Both are required, so a file carrying neither is refused with ``missing field `id` `` or
``missing field `next_id` `` — the same refusal `canvas`, `shapes`, `leaving`, `terminal` and `at`'s
`kind` already give. That asymmetry is the format's existing one and not something this slice adds:
a **misspelled** `id` is caught, because a required field is a `missing field` error, while a
misspelled extra key is dropped in silence, as every unknown field in this format has always been.
It is also why the 44 descriptions have to be edited before the reader starts refusing them, and why
that edit can land as a `refactor`.

`id` rather than `shape`, because `shape` is already what a `reference` calls the figure it names
and one word meaning two things inside one format is the ambiguity `kind` exists to remove.

### `into_diagram` — the whole of the conversion, rewritten

```rust
pub(crate) fn into_diagram(self) -> Diagram {
    let mut diagram = Diagram::numbered_from(self.next_id);
    for shape in self.shapes {
        diagram.add_under(ShapeId::new(shape.id), shape.shape.into());
    }
    diagram
}
```

Three lines of substance and nothing else:

- **The order is still the order.** The array order remains the drawing order, so every picture in
  the repository comes out byte for byte what it did. This is the claim the whole mechanical change
  rests on, and it was measured rather than argued: research.md read three files with and without
  `id` and both sides came out exactly the same, because a field the format does not know is dropped
  in silence.
- **`id` reaches the diagram, not the figure.** `ShapeId` is a `String` and the conversion already
  builds one per `reference` (`description.rs:252`), so a name is a value the reader owns until the
  diagram takes it. Nothing about the returned figure says where it sits, which is 081's arrangement
  and stays it.
- **A name a connector names before its entry is written resolves anyway**, because resolution
  happens at draw time and the whole diagram exists by then. The edge case the specification lists
  needs no code; it is what a completed `Vec<Placed>` means.

The identities `add` used to return were discarded and stay discarded — nothing in `monospace-cli`
reads them back, and `get`'s own rustdoc still says so.

## The demonstration — one workaround removed and three comments corrected

`demonstrate` itself does not change: `#1`, `#3` and `#10` stay written out by hand and still name
the same three figures, because the shipped file now names its entries that way themselves. B2.3
asks for exactly that (Q5).

What changes is everything that was true only because a file named its shapes by position:

- **`demo_without_its_first_entry` loses its renumbering.** It removes the first entry from the
  array and re-reads the text; while identities came from the array order, that shifted every name
  after it and the helper rewrote `"#5"` to `"#4"` to keep the picture. With `id` on every entry the
  remaining twenty-five keep the names they were written with, `#5` is still the same box, and the
  loop goes with the long comment that explains it. **That is this slice's evidence inside the test
  suite rather than beside it**: the fourth picture and the re-read description are the same diagram
  for the first time, and SC-001's claim is what the existing `assert_eq!` now rests on.
- **Two comments become false.** `main.rs:104` and `main.rs:121` both say a description names its
  shapes by position and has nothing to read back. They are corrected in the commit that makes them
  false, beside the code that makes them so (083's Q6).

## What the tests pin

Six contract tests and no characterization, per the specification's _Testing expectations_. None is
a new file; each lands beside the code it covers.

| Rule                                              | Where                           | How it is asked                                                                    |
| ------------------------------------------------- | ------------------------------- | ---------------------------------------------------------------------------------- |
| A chosen identity is found by that identity       | `diagram.rs`                    | `get`, `remove`, `replace`, `forward`, `backward` each name the shape it was given |
| And by no other one                               | `diagram.rs`                    | An identity the file holds for a different shape changes nothing                   |
| A reference follows the name, **both directions** | `diagram.rs` / `description.rs` | Two files differing only in entry order draw the same buffer                       |
| The ordinal, asked rather than drawn              | `diagram.rs`                    | A seeded diagram hands back one no shape holds, then a different one               |
| A freed identity is not reissued                  | `diagram.rs`                    | 081's test, unchanged, on a seeded diagram                                         |
| A repeated identity                               | `diagram.rs`                    | Both entries are held and `get` returns the first — pinned as a cost               |
| A missing identity is refused by name             | `description.rs`                | The message, with nothing added and nothing drawn                                  |
| Free text reads                                   | `description.rs`                | Entries named `right`, `left`, `arrow` draw the ordinal-named picture              |
| The demonstration                                 | `main.rs`, `tests/cli.rs`       | Five pictures byte for byte, and a path prints one picture                         |

The demonstration's test already asserts the four pictures against the re-read description; this
slice removes the renumbering that made the two sides agree, so the same assertion becomes the
claim.

## What this does not add

No dependency, no crate, no change to `xtask` or to the gate, and nothing at all in
`monospace-core`. No way to edit an identity after the fact, none to ask a diagram what identities
it holds, and no `Serialize` anywhere in this path — the format is read and never written, which is
why D1 accepts a counter that can go stale rather than engineering it away. No report on an identity
that resolved to nothing: that is ADR-0041's silent hole arriving on a name, and
[#88](https://github.com/andresmoschini/monospace/issues/88) is where it is written down.

The three model sections this slice amends and the ADR are in `docs/`, in their own commits, and
`docs/diagram-model.md` is the record of what the rules are rather than of when they were decided.
