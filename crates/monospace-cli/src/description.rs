//! The diagram description format: JSON types deserialized from a file and converted into
//! `monospace_core` shapes.
//!
//! Every type here is private to `monospace-cli` and exists only for this conversion (FR-019,
//! [ADR-0035](../../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md)). See
//! `specs/045-simplify-cli-to-demo-shapes/contracts/description-format.md` for the format itself
//! and `data-model.md` for the field-by-field mapping onto `monospace_core`.

use monospace_core::{
    Arrow, Buffer, Direction, Glyph, GlyphCatalog, Layer, Line, Orientation as CoreOrientation,
    Pos as CorePos, Shape, Size as CoreSize, StampMode as CoreStampMode, Stroke,
};
use serde::{Deserialize, Deserializer};

/// A position, mirroring `monospace_core::Pos` for deserialization.
#[derive(Deserialize, Debug, Clone, Copy)]
struct Pos {
    x: i32,
    y: i32,
}

impl From<Pos> for CorePos {
    fn from(pos: Pos) -> Self {
        CorePos { x: pos.x, y: pos.y }
    }
}

/// A size, mirroring `monospace_core::Size` for deserialization.
#[derive(Deserialize, Debug, Clone, Copy)]
struct Size {
    width: u32,
    height: u32,
}

impl From<Size> for CoreSize {
    fn from(size: Size) -> Self {
        CoreSize {
            width: size.width,
            height: size.height,
        }
    }
}

/// Whether a line runs along a row or a column, mirroring `monospace_core::Orientation`.
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Orientation {
    Horizontal,
    Vertical,
}

impl From<Orientation> for CoreOrientation {
    fn from(orientation: Orientation) -> Self {
        match orientation {
            Orientation::Horizontal => CoreOrientation::Horizontal,
            Orientation::Vertical => CoreOrientation::Vertical,
        }
    }
}

/// The direction an arrow endpoint leaves in, mirroring `monospace_core::Direction`.
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Leaving {
    Up,
    Right,
    Down,
    Left,
}

impl From<Leaving> for Direction {
    fn from(leaving: Leaving) -> Self {
        match leaving {
            Leaving::Up => Direction::Up,
            Leaving::Right => Direction::Right,
            Leaving::Down => Direction::Down,
            Leaving::Left => Direction::Left,
        }
    }
}

/// Which side of an already-defined cell decides when a shape's stamp lands on it, mirroring
/// `monospace_core::StampMode`.
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum StampMode {
    Above,
    Below,
}

impl From<StampMode> for CoreStampMode {
    fn from(mode: StampMode) -> Self {
        match mode {
            StampMode::Above => CoreStampMode::Above,
            StampMode::Below => CoreStampMode::Below,
        }
    }
}

/// Deserializes a required glyph field: the text must be exactly one grapheme cluster with no
/// control character, or this reports the same data-error kind `serde_json` reports for anything
/// else wrong with the file (FR-014).
fn deserialize_glyph<'de, D>(deserializer: D) -> Result<Glyph, D::Error>
where
    D: Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    Glyph::new(&text).ok_or_else(|| {
        serde::de::Error::custom(format!("{text:?} is not exactly one grapheme cluster"))
    })
}

/// Deserializes an optional glyph field (`fill`): absent stays `None`; present, it is checked the
/// same way [`deserialize_glyph`] checks a required one.
fn deserialize_optional_glyph<'de, D>(deserializer: D) -> Result<Option<Glyph>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "deserialize_glyph")] Glyph);

    Ok(Option::<Wrapper>::deserialize(deserializer)?.map(|wrapper| wrapper.0))
}

/// The window a diagram is drawn on and rendered from.
#[derive(Deserialize, Debug)]
struct Canvas {
    origin: Pos,
    size: Size,
}

/// One endpoint of an arrow: a position, the direction it leaves in, and its head glyph.
#[derive(Deserialize, Debug, Clone)]
struct Endpoint {
    at: Pos,
    leaving: Leaving,
    #[serde(deserialize_with = "deserialize_glyph")]
    head: Glyph,
}

impl From<Endpoint> for monospace_core::Endpoint {
    fn from(endpoint: Endpoint) -> Self {
        monospace_core::Endpoint {
            at: endpoint.at.into(),
            leaving: endpoint.leaving.into(),
            head: endpoint.head,
        }
    }
}

/// One entry in a `Description`'s `shapes` array, tagged by `kind` (FR-006). An unrecognized
/// `kind` is reported by name, since this enum is internally tagged (FR-014).
#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum ShapeDescription {
    Box {
        at: Pos,
        size: Size,
        stroke: String,
        #[serde(default, deserialize_with = "deserialize_optional_glyph")]
        fill: Option<Glyph>,
        mode: StampMode,
    },
    Line {
        at: Pos,
        len: u32,
        orientation: Orientation,
        stroke: String,
        mode: StampMode,
    },
    Arrow {
        from: Endpoint,
        to: Endpoint,
        stroke: String,
        mode: StampMode,
    },
}

