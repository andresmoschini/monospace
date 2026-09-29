//! The diagram description format: JSON types deserialized from a file and converted into a
//! `monospace_diagram::Diagram` and the `monospace_core::Buffer` its canvas describes.
//!
//! Every type here is private to `monospace-cli` and exists only for this conversion (FR-019,
//! [ADR-0035](../../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md)). See
//! `specs/079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md` for the
//! format itself and `data-model.md` for the field-by-field mapping onto `monospace_diagram`.

use monospace_core::{Direction, Glyph, Orientation as CoreOrientation};
use monospace_diagram::{Diagram, Endpoint as DiagramEndpoint, Position, Shape as DiagramShape};
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

/// The direction a connector endpoint leaves in, mirroring `monospace_core::Direction`.
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

/// What an endpoint's `terminal` is on the wire, tagged by `kind` (FR-006): one chosen glyph or one
/// arm. An unrecognized `kind` is reported by name, and the message names the two accepted.
///
/// Internally tagged rather than externally, because `deserialize_glyph` needs a _named_ field to
/// sit on and the external form has none — `Glyph(Glyph)` does not compile, since `Glyph` derives
/// no `Deserialize` (research.md Q3). A field an `arm` does not know is ignored, as everywhere else
/// in this format.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Terminal {
    Glyph {
        #[serde(deserialize_with = "deserialize_glyph")]
        glyph: Glyph,
    },
    Arm,
}

impl From<Terminal> for monospace_core::Terminal {
    fn from(terminal: Terminal) -> Self {
        match terminal {
            Terminal::Glyph { glyph } => monospace_core::Terminal::Glyph { glyph },
            Terminal::Arm => monospace_core::Terminal::Arm,
        }
    }
}

/// One endpoint of a connector: a position, the direction it leaves in, and its terminal.
///
/// The wire format holds a point and nothing else, so the diagram's `Position` is named explicitly
/// here: `From<Pos> for Position` does not reach this file, because the `into()` below resolves
/// `description::Pos → monospace_core::Pos` on its way to a diagram position that has to be
/// `Absolute`. A reference is in no description format — a caller wanting one builds the picture in
/// code, the way the shipped demonstration does (B5.8).
#[derive(Deserialize, Debug, Clone)]
struct Endpoint {
    at: Pos,
    leaving: Leaving,
    terminal: Terminal,
}

impl From<Endpoint> for DiagramEndpoint {
    fn from(endpoint: Endpoint) -> Self {
        DiagramEndpoint {
            at: Position::Absolute(endpoint.at.into()),
            leaving: endpoint.leaving.into(),
            terminal: endpoint.terminal.into(),
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
    Connector {
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
            ShapeDescription::Connector { from, to, stroke } => DiagramShape::Connector {
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

    /// FR-014 through the rename: a `terminal`'s glyph of more than one grapheme cluster fails to
    /// deserialize. The grapheme check sits on the named field inside the tagged object, which is
    /// the only reason the wire form is internally tagged (research.md Q3).
    #[test]
    fn a_multi_grapheme_terminal_glyph_fails_to_deserialize() {
        let json = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 1 } },
            "shapes": [
                { "kind": "connector",
                  "from": { "at": { "x": 0, "y": 0 }, "leaving": "right",
                            "terminal": { "kind": "glyph", "glyph": "ab" } },
                  "to": { "at": { "x": 6, "y": 0 }, "leaving": "left",
                          "terminal": { "kind": "glyph", "glyph": ">" } },
                  "stroke": "light" }
            ]
        }"#;

        assert!(serde_json::from_str::<Description>(json).is_err());
    }

    /// One connector, with `TERMINAL` standing where the `from` endpoint's terminal goes. The three
    /// refusals below differ only in what is written there, which is what makes them the same test
    /// three times over.
    const ARROW: &str = r#""from": { "at": { "x": 0, "y": 0 }, "leaving": "right", "terminal": TERMINAL },
                  "to": { "at": { "x": 6, "y": 0 }, "leaving": "left",
                          "terminal": { "kind": "glyph", "glyph": ">" } }"#;

    /// The same connector with the field left out altogether.
    const ARROW_WITHOUT_A_TERMINAL: &str = r#""from": { "at": { "x": 0, "y": 0 }, "leaving": "right" },
                  "to": { "at": { "x": 6, "y": 0 }, "leaving": "left",
                          "terminal": { "kind": "glyph", "glyph": ">" } }"#;

    fn description_of(connector: &str) -> String {
        format!(
            r#"{{
            "canvas": {{ "origin": {{ "x": 0, "y": 0 }}, "size": {{ "width": 8, "height": 1 }} }},
            "shapes": [
                {{ "kind": "connector",
                  {connector},
                  "stroke": "light" }}
            ]
        }}"#
        )
    }

    /// The message the binary reports for an endpoint whose terminal is `terminal`.
    ///
    /// `serde_json` appends the position the problem was found at, so what is compared is the
    /// message this crate produced and not the whole of what `serde_json` renders.
    fn error_for(terminal: &str) -> String {
        let json = description_of(&ARROW.replace("TERMINAL", terminal));
        serde_json::from_str::<Description>(&json)
            .expect_err("a terminal this format does not accept must fail")
            .to_string()
    }

    /// SC-005, spec's B1 scenario 3: a `kind` the model has not named is refused by name, and the
    /// message names the two that are accepted — so the value is never read as one of them.
    #[test]
    fn an_unrecognized_terminal_kind_names_it_and_the_two_that_are_accepted() {
        let error = error_for(r#"{ "kind": "dot" }"#);

        assert!(
            error.starts_with("unknown variant `dot`, expected `glyph` or `arm`"),
            "{error}"
        );
    }

    /// FR-014 through the rename: a `glyph` that is not exactly one grapheme cluster is rejected
    /// with the message the old `head` produced, byte for byte.
    #[test]
    fn a_multi_grapheme_terminal_glyph_is_rejected_with_the_message_the_old_head_produced() {
        let error = error_for(r#"{ "kind": "glyph", "glyph": "ab" }"#);

        assert!(
            error.starts_with("\"ab\" is not exactly one grapheme cluster"),
            "{error}"
        );
    }

    /// B1 scenario 3: the field is required for both values, so a file that omits it is refused
    /// rather than read as a connector with no terminal at either end. A terminal's presence never
    /// decides what a description means.
    #[test]
    fn an_omitted_terminal_field_is_refused_by_name() {
        let json = description_of(ARROW_WITHOUT_A_TERMINAL);

        let error = serde_json::from_str::<Description>(&json)
            .expect_err("a connector with no terminal at all must fail")
            .to_string();

        assert!(error.starts_with("missing field `terminal`"), "{error}");
    }

    /// B1 scenario 3, and the externally tagged spelling research.md Q3 measured: every value of
    /// `terminal` is an object tagged by `kind`, so a bare string is refused on purpose rather
    /// than read as an arm.
    #[test]
    fn an_externally_tagged_terminal_is_refused() {
        let error = error_for(r#""arm""#);

        assert!(
            error.starts_with(
                r#"invalid type: string "arm", expected internally tagged enum Terminal"#
            ),
            "{error}"
        );
    }
}
