# Data model: an endpoint is a position, a leaving direction and a terminal

The vocabulary is [`docs/model.md`](../../docs/model.md)'s. This slice changes _The initial set_,
_What a terminal writes_ and one clause of _The route of an arrow_, and the sentences it changes
them to are drafted in [research.md](research.md) Q9 — nothing here restates them. What follows is
the shape the change takes inside the three crates and which of it is public. The public surface
alone is in [contracts/](contracts/); why the type is shaped this way is in
[decisions.md](decisions.md) D2.

## `Terminal` — new, public, in `monospace-core`

What an endpoint contributes to the cell at `at`. Two values, and the field carrying one is present
whatever the value is (spec's B1, scenario 3).

| Variant           | Payload              | The cell it writes                                                                                |
| ----------------- | -------------------- | ------------------------------------------------------------------------------------------------- |
| `Glyph { glyph }` | the caller's `Glyph` | `Cell::Literal` — decided on every side, so nothing composes into it                              |
| `Arm`             | none                 | one `Arm::Set` on the side `leaving` names, in the arrow's own stroke, `Unset` on the other three |

Derives `Debug`, `Clone`, `PartialEq` and `Eq`, and not `Copy` because `Glyph` owns its text.
`Clone` because `Shape::draw` takes `&self` and the terminal is drawn from a borrow.

`Arm` is a unit variant and carries nothing, which is all the wire form says either
(`{"kind":"arm"}`). Nothing is missing: the side is the one `leaving` names and `leaving` is already
on the endpoint, and the stroke is the arrow's own rather than the terminal's, because a terminal
that chose its stroke would be a second thing to keep consistent with the route it joins and the
model's arm is a stroke cell of the figure.

Two variants, and the enum is closed at two. A third terminal is a new variant, a new tag and one
line in the model's vocabulary, with no change to the shape of a description and no change to any
record (research.md Q8 and Q6). What such a terminal writes is not decided here, and the cell it
would leave is the open part — spec's _What this slice does not decide_.

## The one derived value

An arm needs a `cell::Side`, and `Side` is crate-private, so the mapping is a private function
beside the terminal it serves rather than a field on the data:

| `leaving` | the side written |
| --------- | ---------------- |
| `Right`   | `Side::Right`    |
| `Down`    | `Side::Bottom`   |
| `Left`    | `Side::Left`     |
| `Up`      | `Side::Top`      |

It is the leaving direction's own side and not its opposite: the arm is the one the arrow runs on,
so it faces the route, which begins one step from `at` in that direction. A glyph terminal has no
counterpart to this — it names its own character — which is why the field is a sum of two answers
rather than a direction beside a choice.

## What each terminal draws through

Two crate-private fragments already exist and neither changes. What changes is that `End` now has a
second caller.

| Terminal | Fragment                      | The cell                                                                    |
| -------- | ----------------------------- | --------------------------------------------------------------------------- |
| `Glyph`  | `shape::fragment::head::Head` | `Cell::Literal(glyph)`                                                      |
| `Arm`    | `shape::fragment::end::End`   | a `StrokeCell` in the arrow's stroke, one `Arm::Set` and three `Arm::Unset` |

`Line` already reaches `End` for its two outermost cells (`shape/line.rs:56-63`) and worked the side
out from its `Orientation`. The terminal works it out from `leaving`, which is a direction, and the
two agree wherever both are defined — which is what research.md Q1 measured by standing a `line`
where the terminal writes.

`derive_path` does not move and cannot: it takes no terminal, so a terminal cannot reach the route
(research.md Q2). `Arrow::draw` matches on each endpoint's terminal, draws it, and then calls
`derive_path` with the two `at`s and the two `leaving`s exactly as it does today.

## The three `Endpoint`s

One vocabulary type, two endpoint types, and one private mirror of the vocabulary for the wire.

| Crate               | Type       | Its `terminal` field                                        | Visibility                              |
| ------------------- | ---------- | ----------------------------------------------------------- | --------------------------------------- |
| `monospace-core`    | `Endpoint` | `Terminal`, the core's own                                  | public, re-exported from the crate root |
| `monospace-diagram` | `Endpoint` | `monospace_core::Terminal`                                  | public                                  |
| `monospace-cli`     | `Endpoint` | a private type for deserialization, converted on the way in | private to the crate                    |

The diagram layer keeps its own `Endpoint` and does **not** gain its own `Terminal`. The mirror
exists so that a change to how an endpoint is _anchored_ stays inside that crate, and a terminal is
not an anchor (research.md Q5). The mirror already holds the core's `Pos`, `Direction` and `Glyph`
directly, so the terminal is the same treatment and the `From` loses a line rather than gaining a
type. A third `Terminal` and a third `From` would buy nothing, since the two endpoint types have to
convert either way.

`monospace-cli` keeps a private type because it is the only crate that deserializes anything
(ADR-0035) and the wire form is not the core's: `Terminal` in the core carries no `serde` derive and
`monospace-core` gains no dependency. The private type becomes the core's in the same `From` that
already converts `Pos`, `Leaving` and the endpoint itself.

## The invariants a terminal holds

Each is a rule from the spec stated as something a test can ask of a drawn arrow.

1. **A terminal writes the cell at `at` and nothing else.** One stamp, at the endpoint's own
   position — decisions.md D1.
2. **A terminal contributes no cell the route does not.** The body is the same five cells whatever
   the terminal is — spec's B3, scenarios 1 and 2.
3. **A glyph terminal's cell is decided on every side.** Nothing composes into it, in either stamp
   order — spec's B1, scenario 1.
4. **An arm terminal's cell is undecided on three sides.** Whatever reaches it afterwards may still
   join it — spec's B1, scenario 2.
5. **A shared cell belongs to the figure in front.** A glyph and an arm landing on one cell is
   decided by the order the caller wrote them in, and the arrow states no rule of its own over it —
   decisions.md D3, which is a row of the stamping table in [`docs/model.md`](../../docs/model.md)
   _Stamping_ and not a new fact.
6. **A terminal's presence never decides what a description means.** The field is there for both
   values, and a value the model has not named is refused by name — spec's B1, scenario 3.

## What is removed

`Endpoint::head`, in all three crates, and nothing else. `Glyph` keeps its constructor and its
`as_str`. The `Head` fragment keeps a field called `glyph`, which is a fragment's own business and
not the wire's. The model's _An end is an arm; a head is a glyph_ becomes _What a terminal writes_
with ADR-0029's argument entire, per decisions.md D4.

## What is not in here

- No third terminal, and no statement about the cell one would leave (research.md Q8).
- No derivation of `leaving`, and no anchor, reference or offset — issue #82's work.
- No glyph set consulted for a terminal's glyph. The caller's glyph stays the caller's, which the
  model still carries as an open question.
- No field on a description beyond `terminal`, and no second spelling of it. A third terminal
  arrives as a third tag.
