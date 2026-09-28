<!-- The maintainer's decision draft is a Spanish file outside this repository. -->
<!-- cspell:ignore decisiones -->

# Phase 0 research: a connector endpoint hangs from a box's side anchor

Eight questions. Six are module-level and answered here; two record what 081 already answered, with
the measurement that settles them. None takes an ADR: each answer lives inside one crate and is
undone by changing the code that gives it. The domain-level answers are on
[decisions.md](decisions.md) and are not repeated.

## Q1: `Position`, `Reference`, `Anchor` — the three types, and where resolution lives

**Decision**: `Anchor` is its own four-variant enum — `Top`, `Right`, `Bottom`, `Left` — in a new
`position.rs`, beside a `Position` of `Absolute(Pos)` and `Reference(Reference)`, and a `Reference`
of `id: ShapeId` and `anchor: Anchor` (D1, D2). Only `Endpoint.at` becomes a `Position`. Resolution
is `Diagram::draw`'s work: a private `Diagram::resolve(&self, &Position) -> Option<Pos>` looks the
identity up, asks that shape for the anchor, adds the offsets, and a `None` skips the shape.

**Rationale**: the model's _Vocabulary_ gives all three a row. `Anchor` is not `Direction`
(geometry.rs:34, `Up`/`Right`/`Down`/`Left`) because a direction is which way a figure leaves and a
side is where it is; the two coincide today and one name would mean the wrong thing the day a corner
arrives with #90. `Position` on one field is what makes ADR-0041's restriction a type rather than a
comment. `Diagram::draw` is already the only door — `Shape::draw` is `pub(crate)` (shape.rs:85) —
and it takes `&self`, so the `find` it needs (diagram.rs:73) is in hand. `ShapeId` is `Clone` and
not `Copy`, so a `Reference` is not; nothing here needs it to be.

**Alternatives considered**: one `Position` on all three kinds, which is #89's shape built before
the cycle obligation ADR-0041 attaches to widening it (D2); a pass rewriting each shape into an
absolute one before drawing, which is a second shape type or a clone per drawing.

## Q2: What replaces `From<Endpoint> for monospace_core::Endpoint`?

**Decision**: `Shape::draw` takes a second argument, `&impl Fn(&Position) -> Option<Pos>`, and its
`Connector` arm calls it for both endpoints and returns before writing anything if either is `None`.
The core's `Endpoint` is then built by hand from the resolved `Pos`, the way every other arm already
builds its core shape, and the two match sites stay in one file.

**Rationale**: the impl cannot survive, and this is measured rather than predicted: the core's
`Endpoint.at` is a `Pos` (connector.rs:106) and a `Position` has no `Pos` to give it without a
diagram. A second argument keeps the single `match` over `Shape` in `shape.rs`, and the `kind_of`
match in `gallery.rs` beside it, as the closed-set checks they are. The `Box` and `Line` arms ignore
the argument, which is ADR-0041's restriction stated as a signature rather than as a rule to enforce
at run time. A `Resolved` newtype the `Connector` arm takes instead is the same thing spelled with a
type and one more name.

## Q3: The removal test on two things this slice could add

**Decision**: neither is added. The anchor query stays `pub(crate)`, and no method attaches an
endpoint.

**Rationale**: the specification's testing expectations ask for each anchor to be "asked for", and a
`#[cfg(test)]` module in this crate can ask a `pub(crate)` one — `diagram.rs` and `shape.rs` already
hold all 37 tests that run today. Principle III's removal test is the constitution's own argument,
and ADR-0041 applies it to public API in as many words: "an entry has to be shown to break something
when taken out. It applies to public API as much as to configuration". Nothing outside the crate
asks for an anchor: `cargo xtask render` cannot, because a reference is in no description format,
and `monospace-cli` does not — the demonstration rebuilds the connector by matching on it, which
`Shape`'s public fields already permit. A public anchor query is not what ADR-0040's _Confirmation_
needs, since "positions that can be asserted directly" is satisfied from inside; and
`Shape::attached_at(&self, which, Position) -> Self` is four lines and one more method for the
single caller that can already match.

## Q4: What the fifth picture costs, and why no marker can draw it

**Decision**: the demonstration builds it in code, from the same diagram it has been mutating, and
no field grows in the description format.

