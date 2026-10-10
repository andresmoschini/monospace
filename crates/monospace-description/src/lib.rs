//! The diagram description format: JSON deserialized into a `monospace_diagram::Diagram` and the
//! window it is drawn in.
//!
//! One call does the whole of it — [`parse`] takes the text and hands back both — and every type
//! behind that call is private to this crate. The public surface is three things: [`parse`],
//! [`Window`] and [`ParseError`]. Nothing here reaches a file, a terminal or a screen: a caller
//! that wants to read from disk reads the disk itself and hands the text over.
//!
//! **This crate is a layer above the diagram, and it knows nothing of the editing layer.** Both
//! hang off `monospace-diagram` and neither depends on the other; what would join them is a
//! convenience that opens a file into a session, and that belongs to whoever needs it rather than
//! to either layer. See [`docs/diagram-model.md`](../../../docs/diagram-model.md).
//!
//! # The envelope
//!
//! One file is one JSON object of three fields, and all three are required:
//!
//! ```json
//! {
//!   "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 20, "height": 10 } },
//!   "next_id": 3,
//!   "shapes": []
//! }
//! ```
//!
//! - **`canvas`** is the window drawn into and rendered from: an `origin` of `x` and `y` beside a
//!   `size` of `width` and `height`. It becomes a [`Window`], which is **temporary** — see that
//!   type.
//! - **`next_id`** is the ordinal the next shape added takes, which is where numbering **resumes**
//!   rather than a count of what was read: `1`, `7` and `9` under `next_id: 10` hands back `10`. It
//!   is a **nonzero** ordinal, because zero is not an identity; `"next_id": 0` is refused here
//!   rather than seeded with something no shape can carry.
//! - **`shapes`** is the figures in drawing order, empty or not, the last front-most and deciding a
//!   shared cell first. Each entry names a `kind` and carries the `id` its shape is held under.
//!
//! # What is not here
//!
//! **No writer.** The format is read and not written, and [`ParseError`] says nothing about what a
//! writer would owe a reader. **No file.** [`parse`] takes text rather than a path, so nothing in
//! this crate reaches a filesystem — which is also what keeps it portable by naming rather than by
//! anything it avoids.

use std::num::NonZeroU32;

use monospace_core::{Direction, Glyph, Orientation as CoreOrientation};
use monospace_diagram::{
    Anchor, Delta, Diagram, Endpoint as DiagramEndpoint, Position, Reference,
    Shape as DiagramShape, ShapeId,
};
use serde::{Deserialize, Deserializer};

/// Where a description says a diagram is drawn, as an origin and a size.
///
/// **This type is temporary and it is expected to leave the description.** A window is not part of
/// what a diagram *is* — see [`docs/diagram-model.md`](../../../docs/diagram-model.md), which holds
/// that the window belongs to the caller and that which part of a diagram to draw is the caller's
/// question in any case. An interactive application draws what fits the screen at the scroll
/// position it is at, and reads no window out of a file to do it.
///
/// It is here because the file still carries a `canvas` and the command-line application still
/// renders into one. It is named rather than returned as a bare `(Pos, Size)` so that the day it
/// goes, the thing that goes is one type with one name.
///
/// Two fields rather than a pair of its own: `monospace_core::Pos` and `monospace_core::Size`
/// already are that pair, and a second pair of coordinates would be a second thing to convert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    /// The window's top-left corner, which may be negative.
    pub origin: monospace_core::Pos,
    /// The window's extent in cells.
    pub size: monospace_core::Size,
}

/// A description that could not be read.
///
/// **The message is the one `serde_json` produced, unchanged.** A newtype rather than a
/// re-wrap: the value of this type is that `serde` does not appear in this crate's public API, not
/// that it says anything the underlying error did not. A caller prints it and the words are the
/// ones the format has always reported — a missing field named, a `kind` this build does not know
/// named beside the two it does, an identity written as free text and quoted back — which is what
/// the contract tests here and in `monospace-cli` assert on.
///
/// What would earn it more is a writer: a file whose `kind` this build does not know is a
/// different kind of problem from one that is spelled wrong, and only then is there something to add
/// that `serde` cannot say.
#[derive(Debug)]
pub struct ParseError(serde_json::Error);

