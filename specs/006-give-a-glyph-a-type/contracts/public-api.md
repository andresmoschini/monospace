# Public API contract: Give a glyph a type of its own

**Feature**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Date**: 2026-09-08

The interface `monospace-core` exposes is its public Rust API; that is the whole contract this
feature changes. It is identical after P1 and after P2 — the widening changes what `new` accepts,
not what it looks like, and FR-014 with SC-006 are what hold that.

## Added

```rust
/// What a cell renders to: one grapheme cluster, never a control character.
pub struct Glyph(/* private */);

impl Glyph {
    /// `Some` when `text` is exactly one grapheme cluster containing no control character.
    pub fn new(text: &str) -> Option<Self>;

    /// The glyph's text.
    pub fn as_str(&self) -> &str;
}
```

`Glyph` derives `Debug`, `Clone`, `PartialEq` and `Eq`, and is re-exported from the crate root
alongside `GlyphCatalog` and `GlyphKey`.

## Changed

```rust
impl GlyphCatalog {
    // Was: pub fn glyph(&self, key: &GlyphKey) -> Option<char>;
    pub fn glyph(&self, key: &GlyphKey) -> Option<&Glyph>;
}
```

Every caller of `glyph` moves from comparing a `char` to comparing text through `as_str`. Inside
this repository that is the crate's own tests and nothing else — `monospace-cli` names neither a
glyph nor a character.

## Unchanged

`Buffer`, `Cell`, `Arm`, `StampMode`, `Pos`, `Size`, `Stroke`, `GlyphKey`, `GlyphCatalog::light` and
`render`'s signature. `render` still returns a `String` of exactly `size.height` lines; what changes
is that the promise is read in glyphs rather than characters (FR-017), which is the same rectangle.

## Behavioral guarantees a caller can rely on

| Guarantee                                                                           | From     |
| ----------------------------------------------------------------------------------- | -------- |
| An invalid glyph cannot be observed, because construction is the only way in        | FR-005   |
| Refusal carries no detail: there is nothing to match on beyond its absence          | FR-006   |
| `light()` answers every key the Light table gives a glyph for, and no other key     | FR-008   |
| `light()` panics only on a bug in data this library ships, never on caller input    | FR-009   |
| Equality is by text, not by appearance; nothing is normalized                       | FR-018   |
| Width is not part of the invariant: a wide grapheme is one glyph and shifts its row | ADR-0019 |

## Documentation contract

Every item above carries rustdoc as it is introduced. `Glyph`'s says what it accepts and, in the
same breath, that neither width nor normal form is part of the invariant. `GlyphCatalog::light`'s
carries a `# Panics` section, which `clippy::missing_panics_doc` will require as soon as the panic
exists.