impl ShapeDescription {
    /// Converts this description into the matching `monospace_core` shape and stamps it into
    /// `buffer` under its own stamp mode (FR-008, FR-009).
    fn draw(&self, buffer: &mut Buffer) {
        match self {
            Self::Box {
                at,
                size,
                stroke,
                fill,
                mode,
            } => {
                monospace_core::BoxShape {
                    at: (*at).into(),
                    size: (*size).into(),
                    stroke: Stroke::from(stroke.as_str()),
                    fill: fill.clone(),
                }
                .draw(&mut Layer::new(buffer, (*mode).into()));
            }
            Self::Line {
                at,
                len,
                orientation,
                stroke,
                mode,
            } => {
                Line {
                    at: (*at).into(),
                    len: *len,
                    orientation: (*orientation).into(),
                    stroke: Stroke::from(stroke.as_str()),
                }
                .draw(&mut Layer::new(buffer, (*mode).into()));
            }
            Self::Arrow {
                from,
                to,
                stroke,
                mode,
            } => {
                Arrow {
                    from: from.clone().into(),
                    to: to.clone().into(),
                    stroke: Stroke::from(stroke.as_str()),
                }
                .draw(&mut Layer::new(buffer, (*mode).into()));
            }
        }
    }
}

/// The whole of one description file: a canvas and an ordered list of shapes.
#[derive(Deserialize, Debug)]
pub struct Description {
    canvas: Canvas,
    shapes: Vec<ShapeDescription>,
}

impl Description {
    /// Draws every shape in `shapes`, in order, into a buffer the size of `canvas`, then renders
    /// that buffer to text (FR-005, FR-009).
    #[must_use]
    pub fn render(&self) -> String {
        let origin = self.canvas.origin.into();
        let size = self.canvas.size.into();
        let mut buffer = Buffer::new(origin, size);
        for shape in &self.shapes {
            shape.draw(&mut buffer);
        }
        let catalog = GlyphCatalog::union([
            GlyphCatalog::light(),
            monospace_glyph_sets::ascii(),
            monospace_glyph_sets::double(),
            monospace_glyph_sets::heavy(),
            monospace_glyph_sets::light_round(),
        ]);
        monospace_core::render(&buffer, &catalog, origin, size)
    }
}

#[cfg(test)]
mod tests {
    use super::Description;
    use monospace_core::{
        BoxShape, Buffer, Glyph, GlyphCatalog, Layer, Pos, Shape, Size, StampMode, Stroke, render,
    };

    fn one_box_json() -> &'static str {
        r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "shapes": [
                { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░", "mode": "above" }
            ]
        }"#
    }

    /// A one-box `Description` renders the same text as a `BoxShape` drawn directly with the same
    /// parameters.
    #[test]
    fn a_one_box_description_renders_the_same_as_a_box_shape_drawn_directly() {
        let description: Description =
            serde_json::from_str(one_box_json()).expect("well-formed description");

        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 4,
            height: 3,
        };
        let mut buffer = Buffer::new(origin, size);
        BoxShape {
            at: origin,
            size,
            stroke: Stroke::from("light"),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));
        let expected = render(&buffer, &GlyphCatalog::light(), origin, size);

        assert_eq!(description.render(), expected);
    }

    /// An unrecognized `kind` fails to deserialize and names the unrecognized value (FR-014).
    #[test]
    fn an_unrecognized_kind_fails_to_deserialize_and_names_it() {
        let json = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "shapes": [ { "kind": "triangle", "mode": "above" } ]
        }"#;

        let error =
            serde_json::from_str::<Description>(json).expect_err("unrecognized kind must fail");

        assert!(error.to_string().contains("triangle"), "{error}");
    }

    /// An unrecognized `mode` fails to deserialize.
    #[test]
    fn an_unrecognized_mode_fails_to_deserialize() {
        let json = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "shapes": [
                { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "mode": "sideways" }
            ]
        }"#;

        assert!(serde_json::from_str::<Description>(json).is_err());
    }

    /// A `fill` of more than one grapheme cluster fails to deserialize.
    #[test]
    fn a_multi_grapheme_fill_fails_to_deserialize() {
        let json = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "shapes": [
                { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "ab", "mode": "above" }
            ]
        }"#;

        assert!(serde_json::from_str::<Description>(json).is_err());
    }

    /// A `head` of more than one grapheme cluster fails to deserialize.
    #[test]
    fn a_multi_grapheme_head_fails_to_deserialize() {
        let json = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 1 } },
            "shapes": [
                { "kind": "arrow",
                  "from": { "at": { "x": 0, "y": 0 }, "leaving": "right", "head": "ab" },
                  "to": { "at": { "x": 6, "y": 0 }, "leaving": "left", "head": ">" },
                  "stroke": "light", "mode": "above" }
            ]
        }"#;

        assert!(serde_json::from_str::<Description>(json).is_err());
    }
}