impl std::fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

/// Reads a description and hands back the diagram it holds and the window it names.
///
/// The whole of this crate's public surface for reading, in one call: there is no value to build,
/// hold or convert afterwards, so no method can be called in the wrong order and none leaves a
/// diagram half-converted.
///
/// # Errors
///
/// [`ParseError`] when the text is not a description this crate reads — a field left out, a `kind`
/// it does not name, an identity written as free text, a `next_id` of zero. The message names the
/// field it was refused on and is the one `serde_json` produced.
///
/// # Examples
///
/// ```
/// # use monospace_description::parse;
/// let (diagram, window) = parse(
///     r#"{
///         "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
///         "next_id": 2,
///         "shapes": [ { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
///                       "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
///     }"#,
/// )
/// .expect("a well-formed description");
///
/// assert_eq!(window.size.width, 4);
/// ```
pub fn parse(json: &str) -> Result<(Diagram, Window), ParseError> {
    let description: Description = serde_json::from_str(json).map_err(ParseError)?;
    let window = Window {
        origin: description.canvas.origin.into(),
        size: description.canvas.size.into(),
    };
    Ok((description.into_diagram(), window))
}

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

/// The ordinal the three identity fields share, and the reason each names itself on a refusal.
///
/// An identity is a number greater than zero — `1`, not `"#1"` — so a file spelling it any other
/// way is a **data error**, and the message says which field was wrong rather than leaving a reader
/// to work back from a position: `serde` reports `invalid type: string "#1", expected a nonzero
/// u32`, which names neither `id` nor `shape` nor `next_id`, and the three are the whole of what a
/// caller has to go on.
///
/// The name is the field's, written beside the field rather than read from serde's error, because
/// serde cannot know it: the same [`NonZeroU32`] is deserialized from three different keys. The
/// underlying message is carried through whole, so what the file actually said survives next to the
/// field it was said in.
fn ordinal<'de, D>(field: &'static str, deserializer: D) -> Result<NonZeroU32, D::Error>
where
    D: Deserializer<'de>,
{
    NonZeroU32::deserialize(deserializer)
        .map_err(|error| serde::de::Error::custom(format!("`{field}` is an ordinal: {error}")))
}

/// The `id` of an entry in `shapes`. A thin wrapper over [`ordinal`] so that the name in the message
/// is the field's own, beside the field it belongs to.
fn id<'de, D>(deserializer: D) -> Result<NonZeroU32, D::Error>
where
    D: Deserializer<'de>,
{
    ordinal("id", deserializer)
}

/// The `next_id` a diagram's numbering resumes from.
fn next_id<'de, D>(deserializer: D) -> Result<NonZeroU32, D::Error>
where
    D: Deserializer<'de>,
{
    ordinal("next_id", deserializer)
}

/// The `shape` a reference names.
fn shape<'de, D>(deserializer: D) -> Result<NonZeroU32, D::Error>
where
    D: Deserializer<'de>,
{
    ordinal("shape", deserializer)
}

/// The window a diagram is drawn on and rendered from: the `origin` is its top left corner and the
/// `size` its width and height. Both are required, so a file leaving either out is refused by name.
#[derive(Deserialize, Debug)]
struct Canvas {
    origin: Pos,
    size: Size,
}

