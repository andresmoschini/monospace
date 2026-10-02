//! Minimal non-interactive command-line front end for Monospace.
//!
//! It holds no domain logic of its own: it converts its description format — documented in
//! `specs/079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md` — into a
//! `monospace_diagram::Diagram` and draws it.
//!
//! # Design notes
//!
//! **A file is rendered once; only the bare run demonstrates.** Given a path this prints one
//! picture and nothing else — no caption, and no shape moved forward. Given no argument it prints
//! the shipped demonstration the way spec 080 asks for it: **seven** captioned pictures — as
//! written, with the back-most shape moved one place toward the front, with that same shape
//! displaced, with that same shape taken out, with the arrow rehung from the box it already pointed
//! at and that box displaced, with the arrow itself displaced as well, and with the box the arrow
//! hangs from taken out — which leaves the arrow exactly where it stood.
//!
//! That split exists because the changes the bare run shows say something only about the
//! shipped demonstration, whose first two entries are two partially overlapping opaque boxes.
//! Applied to a file someone hands the binary they are a demonstration's assumptions imposed on
//! their description. It is also what lets `cargo xtask render` embed this output in a document
//! ([ADR-0052](../../../docs/decisions/0052-show-the-rendering.md)): a picture that goes into a
//! Markdown fence cannot arrive wrapped in prose.

mod description;

use std::process::ExitCode;

use description::Description;
use monospace_core::{Buffer, Direction, GlyphCatalog, Pos, Size, Terminal};
use monospace_diagram::{Anchor, Delta, Diagram, Endpoint, Position, Reference, Shape, ShapeId};

/// The shipped demonstration description, embedded at compile time so the no-argument run works
/// from any working directory and from a binary copied outside a checkout (FR-022, FR-023).
const DEMO: &str = include_str!("../assets/demo.json");

#[cfg(test)]
mod sweep;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let demonstrating = args.is_empty();
    let text = match args.as_slice() {
        [] => DEMO.to_owned(),
        [path] => match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                eprintln!("{path}: {error}");
                return ExitCode::FAILURE;
            }
        },
        _ => {
            eprintln!("usage: monospace-cli [path]");
            return ExitCode::FAILURE;
        }
    };

    let description = match serde_json::from_str::<Description>(&text) {
        Ok(description) => description,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    print!(
        "{}",
        if demonstrating {
            demonstrate(description)
        } else {
            render_once(description)
        }
    );
    ExitCode::SUCCESS
}

/// Renders `description` into its own window, as one picture and nothing else (FR-013, FR-014,
/// FR-015, FR-018, FR-019).
fn render_once(description: Description) -> String {
    let (origin, size) = description.window();
    let diagram = description.into_diagram();

    picture(&diagram, &glyph_catalog(), origin, size)
}

/// Draws `diagram` into a fresh window of `size` and renders it. Drawing changes nothing about the
/// diagram, which is what lets the demonstration draw one diagram four times and change it between
/// two of them.
fn picture(diagram: &Diagram, catalog: &GlyphCatalog, origin: Pos, size: Size) -> String {
    let mut buffer = Buffer::new(origin, size);
    diagram.draw(&mut buffer);
    monospace_core::render(&buffer, catalog, origin, size)
}

