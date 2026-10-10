---
status: draft
date: 2026-10-10
---

# The description format moves to TOML

## Why now

[#203](https://github.com/andresmoschini/monospace/issues/203) is the change that takes the window
out of the description, and it is agreed and not yet built. What it leaves behind is the smallest
envelope this format has ever had: two fields, `next_id` and `shapes`, and nothing else. That is the
moment to change the spelling, because there is almost nothing to change.

The shape of what is left is the shape TOML speaks natively. `shapes` is a flat list of entries,
each carrying a `kind` beside its own parameters, and a list of tables with a tag field is exactly
what TOML's array-of-tables is. Nothing in the format needs a JSON-only construct: no unions over
more than one variant of a tag, no arbitrary nesting, no keys that a TOML reader could mistake for
structure.

The reason this is cheap is the same reason #203 was cheap, and it is worth saying again because it
is the whole argument: **the format is read and not written, and nothing outside this repository has
ever produced a file.** A spelling change that would be a breaking change for anybody is, here, a
mechanical rewrite of twenty-eight markers and one binary's embedded description, in one commit,
with `cargo xtask render` as the check that nothing a reader sees moved.

**This change lands on top of [#203](https://github/andresmoschini/monospace/issues/203), and that
is a dependency rather than an assumption.** Its building stage has not merged, so the envelope this
spec writes down — `next_id` and `shapes`, no `canvas` — does not exist in the code yet. If
[#203](https://github.com/andresmoschini/monospace/issues/203) has not landed when this one is
built, the first commit of this change is #203's, and this spec's examples are wrong until then.

## Scope

### In

- `crates/monospace-description`: `serde_json` out, `toml` in. The private types keep their names
  and their `From` conversions; only the deserialization behind them and the crate's module doc
  change.
- `monospace-cli`: `assets/demo.json` becomes `assets/demo.toml`, the `include_str!` path with it,
  and the three tests that vary one field of the embedded description by rewriting it.
- `xtask/src/render.rs`: `render_one`'s scratch file changes suffix, and nothing else — the body is
  already written verbatim, and the marker's grammar is already a prefix after #203.
- Every `<!-- render -->` marker in the repository. All twenty-eight in tracked documents change in
  the same increment, and the four documents and four specs carrying them are redrawn by
  `cargo xtask render`.
- The four documents that name the format's spelling: `docs/diagram-model.md` §11, `docs/model.md`
  §The model, `README.md`, and `CONTRIBUTING.md`'s illustration and the sentence beside it.

### Out

- **A writer.** The format is read and not written, and what a writer would owe a reader is the open
  question `docs/diagram-model.md` §11 already defers. A second spelling is what reopens it.
- **`deny_unknown_fields`.** #203 declined to settle whether a deprecated field is refused by name
  or ignored, on the grounds that there is no writer and fifteen tests assert the message. A second
  format would reopen that question twice, and this change does not.
- **Renaming the crate.** D7.

## The decision

**D1 — TOML only, and a JSON description is refused by name.**

**Answer:** `parse` reads TOML and nothing else. A file in the old spelling handed to
`monospace-cli` fails with a message that names the format it was given and the one it reads. **Why
not** reading both, by sniffing the first non-space byte or by extension: it doubles the grammar,
the error messages and the tests, to serve a reader that does not exist — nothing outside this
repository has written a JSON description, and the three documents that carry one are rewritten in
the same increment. **Why not** keeping JSON and adding TOML beside it: that is two formats, and a
crate that promises to read two promises to keep both. **What it costs:** a person with an old file
gets an error rather than a picture, which is the point — the format changed, and the error says so.
**Answered by** the maintainer, in the session that wrote this.

**D2 — the envelope is flat: `next_id` and `[[shapes]]`, with `next_id` first.**

**Answer:** two top-level keys and nothing between them. **Why not** a `[diagram]` wrapper table: it
adds a level to every file and every marker against a problem that does not exist yet, and the flat
form is what makes a marker read like a drawing rather than a configuration file. **What it costs:**
TOML's table ordering is load-bearing in a way JSON's is not. A `next_id` written after the first
`[[shapes]]` belongs to that shape, and the missing top-level field is then refused by name — the
right error, for a cause the message does not name. The crate's module doc says where `next_id`
goes, and one contract test pins the refusal. **Answered by** the maintainer, in the session that
wrote this.

**D3 — `[[shapes]]` is one array, and `kind` is a field like any other.**

**Answer:** every entry, of all three kinds, in one array, in drawing order. **Why not** grouping by
kind — `[[shape.box]]`, `[[shape.line]]`, `[[shape.connector]]`: drawing order is the array order,
and the model holds it as load-bearing, the last entry front-most and deciding a shared cell first.
Grouping by kind destroys the interleaving and needs a new rule for how three groups compose back
into one order, which is a decision about the format this change is not taking. **What it costs:**
nothing. An identity is a value here rather than a key, so TOML's bare-key rules do not touch it,
and the ordinal the model already settles is the one TOML's integers read. **Answered by** the
maintainer, in the session that wrote this.

**D4 — nested values are inline tables.**

**Answer:** `at = { x = 0, y = 0 }`, `size = { width = 4, height = 3 }`,
`terminal = { kind = "glyph", glyph = "◀" }`, and a connector's endpoints written the same way.
**Why not** sub-tables with headers — `[shapes.from.at]`: legal TOML inside an array of tables, but
it multiplies headers, hides the connector's shape behind them, and repeats D2's ordering problem at
every level, since every simple key of an entry must be written before that entry's first sub-table
header or it belongs to the wrong table. **What it costs:** an inline table must fit on one line in
TOML 1.0, so a connector endpoint is a long line, and a marker with three shapes is about twenty
lines where the JSON was fifteen. The alternative is a marker that is headers rather than a drawing,
and the picture is what the marker is for. **Answered by** the maintainer, in the session that wrote
this.

**D5 — the marker body is TOML, and `render_one` writes a `.toml` scratch file.**

**Answer:** `OPEN` stays the prefix it became in #203, the body is the TOML description verbatim,
and the scratch file is named `monospace-render-{pid}-{line}.toml`. **Why not** keeping the scratch
`.json`: the binary reads one format, and a scratch file in the other is a lie the gate would then
have to catch. **What it costs:** none that is not already counted — #203 already made `OPEN` a
prefix, and the body was already written as-is. **Answered by** the maintainer, in the session that
wrote this.

**D6 — `ParseError` wraps `toml::de::Error`, and every message changes wording.**

**Answer:** a newtype over `toml::de::Error` whose `Display` delegates, exactly as it does over
`serde_json::Error` today. The fifteen tests that assert serde's wording are updated to toml's.
**Why not** enriching or normalizing the messages: the same reason #201's D2 gave — the type's value
is that the underlying library does not appear in this crate's public API, not that it says anything
the error did not. **What it costs, and it is the largest in this change:** every error message in
the crate and the CLI changes, and toml's `Display` is multi-line where serde's was one line —
`TOML parse error at line 3, column 5`, then the offending line, then a caret. The gain is real and
worth naming: **line and column**, which serde's messages do not give, and which a format spelled by
hand has always needed. The crate's module doc documents the shape so a reader knows what to expect.
**Answered by** the maintainer, in the session that wrote this.

**D7 — the crate keeps its name.**

**Answer:** `monospace-description` still names what it does. **Why not** renaming it to something
spelling-specific: the crate is named after the format, not after JSON, and a rename is churn with
no reader. **Answered by** the maintainer, in the session that wrote this.

**D8 — the tests that vary a field of the embedded description do it through `toml::Value`.**

**Answer:** the three tests that rewrite `DEMO` — a connector's `at` becoming a reference, a shape's
parameters — rewrite it through `toml::Value` rather than by replacing text. **Why not** by text, as
they do today through `serde_json::Value`: a needle written against one formatting of a file stops
matching the day the formatter disagrees, which is the argument #201 recorded for doing it this way.
**What it costs:** `toml` becomes a dev-dependency of `monospace-cli` beside the dependency it
already is. **Answered by** the maintainer, in the session that wrote this.

## Model slice

- `docs/diagram-model.md` §11: the question _Is a diagram serializable, and in what?_ reads
  _Settled: in JSON_ today. It reads _Settled: in TOML_. Nothing else in the paragraph moves — the
  format is still not provisional, the writer is still what it defers, and what a writer owes a
  reader is still unsettled. The `canvas` paragraph beside it belongs to #203.
- `docs/model.md` §The model: one sentence says the format is decided by the JSON. It is decided by
  the TOML.
- The format's own documentation is the crate doc of `monospace-description`, which is rewritten
  with the format rather than amended section by section — a change of spelling is a change of every
  line that shows one.

**There is no model rule to add.** The model says what a diagram is; the format is how one is
written down, and the model has never said in what.

## Public surface

The deciding stage adds none of this; it is what the building stage publishes.

```rust
// monospace-description
pub fn parse(toml_text: &str) -> Result<Diagram, ParseError>;

pub struct ParseError(toml::de::Error);
```

`parse` returns a `Diagram` and nothing else — #203's D6 is what makes that true, and this change
does not reopen it. Every type behind the call stays private to the crate, and the `Pos`, `Size`,
`Orientation`, `Leaving`, `Terminal`, `AnchorDescription`, `OffsetDescription`, `At`, `Endpoint` and
`ShapeDescription` types keep their names and their `From` conversions; only the deserializer behind
them changes.

## Behavior

1. `parse` reads a TOML description and hands back the diagram it holds, and nothing else.
2. A description in JSON is refused, and the message names the format it was given and the one it
   reads.
3. The shapes are read in the order they appear, and that order is the drawing order.
4. `next_id` is required, is a nonzero ordinal, and is trusted beyond being nonzero.
5. An identity written as free text is refused by name.
6. `at` as a point and `at` as a reference are the two accepted kinds, and an unknown one is
   reported by name beside the two that are.
7. `terminal` as a glyph and `terminal` as an arm are the two accepted kinds, and an unknown one is
   reported by name beside the two that are.
8. A `fill` or a terminal glyph of more than one grapheme cluster is refused, naming the field.
9. `next_id` written after the first `[[shapes]]` is refused as a missing top-level field.
10. A render marker carries a TOML description verbatim, and `render_one` writes it to a `.toml`
    scratch file and reads it back.
11. The command-line application's `--size`, `--origin`, its default window and its bare
    demonstration are unchanged by this change.
12. Every picture in every tracked document is the picture its marker declares, which
    `cargo xtask render --check` decides.

## Examples

**One box, one line, one connector.** Hypothetical in its new spelling — no code produces it yet —
but not in its picture or its meaning: it is the description `docs/diagram-demo.md` and `specs/173`
carry today, with `[[shapes]]` and inline tables in place of braces. **The picture below was
observed**, by running `monospace-cli` on the description as it stands today.

````markdown
<!-- render: 17x3
next_id = 4

[[shapes]]
kind = "box"
id = 1
at = { x = 0, y = 0 }
size = { width = 4, height = 3 }
stroke = "light"
fill = "░"

[[shapes]]
kind = "line"
id = 2
at = { x = 6, y = 1 }
len = 4
orientation = "horizontal"
stroke = "light"

[[shapes]]
kind = "connector"
id = 3
from = { at = { kind = "point", x = 12, y = 1 }, leaving = "right",
         terminal = { kind = "glyph", glyph = "◀" } }
to = { at = { kind = "point", x = 16, y = 1 }, leaving = "left",
       terminal = { kind = "arm" } }
stroke = "light"
-->

```text
┌──┐
│░░│  ────  ◀────
└──┘
```

<!-- /render -->
````

The two-line `from` and `to` are the one place D4's cost is visible. An inline table must fit on one
line in TOML 1.0; the line breaks above are how a hand writes it around that rule, and the picture
is unchanged because the file is read as the same table either way.

**A reference endpoint.** Also hypothetical, and the one example a reader should look at twice,
because the reference is the format's one genuinely nested construct:

```toml
[[shapes]]
kind = "connector"
id = 3
from = { at = { kind = "point", x = 12, y = 1 }, leaving = "right", terminal = { kind = "arm" } }
to = { at = { kind = "reference", shape = 1, anchor = "bottom", out = 1 },
       leaving = "left", terminal = { kind = "arm" } }
stroke = "light"
```

Three levels of table, one line each, and no braces anywhere. That is what D4 buys, and D2's
ordering rule is what keeps it readable: every key that belongs to the endpoint is written before
the endpoint's own sub-tables, and every key that belongs to the shape is written before
`[[shapes]]`' next entry.

**A description in the old spelling.** Also hypothetical, and the only one here whose output is a
claim about behavior rather than about a window: with D1 this is refused, and the error names both
formats.

```json
{ "next_id": 4, "shapes": [ ... ] }

  monospace-cli file.json    →    the format is JSON; this crate reads TOML
```

Rule 2 is the whole of that block. A reader with an old file gets a sentence that tells them what to
do, which is the entire argument for D1's refusal.

## What proves it

None of these tests exists yet; the deciding stage adds no code. The names are what the building
stage is held to.

| Rule | Test                                                                     |
| ---- | ------------------------------------------------------------------------ |
| 1    | `parse_reads_toml_and_hands_back_the_diagram_and_nothing_else`           |
| 2    | `a_json_description_is_refused_and_names_both_formats`                   |
| 3    | `shapes_are_read_in_the_order_they_appear_and_that_is_the_drawing_order` |
| 4    | `a_missing_next_id_is_refused_by_name`                                   |
| 5    | `an_identity_written_as_free_text_is_refused_and_names_the_field`        |
| 6    | `an_unknown_at_kind_names_it_and_the_two_that_are_accepted`              |
| 7    | `an_unknown_terminal_kind_names_it_and_the_two_that_are_accepted`        |
| 8    | `a_multi_grapheme_fill_is_refused_naming_the_field`                      |
| 9    | `a_next_id_after_the_shapes_is_refused_as_a_missing_top_level_field`     |
| 10   | `a_marker_body_is_written_to_a_toml_scratch_file_and_read_back`          |
| 11   | `the_flags_the_default_and_the_demonstration_are_unchanged`              |
| 12   | `cargo xtask render --check`                                             |

Rule 12 is the gate's own `render` step rather than a test, and it is the only line here that covers
all twenty-eight markers at once.

**Measured rather than tested, and reported in the building pull request.** That every picture is
unchanged: every marker rewritten, `cargo xtask render` run, and the number of pictures that moved
reported — which is the honest answer to whether changing the spelling changed anything a reader
sees, and it should be `0`. That the three tests rewriting `DEMO` still draw the same pictures,
which is the one place D8 could have moved a snapshot.

## Open questions

- **The sequence with #203.** This spec assumes #203's building stage has merged. It has not. Either
  this change waits for it, or its first commit is #203's and the two are one branch — which is a
  question for the maintainer, not a section here.
- **Whether `At::Point`'s internally tagged newtype variant deserializes through `toml`.** Serde's
  own documentation says such a variant _deserializes and does not serialize_, and this format is
  read-only, so the existing code has never had to check. Whether `toml`'s deserializer takes that
  path is a measurement for the building stage, and a failure here is a blocker that changes D4
  rather than a detail: the point would have to become a struct variant with `x` and `y` written
  out, and every marker with a connector would grow a line.
- **`toml` or `toml_edit`, if the writer comes.** `toml` reads and is what this change adds. A
  writer that has to preserve a file's formatting and comments is `toml_edit`'s argument, and it is
  an argument for another day — but it is worth recording here, because the day the writer arrives,
  the choice is already made by this change's dependency.
