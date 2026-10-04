//! The diagram description format: JSON types deserialized from a file and converted into a
//! `monospace_diagram::Diagram` and the `monospace_core::Buffer` its canvas describes.
//!
//! Every type here is private to `monospace-cli` and exists only for this conversion. This module
//! is where the format is documented, so it describes the envelope as well as each type.
//!
//! The envelope is three keys, and all three are required — a file leaving one out is refused by
//! name:
//!
//! ```json
//! {
//!   "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 20, "height": 7 } },
//!   "next_id": 7,
//!   "shapes": [ { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 },
//!                "size": { "width": 4, "height": 3 }, "stroke": "light", "fill": "░" } ]
//! }
//! ```
//!
//! `shapes` is the drawing order, back to front. Three kinds, each tagged by `kind` and each
//! carrying an `id` that names it: `box` (`at`, `size`, `stroke`, optional `fill`), `line` (`at`,
//! `len`, `orientation`, `stroke`) and `connector` (`from`, `to`, `stroke`), whose two endpoints
//! are [`Endpoint`]. `at` is [`At`] and `terminal` is [`Terminal`], both internally tagged, so the
//! wire form names a `kind` on each rather than nesting one shape's spelling inside another's.
//!
//! Every further rule is on the type that owns it: what `stroke` may be spelled, which glyph counts
//! as one, what a `reference` that resolves to nothing means, and why the order is the order.

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
/// else wrong with the file.
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

/// What an endpoint's `terminal` is on the wire, tagged by `kind`: one chosen glyph or one
/// arm. An unrecognized `kind` is reported by name, and the message names the two accepted.
///
/// Internally tagged rather than externally, because `deserialize_glyph` needs a _named_ field to
/// sit on and the external form has none — `Glyph(Glyph)` does not compile, since `Glyph` derives
/// no `Deserialize`. A field an `arm` does not know is ignored, as everywhere else
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
/// another figure.
///
/// **The tag is the rule, not a sentence.** A file cannot say both, cannot say neither, and cannot
/// say one by mistake — all three are compile-time facts about this enum rather than conventions a
/// reader has to remember. The three spellings were measured against this crate's own `serde`
/// rather than argued: an untagged union reads every existing description and
/// answers anything wrong with `data did not match any variant of untagged enum At`, and a sibling
/// `reference` beside an optional `at` refuses nothing at all.
///
/// A reference to a shape the diagram does not hold, or to an anchor that kind does not answer, is
/// **not** an error here or anywhere downstream: the figure holding it is simply not drawn and the
/// run succeeds. That is a cost this format accepts by design.
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
/// other than an endpoint's may not be a reference, and the
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

/// One entry in a `Description`'s `shapes` array, tagged by `kind`. An unrecognized
/// `kind` is reported by name, since this enum is internally tagged.
///
/// Every variant carries an `id`: the identity its shape is held under, and the one a `reference`
/// names. It is required, so a file leaving it out is refused **by name**, exactly as `canvas`,
/// `shapes`, `leaving`, `terminal` and `at`'s `kind` already are. It sits **first** in every
/// variant because a variant's fields are read in declaration order, and `id` immediately after
/// `kind` on the wire is where all the descriptions in this repository already put it (Q3).
///
/// The identity is **not** checked for uniqueness, and two entries may carry one: both are read,
/// the first is what every change and every reference finds, and the second is reachable by no
/// identity until the first is removed. That is an accepted cost, and it is why the model's
/// "unique within that diagram" is a model document statement rather than something enforced here.
#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum ShapeDescription {
    Box {
        id: String,
        at: Pos,
        size: Size,
        stroke: String,
        #[serde(default, deserialize_with = "deserialize_optional_glyph")]
        fill: Option<Glyph>,
    },
    Line {
        id: String,
        at: Pos,
        len: u32,
        orientation: Orientation,
        stroke: String,
    },
    Connector {
        id: String,
        from: Endpoint,
        to: Endpoint,
        stroke: String,
    },
}

impl ShapeDescription {
    /// The identity this entry's shape is held under, read before the shape is converted.
    ///
    /// One method rather than three because the field is on every variant and the conversion below
    /// drops it: the figure carries no name, and it is the diagram that holds the one the file
    /// wrote.
    fn id(&self) -> String {
        match self {
            ShapeDescription::Box { id, .. }
            | ShapeDescription::Line { id, .. }
            | ShapeDescription::Connector { id, .. } => id.clone(),
        }
    }
}

