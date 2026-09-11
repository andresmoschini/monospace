# Data Model: Simplify CLI to demo shapes

Every type below is new, private to `monospace-cli`, and exists only to be deserialized from JSON
and then converted into the matching `monospace-core` value. None of it is exposed outside the
binary, and none of it is added to `monospace-core` (FR-019, ADR-0035).

## Description

The whole of one file (Key Entities, _Diagram description_).

| Field    | Type                    | Maps to (core)                    | Notes                                |
| -------- | ----------------------- | --------------------------------- | ------------------------------------ |
| `canvas` | `Canvas`                | `Buffer::new` args, `render` args | Required (FR-005).                   |
| `shapes` | `Vec<ShapeDescription>` | drawn in order via `Layer`        | Required; may be empty (Edge Cases). |

## Canvas

The window the diagram is drawn on (Key Entities, _Canvas_).

| Field    | Type   | Maps to (core)           | Notes                                                                          |
| -------- | ------ | ------------------------ | ------------------------------------------------------------------------------ |
| `origin` | `Pos`  | `Pos { x, y }`           | May be negative (core places no restriction).                                  |
| `size`   | `Size` | `Size { width, height }` | `width`/`height` are `u32`; zero is valid (Edge Cases: "canvas size is zero"). |

## Pos (shared shape)

| Field | Type  |
| ----- | ----- |
| `x`   | `i32` |
| `y`   | `i32` |

## Size (shared shape)

| Field    | Type  |
| -------- | ----- |
| `width`  | `u32` |
| `height` | `u32` |

## ShapeDescription

One entry in `shapes`, tagged by `kind` (FR-006). Internally tagged (`#[serde(tag = "kind")]`) so an
unrecognized value of `kind` is what FR-014's message names.

### `kind: "box"` → `monospace_core::BoxShape`

| Field    | Type                    | Maps to                                 | Notes                                                                                                                                                                     |
| -------- | ----------------------- | --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `at`     | `Pos`                   | `at`                                    | Top-left corner.                                                                                                                                                          |
| `size`   | `Size`                  | `size`                                  | Below 2 in either dimension draws nothing — core's own rule, not re-validated here (Edge Cases).                                                                          |
| `stroke` | `String`                | `Stroke::from`                          | Any name is accepted; an unknown one renders per the catalog's miss behavior (Edge Cases).                                                                                |
| `fill`   | `Option<String>`        | `fill`                                  | `None`/absent = no fill. When present, must be one grapheme cluster with no control character — `Glyph::new` returns `None` otherwise, reported as a data error (FR-014). |
| `mode`   | `StampMode` (see below) | stamp mode used when drawing this shape | Per-shape, per FR-008.                                                                                                                                                    |

### `kind: "line"` → `monospace_core::Line`

| Field         | Type                           | Maps to        | Notes                         |
| ------------- | ------------------------------ | -------------- | ----------------------------- |
| `at`          | `Pos`                          | `at`           | First cell.                   |
| `len`         | `u32`                          | `len`          | 0 is valid and draws nothing. |
| `orientation` | `"horizontal"` \| `"vertical"` | `orientation`  |                               |
| `stroke`      | `String`                       | `Stroke::from` | Same miss behavior as above.  |
| `mode`        | `StampMode`                    | stamp mode     |                               |

### `kind: "arrow"` → `monospace_core::Arrow`

| Field    | Type        | Maps to    | Notes                                              |
| -------- | ----------- | ---------- | -------------------------------------------------- |
| `from`   | `Endpoint`  | `from`     |                                                    |
| `to`     | `Endpoint`  | `to`       |                                                    |
| `stroke` | `String`    | `stroke`   | Colors the route only; each head is its own glyph. |
| `mode`   | `StampMode` | stamp mode |                                                    |

### Endpoint (shared by `from`/`to` on an arrow)

| Field     | Type                                        | Maps to   | Notes                                      |
| --------- | ------------------------------------------- | --------- | ------------------------------------------ |
| `at`      | `Pos`                                       | `at`      |                                            |
| `leaving` | `"up"` \| `"right"` \| `"down"` \| `"left"` | `leaving` |                                            |
| `head`    | `String`                                    | `head`    | One grapheme cluster, same rule as `fill`. |

### StampMode

`"above"` → `StampMode::Above`, `"below"` → `StampMode::Below` (FR-008). Any other string is a data
error naming the unrecognized value (FR-014).

## Validation summary

| Rule                                                     | Enforced by                                                              | FR     |
| -------------------------------------------------------- | ------------------------------------------------------------------------ | ------ |
| File must exist and be readable                          | `std::fs::read_to_string` error → message naming the path                | FR-012 |
| Text must be well-formed JSON                            | `serde_json::from_str` syntax error → location message                   | FR-013 |
| `kind` must be one of `box`, `line`, `arrow`             | internally tagged enum → names the unrecognized kind                     | FR-014 |
| Required fields must be present, of the right type       | `serde`'s derived `Deserialize`                                          | FR-014 |
| `fill`/`head` must be one grapheme, no control character | `Glyph::new` at conversion time, checked after deserializing             | FR-014 |
| Shapes drawn in file order                               | `Vec` preserves JSON array order; shapes stamped in a `for` loop over it | FR-009 |

## Relationships

A `Description` owns one `Canvas` and an ordered `Vec<ShapeDescription>`. A `ShapeDescription` names
no other shape and carries no reference into the rest of the file — flat data, per Key Entities:
"holds no conditionals, variables or references between shapes."
