# Phase 1 Data Model: Ship the mixing tables

This feature's "data" is Rust `const` tables and the functions that turn them into a `GlyphCatalog`,
plus a corrected JSON fixture — nothing persisted. This file names what each one holds and what it
does and doesn't let a caller do, the mapping `research.md`'s decisions need before code exists.

## `Row` (existing private type, unchanged)

Already defined in `monospace-glyph-sets/src/lib.rs`, reused as-is — no field change, no new
variant:

```rust
type Row = (
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    &'static str,
);
```

A mixing table's rows differ from a single-stroke table's only in that a row's four `Option`s can
name either of the pair's two stroke strings (`"light"`/`"double"`, `"light"`/`"heavy"`,
`"light-round"`/`"double"`, `"light-round"`/`"heavy"`) instead of always the same one. Nothing about
the type changes to allow that — it always allowed any `&'static str`.

## `MIXING_LIGHT_AND_DOUBLE`, `MIXING_LIGHT_AND_HEAVY`, `MIXING_LIGHT_ROUND_AND_DOUBLE`, `MIXING_LIGHT_ROUND_AND_HEAVY` (new private `const` tables)

Each `&'static [Row]`, copied verbatim, in the same row order, from `docs/glyph-sets.md`'s four
_Mixing ..._ sections:

| Table                           | Rows | Stroke names on a row's four sides |
| ------------------------------- | ---- | ---------------------------------- |
| `MIXING_LIGHT_AND_DOUBLE`       | 18   | `"light"`, `"double"`              |
| `MIXING_LIGHT_AND_HEAVY`        | 50   | `"light"`, `"heavy"`               |
| `MIXING_LIGHT_ROUND_AND_DOUBLE` | 18   | `"light-round"`, `"double"`        |
| `MIXING_LIGHT_ROUND_AND_HEAVY`  | 50   | `"light-round"`, `"heavy"`         |

**Invariant carried over from every existing table**: every row is a valid `(GlyphKey, Glyph)` pair
once converted through `build` — `Glyph::new` on the character succeeds, so there is nothing to
validate beyond `build`'s existing panic (FR-001 through FR-004). None of the 136 rows is the empty
key, matching what "the rows `docs/glyph-sets.md` records" already means for these four sections.

**FR-007 (no key collision)**: no row in any of the four new tables uses only one stroke name on
every non-empty side — every row names both strokes of its pair on at least the sides the document's
table gives it — so no mixing table's key can collide with a single-stroke table's key (which by
definition names only one stroke) or with another mixing table's key (which names a different pair).
This is checked, not assumed: a test builds a catalog from all nine tables now shipped and confirms
every key answers from whichever table's rows define it, independent of union order.

## `build` (existing private helper, unchanged)

```rust
fn build(table: &str, rows: &[Row]) -> GlyphCatalog
```

Already generic over any `&[Row]` of any length; the four new tables call it exactly as `ascii()`,
`double()`, `heavy()` and `light_round()` already do, with their own name and rows. No change to its
signature, its body, or its panic condition.

## `mixing_light_and_double`, `mixing_light_and_heavy`, `mixing_light_round_and_double`, `mixing_light_round_and_heavy` (new public functions)

```rust
#[must_use]
pub fn mixing_light_and_double() -> GlyphCatalog        // build("Mixing Light and Double", ...)
#[must_use]
pub fn mixing_light_and_heavy() -> GlyphCatalog         // build("Mixing Light and Heavy", ...)
#[must_use]
pub fn mixing_light_round_and_double() -> GlyphCatalog  // build("Mixing Light Round and Double", ...)
#[must_use]
pub fn mixing_light_round_and_heavy() -> GlyphCatalog   // build("Mixing Light Round and Heavy", ...)
```

Same shape as every existing table function (FR-005): no argument, no privileged access to anything
`monospace-core` does not already expose publicly, `#[must_use]` because discarding the built
catalog is always a mistake.