/// What an endpoint's `terminal` is on the wire, tagged by `kind`: one chosen glyph or one arm.
/// An unrecognized `kind` is reported by name, and the message names the two accepted.
///
/// Internally tagged rather than externally, because `deserialize_glyph` needs a _named_ field to
/// sit on and the external form has none — `Glyph(Glyph)` does not compile, since `Glyph` derives
/// no `Deserialize`. A field an `arm` does not know is ignored, as everywhere else in this format.
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
/// run succeeds. **The silence is the answer, on purpose** — a reference naming a shape that is not
/// there yet is an ordinary state of a diagram being built rather than an invalid one, so the only
/// thing this layer owes anybody is to refuse to draw something it cannot place. What it pays for
/// that is that a diagram which drew nothing and a diagram whose every reference is broken look
/// identical, and there is nothing to ask about the difference.
///
/// `Point` is a **newtype** over the file's own [`Pos`] rather than a struct variant with `x` and
/// `y` written out, which keeps one `Pos` in this file instead of a second pair of coordinates to
/// convert. Both spellings were measured and read `{"kind": "point", "x": 1, "y": 1}` and refused
/// `{"x": 1, "y": 1}` identically, so the newtype is taken for the one `Pos` it saves. It carries a
/// trap worth knowing: **an internally tagged newtype variant deserializes and does not serialize**.
/// Nothing here is broken, because the format is read and never written — but a caller who later
/// wanted a `Serialize` on this type would have to widen the variant, and this is where to find out
/// why.
///
/// The `Reference` variant's two gap fields are **two spellings of one gap**, and which one a file
/// reaches for is the caller's: `offset` is the screen axes whatever side the anchor names, and
/// `out` is the side's own words, so `"anchor": "bottom", "out": 1` is one cell below the border
/// without the file knowing that "below" is `dy: 1`. Both are optional and absent is zero, so a
/// reference on the side itself is three fields; `out` is added to `offset` rather than replacing it,
/// and a file may write either or both. **Only the two signs are decided here and they are decided
/// by the diagram crate, not by this one** — `Reference::new` turns `out` into an offset and never
/// sees the anchor twice, which is what keeps four files spelling four different gaps from becoming
/// four copies of the same arithmetic.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum At {
    Point(Pos),
    Reference {
        #[serde(deserialize_with = "shape")]
        shape: NonZeroU32,
        anchor: AnchorDescription,
        #[serde(default)]
        offset: OffsetDescription,
        #[serde(default)]
        out: i32,
    },
}

/// One endpoint of a connector: where it stands, the direction it leaves in, and its terminal.
///
/// `at` is an [`At`], so the diagram's `Position` is spelled by name in the conversion below: the
/// `into()` on a [`Pos`] resolves `description::Pos → monospace_core::Pos` and lands on
/// `Position::Absolute`, while a reference is built field by field. A reference may only be held by
/// a **connector's endpoint** — a `box` and a `line` keep their bare `Pos` — and the types say so
/// rather than the prose. The restriction is what keeps a chain of references one link long: a
/// reference names a box or a line, both positioned absolutely, or a connector, which answers no
/// side at all, so nothing recurses and no cycle can be built.
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
                    out,
                } => Position::Reference(Reference::new(
                    ShapeId::new(shape),
                    anchor.into(),
                    offset.into(),
                    out,
                )),
            },
            leaving: endpoint.leaving.into(),
            terminal: endpoint.terminal.into(),
        }
    }
}

/// One entry in a `Description`'s `shapes` array, tagged by `kind`. An unrecognized `kind` is
/// reported by name, since this enum is internally tagged.
///
/// Every variant carries an `id`: the identity its shape is held under, and the one a `reference`
/// names. It is required, so a file leaving it out is refused **by name**, exactly as `canvas`,
/// `shapes`, `leaving`, `terminal` and `at`'s `kind` already are. It sits **first** in every
/// variant because a variant's fields are read in declaration order, and `id` immediately after
/// `kind` on the wire is where all the descriptions in this repository already put it.
///
/// **The identity is an ordinal rather than any text.** One number greater than zero, deserialized
/// by [`id`] so that `"#1"` is refused by name rather than read as a string nobody can resolve,
/// and the same spelling the `next_id` beside it has always used.
///
/// The identity is **not** checked for uniqueness, and two entries may carry one: both are read,
/// the first is what every change and every reference finds, and the second is reachable by no
/// identity until the first is removed. That cost is accepted, and it is why the model's "unique
/// within that diagram" is amended in the `docs` commit rather than enforced here.
#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum ShapeDescription {
    Box {
        #[serde(deserialize_with = "id")]
        id: NonZeroU32,
        at: Pos,
        size: Size,
        stroke: String,
        #[serde(default, deserialize_with = "deserialize_optional_glyph")]
        fill: Option<Glyph>,
    },
    Line {
        #[serde(deserialize_with = "id")]
        id: NonZeroU32,
        at: Pos,
        len: u32,
        orientation: Orientation,
        stroke: String,
    },
    Connector {
        #[serde(deserialize_with = "id")]
        id: NonZeroU32,
        from: Endpoint,
        to: Endpoint,
        stroke: String,
    },
}