**Rationale**: measured. `assets/demo.json:18-24` is the third entry, a box at `{9, 2}` four by
three, so its right side centre is `(12, 3)` — and the connector's `from` at line 68 is exactly
`(12, 3)`. B5.1's "the box the arrow already hangs from" is therefore the third shape, and the arrow
already stands on the side the reference would name. The fifth picture replaces the connector with
one whose `from` is a reference to that shape's right side and then displaces the box, as pictures
three and four do with `#1`. ADR-0064 is why a marker cannot help: a marker is a description a
subprocess reads, and a reference is in none, so a picture of one is a gallery's business — and the
gallery, wired in by Q5, is the only carrier in the crate that reaches it. A `reference` field in
the description format is what the specification's "What this slice does not decide" and B5.8 both
refuse, and ADR-0035 with them.

## Q5: `gallery.rs` is in `src/` and is not compiled

**Decision**: wire it in this slice, as a `test` commit of its own before any anchor code —
`#[cfg(test)] mod gallery;` in `lib.rs`.

**Rationale**: measured both ways. `lib.rs:6-8` declares `delta`, `diagram` and `shape` and not
`gallery`, so `cargo test -p monospace-diagram` reports 37 tests and none under `gallery`; its three
tests and its three committed snapshots never run, and the module doc's claim that `kind_of` is the
completeness guarantee is a claim the build does not enforce. Adding the one line makes it 40 tests,
all three green against the snapshots exactly as committed, and
`cargo clippy -p monospace-diagram --all-targets -- -D warnings` stays clean. ADR-0064 makes this
the only carrier here that reaches a fragment, which is what the anchors and a reference are.

**Alternatives considered**: the six contract tests in `diagram.rs`'s `#[cfg(test)] mod tests`
beside the 37 that do run, which is where 079, 080 and 081 put theirs — but then the crate keeps a
file whose doc claims a guarantee nothing enforces, which is the failure `kind_of` exists to
prevent. Moving 079–081's picture claims in as well is out of scope, and a second slice's work.

## Q6: Does 081's D1 still hold, now that the core's endpoint cannot be converted?

**Decision**: yes, and no entry is raised about it.

**Rationale**: 081's D1 answered "whether the core grows a capability" with "`monospace-diagram`
only, the core untouched", giving the reason in the future tense: "the core knows no reference, so a
displacement there is rewritten when an endpoint becomes a `Position`". The impl that answer names
is shape.rs:31, and Q2 measures that it cannot survive. The core's `BoxShape` and `Line` already
expose everything the anchors need — `at`, `size`, `len` and `orientation` are all `pub` — so the
four side centres are computable above the core with no core item at all. Raising it on the sheet
would re-ask in the present tense a question 081 answered in the future, and a core `Shape::anchor`
would put a fragment on an open trait, where nothing forces the closed set `kind_of` needs.

## Q7: Which records become false, and which of them is corrected

**Decision**: `contracts/diagram-api.md` is corrected, because a contract describes what a caller
sees now. The other four stand, and the rustdoc on `displaced_by` is rewritten.

**Rationale**: `4a851ac` drew the line and 081's Q7 applied it — a contract contradicting the code
is worse than none, while a merged spec is not rewritten to match a correction found later.
Measured: `contracts/diagram-api.md:71-72,123-124,133` says "no figure can hold a reference yet" in
three places, and `data-model.md:108`, `quickstart.md:88-89`, `tasks.md:387-388` and
`checklists/requirements.md:72-73` each say it once. The rustdoc is neither a contract nor a merged
spec: it is code, and shape.rs:134's "the issue that introduces one settles it" becomes false the
moment this slice lands, so it is corrected in the commit that makes it wrong. Correcting all five
would be `4a851ac`'s rule rejected; correcting none leaves the one artifact a caller reads
describing a crate that cannot do what this slice gives it.

## Q8: Is 081's D6 decided yet?

**Decision**: yes, and it is not raised on the sheet. The specification's 2026-09-28 clarification
answers it and #142 carries what we would rather it did.

**Rationale**: 081's D6 recorded "what `remove` says about references" as "nothing... #82 settles
it, where there is a `Reference` to test against", and the clarification answers it: the reference
is left exactly as it was, it does not resolve, the shape that held it is not drawn, and nothing
errors, reports or rewrites. That is `docs/diagram-model.md` §9's paragraph as written, so there is
nothing to decide and no amendment owed to it. An entry would re-ask a clarification session already
answered, and rewriting §9's paragraph would change nothing about what the code does.