/// Renders `description` as written, then again with its first entry moved one place toward the
/// front, then again with that same entry displaced, then again with it taken out, then once with the
/// arrow rehung from the box it already pointed at and that box displaced, then once more with the
/// arrow itself displaced, and once more again with the box the arrow hangs from taken out. Each
/// picture is under a caption.
///
/// This is the shipped demonstration's output, and spec 080's FR-018 is what asks for the captions.
/// The changes it shows are meaningful only for that description, which is why a file the binary is
/// handed goes through [`render_once`] instead.
///
/// The first four pictures are about one figure, the entry the description lists first. The fifth
/// and sixth are appended after them rather than interleaved, and are about two others; the seventh
/// is appended after those and is about one of them again, which is what makes it the sixth with the
/// box gone rather than a picture of its own arrangement. The identities are written out at each call
/// rather than read back: a description names its shapes by the identity it wrote, so the
/// demonstration already knows which entry it means, and `get` offers no listing to read the names
/// from.
fn demonstrate(description: Description) -> String {
    let (origin, size) = description.window();
    let mut diagram = description.into_diagram();
    let catalog = glyph_catalog();
    let the_back_most = ShapeId::new("#1");

    // How far the third picture's figure moves. A fixed value this function carries rather than a
    // field in the description format or an argument on the binary, so a file's picture is exactly
    // the one it was (B5.8, [ADR-0035]). An assumption about this demonstration, like the reorder
    // below, and not a rule about descriptions. The value lands the figure on cells the shipped
    // description already draws, which is what lets the third picture tell a displacement from the
    // reorder the second one shows.
    let by = Delta { dx: 0, dy: 3 };

    // The box the arrow already hangs from, and the arrow itself, both named by hand for the reason
    // `#1` above is: a description names its shapes by the identity it wrote, so the demonstration
    // already knows which entry it means, and `get` offers no listing to read the names from. A
    // figure added before either would make the written value name the wrong shape, and the fifth
    // picture is what catches that.
    let the_hung_from = ShapeId::new("#3");
    let the_arrow = ShapeId::new("#10");

    // How far the fifth picture's box moves, beside the delta above. The destination was measured
    // against the shipped description's own rows rather than chosen by eye: the box lands at
    // `{13, 2}` occupying `x 13..16, y 2..4`, and the only cells anything else writes in that
    // rectangle are the arrow's own arm at `(13, 3)` through `(15, 3)` — the arm the fifth picture
    // replaces. So the box lands on cells the picture's own change has cleared.
    let four_right = Delta { dx: 4, dy: 0 };

    // How far the sixth picture's arrow moves, beside the two deltas above and **reusing neither**.
    // Two down and not three, for the same reason the third picture's figure moves three rather
    // than four: this function holds its deltas deliberately rather than taking them from the format
    // or the binary, and three rows would land the arrow on cells the shipped description already
    // draws. Two rows lands it clear of both boxes, which is what the picture is for.
    let two_down = Delta { dx: 0, dy: 2 };

    let mut out = String::from("As written:\n");
    out.push_str(&picture(&diagram, &catalog, origin, size));

    // `forward` is a safe no-op when there is no such shape — e.g. an empty description (FR-016).
    diagram.forward(&the_back_most);
    out.push_str("\nWith the back-most shape moved one place forward:\n");
    out.push_str(&picture(&diagram, &catalog, origin, size));

    // `get` and `replace` are no-ops on an identity this diagram does not hold, so an empty
    // description demonstrates through both of them unchanged (B5.7).
    if let Some(moved) = diagram
        .get(&the_back_most)
        .map(|shape| shape.displaced_by(by))
    {
        diagram.replace(&the_back_most, moved);
    }
    out.push_str("\nWith that same shape displaced:\n");
    out.push_str(&picture(&diagram, &catalog, origin, size));

    diagram.remove(&the_back_most);
    out.push_str("\nWith that same shape taken out:\n");
    out.push_str(&picture(&diagram, &catalog, origin, size));

    // The arrow's `from` is rehung from the box's right side — the point the description already
    // spells for it, so the fifth picture opens on the same picture the fourth does and the only
    // thing that changes about the arrow is where it starts. It is **read back** rather than
    // restated: `to`, its direction, its terminal and the stroke are then the description's by
    // construction, and a restated connector would be four values the demonstration agreed with
    // the description about, where a disagreement would change the picture instead of failing.
    //
    // The direction and the terminal are written out beside the anchor and are not derived from it:
    // §6 says both are the caller's, and deriving the direction from the side is
    // [#89](https://github.com/andresmoschini/monospace/issues/89)'s.
    // A description whose tenth entry is not a connector, or one with no tenth entry at all, has
    // nothing to rehang: the `if let` falls through and the picture is the fourth's.
    if let Some(Shape::Connector { to, stroke, .. }) = diagram.get(&the_arrow).cloned() {
        diagram.replace(
            &the_arrow,
            Shape::Connector {
                from: Endpoint {
                    at: Position::Reference(Reference {
                        id: the_hung_from.clone(),
                        anchor: Anchor::Right,
                        offset: Delta { dx: 0, dy: 0 },
                    }),
                    leaving: Direction::Right,
                    terminal: Terminal::Arm,
                },
                to,
                stroke,
            },
        );
    }

    // And now the figure it hangs from moves, which is what the reference is for: the arrow lands
    // on the box's new side and re-routes to the end that did not move, because that endpoint is
    // still a point and a displacement reaches points.
    if let Some(moved) = diagram
        .get(&the_hung_from)
        .map(|shape| shape.displaced_by(four_right))
    {
        diagram.replace(&the_hung_from, moved);
    }
    out.push_str("\nWith the arrow now hanging from that box, and the box displaced:\n");
    out.push_str(&picture(&diagram, &catalog, origin, size));

    // And now the arrow itself moves, which is the sixth picture and the reason the fifth exists:
    // at this point **both** of the arrow's endpoints are references — its `from` rehung above and
    // the shipped `to`, which names `#5`'s bottom with an offset of one in each axis — so a
    // displacement that did not reach a reference's offsets would draw a picture byte for byte
    // identical to the fifth, and that is the defect this step exists to show is gone. The sixth is
    // therefore the fifth with the arrow two rows lower and **both boxes standing exactly where
    // they stood**, which is the claim a reader checks with their eyes.
    //
    // The `if let` is not optional and is the same one the third and fifth steps carry: `get` and
    // `replace` are no-ops on an identity this diagram does not hold, which is what keeps a
    // one-shape description — and an empty one — demonstrating at all.
    if let Some(moved) = diagram
        .get(&the_arrow)
        .map(|shape| shape.displaced_by(two_down))
    {
        diagram.replace(&the_arrow, moved);
    }
    out.push_str("\nWith the arrow displaced as well:\n");
    out.push_str(&picture(&diagram, &catalog, origin, size));

    // And the seventh takes the figure the arrow hangs from **away**, which is the sixth picture with
    // the box gone and **the arrow exactly where it stood**. `remove` freezes the end that named the
    // box at the point it was resolving to, so the box leaves the picture and the connector does
    // not: the seventh is the sixth with twelve cells blanked and nothing else touched.
    //
    // This is the evidence the rule is for. Every picture above shows what the demonstration does to
    // itself, and the sixth removes a figure that holds no reference — so nothing the shipped binary
    // printed before this step showed the rule at all. The seventh is where a reader sees it.
    //
    // **No `if let` and no `get`**, which is what makes this step read differently from the five
    // beside it and is D4's answer rather than an omission: `remove` hands back nothing, so there
    // is nothing here to ask and a caller cannot get the old behavior back by wrapping the call in a
    // conditional. `#3` is the box the demonstration hangs the arrow from — the same identity the
    // fifth picture displaced — so this step takes out exactly what the arrow hangs from.
    diagram.remove(&the_hung_from);
    out.push_str("\nWith the box the arrow hangs from taken out:\n");
    out.push_str(&picture(&diagram, &catalog, origin, size));

    out
}

