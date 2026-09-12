# Phase 0 Research: Ship the Double, Heavy and Light Round tables

The spec's Assumptions already settle the questions a normal Phase 0 would raise here — no new
crate, no new dependency, no ADR, the interface mirrors ASCII's. What is left is the one small
choice the spec deliberately leaves open: how `double()`, `heavy()` and `light_round()` are
expressed alongside the existing `ascii()`.

## The shape of the three new functions

**Decision**: Each of the three gets its own private `const` row table (`DOUBLE`, `HEAVY`,
`LIGHT_ROUND`), using the same `Row` tuple `ascii()` already defines, and its own public function
(`double()`, `heavy()`, `light_round()`) with the same signature and the same `#[must_use]` as
`ascii()`. The row-to-catalog conversion currently inlined in `ascii()` — map each `Row` to a
`(GlyphKey, Glyph)` pair, panicking if a row is not a valid glyph — is pulled out into one private
helper, `fn build(table: &str, rows: &[Row]) -> GlyphCatalog`, that all four public functions call
with their own table name and rows.

```rust
fn build(table: &str, rows: &[Row]) -> GlyphCatalog {
    GlyphCatalog::from_rules(rows.iter().map(|&(top, right, bottom, left, glyph)| {
        let key = GlyphKey {
            top: top.map(Stroke::from),
            right: right.map(Stroke::from),
            bottom: bottom.map(Stroke::from),
            left: left.map(Stroke::from),
        };
        let glyph = Glyph::new(glyph)
            .unwrap_or_else(|| panic!("{table} table row {key:?} is not a valid glyph"));
        (key, glyph)
    }))
}

#[must_use]
pub fn ascii() -> GlyphCatalog {
    build("ASCII", ASCII)
}

#[must_use]
pub fn double() -> GlyphCatalog {
    build("Double", DOUBLE)
}

#[must_use]
pub fn heavy() -> GlyphCatalog {
    build("Heavy", HEAVY)
}

#[must_use]
pub fn light_round() -> GlyphCatalog {
    build("Light Round", LIGHT_ROUND)
}
```

**Rationale**:

- Four tables sharing one four-line conversion is the case a helper is for: without it, adding
  Double, Heavy and Light Round would paste the same loop three more times, and a mistake in one
  copy (a dropped `.map(Stroke::from)`, a different panic wording) would not show up as a compile
  error anywhere.
- The helper is pulled out before the new tables are added, not alongside them, so the extraction is
  provably behavior-preserving on its own: `ascii()`'s existing tests are the only tests that run
  against it, and they must still pass unchanged. Principle V calls this out directly — a `refactor`
  commit adds no test — so the new tables' tests arrive only in the following `feat` commit(s).
- No new type and no new crate: a "table" stays exactly what feature 054 already decided it is, a
  `GlyphCatalog` built from its own rows. This feature adds data and one small helper, not
  architecture.

**Alternatives considered**:

- **Copy `ascii()`'s body three times, one per table.** Rejected: three near-identical closures are
  exactly the kind of duplication a shared four-line helper removes for the cost of one function,
  and a future fourth or fifth table (Light Round's mixing sets, say, if they are ever promoted)
  would keep paying the copy-paste tax instead of just adding a row table and a one-line function.
- **One function taking a `Stroke` enum or a table identifier and dispatching internally**
  (`fn table(which: Table) -> GlyphCatalog`). Rejected: it does not mirror `ascii()`'s existing
  shape (FR-007's "reachable the same way ASCII already is"), forces a new public enum into
  existence for no caller need the spec names, and makes each table's public name a string or enum
  variant to look up instead of a function to call and autocomplete.
- **A macro generating the four functions.** Rejected: four functions of four lines each do not
  justify a macro's cost in readability, and the constitution's preference for the standard library
  and idiomatic, minimal machinery over cleverness (Dependencies) argues the same way even though a
  macro is not a dependency.

## Where the three tables' rows come from

The forty-five rows (fifteen each) are copied verbatim, in the same row order, from
`docs/glyph-sets.md`'s _Double_, _Heavy_ and _Light Round_ tables — the same way `ascii()`'s `ASCII`
table was copied from that document's _ASCII_ table in feature 054. Each row is checked against the
document's row for the same key before the corresponding test is written (SC-004), rather than
transcribed once and trusted.

## No new dependency, no new crate, no ADR

Unchanged from feature 054's finding for `monospace-glyph-sets`: it needs nothing beyond
`monospace-core`, already depended on. The spec's own Assumptions record that no ADR is needed —
ADR-0036 and feature 054 already settled the extension point and the crate boundary this feature
reuses without change.