impl From<ShapeDescription> for DiagramShape {
    /// Converts this description into the matching `monospace_diagram` shape, dropping no
    /// parameter. The `id` goes with the diagram rather than with the figure:
    /// nothing about a returned shape says where it sits, which is 081's arrangement and stays it.
    fn from(description: ShapeDescription) -> Self {
        match description {
            ShapeDescription::Box {
                at,
                size,
                stroke,
                fill,
                ..
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
                ..
            } => DiagramShape::Line {
                at: at.into(),
                len,
                orientation: orientation.into(),
                stroke: stroke.as_str().into(),
            },
            ShapeDescription::Connector {
                from, to, stroke, ..
            } => DiagramShape::Connector {
                from: from.into(),
                to: to.into(),
                stroke: stroke.as_str().into(),
            },
        }
    }
}

/// The whole of one description file: a canvas, the ordinal the next shape takes, and an ordered
/// list of shapes.
#[derive(Deserialize, Debug)]
pub struct Description {
    canvas: Canvas,
    /// The ordinal the next `add` takes: a number rather than a container holding one, because it
    /// is one number (Q3). Required, so a file leaving it out is refused by name, the way `canvas`
    /// and `shapes` already are.
    ///
    /// **It is trusted, not checked.** A stale value — an entry renamed, a `#2` deleted — hands back
    /// an identity already in use, and the shape that arrives is one nobody can name. That is D2's
    /// accepted cost, and the repair is one line in `monospace-diagram`; nothing in this format
    /// checks it.
    next_id: u32,
    shapes: Vec<ShapeDescription>,
}

impl Description {
    /// The canvas's origin and size, converted to `monospace_core` types.
    pub(crate) fn window(&self) -> (monospace_core::Pos, monospace_core::Size) {
        (self.canvas.origin.into(), self.canvas.size.into())
    }

    /// Builds a diagram from `shapes`, in order, under the identities the file wrote and with its
    /// numbering resuming where it says.
    ///
    /// **The order is still the order.** The array order remains the drawing order, so every
    /// picture in the repository comes out byte for byte what it did — that is the claim the whole
    /// mechanical change rests on, and it was measured rather than argued.
    ///
    /// A name a connector names before the entry carrying it is written still resolves, because
    /// resolution happens at draw time and the whole diagram exists by then. That edge case needs
    /// no code here; it is what a completed vector of placements means.
    pub(crate) fn into_diagram(self) -> Diagram {
        let mut diagram = Diagram::numbered_from(self.next_id);
        for shape in self.shapes {
            diagram.add_under(ShapeId::new(shape.id()), shape.into());
        }
        diagram
    }
}

#[cfg(test)]
mod tests {
    use super::Description;
    use crate::render_once;

    /// An unrecognized `kind` fails to deserialize and names the unrecognized value.
    #[test]
    fn an_unrecognized_kind_fails_to_deserialize_and_names_it() {
        let json = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 1,
            "shapes": [ { "kind": "triangle" } ]
        }"#;

        let error =
            serde_json::from_str::<Description>(json).expect_err("unrecognized kind must fail");