impl ShapeDescription {
    /// The identity this entry's shape is held under, read before the shape is converted.
    ///
    /// One method rather than three because the field is on every variant and the conversion below
    /// drops it: the figure carries no identity, and it is the diagram that holds the one the file
    /// wrote.
    fn id(&self) -> ShapeId {
        match self {
            ShapeDescription::Box { id, .. }
            | ShapeDescription::Line { id, .. }
            | ShapeDescription::Connector { id, .. } => ShapeId::new(*id),
        }
    }
}

impl From<ShapeDescription> for DiagramShape {
    /// Converts this description into the matching `monospace_diagram` shape, dropping no
    /// parameter. The `id` goes with the diagram rather than with the figure: nothing about a
    /// returned shape says where it sits, which is the arrangement here and stays it.
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
///
/// Private because nothing outside this crate needs it: [`parse`] reads one and hands back what it
/// holds, and a caller that could hold a `Description` would be able to look at a format this
/// crate does not promise to keep.
#[derive(Deserialize, Debug)]
struct Description {
    canvas: Canvas,
    /// The ordinal the next `add` takes: a number rather than a container holding one, because it
    /// is one number. Required, so a file leaving it out is refused by name, the way `canvas`
    /// and `shapes` already are.
    ///
    /// **It is trusted, not checked, beyond being nonzero.** A stale value — an entry removed, a
    /// `2` deleted — hands back an identity already in use, and the shape that arrives is one
    /// nobody can name. That is D6's accepted cost, and the repair is one line in
    /// `monospace-diagram`; nothing in this format checks it. What **is** checked is the zero: zero
    /// is not an identity, so `"next_id": 0` is a data error naming the field rather than a counter
    /// seeded with something the diagram could not carry.
    #[serde(deserialize_with = "next_id")]
    next_id: NonZeroU32,
    /// The figures, in the order they are drawn. Required, and the order is load-bearing: the array
    /// order is the drawing order, so the last entry is front-most and decides a shared cell first.
    shapes: Vec<ShapeDescription>,
}

impl Description {
    /// Builds a diagram from `shapes`, in order, under the identities the file wrote and with its
    /// numbering resuming where it says.
    ///
    /// **The order is still the order.** The array order remains the drawing order, so every
    /// picture in the repository comes out byte for byte what it did — that is the claim the whole
    /// mechanical change rests on, and it was measured rather than argued.
    ///
    /// A shape a connector names before the entry carrying it is written still resolves, because
    /// resolution happens at draw time and the whole diagram exists by then. That edge case needs
    /// no code here; it is what a completed vector of placements means.
    fn into_diagram(self) -> Diagram {
        let mut diagram = Diagram::numbered_from(self.next_id);
        for shape in self.shapes {
            diagram.add_under(shape.id(), shape.into());
        }
        diagram
    }
}

#[cfg(test)]
mod tests {
    use monospace_core::{Buffer, GlyphCatalog};

    use super::{Description, ParseError, Window, parse};

    /// The error a description that must not read hands back.
    ///
    /// **A helper rather than `expect_err`**, which would need `Debug` on the pair `parse` returns:
    /// `Diagram` derives none, and deriving it here would mean the diagram crate grew a `Debug`
    /// every consumer inherits for the sake of one test module. `Result::err` does not.
    fn refusal(json: &str) -> ParseError {
        parse(json)
            .err()
            .expect("a description this format refuses must fail")
    }

