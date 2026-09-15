//! The diagram description format: JSON types deserialized from a file and converted into a
//! `monospace_diagram::Diagram` and the `monospace_core::Buffer` its canvas describes.
//!
//! Every type here is private to `monospace-cli` and exists only for this conversion (FR-019,
//! [ADR-0035](../../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md)). See
//! `specs/079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md` for the
//! format itself and `data-model.md` for the field-by-field mapping onto `monospace_diagram`.

use monospace_core::{Buffer, Direction, Glyph, Orientation as CoreOrientation};
use monospace_diagram::{Diagram, Endpoint as DiagramEndpoint, Shape as DiagramShape};
use serde::{Deserialize, Deserializer};

/// A position, mirroring `monospace_core::Pos` for deserialization.
#[derive(Deserialize, Debug, Clone, Copy)]
struct Pos {
    x: i32,
    y: i32,
}

impl From<Pos> for monospace_core::Pos {
    fn from(pos: Pos) -> Self {
        monospace_core::Pos { x: pos.x, y: pos.y }
    }
}

/// A size, mirroring `monospace_core::Size` for deserialization.
#[derive(Deserialize, Debug, Clone, Copy)]
struct Size {
    width: u32,
    height: u32,
}

impl From<Size> for monospace_core::Size {
    fn from(size: Size) -> Self {
        monospace_core::Size {
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

impl From<Endpoint> for DiagramEndpoint {
    fn from(endpoint: Endpoint) -> Self {
        DiagramEndpoint {
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
    },
    Line {
        at: Pos,
        len: u32,
        orientation: Orientation,
        stroke: String,
    },
    Arrow {
        from: Endpoint,
        to: Endpoint,
        stroke: String,
    },
}

impl From<ShapeDescription> for DiagramShape {
    /// Converts this description into the matching `monospace_diagram` shape, dropping no
    /// parameter (FR-008, FR-009).
    fn from(description: ShapeDescription) -> Self {
        match description {
            ShapeDescription::Box {
                at,
                size,
                stroke,
                fill,
            } => DiagramShape::Box {
                at: at.into(),
                size: size.into(),
                stroke: stroke.as_str().into(),
                fill,
            },
            ShapeDescription::Line {
                at,
                len,
                orientation,
                stroke,
            } => DiagramShape::Line {
                at: at.into(),
                len,
                orientation: orientation.into(),
                stroke: stroke.as_str().into(),
            },
            ShapeDescription::Arrow { from, to, stroke } => DiagramShape::Arrow {
                from: from.into(),
                to: to.into(),
                stroke: stroke.as_str().into(),
            },
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
    /// The canvas's origin and size, converted to `monospace_core` types.
    pub(crate) fn window(&self) -> (monospace_core::Pos, monospace_core::Size) {
        (self.canvas.origin.into(), self.canvas.size.into())
    }

    /// A buffer the size of `canvas`, with no positions defined yet.
    pub(crate) fn buffer(&self) -> Buffer {
        let (origin, size) = self.window();
        Buffer::new(origin, size)
    }

    /// Builds a diagram from `shapes`, in order (FR-016).
    pub(crate) fn into_diagram(self) -> Diagram {
        let mut diagram = Diagram::new();
        for shape in self.shapes {
            diagram.add(shape.into());
        }
        diagram
    }
}

#[cfg(test)]
mod tests {
    use super::Description;

    /// An unrecognized `kind` fails to deserialize and names the unrecognized value (FR-014).
    #[test]
    fn an_unrecognized_kind_fails_to_deserialize_and_names_it() {
        let json = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "shapes": [ { "kind": "triangle" } ]
        }"#;

        let error =
            serde_json::from_str::<Description>(json).expect_err("unrecognized kind must fail");

        assert!(error.to_string().contains("triangle"), "{error}");
    }

    /// A `fill` of more than one grapheme cluster fails to deserialize.
    #[test]
    fn a_multi_grapheme_fill_fails_to_deserialize() {
        let json = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "shapes": [
                { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "ab" }
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
                  "stroke": "light" }
            ]
        }"#;

        assert!(serde_json::from_str::<Description>(json).is_err());
    }
}