        assert!(error.to_string().contains("triangle"), "{error}");
    }

    /// A `fill` of more than one grapheme cluster fails to deserialize.
    #[test]
    fn a_multi_grapheme_fill_fails_to_deserialize() {
        let json = r##"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 2,
            "shapes": [
                { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "ab" }
            ]
        }"##;

        assert!(serde_json::from_str::<Description>(json).is_err());
    }

    /// A `terminal`'s glyph of more than one grapheme cluster fails to
    /// deserialize. The grapheme check sits on the named field inside the tagged object, which is
    /// the only reason the wire form is internally tagged.
    #[test]
    fn a_multi_grapheme_terminal_glyph_fails_to_deserialize() {
        let json = r##"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 1 } },
            "next_id": 2,
            "shapes": [
                { "kind": "connector", "id": "#1",
                  "from": { "at": { "kind": "point", "x": 0, "y": 0 }, "leaving": "right",
                            "terminal": { "kind": "glyph", "glyph": "ab" } },
                  "to": { "at": { "kind": "point", "x": 6, "y": 0 }, "leaving": "left",
                          "terminal": { "kind": "glyph", "glyph": ">" } },
                  "stroke": "light" }
            ]
        }"##;

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
            r##"{{
            "canvas": {{ "origin": {{ "x": 0, "y": 0 }}, "size": {{ "width": 8, "height": 1 }} }},
            "next_id": 2,
            "shapes": [
                {{ "kind": "connector", "id": "#1",
                  {connector},
                  "stroke": "light" }}
            ]
        }}"##
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

    /// A `kind` the model has not named is refused by name, and the
    /// message names the two that are accepted — so the value is never read as one of them.
    #[test]
    fn an_unrecognized_terminal_kind_names_it_and_the_two_that_are_accepted() {
        let error = error_for(r#"{ "kind": "dot" }"#);

        assert!(
            error.starts_with("unknown variant `dot`, expected `glyph` or `arm`"),
            "{error}"
        );
    }

    /// A `glyph` that is not exactly one grapheme cluster is rejected
    /// with the message the old `head` produced, byte for byte.
    #[test]
    fn a_multi_grapheme_terminal_glyph_is_rejected_with_the_message_the_old_head_produced() {
        let error = error_for(r#"{ "kind": "glyph", "glyph": "ab" }"#);

        assert!(
            error.starts_with("\"ab\" is not exactly one grapheme cluster"),
            "{error}"
        );
    }

    /// The field is required for both values, so a file that omits it is refused
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

    /// The externally tagged spelling was measured: every value of
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

    /// The same connector as `ARROW`, with `AT` standing where the `from` endpoint's `at` goes, so
    /// the three cases below differ only in what an `at` says and the file around it is fixed.
    const ARROW_WITH_AN_AT: &str = r#""from": { "at": AT, "leaving": "right",
        "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "point", "x": 6, "y": 0 }, "leaving": "left",
              "terminal": { "kind": "glyph", "glyph": ">" } }"#;

    /// The message the binary reports for an endpoint whose `at` is `at`, read the way a reader
    /// reads it: the message, not the `serde_json` position that follows it.
    fn error_for_an_at(at: &str) -> String {
        let json = description_of(&ARROW_WITH_AN_AT.replace("AT", at));
        serde_json::from_str::<Description>(&json)
            .expect_err("an `at` this format does not accept must fail")
            .to_string()
    }

    /// A point written without a tag is **refused**, not read as
    /// one of the two.
    ///
    /// The position that follows the message is `serde_json`'s and moves with the bytes, which is
    /// why what is compared is the message. Before the tag landed this file rendered and exited
    /// successfully — the silence the tag is there to end, measured both ways.
    #[test]
    fn an_untagged_at_is_refused_by_name() {
        let error = error_for_an_at(r#"{ "x": 0, "y": 0 }"#);

        assert!(error.starts_with("missing field `kind`"), "{error}");
    }

    /// A `kind` the union has no arm for is refused by name, **and the
    /// message names the two it does have** — so the value is never read as one of them.
    ///
    /// The same rule §6's own `kind` fields have obeyed since 079, and the message is the whole of
    /// what choosing the tag bought over an untagged union, which would have answered the same file
    /// with `data did not match any variant of untagged enum At`.
    #[test]
    fn an_unknown_at_kind_names_it_and_the_two_that_are_accepted() {
        let error = error_for_an_at(r#"{ "kind": "arrows", "x": 0, "y": 0 }"#);

        assert!(
            error.starts_with("unknown variant `arrows`, expected `point` or `reference`"),
            "{error}"
        );
    }

    /// The third measurement of the tag's cost: a `point` written beside a stray `shape` is
    /// **accepted**, and the `shape` is dropped in silence.
    ///
    /// This is the format's existing behavior rather than anything this slice adds, and it is the
    /// cost of an internal tag rather than a defect: the tag is the rule, and a field beside it is
    /// an unknown field exactly as a description carrying `mode` was before 079 removed it. Pinned
    /// because the alternative — refusing a file for a harmless extra key — is the change a later
    /// slice would make silently, and this is where the present answer is written down.
    #[test]
    fn a_stray_field_beside_an_at_is_dropped_in_silence() {
        let json = description_of(&ARROW_WITH_AN_AT.replace(
            "AT",
            r##"{ "kind": "point", "x": 1, "y": 1, "shape": "#5" }"##,
        ));

        assert!(
            serde_json::from_str::<Description>(&json).is_ok(),
            "a stray field beside a tagged `at` must not refuse the file"
        );
    }

    /// The case `offset` being optional exists for: a reference
    /// carrying **no** `offset` at all reads, and is a reference standing on the side itself.
    ///
    /// Without `#[serde(default)]` this would be a missing-field error on `offset`, and every
    /// reference meaning "on the border" would have to spell two zeros to say nothing — the reason
    /// the contract calls the field optional and absent zero.
    #[test]
    fn a_reference_with_no_offset_is_read_as_a_reference_on_the_side_itself() {
        let json = description_of(&ARROW_WITH_AN_AT.replace(
            "AT",
            r##"{ "kind": "reference", "shape": "#1", "anchor": "right" }"##,
        ));

        assert!(
            serde_json::from_str::<Description>(&json).is_ok(),
            "a reference without an `offset` must read"
        );
    }

    /// A reference follows the name, **both directions**.
    ///
    /// Two descriptions over the same window, the same two boxes and the same connector, differing
    /// only in the order their entries are listed in. The connector naming the **box** draws the
    /// same buffer whichever order the entries are in; the connector naming the **place** draws two
    /// different pictures, which is what those files mean today.
    ///
    /// Both directions, because a reader that made the name win in one and not in the other would
    /// pass a single test — and the second half is the one that is easy to leave out, since the
    /// first half is what the slice is for.
    #[test]
    fn a_reference_follows_the_name_and_not_the_place_where_the_entry_is_written() {
        // Two boxes at **fixed** positions, far enough apart that the route to either is a
        // different drawing: one at `{0, 3}` and one at `{11, 3}`, and a connector falling from
        // `{7, 0}` onto the **top** of whichever box it is told to name. The two boxes do not
        // touch, which is what makes the two routes distinguishable at all — measured, because a
        // pair that shares cells composes to the same picture whichever way round it is built.
        //
        // `left` and `right` are the identity given to the box at `{0, 3}` and the one at `{11, 3}`
        // respectively, and `right_listed_first` says which of them the `shapes` array lists
        // first. Nothing else changes between the four descriptions below, so the route can only
        // move if the reader is following the place rather than the name.
        let description_with =
            |left: &str, right: &str, right_listed_first: bool, connector_names: &str| {
                let left_box = format!(
                    r#"{{ "kind": "box", "id": "{left}", "at": {{ "x": 0, "y": 3 }},
                  "size": {{ "width": 4, "height": 3 }}, "stroke": "light" }}"#
                );
                let right_box = format!(
                    r#"{{ "kind": "box", "id": "{right}", "at": {{ "x": 11, "y": 3 }},
                  "size": {{ "width": 4, "height": 3 }}, "stroke": "light" }}"#
                );
                let boxes = if right_listed_first {
                    format!("{right_box},\n                {left_box}")
                } else {
                    format!("{left_box},\n                {right_box}")
                };
                format!(
                    r#"{{
            "canvas": {{ "origin": {{ "x": 0, "y": 0 }}, "size": {{ "width": 16, "height": 6 }} }},
            "next_id": 3,
            "shapes": [
                {boxes},
                {{ "kind": "connector", "id": "arrow",
                  "from": {{ "at": {{ "kind": "point", "x": 7, "y": 0 }}, "leaving": "down",
                             "terminal": {{ "kind": "glyph", "glyph": "▼" }} }},
                  "to": {{ "at": {{ "kind": "reference", "shape": "{connector_names}",
                                      "anchor": "top" }},
                          "leaving": "up",
                          "terminal": {{ "kind": "arm" }} }},
                  "stroke": "light" }}
            ]
        }}"#
                )
            };
        let parse = |json: String| serde_json::from_str::<Description>(&json).expect("well-formed");

        // Naming the **box**: the box at `{11, 3}` is `right` in both files and is listed first in
        // one and second in the other, so both draw the same picture.
        let right_listed_first =
            render_once(parse(description_with("left", "right", true, "right")));
        let right_listed_second =
            render_once(parse(description_with("left", "right", false, "right")));
        assert_eq!(
            right_listed_first, right_listed_second,
            "a reference naming a shape must not move when the entries are listed in another order"
        );

        // Naming the **place**: `"#2"` is the second entry in each file, and the two files put a
        // different box there, so the connector lands on a different box in each. That is
        // what those files mean today, and the half that says the name won in one direction and
        // not in the other — a reader that made it win in both would have failed the first half.
        let place_right_first = render_once(parse(description_with("#2", "#1", true, "#2")));
        let place_right_second = render_once(parse(description_with("#1", "#2", false, "#2")));
        assert_ne!(
            place_right_first, place_right_second,
            "a reference naming a place must still follow the place it is written at"
        );
    }

    /// A missing identity is refused **by name**, exactly as `canvas`, `shapes`, `leaving`,
    /// `terminal` and `at`'s `kind` already are.
    ///
    /// Asserted on the message and not on the line and column that follow it, which are serde's
    /// and move with the bytes. The `display_with_line_column` half of the claim is the rest of the
    /// format's existing behavior and is already pinned by
    /// `an_omitted_terminal_field_is_refused_by_name`; this pins the two new names beside it.
    #[test]
    fn a_missing_identity_is_refused_by_name() {
        let without_next_id = r##"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "shapes": [ { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 },
                          "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
        }"##;
        let without_an_id = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 2,
            "shapes": [ { "kind": "box", "at": { "x": 0, "y": 0 },
                          "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
        }"#;

        for (json, named) in [(without_next_id, "next_id"), (without_an_id, "id")] {
            let error = serde_json::from_str::<Description>(json)
                .expect_err("a missing identity must fail");
            assert!(
                error
                    .to_string()
                    .contains(&format!("missing field `{named}`")),
                "expected `missing field `{named}``, got: {error}"
            );
        }
    }

    /// Free text reads. A description whose entries are named `right`, `left` and
    /// `arrow` draws **byte for byte** what the same description named `#1`, `#2` and `#3` draws.
    ///
    /// The format takes any string, and a format that insisted on an ordinal could not pass this
    /// test. The `next_id` is untouched by the substitution, which is what D1 chose over deriving
    /// an ordinal from the names: an ordinal derived from the names would have nothing to resume
    /// from, and a name is not a number to count.
    #[test]
    fn free_text_reads_and_draws_what_the_ordinal_named_description_draws() {
        let named = |ids: (&str, &str, &str)| {
            format!(
                r#"{{
            "canvas": {{ "origin": {{ "x": 0, "y": 0 }}, "size": {{ "width": 9, "height": 3 }} }},
            "next_id": 4,
            "shapes": [
                {{ "kind": "box", "id": "{}", "at": {{ "x": 0, "y": 0 }},
                  "size": {{ "width": 4, "height": 3 }}, "stroke": "light" }},
                {{ "kind": "box", "id": "{}", "at": {{ "x": 0, "y": 0 }},
                  "size": {{ "width": 4, "height": 3 }}, "stroke": "light" }},
                {{ "kind": "connector", "id": "{}",
                  "from": {{ "at": {{ "kind": "reference", "shape": "{}",
                                      "anchor": "right" }},
                             "leaving": "right",
                             "terminal": {{ "kind": "glyph", "glyph": ">" }} }},
                  "to": {{ "at": {{ "kind": "point", "x": 8, "y": 1 }}, "leaving": "left",
                           "terminal": {{ "kind": "glyph", "glyph": ">" }} }},
                  "stroke": "light" }}
            ]
        }}"#,
                ids.0, ids.1, ids.2, ids.0
            )
        };
        let parse = |json: String| serde_json::from_str::<Description>(&json).expect("well-formed");

        let ordinal_named = render_once(parse(named(("#1", "#2", "#3"))));
        let free_text = render_once(parse(named(("left", "right", "arrow"))));
        assert_eq!(
            ordinal_named, free_text,
            "free text must draw exactly what the ordinal-named description draws"
        );
        // The reference follows the name it was given, not the one in the other file.
        assert!(
            free_text.contains('>'),
            "the connector must draw, so the reference resolved through the name"
        );
    }
}