    /// The rendering of a parsed description, drawn into the window it names.
    ///
    /// **The core's own Light table, and no glyph set beside it.** Every shape here is drawn in
    /// `light`, which is what Light covers, and the two tests that need a picture are about what a
    /// reference resolves to rather than about how a stroke draws — so a second catalog would add a
    /// dependency to answer nothing they ask. Measured rather than argued: the same three shapes
    /// drawn in `double` come out of this catalog as an empty buffer.
    fn render_once(description: Description) -> String {
        let window = Window {
            origin: description.canvas.origin.into(),
            size: description.canvas.size.into(),
        };
        let diagram = description.into_diagram();

        let mut buffer = Buffer::new(window.origin, window.size);
        diagram.draw(&mut buffer);
        monospace_core::render(&buffer, &GlyphCatalog::light(), window.origin, window.size)
    }

    /// An unrecognized `kind` fails to deserialize and names the unrecognized value.
    #[test]
    fn an_unrecognized_kind_fails_to_deserialize_and_names_it() {
        let json = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 1,
            "shapes": [ { "kind": "triangle" } ]
        }"#;

        let error = refusal(json);

        assert!(error.to_string().contains("triangle"), "{error}");
    }

    /// A `fill` of more than one grapheme cluster fails to deserialize.
    #[test]
    fn a_multi_grapheme_fill_fails_to_deserialize() {
        let json = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 2,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "ab" }
            ]
        }"#;

        assert!(parse(json).is_err());
    }

    /// A `terminal`'s glyph of more than one grapheme cluster fails to deserialize. The grapheme
    /// check sits on the named field inside the tagged object, which is the only reason the wire
    /// form is internally tagged.
    #[test]
    fn a_multi_grapheme_terminal_glyph_fails_to_deserialize() {
        let json = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 1 } },
            "next_id": 2,
            "shapes": [
                { "kind": "connector", "id": 1,
                  "from": { "at": { "kind": "point", "x": 0, "y": 0 }, "leaving": "right",
                            "terminal": { "kind": "glyph", "glyph": "ab" } },
                  "to": { "at": { "kind": "point", "x": 6, "y": 0 }, "leaving": "left",
                          "terminal": { "kind": "glyph", "glyph": ">" } },
                  "stroke": "light" }
            ]
        }"#;

        assert!(parse(json).is_err());
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
            "next_id": 2,
            "shapes": [
                {{ "kind": "connector", "id": 1,
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
        refusal(&json).to_string()
    }

    /// A `kind` the model has not named is refused by name, and the message names the two that are
    /// accepted — so the value is never read as one of them.
    #[test]
    fn an_unrecognized_terminal_kind_names_it_and_the_two_that_are_accepted() {
        let error = error_for(r#"{ "kind": "dot" }"#);

        assert!(
            error.starts_with("unknown variant `dot`, expected `glyph` or `arm`"),
            "{error}"
        );
    }

    /// A `glyph` that is not exactly one grapheme cluster is rejected with the message the old
    /// `head` produced, byte for byte.
    #[test]
    fn a_multi_grapheme_terminal_glyph_is_rejected_with_the_message_the_old_head_produced() {
        let error = error_for(r#"{ "kind": "glyph", "glyph": "ab" }"#);

        assert!(
            error.starts_with("\"ab\" is not exactly one grapheme cluster"),
            "{error}"
        );
    }

    /// The field is required for both values, so a file that omits it is refused rather than read
    /// as a connector with no terminal at either end. A terminal's presence never decides what a
    /// description means.
    #[test]
    fn an_omitted_terminal_field_is_refused_by_name() {
        let json = description_of(ARROW_WITHOUT_A_TERMINAL);

        let error = refusal(&json).to_string();

        assert!(error.starts_with("missing field `terminal`"), "{error}");
    }

    /// The externally tagged spelling was measured too: every value of `terminal` is an object
    /// tagged by `kind`, so a bare string is refused on purpose rather than read as an arm.
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
        refusal(&json).to_string()
    }

    /// A point written without a tag is **refused**, not read as one of the two.
    ///
    /// The position that follows the message is `serde_json`'s and moves with the bytes, which is
    /// why what is compared is the message. Before the tag landed this file rendered and exited
    /// successfully — the silence the tag is there to end, measured both ways.
    #[test]
    fn an_untagged_at_is_refused_by_name() {
        let error = error_for_an_at(r#"{ "x": 0, "y": 0 }"#);

        assert!(error.starts_with("missing field `kind`"), "{error}");
    }

    /// A `kind` the union has no arm for is refused by name, **and the message names the two it
    /// does have** — so the value is never read as one of them.
    ///
    /// The same rule this format's other `kind` fields have always obeyed, and the message is the
    /// whole of what choosing the tag bought over an untagged union, which would have answered the
    /// same file with `data did not match any variant of untagged enum At`.
    #[test]
    fn an_unknown_at_kind_names_it_and_the_two_that_are_accepted() {
        let error = error_for_an_at(r#"{ "kind": "arrows", "x": 0, "y": 0 }"#);

        assert!(
            error.starts_with("unknown variant `arrows`, expected `point` or `reference`"),
            "{error}"
        );
    }

    /// A `point` written beside a stray `shape` is **accepted**, and the `shape` is dropped in
    /// silence.
    ///
    /// This is the format's existing behavior rather than anything this slice adds, and it is the
    /// cost of an internal tag rather than a defect: the tag is the rule, and a field beside it is
    /// an unknown field exactly as a description carrying `mode` was before the format dropped it.
    /// Pinned because the alternative — refusing a file for a harmless extra key — is the change a
    /// later slice would make silently, and this is where the present answer is written down.
    #[test]
    fn a_stray_field_beside_an_at_is_dropped_in_silence() {
        let json = description_of(
            &ARROW_WITH_AN_AT.replace("AT", r#"{ "kind": "point", "x": 1, "y": 1, "shape": 5 }"#),
        );

        assert!(
            parse(&json).is_ok(),
            "a stray field beside a tagged `at` must not refuse the file"
        );
    }

    /// A reference carrying **no** `offset` at all reads, and is a reference standing on the side
    /// itself — which is the case the field's being optional exists for.
    ///
    /// Without `#[serde(default)]` this would be a missing-field error on `offset`, and every
    /// reference meaning "on the border" would have to spell two zeros to say nothing — the reason
    /// the field is optional and absent zero.
    #[test]
    fn a_reference_with_no_offset_is_read_as_a_reference_on_the_side_itself() {
        let json = description_of(&ARROW_WITH_AN_AT.replace(
            "AT",
            r#"{ "kind": "reference", "shape": 1, "anchor": "right" }"#,
        ));

        assert!(
            parse(&json).is_ok(),
            "a reference without an `offset` must read"
        );
    }

    /// An unknown key **inside** a reference is dropped in silence — which is why a file could not
    /// spell `out` before the field existed, and why `"out": 1` read by a build without it draws the
    /// point on the border and says nothing at all.
    ///
    /// One level in from `a_stray_field_beside_an_at_is_dropped_in_silence`: that one puts a field
    /// beside the tag and this one beside a reference's own fields, and both are the same cost of an
    /// internal tag. **`along` is the key this is really about** — the second spelling of the amount
    /// that runs along the side, declined because the offset already says it — so a file naming it
    /// reads, and reads as a reference that says nothing about it.
    #[test]
    fn an_unknown_key_inside_a_reference_is_dropped_in_silence() {
        let json = description_of(&ARROW_WITH_AN_AT.replace(
            "AT",
            r#"{ "kind": "reference", "shape": 1, "anchor": "right", "along": 2 }"#,
        ));

        assert!(
            parse(&json).is_ok(),
            "an unknown key beside a reference's own fields must not refuse the file"
        );
    }

    /// A reference follows the identity, **both directions**.
    ///
    /// Two descriptions over the same window, the same two boxes and the same connector, differing
    /// only in the order their entries are listed in. The connector naming the **box** draws the
    /// same buffer whichever order the entries are in; the connector naming the **place** draws two
    /// different pictures, which is what those files mean today.
    ///
    /// Both directions, because a reader that made the identity win in one and not in the other
    /// would pass a single test — and the second half is the one that is easy to leave out, since
    /// the first half is what the slice is for.
    ///
    /// **The identities are ordinals rather than the free text this test used to give them**,
    /// which is the whole cost the format change names: the two halves are still about the same
    /// thing — an identity is not a place — and they say it with `1` and `2`.
    #[test]
    fn a_reference_follows_the_identity_and_not_the_place_where_the_entry_is_written() {
        // Two boxes at **fixed** positions, far enough apart that the route to either is a
        // different drawing: one at `{0, 3}` and one at `{11, 3}`, and a connector falling from
        // `{7, 0}` onto the **top** of whichever box it is told to name. The two boxes do not
        // touch, which is what makes the two routes distinguishable at all — measured, because a
        // pair that shares cells composes to the same picture whichever way round it is built.
        //
        // `left` and `right` are the ordinals given to the box at `{0, 3}` and the one at `{11, 3}`
        // respectively, and `right_listed_first` says which of them the `shapes` array lists
        // first. Nothing else changes between the four descriptions below, so the route can only
        // move if the reader is following the place rather than the identity.
        let description_with =
            |left: u32, right: u32, right_listed_first: bool, connector_names| {
                let left_box = format!(
                    r#"{{ "kind": "box", "id": {left}, "at": {{ "x": 0, "y": 3 }},
                  "size": {{ "width": 4, "height": 3 }}, "stroke": "light" }}"#
                );
                let right_box = format!(
                    r#"{{ "kind": "box", "id": {right}, "at": {{ "x": 11, "y": 3 }},
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
                {{ "kind": "connector", "id": 3,
                  "from": {{ "at": {{ "kind": "point", "x": 7, "y": 0 }}, "leaving": "down",
                             "terminal": {{ "kind": "glyph", "glyph": "▼" }} }},
                  "to": {{ "at": {{ "kind": "reference", "shape": {connector_names},
                                      "anchor": "top" }},
                          "leaving": "up",
                          "terminal": {{ "kind": "arm" }} }},
                  "stroke": "light" }}
            ]
        }}"#
                )
            };
        // `serde_json` and not [`parse`]: `render_once` takes the `Description` so that it can read
        // the window off it, and `parse` hands back the diagram and window already built.
        let read = |json: String| serde_json::from_str::<Description>(&json).expect("well-formed");

        // Naming the **box**: the box at `{11, 3}` is `2` in both files and is listed first in one
        // and second in the other, so both draw the same picture.
        let right_listed_first = render_once(read(description_with(1, 2, true, 2)));
        let right_listed_second = render_once(read(description_with(1, 2, false, 2)));
        assert_eq!(
            right_listed_first, right_listed_second,
            "a reference naming a shape must not move when the entries are listed in another order"
        );

        // Naming the **place**: `2` is the identity the second entry carries in each file, and the
        // two files put a different box there, so the connector lands on a different box in each.
        // That is what those files mean today, and the half that says the identity won in one
        // direction and not in the other — a reader that made it win in both would have failed the
        // first half.
        let place_right_first = render_once(read(description_with(2, 1, true, 2)));
        let place_right_second = render_once(read(description_with(1, 2, false, 2)));
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
        let without_next_id = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "shapes": [ { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                          "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
        }"#;
        let without_an_id = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 2,
            "shapes": [ { "kind": "box", "at": { "x": 0, "y": 0 },
                          "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
        }"#;

        for (json, named) in [(without_next_id, "next_id"), (without_an_id, "id")] {
            let error = refusal(json);
            assert!(
                error
                    .to_string()
                    .contains(&format!("missing field `{named}`")),
                "expected `missing field `{named}``, got: {error}"
            );
        }
    }

    /// Free text is **refused**, and the message names the field it was refused on.
    ///
    /// **Both fields, and both spellings**, because the three identity fields share one
    /// deserializer and the only thing that differs between them is the name it puts in the
    /// message. `serde`'s own wording — `invalid type: string "#1", expected a nonzero u32` — names
    /// neither, so a reader handed it has nothing to work back from; and a wrapper that named
    /// `id` for all three would send a reader after the wrong field on the other two.
    ///
    /// **The spelling is quoted and not summarized**, so a file that wrote `"arrow"` says so rather
    /// than being told it wrote something else. The `next_id` half is
    /// `a_next_id_of_zero_is_refused_and_names_the_field_it_was_refused_on` beside this one.
    #[test]
    fn an_identity_written_as_free_text_is_refused_and_names_the_field() {
        let a_string_id = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 2,
            "shapes": [ { "kind": "box", "id": "arrow", "at": { "x": 0, "y": 0 },
                          "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
        }"#;
        let a_string_shape = &description_of(&ARROW_WITH_AN_AT.replace(
            "AT",
            r##"{ "kind": "reference", "shape": "#1", "anchor": "right" }"##,
        ));

        for (json, named, said) in [
            (a_string_id, "id", "\"arrow\""),
            (a_string_shape, "shape", "\"#1\""),
        ] {
            let error = refusal(json).to_string();
            assert!(
                error.starts_with(&format!("`{named}` is an ordinal: ")),
                "expected `{named}` to be named first, got: {error}"
            );
            assert!(
                error.contains(said),
                "expected the message to quote what the file said, `{said}`, got: {error}"
            );
        }
    }

    /// A `next_id` of zero is refused and named, because zero is not an identity.
    ///
    /// **`0` is the one value the whole format cannot express**, so a counter that took it would be
    /// a counter seeded with nothing the diagram could carry — and the old reader held it and issued
    /// `"#0"` from it. The refusal names `next_id` for the same reason the other two name theirs.
    #[test]
    fn a_next_id_of_zero_is_refused_and_names_the_field_it_was_refused_on() {
        let json = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 0,
            "shapes": [ { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                          "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
        }"#;

        let error = refusal(json).to_string();

        assert!(
            error.starts_with("`next_id` is an ordinal: "),
            "expected `next_id` to be named first, got: {error}"
        );
        assert!(
            error.contains('0'),
            "expected the message to quote the zero it was handed, got: {error}"
        );
    }

    /// An ordinal is read everywhere the format names a shape, and the file still draws.
    ///
    /// **The positive case beside the two refusals**, because a reader that refused everything
    /// would pass both of those. The same three fields are here written as `1` and `2`, and the
    /// connector's reference resolves through the ordinal the box carries: **a connector answers no
    /// anchor**, so this draws the connector rather than the window an unresolvable reference leaves,
    /// which is what makes the refusal above and this the same field read two ways.
    #[test]
    fn an_ordinal_is_read_where_the_format_names_a_shape() {
        let json = r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 3 } },
            "next_id": 3,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                  "size": { "width": 4, "height": 3 }, "stroke": "light" },
                { "kind": "connector", "id": 2,
                  "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" },
                            "leaving": "right", "terminal": { "kind": "glyph", "glyph": ">" } },
                  "to": { "at": { "kind": "point", "x": 7, "y": 1 }, "leaving": "left",
                          "terminal": { "kind": "arm" } },
                  "stroke": "light" }
            ]
        }"#;

        let picture = render_once(
            serde_json::from_str::<Description>(json).expect("an ordinal is an ordinal"),
        );

        assert!(
            picture.contains('>'),
            "the connector must draw, so the reference resolved through the ordinal: {picture:?}"
        );
    }
}