/// The glyph catalog the CLI renders with: the union of every glyph set the core does not ship
/// and the core's own light table.
fn glyph_catalog() -> GlyphCatalog {
    GlyphCatalog::union([
        GlyphCatalog::light(),
        monospace_glyph_sets::ascii(),
        monospace_glyph_sets::double(),
        monospace_glyph_sets::heavy(),
        monospace_glyph_sets::light_round(),
        monospace_glyph_sets::light_double(),
        monospace_glyph_sets::light_heavy(),
        monospace_glyph_sets::light_round_double(),
        monospace_glyph_sets::light_round_heavy(),
    ])
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use monospace_core::{
        BoxShape, Buffer, Glyph, GlyphCatalog, Layer, Pos, Shape, Size, StampMode, Stroke, render,
    };

    use super::{Description, demonstrate, render_once};

    fn one_box_json() -> &'static str {
        r##"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 2,
            "shapes": [
                { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░" }
            ]
        }"##
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

        assert_eq!(render_once(description), expected);
    }

    /// A file's picture is the demonstration's first picture with nothing around it: no caption
    /// line, no blank line, and no second picture.
    ///
    /// This is what lets `cargo xtask render` paste the output straight into a Markdown fence.
    #[test]
    fn rendering_once_is_the_demonstrations_first_picture_and_nothing_else() {
        let (first, ..) = demonstrated_pictures(one_box_json());

        assert_eq!(render_once(parse(one_box_json())), first);
    }

    /// Parses `json` as a description, which every test here writes by hand.
    fn parse(json: &str) -> Description {
        serde_json::from_str(json).expect("well-formed description")
    }

    /// The demonstration's **seven** pictures, found by the blank line between them and returned
    /// without their captions, so nothing here pins a caption's wording.
    ///
    /// Each carries exactly the trailing newline `render` gives it. The last block already holds
    /// one, since nothing follows it, so it is stripped and put back rather than doubled, and the
    /// seven are then comparable with each other and with a picture drawn on its own.
    ///
    /// **The closure needed no change when the seventh arrived**, and that is worth knowing rather
    /// than assuming: it splits on the blank line, strips the trailing newline and puts one back, and
    /// the seventh block is the last of the output so the normalization the first six already get
    /// applies to it identically. It is also why the sixth compared **equal to the fifth** before the
    /// rule landed rather than one character apart — measured, and the reason no test pins the raw
    /// text.
    fn demonstrated_pictures(
        json: &str,
    ) -> (String, String, String, String, String, String, String) {
        let output = demonstrate(parse(json));
        let mut blocks = output.split("\n\n");
        let mut next_picture = || {
            let block = blocks
                .next()
                .expect("seven captioned pictures, each after a blank line");
            let (_caption, picture) = block
                .split_once('\n')
                .expect("a caption line precedes each picture");
            format!("{}\n", picture.strip_suffix('\n').unwrap_or(picture))
        };

        (
            next_picture(),
            next_picture(),
            next_picture(),
            next_picture(),
            next_picture(),
            next_picture(),
            next_picture(),
        )
    }

    /// TE-007: two partially overlapping opaque boxes demonstrate as two pictures — the first the
    /// two boxes in the order written, the second the two in the opposite order.
    ///
    /// This used to run the binary against a file. A file no longer demonstrates, so the claim is
    /// made here, against the function that does.
    #[test]
    fn two_overlapping_boxes_demonstrate_in_opposite_orders() {
        let (first, second, ..) = demonstrated_pictures(
            r##"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 4 } },
            "next_id": 3,
            "shapes": [
                { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░" },
                { "kind": "box", "id": "#2", "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "▓" }
            ]
        }"##,
        );

        let size = Size {
            width: 4,
            height: 3,
        };
        let a = BoxShape {
            at: Pos { x: 0, y: 0 },
            size,
            stroke: Stroke::from("light"),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        };
        let b = BoxShape {
            at: Pos { x: 2, y: 1 },
            size,
            stroke: Stroke::from("light"),
            fill: Some(Glyph::new("▓").expect("\"▓\" is one glyph")),
        };

        assert_eq!(first, stamped_back_to_front(&a, &b));
        assert_eq!(second, stamped_back_to_front(&b, &a));
    }

    /// Stamps `back` then `front` with `StampMode::Above` into a six-by-four window — the picture
    /// a description listing them in that order produces, per _The two orders are equivalent_.
    fn stamped_back_to_front(back: &BoxShape, front: &BoxShape) -> String {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let mut buffer = Buffer::new(origin, size);
        back.draw(&mut Layer::new(&mut buffer, StampMode::Above));
        front.draw(&mut Layer::new(&mut buffer, StampMode::Above));
        render(&buffer, &GlyphCatalog::light(), origin, size)
    }

    /// User Story 5, spec's B5.1 and B5.2, SC-005: the tenth entry's far endpoint stops being a
    /// point and becomes a reference to `#5`'s bottom with an offset of one in each axis, and
    /// **the first five pictures come out byte for byte what they were**.
    ///
    /// The evidence is in a file and the picture not moving is what proves the arithmetic. The
    /// other side of each comparison is not pinned as text but built: the same demonstration with
    /// that one entry's `to` put back to the point it spells, so a difference can only be the
    /// entry. `{21, 3} + (1, 1)` is `{22, 4}`, which is what the entry said outright, and the
    /// demonstration removes `#1`, displaces `#1` and displaces `#3`, so the fifth shape is none of
    /// them and the reference resolves the same in every picture.
    ///
    /// **The sixth is compared between the two runs and not against a "before",** because it has no
    /// before: it is the demonstration's own step, added by this slice, and both runs produce it. The
    /// two are equal, and the reason is the rule rather than a coincidence: the run that spells `to`
    /// outright has that endpoint grown from `{22, 4}` as an absolute point, while the run that
    /// names a reference to `#5`'s bottom with offset `(1, 1)` has the **offset** grown to `(1, 3)`
    /// and the reference resolving to `{22, 6}`. Same cell, two routes to it, which is the whole
    /// claim.
    ///
    /// A path still prints one picture and nothing else, which is what `cargo xtask render` embeds.
    #[test]
    fn the_tenth_entry_naming_a_reference_leaves_the_first_five_pictures_exactly_as_they_were() {
        // Built through `serde_json` rather than by replacing text in the file, because the file is
        // formatted and a needle written against one formatting of it is a test that stops matching
        // the day prettier disagrees.
        let mut value: serde_json::Value =
            serde_json::from_str(super::DEMO).expect("the embedded description is well-formed");
        for shape in value["shapes"]
            .as_array_mut()
            .expect("the description lists its shapes")
        {
            if shape["kind"] == serde_json::json!("connector")
                && shape["from"]["at"]["kind"] == serde_json::json!("point")
                && shape["from"]["at"]["x"] == serde_json::json!(12)
            {
                shape["to"]["at"] = serde_json::json!({ "kind": "point", "x": 22, "y": 4 });
            }
        }
        let with_the_point = value.to_string();
        assert_ne!(
            super::DEMO,
            with_the_point,
            "the tenth entry still spells its far endpoint outright"
        );

        let (first, second, third, fourth, fifth, sixth, seventh) =
            demonstrated_pictures(super::DEMO);
        let (first2, second2, third2, fourth2, fifth2, sixth2, seventh2) =
            demonstrated_pictures(&with_the_point);
        assert_eq!(
            (&first, &second, &third, &fourth, &fifth),
            (&first2, &second2, &third2, &fourth2, &fifth2),
            "naming the far endpoint as a reference changed one of the first five pictures"
        );
        assert_eq!(
            sixth, sixth2,
            "the sixth picture differs between a spelled endpoint and a named one: both are this \
             slice's own step, and both reach the same cell two rows lower"
        );
        // **The seventh joins the sixth for the same stated reason, and the reason is the freeze.**
        // `remove(&#3)` freezes the arrow's `from` at `{16, 5}` in both runs whatever route it took
        // to get there: the run that spells `to` outright grows that absolute point from `{22, 4}`,
        // and the run that names a reference to `#5`'s bottom grows the **offset** to `(1, 3)` and
        // freezes the other end there. Same cell, two routes to it — the claim the sixth's existing
        // comment already made, now with the seventh beside it. Neither has a "before": both are the
        // demonstration's own step and both runs produce them.
        assert_eq!(
            seventh, seventh2,
            "the seventh picture differs between a spelled endpoint and a named one"
        );

        // The first picture is the shipped file's own, byte for byte, and a path prints that and
        // nothing else.
        assert_eq!(first, render_once(parse(super::DEMO)));
        assert_eq!(first2, render_once(parse(&with_the_point)));

        // The fifth picture still shows the box the arrow hangs from displaced four cells right,
        // because that is the demonstration's own change to the picture and not the file's.
        assert_ne!(
            fourth, fifth,
            "the fifth picture must be the fourth with a figure moved"
        );
    }

    /// User Story 3, spec's B3.1, B3.3 and SC-006: a bare run prints **seven** captioned pictures and
    /// the first is the description as written.
    ///
    /// The count comes from the blank lines the output holds, and no caption's wording is pinned —
    /// what is claimed is that there are seven of them and that the first is the one a file's run
    /// prints on its own. The sixth and the seventh are captions like the other five, so the count
    /// is all this test says about them; which picture the sixth holds is
    /// `the_sixth_picture_moves_only_the_arrow`'s claim and which the seventh holds is
    /// `the_seventh_picture_takes_the_box_away_and_leaves_the_arrow`'s.
    #[test]
    fn a_bare_run_prints_seven_captioned_pictures_the_first_being_the_description_as_written() {
        let output = demonstrate(parse(super::DEMO));

        assert_eq!(
            output.split("\n\n").count(),
            7,
            "seven captioned pictures, each after a blank line: {output:?}"
        );

        let (first, ..) = demonstrated_pictures(super::DEMO);
        assert_eq!(first, render_once(parse(super::DEMO)));
    }

    /// User Story 4, B4.2, SC-005: `render_once` over a path prints one picture and nothing else,
    /// which is what `cargo xtask render` embeds in a document.
    ///
    /// Pinned as a **count and a shape**, not as a literal: one window's worth of rows, and the
    /// first picture's own first row. The caption is absent, because a file's run is a picture with
    /// nothing around it and the demonstration's is a picture under a caption.
    #[test]
    fn a_path_prints_one_picture_and_nothing_else() {
        let (first, ..) = demonstrated_pictures(super::DEMO);
        let once = render_once(parse(super::DEMO));

        assert_eq!(
            once, first,
            "a path prints the first picture, byte for byte"
        );
        assert!(
            !once.contains("\n\n"),
            "a path prints no caption and no second picture: {once:?}"
        );
        assert_eq!(
            once.lines().count(),
            13,
            "the shipped window is thirteen rows tall and nothing is added: {once:?}"
        );
    }

    /// The glyph a picture shows at `(column, row)`, or a space where it draws nothing.
    ///
    /// What `render` gives for a cell holding nothing, so a claim about a cell being blank or about
    /// two pictures agreeing on one is made in the vocabulary a reader reads the picture in.
    fn the_glyph_at(picture: &str, at: (usize, usize)) -> char {
        let (x, y) = at;
        picture
            .lines()
            .nth(y)
            .and_then(|row| row.chars().nth(x))
            .unwrap_or(' ')
    }

    /// The columns and rows the figure the third picture displaces holds, in the second and the
    /// third picture of the shipped demonstration.
    ///
    /// Quoted rather than read from the code that produces them, because a contract test that asks
    /// the demonstration the same questions it answers itself checks nothing. The numbers are the
    /// shipped description's first entry, a four-by-three box at the origin, and the demonstration's
    /// own fixed delta of three rows down.
    fn the_first_figures_footprint() -> (Range<usize>, Range<usize>) {
        (0..4, 0..6)
    }

    /// The positions where two pictures of one window differ, as `(column, row)`.
    fn differing(first: &str, second: &str) -> Vec<(usize, usize)> {
        first
            .lines()
            .zip(second.lines())
            .enumerate()
            .flat_map(|(y, (one, other))| {
                one.chars()
                    .zip(other.chars())
                    .enumerate()
                    .filter(|(_, (one, other))| one != other)
                    .map(move |(x, _)| (x, y))
            })
            .collect()
    }

    /// User Story 3, spec's B3.3, SC-004: the third picture differs from the second only in the
    /// cells the displaced figure holds, and the fourth differs from the third only in the cells that
    /// figure occupies.
    ///
    /// The fourth is pinned against the shipped description with its first entry left out and drawn
    /// on its own, which is the whole claim in one comparison: a removal leaves a gap rather than a
    /// hole punched in what was around it, and the cell each gap holds is whatever the figure
    /// behind it decides, or empty. Comparing the two pictures with each other would only say they
    /// differ.
    #[test]
    fn the_third_picture_moves_one_figure_and_the_fourth_takes_that_figure_out() {
        let (_first, second, third, fourth, ..) = demonstrated_pictures(super::DEMO);

        let (columns, rows) = the_first_figures_footprint();
        let moved = differing(&second, &third);
        assert!(!moved.is_empty(), "the displacement changed nothing at all");
        assert!(
            moved
                .iter()
                .all(|(x, y)| columns.contains(x) && rows.contains(y)),
            "the displacement reached outside the figure's own cells: {moved:?}"
        );

        assert_eq!(fourth, render_once(parse(&demo_without_its_first_entry())));
    }

    /// The columns and rows the fifth picture's box holds before and after, and the column the
    /// connector's far endpoint holds throughout.
    ///
    /// Quoted rather than read from the code that produces them, because a contract test that asks
    /// the demonstration the same questions it answers itself checks nothing. The numbers are the
    /// shipped description's third entry — a four-by-three box at `{9, 2}` — the demonstration's own
    /// fixed delta of four columns right, and the tenth entry's `to` at `{22, 4}`, which the fifth
    /// picture does not touch.
    fn the_hung_figures_footprint() -> (
        Range<usize>,
        Range<usize>,
        Range<usize>,
        Range<usize>,
        usize,
    ) {
        ((9..13), (2..5), (13..17), (2..5), 22)
    }

    /// User Story 5, spec's B5.1, SC-006: the fifth picture differs from the fourth in exactly the
    /// box's old cells, the box's new cells and the connector's route — the same shape of claim the
    /// third picture's own test makes, reached with the same `differing` helper.
    ///
    /// The route is bounded rather than listed: it is the cells from the box's **new** side across
    /// to the far endpoint, on the rows it runs along, so the range is read off the box's own
    /// footprint rather than written down. The second assertion is the one that says the arrow
    /// re-routed rather than carried along: nothing at or past the far endpoint's column moved,
    /// which is the endpoint the displacement does not reach.
    #[test]
    fn the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it() {
        let (_first, _second, _third, fourth, fifth, ..) = demonstrated_pictures(super::DEMO);

        let (old_columns, old_rows, new_columns, new_rows, the_far_end_column) =
            the_hung_figures_footprint();

        let reached = |at: (usize, usize)| {
            let (x, y) = at;
            let in_a_footprint = (old_columns.contains(&x) && old_rows.contains(&y))
                || (new_columns.contains(&x) && new_rows.contains(&y));
            let on_the_route = (new_rows.start..=new_rows.end + 1).contains(&y)
                && (new_columns.end..=22).contains(&x);
            in_a_footprint || on_the_route
        };

        let changed = differing(&fourth, &fifth);
        assert!(
            !changed.is_empty(),
            "the fifth picture changed nothing at all"
        );
        assert!(
            changed.iter().copied().all(reached),
            "the fifth picture changed a cell neither box holds nor the arrow runs through: {changed:?}"
        );
        assert!(
            changed.iter().all(|(x, _)| *x <= the_far_end_column),
            "the change reached past the endpoint that did not move: {changed:?}"
        );

        // And the box is on its new side rather than still on the old one: a cell inside the new
        // footprint is written now and held nothing before.
        let occupied_before = |(x, y): (usize, usize)| {
            fourth
                .lines()
                .nth(y)
                .and_then(|row| row.chars().nth(x))
                .is_some_and(|glyph| glyph != ' ')
        };
        assert!(
            (new_columns.clone())
                .any(|x| (new_rows.clone()).any(|y| reached((x, y)) && !occupied_before((x, y)))),
            "the box did not land where the demonstration says it lands"
        );
    }

    /// The shipped demonstration with its first entry left out, as the text `render_once` reads.
    ///
    /// Rewritten through `serde_json` rather than by hand, so the description the test draws is the
    /// one the binary embeds and only the first entry differs.
    ///
    /// **It renumbers nothing, and that is the cost this slice removes rather than a fix to this
    /// helper.** While an identity named a place in a list, removing the first listing shifted
    /// every name after it: the tenth entry's `to` named `"#5"`, the fifth entry of *this* text, and
    /// re-reading the text issued the identities from scratch in array order, so `#5` became the
    /// box at `{18, 0}` rather than the one at `{20, 1}`. The fourth picture would then have been a
    /// different picture from the one the test compares it to, and the difference begins at row 3.
    /// That loop rewriting `"#5"` to `"#4"` is what kept the two sides the same diagram.
    ///
    /// Every entry now carries the identity it was written with, so the twenty-five that survive
    /// keep the names they had and `remove(0)` shifts nothing at all. The same `assert_eq!` in
    /// `the_third_picture_moves_one_figure_and_the_fourth_takes_that_figure_out` is SC-001's claim
    /// now rather than a fixture a loop had to hold in place.
    fn demo_without_its_first_entry() -> String {
        let mut value: serde_json::Value =
            serde_json::from_str(super::DEMO).expect("the embedded description is well-formed");
        let shapes = value["shapes"]
            .as_array_mut()
            .expect("the description lists its shapes");
        shapes.remove(0);
        value.to_string()
    }

    /// User Story 3, spec's B3.3, SC-004: an empty description demonstrates as **seven** identical
    /// pictures and fails nothing.
    ///
    /// There is no back-most shape to move, no figure to displace, no shape to take out, no tenth
    /// entry to rehang and no third entry to displace, so every call in every picture is a no-op on
    /// an identity this diagram does not hold, and there is no branch here to get wrong.
    ///
    /// **The seventh is the case worth having**, and it is worth having because it is a `remove`
    /// rather than an `if let`. An empty description holds no `#3`, so the seventh step calls
    /// `remove` on an identity this diagram does not hold: `find` answers `None` and `remove`
    /// returns before its loop, which is a different guard from the `if let` the sixth step carries
    /// and is the only reason a removal can be called unconditionally at all. The seventh picture is
    /// therefore the sixth, and a seventh `assert_eq!` is what makes the count seven mean something
    /// rather than being a count of pictures the helper happened to return.
    #[test]
    fn an_empty_description_demonstrates_as_seven_identical_pictures() {
        let pictures = demonstrated_pictures(
            r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 2,
            "shapes": []
        }"#,
        );

        assert_eq!(pictures.0, pictures.1);
        assert_eq!(pictures.0, pictures.2);
        assert_eq!(pictures.0, pictures.3);
        assert_eq!(pictures.0, pictures.4);
        assert_eq!(pictures.0, pictures.5);
        assert_eq!(pictures.0, pictures.6);
    }

    /// User Story 3, spec's B3.3, SC-004: a description holding exactly one shape demonstrates **seven**
    /// pictures and fails nothing.
    ///
    /// The reorder changes nothing, because that shape is both front-most and back-most. The
    /// displacement does not: the demonstration's fixed delta carries the only figure out of this
    /// three-row window, which the figure filled, so the third picture is that window with nothing
    /// in it. The fourth is the same, because the figure is taken out.
    ///
    /// **These seven are therefore not identical**, and no value of the delta would make them so: a
    /// figure that fills its own window is moved partly or wholly out of it by any delta other than
    /// none. What the rule asks of this case is that the run succeeds, which it does — and the fifth,
    /// sixth and seventh add three more no-ops on identities a one-shape description does not hold,
    /// since it has no `#3` and no `#10`. See the specification's Clarifications for the 2026-09-28
    /// session, which corrected this scenario on the evidence of this test.
    #[test]
    fn one_shape_demonstrates_as_two_copies_of_itself_and_then_an_empty_window() {
        let pictures = demonstrated_pictures(one_box_json());

        assert_eq!(pictures.0, pictures.1);
        assert_eq!(pictures.2, pictures.3);
        assert_ne!(pictures.0, pictures.2);
        assert_eq!(pictures.3, pictures.4);
        // The sixth equals the fifth for the same reason the fourth equals the third: the
        // description holds no `#10`, so the sixth step's `get` returns `None` and the step changes
        // nothing. **A one-shape description is the case that would break if the `if let` around
        // that step were dropped**, which is why it is asserted rather than assumed.
        assert_eq!(pictures.4, pictures.5);
        // And the seventh equals the sixth for the reason of its own: the description holds no
        // `#3` either, so `remove` finds nothing and returns before its loop. The name is unchanged
        // and still true — this is two copies of itself and then an empty window, whatever comes
        // after — and the count beside it is what changed.
        assert_eq!(pictures.5, pictures.6);
    }

    /// The two boxes' footprints at the fifth picture, and the one cell of the first that the arrow
    /// legitimately writes.
    ///
    /// **Quoted rather than read from the code that produces them**, because a contract test that
    /// asks the demonstration the same questions it answers itself checks nothing. `#3` is the
    /// shipped description's third entry, a four-by-three box at `{9, 2}` that the fifth picture
    /// displaces four columns right into `x 13..16, y 2..4`; `#5` is its fifth entry, a four-by-three
    /// box at `{20, 1}` occupying `x 20..23, y 1..3`, which neither the fifth nor the sixth touches.
    ///
    /// **The attachment cell is named because a footprint is not the same as a figure's own cells.**
    /// The arrow's `from` hangs from `#3`'s **right side**, whose centre is `{16, 3}` — and `{16, 3}`
    /// is inside `#3`'s own rectangle, because the rectangle is the box and the box's border is its
    /// rightmost column. The fifth picture has the arrow's arm welded to that border cell and the
    /// sixth has it detached, so that one cell inside a footprint **must** change for the arrow to
    /// have moved at all. A test that forbade every cell inside either footprint would therefore
    /// fail on the very behavior it is meant to certify, and quoting the footprint alone would hide
    /// that. The far end is not in this position: `to` names `#5`'s bottom with offset `(1, 1)`, and
    /// `{21, 3} + (1, 1)` is `{22, 4}` — **one row below** `#5`, outside its rectangle — which is
    /// why the second box's footprint is untouched entire.
    fn the_two_boxes_and_the_attachment() -> [(Range<usize>, Range<usize>); 2] {
        [(13..17, 2..5), (20..24, 1..4)]
    }

    /// The one cell inside a box's footprint the arrow writes: `#3`'s right side centre, `{16, 3}`.
    const THE_ATTACHMENT: (usize, usize) = (16, 3);

    /// The rectangle the arrow's own cells fall inside at the sixth picture, and the rectangle the
    /// box it hangs from stands in at the sixth and is gone by the seventh.
    ///
    /// **Quoted rather than read from the code that produces them**, because a contract test that
    /// asks the demonstration the same questions it answers itself checks nothing. `#3` is the
    /// shipped description's third entry, a four-by-three box at `{9, 2}` that the fifth picture
    /// displaces four columns right into `x 13..16, y 2..4`. The arrow's `from` stands at
    /// `{16, 5}` — `#3`'s right side centre at `{16, 3}` plus the offset `(0, 2)` the sixth picture
    /// grew — and its `to` names `#5`'s bottom with offset `(1, 3)`, which is `{22, 6}`.
    ///
    /// **The arrow's rectangle is a bound and not its footprint, and that is the point.** Between
    /// those two cells the route writes **ten** of the twenty-one the rectangle holds, and it writes
    /// them in three rows: four across the top, two in the middle and four along the bottom. It is
    /// neither contiguous nor a rectangle, so nothing that reads it off a picture gets it right —
    /// which is how the specification's own `22 − 10 = 12` was wrong and how research.md Q7 corrected
    /// it. The count is therefore **asserted against the pictures** and the rectangle is only what
    /// bounds where to look.
    fn the_arrow_and_its_removed_box() -> [(Range<usize>, Range<usize>); 2] {
        [(16..23, 5..8), (13..17, 2..5)]
    }

    /// User Story 3, spec's B3.1 and SC-001: the seventh picture differs from the sixth **only** in
    /// the cells `#3` held, which is the only statement in the slice that says the arrow stood
    /// still.
    ///
    /// Modelled on `the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it` and reusing its
    /// `differing` helper, with the claim **in both directions**: what the removal may reach, and
    /// what it may not touch. The first alone is not enough — a removal that moved the arrow
    /// elsewhere and blanked two rows of box would satisfy "twelve cells differ", and the second is
    /// what rules that out.
    #[test]
    fn the_seventh_picture_takes_the_box_away_and_leaves_the_arrow() {
        let (_first, _second, _third, _fourth, _fifth, sixth, seventh) =
            demonstrated_pictures(super::DEMO);

        let [arrow, the_box] = the_arrow_and_its_removed_box();
        let (arrow_columns, arrow_rows) = (arrow.0, arrow.1);

        // What the removal may reach: exactly the twelve cells `#3` stood in, and all twelve blank
        // rather than carrying a different glyph — a box's border cells are the only thing that
        // could still be written there.
        let changed = differing(&sixth, &seventh);
        assert_eq!(
            changed.len(),
            12,
            "the seventh differs from the sixth in something other than the twelve cells the box \
             held: {changed:?}"
        );
        assert!(
            changed
                .iter()
                .all(|(x, y)| the_box.0.contains(x) && the_box.1.contains(y)),
            "the removal reached outside the box's own rectangle: {changed:?}"
        );
        assert!(
            changed.iter().all(|at| the_glyph_at(&seventh, *at) == ' '),
            "a cell the removed box held is still drawn in the seventh: {changed:?}"
        );

        // What it may not touch: every cell the arrow held in the sixth, found rather than quoted —
        // the written cells inside the bound `the_arrow_and_its_removed_box` names, which is a
        // rectangle holding ten written cells out of twenty-one.
        let the_arrow_held: Vec<(usize, usize)> = arrow_rows
            .flat_map(|y| arrow_columns.clone().map(move |x| (x, y)))
            .filter(|at| the_glyph_at(&sixth, *at) != ' ')
            .collect();
        assert_eq!(
            the_arrow_held.len(),
            10,
            "the arrow's footprint is not the ten cells this test measured: {the_arrow_held:?}"
        );
        for at in the_arrow_held {
            assert_eq!(
                the_glyph_at(&sixth, at),
                the_glyph_at(&seventh, at),
                "the arrow did not stand where it stood, at {at:?}"
            );
        }
    }

    /// User Story 3, spec's B3.1, SC-004: the sixth picture differs from the fifth **only** in the
    /// cells the arrow holds before and after, which is the only statement that says both boxes
    /// stood still.
    ///
    /// Modelled on `the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it` and reusing its
    /// `differing` helper, with the claim **mirrored**: that test bounds what the fifth may reach,
    /// and this one bounds what the sixth may reach in the same shape.
    ///
    /// The bound is **exact rather than one-sided**, and that is what makes it say "both boxes stood
    /// still" rather than "no box moved very far": the cells that changed inside the two footprints
    /// are exactly the one attachment cell and nothing else. A displacement that rewrote the shape a
    /// reference names — moving `#3` or `#5` to follow the arrow — would change cells inside a
    /// footprint that are not the attachment, and the assertion below is what rules it out.
    #[test]
    fn the_sixth_picture_moves_only_the_arrow() {
        let (_first, _second, _third, _fourth, fifth, sixth, _seventh) =
            demonstrated_pictures(super::DEMO);

        let changed = differing(&fifth, &sixth);
        assert!(
            !changed.is_empty(),
            "the sixth picture changed nothing at all"
        );

        let inside: Vec<(usize, usize)> = changed
            .iter()
            .copied()
            .filter(|(x, y)| {
                the_two_boxes_and_the_attachment()
                    .iter()
                    .any(|(columns, rows)| columns.contains(x) && rows.contains(y))
            })
            .collect();
        assert_eq!(
            inside,
            vec![THE_ATTACHMENT],
            "the sixth picture changed a cell inside a box's own footprint other than the one the \
             arrow attaches to, so a box moved: {changed:?}"
        );

        // And the arrow did move, which the bound above cannot say on its own: the far end left the
        // cell it welded itself to. Named rather than derived, and it is a cell **outside** both
        // footprints — `{22, 4}` is `#5`'s bottom centre plus the shipped offset, one row below the
        // box — so a displacement that moved nothing and a displacement that moved a box could not
        // both satisfy this.
        assert!(
            changed.contains(&(22, 4)),
            "the arrow's far end is still welded where it was: {changed:?}"
        );
        assert!(
            changed.contains(&(22, 6)),
            "the arrow's far end did not travel the two rows the demonstration's delta names: \
             {changed:?}"
        );
    }
}
