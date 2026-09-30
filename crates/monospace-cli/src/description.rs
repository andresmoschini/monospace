//! The diagram description format: JSON types deserialized from a file and converted into a
//! `monospace_diagram::Diagram` and the `monospace_core::Buffer` its canvas describes.
//!
//! Every type here is private to `monospace-cli` and exists only for this conversion (FR-019,
//! [ADR-0035](../../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md)). The
//! format itself is
//! `specs/083-a-reference-carries-a-horizontal-and-a-v/contracts/description-format.md`, which
//! supersedes
//! [`079's`](../../specs/079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md)
//! and stays the record of what the format was before an endpoint's `at` could hold a reference.
//! The field-by-field mapping onto `monospace_diagram` is in 083's `data-model.md`.

use monospace_core::{Direction, Glyph, Orientation as CoreOrientation};
use monospace_diagram::{
    Anchor, Delta, Diagram, Endpoint as DiagramEndpoint, Position, Reference,
    Shape as DiagramShape, ShapeId,
};
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

/// Which of a figure's four sides an endpoint hangs from, mirroring `monospace_diagram::Anchor`.
///
/// The same four and no fifth: there is no corner and no center, and the file cannot name one.
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum AnchorDescription {
    Top,
    Right,
    Bottom,
    Left,
}

impl From<AnchorDescription> for Anchor {
    fn from(anchor: AnchorDescription) -> Self {
        match anchor {
            AnchorDescription::Top => Anchor::Top,
            AnchorDescription::Right => Anchor::Right,
            AnchorDescription::Bottom => Anchor::Bottom,
            AnchorDescription::Left => Anchor::Left,
        }
    }
}

/// How far from that side an endpoint stands, mirroring `monospace_diagram::Delta`.
///
/// `Default` is what makes the field optional on the wire: `#[serde(default)]` on the field below
/// builds a zero delta when the key is absent, so a reference that means "on the side itself" spells
/// three fields rather than five. The two amounts are signed and are in the **screen** axes whatever
/// side the anchor names — `dx` is cells right, `dy` cells down — and neither is checked against the
/// side it is measured from.
#[derive(Deserialize, Debug, Clone, Copy, Default)]
struct OffsetDescription {
    dx: i32,
    dy: i32,
}

impl From<OffsetDescription> for Delta {
    fn from(offset: OffsetDescription) -> Self {
        Delta {
            dx: offset.dx,
            dy: offset.dy,
        }
    }
}

/// What an endpoint's `at` is on the wire, tagged by `kind`: a point, or a reference to a side of
/// another figure (SC-004).
///
/// **The tag is the rule, not a sentence.** A file cannot say both, cannot say neither, and cannot
/// say one by mistake — all three are compile-time facts about this enum rather than conventions a
/// reader has to remember. The three spellings were measured against this crate's own `serde`
/// rather than argued (research.md Q2): an untagged union reads every existing description and
/// answers anything wrong with `data did not match any variant of untagged enum At`, and a sibling
/// `reference` beside an optional `at` refuses nothing at all.
///
/// A reference to a shape the diagram does not hold, or to an anchor that kind does not answer, is
/// **not** an error here or anywhere downstream: the figure holding it is simply not drawn and the
/// run succeeds. That is the cost
/// [ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md) already
/// accepts.
///
/// `Point` is a **newtype** over the file's own [`Pos`] rather than a struct variant with `x` and
/// `y` written out, which keeps one `Pos` in this file instead of a second pair of coordinates to
/// convert. Both spellings were measured and read `{"kind": "point", "x": 1, "y": 1}` and refused
/// `{"x": 1, "y": 1}` identically, so the newtype is taken for the one `Pos` it saves. It carries a
/// trap worth knowing: **an internally tagged newtype variant deserializes and does not serialize**.
/// Nothing here is broken, because the format is read and never written — but a caller who later
/// wanted a `Serialize` on this type would have to widen the variant, and this is where to find out
/// why.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum At {
    Point(Pos),
    Reference {
        shape: String,
        anchor: AnchorDescription,
        #[serde(default)]
        offset: OffsetDescription,
    },
}

/// One endpoint of a connector: where it stands, the direction it leaves in, and its terminal.
///
/// `at` is an [`At`], so the diagram's `Position` is spelled by name in the conversion below: the
/// `into()` on a [`Pos`] resolves `description::Pos → monospace_core::Pos` and lands on
/// `Position::Absolute`, while a reference is built field by field. A reference may only be held by
/// a **connector's endpoint** — a `box` and a `line` keep their bare `Pos`, because a position
/// other than an endpoint's may not be a reference
/// ([ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md)) and the
/// types say so rather than the prose.
#[derive(Deserialize, Debug, Clone)]
struct Endpoint {
    at: At,
    leaving: Leaving,
    terminal: Terminal,
}

impl From<Endpoint> for DiagramEndpoint {
    fn from(endpoint: Endpoint) -> Self {
        DiagramEndpoint {
            at: match endpoint.at {
                At::Point(at) => Position::Absolute(at.into()),
                At::Reference {
                    shape,
                    anchor,
                    offset,
                } => Position::Reference(Reference {
                    id: ShapeId::new(shape),
                    anchor: anchor.into(),
                    offset: offset.into(),
                }),
            },
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
                  "from": { "at": { "kind": "point", "x": 0, "y": 0 }, "leaving": "right",
                            "terminal": { "kind": "glyph", "glyph": "ab" } },
                  "to": { "at": { "kind": "point", "x": 6, "y": 0 }, "leaving": "left",
                          "terminal": { "kind": "glyph", "glyph": ">" } },
                  "stroke": "light" }
            ]
        }"#;

        assert!(serde_json::from_str::<Description>(json).is_err());
    }

    /// One connector, with `TERMINAL` standing where the `from` endpoint's terminal goes. The three
    /// refusals below differ only in what is written there, which is what makes them the same test
    /// three times over.
    const ARROW: &str = r#""from": { "at": { "kind": "point", "x": 0, "y": 0 }, "leaving": "right", "terminal": TERMINAL },
                  "to": { "at": { "kind": "point", "x": 6, "y": 0 }, "leaving": "left",
                          "terminal": { "kind": "glyph", "glyph": ">" } }"#;

    /// The same connector with the field left out altogether.
    const ARROW_WITHOUT_A_TERMINAL: &str = r#""from": { "at": { "kind": "point", "x": 0, "y": 0 }, "leaving": "right" },
                  "to": { "at": { "kind": "point", "x": 6, "y": 0 }, "leaving": "left",
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