**What stays impossible**: none of the four accepts a stroke to filter by or a table to merge with —
combining a mixing table with the single-stroke tables it draws on remains the caller's job, done
with the existing `GlyphCatalog::union` (FR-006, unchanged by this feature).

## `crates/monospace-cli/assets/demo.json` (corrected fixture)

No new field and no new shape kind — the same `Canvas`/`ShapeDescription` format `description.rs`
already deserializes. Two of the six box pairs feature 056 added below the original demonstration
change their `stroke` field only:

| Position pair    | Before                  | After                    |
| ---------------- | ----------------------- | ------------------------ |
| (32,9) / (34,10) | `light` / `light-round` | `light-round` / `double` |
| (40,9) / (42,10) | `heavy` / `double`      | `light-round` / `heavy`  |

Every other field on those boxes (`at`, `size`, `mode`, absence of `fill`) is unchanged, and every
other shape in the file is untouched (FR-010). If checking the existing Light+Double crossing
against _Mixing Light and Double_'s eighteen covered rows shows it lands outside them, its two
boxes' `at` positions move by whatever offset lands it on a covered row — their `stroke` fields do
not change, since Light+Double is already the pair FR-009 asks that crossing to exercise.

## `Description::render`'s catalog (corrected union)

`crates/monospace-cli/src/description.rs`'s `Description::render` gains four more entries in its
existing `GlyphCatalog::union([...])` call:

```rust
let catalog = GlyphCatalog::union([
    GlyphCatalog::light(),
    monospace_glyph_sets::ascii(),
    monospace_glyph_sets::double(),
    monospace_glyph_sets::heavy(),
    monospace_glyph_sets::light_round(),
    monospace_glyph_sets::mixing_light_and_double(),
    monospace_glyph_sets::mixing_light_and_heavy(),
    monospace_glyph_sets::mixing_light_round_and_double(),
    monospace_glyph_sets::mixing_light_round_and_heavy(),
]);
```

No other line of `description.rs` changes.

## Relationships at a glance

```text
docs/glyph-sets.md
  _Mixing Light and Double_       ──copied verbatim──> MIXING_LIGHT_AND_DOUBLE:       &[Row; 18]
  _Mixing Light and Heavy_        ──copied verbatim──> MIXING_LIGHT_AND_HEAVY:        &[Row; 50]
  _Mixing Light Round and Double_ ──copied verbatim──> MIXING_LIGHT_ROUND_AND_DOUBLE: &[Row; 18]
  _Mixing Light Round and Heavy_  ──copied verbatim──> MIXING_LIGHT_ROUND_AND_HEAVY:  &[Row; 50]

MIXING_* ──build(name, rows)──> GlyphCatalog::from_rules(..) ──> GlyphCatalog

mixing_light_and_double(), mixing_light_and_heavy(),
mixing_light_round_and_double(), mixing_light_round_and_heavy()   # same shape as every table so far

GlyphCatalog::union([light(), ascii(), double(), heavy(), light_round(),
                      mixing_light_and_double(), mixing_light_and_heavy(),
                      mixing_light_round_and_double(), mixing_light_round_and_heavy()])
  ──> a key from any of the nine answers independently of union order (FR-007)

crates/monospace-cli/assets/demo.json ──corrected stroke fields──> exercises all four mixing pairs
crates/monospace-cli/src/description.rs ──corrected union──> renders the mixed junctions
```

## The documents (no new entity, corrected prose)

`docs/glyph-sets.md`'s opening note and `docs/model.md`'s _Strokes, glyph sets and the catalog_ both
currently say the mixing sets are reference-only, loaded from a file rather than shipped as data.
Both are corrected to name the four mixing tables alongside the five single-stroke ones as data
`monospace-glyph-sets` ships (see `research.md`'s closing section for why, given FR-010's scope
wording). Neither document's tables of rows change — only the sentence describing where a mixing
set's rules come from.
