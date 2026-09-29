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
//! the shipped demonstration the way spec 080 asks for it: five captioned pictures about one
//! figure, as written, with the back-most shape moved one place toward the front, with that same
//! shape displaced, with that same shape taken out, and with the arrow rehung from the box it
//! already pointed at and that box displaced.
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
/// front, then again with that same entry displaced, then again with it taken out, and then once
/// more with the arrow rehung from the box it already pointed at and that box displaced. Each
/// picture is under a caption.
///
/// This is the shipped demonstration's output, and spec 080's FR-018 is what asks for the captions.
/// The changes it shows are meaningful only for that description, which is why a file the binary is
/// handed goes through [`render_once`] instead.
///
/// The first four pictures are about one figure, the entry the description lists first. The fifth
/// is appended after them rather than interleaved, and is about two others. The identities are
/// written out at each call rather than read back: a description names its shapes by position, so
/// the demonstration already knows which entry it means and has nothing to read.
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
    // `#1` above is: a description names its shapes by position, so the demonstration already knows
    // which entry it means and has nothing to read back. A figure added before either would make
    // the written value name the wrong shape, and the fifth picture is what catches that.
    let the_hung_from = ShapeId::new("#3");
    let the_arrow = ShapeId::new("#10");

    // How far the fifth picture's box moves, beside the delta above. The destination was measured
    // against the shipped description's own rows rather than chosen by eye: the box lands at
    // `{13, 2}` occupying `x 13..16, y 2..4`, and the only cells anything else writes in that
    // rectangle are the arrow's own arm at `(13, 3)` through `(15, 3)` — the arm the fifth picture
    // replaces. So the box lands on cells the picture's own change has cleared.
    let four_right = Delta { dx: 4, dy: 0 };

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
        r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "shapes": [
                { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░" }
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

    /// The demonstration's five pictures, found by the blank line between them and returned
    /// without their captions, so nothing here pins a caption's wording.
    ///
    /// Each carries exactly the trailing newline `render` gives it. The last block already holds
    /// one, since nothing follows it, so it is stripped and put back rather than doubled, and the
    /// five are then comparable with each other and with a picture drawn on its own.
    fn demonstrated_pictures(json: &str) -> (String, String, String, String, String) {
        let output = demonstrate(parse(json));
        let mut blocks = output.split("\n\n");
        let mut next_picture = || {
            let block = blocks
                .next()
                .expect("five captioned pictures, each after a blank line");
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
            r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 4 } },
            "shapes": [
                { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░" },
                { "kind": "box", "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "▓" }
            ]
        }"#,
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

    /// User Story 5, spec's B5.1 scenario, SC-006: a bare run prints five captioned pictures and
    /// the first is the description as written.
    ///
    /// The count comes from the blank lines the output holds, and no caption's wording is pinned:
    /// what is claimed is that there are five of them and that the first is the one a file's run
    /// prints on its own.
    #[test]
    fn a_bare_run_prints_five_captioned_pictures_the_first_being_the_description_as_written() {
        let output = demonstrate(parse(super::DEMO));

        assert_eq!(
            output.split("\n\n").count(),
            5,
            "five captioned pictures, each after a blank line: {output:?}"
        );

        let (first, ..) = demonstrated_pictures(super::DEMO);
        assert_eq!(first, render_once(parse(super::DEMO)));
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

    /// User Story 5, spec's B5.3 and B5.5, SC-006: the third picture differs from the second only
    /// in the cells the displaced figure holds, and the fourth differs from the third only in the
    /// cells that figure occupies.
    ///
    /// The fourth is pinned against the shipped description with its first entry left out and drawn
    /// on its own, which is the whole claim in one comparison: a removal leaves a gap rather than a
    /// hole punched in what was around it, and the cell each gap holds is whatever the figure
    /// behind it decides, or empty. Comparing the two pictures with each other would only say they
    /// differ.
    #[test]
    fn the_third_picture_moves_one_figure_and_the_fourth_takes_that_figure_out() {
        let (_first, second, third, fourth, _fifth) = demonstrated_pictures(super::DEMO);

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
        let (_first, _second, _third, fourth, fifth) = demonstrated_pictures(super::DEMO);

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
    /// one the binary embeds and only the first entry differs. The identities it is given on the
    /// way are not the demonstration's, and nothing here can tell: an identity names an entry in
    /// the order, and the order is the same either way.
    fn demo_without_its_first_entry() -> String {
        let mut value: serde_json::Value =
            serde_json::from_str(super::DEMO).expect("the embedded description is well-formed");
        value["shapes"]
            .as_array_mut()
            .expect("the description lists its shapes")
            .remove(0);
        value.to_string()
    }

    /// User Story 5, spec's B5.7 scenario: an empty description demonstrates as five identical
    /// pictures and fails nothing.
    ///
    /// There is no back-most shape to move, no figure to displace, no shape to take out, no tenth
    /// entry to rehang and no third entry to displace, so every call in every picture is a no-op on
    /// an identity this diagram does not hold, and there is no branch here to get wrong.
    #[test]
    fn an_empty_description_demonstrates_as_five_identical_pictures() {
        let pictures = demonstrated_pictures(
            r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "shapes": []
        }"#,
        );

        assert_eq!(pictures.0, pictures.1);
        assert_eq!(pictures.0, pictures.2);
        assert_eq!(pictures.0, pictures.3);
        assert_eq!(pictures.0, pictures.4);
    }

    /// User Story 5, spec's B5.7 scenario: a description holding exactly one shape demonstrates
    /// five pictures and fails nothing.
    ///
    /// The reorder changes nothing, because that shape is both front-most and back-most. The
    /// displacement does not: the demonstration's fixed delta carries the only figure out of this
    /// three-row window, which the figure filled, so the third picture is that window with nothing
    /// in it. The fourth is the same, because the figure is taken out.
    ///
    /// These five are therefore not identical, and no value of the delta would make them so: a
    /// figure that fills its own window is moved partly or wholly out of it by any delta other than
    /// none. What the rule asks of this case is that the run succeeds, which it does — and the fifth
    /// adds two more no-ops on identities a one-shape description does not hold. See the
    /// specification's Clarifications for the 2026-09-28 session, which corrected B5.7 on the
    /// evidence of this test.
    #[test]
    fn one_shape_demonstrates_as_two_copies_of_itself_and_then_an_empty_window() {
        let pictures = demonstrated_pictures(one_box_json());

        assert_eq!(pictures.0, pictures.1);
        assert_eq!(pictures.2, pictures.3);
        assert_ne!(pictures.0, pictures.2);
        assert_eq!(pictures.3, pictures.4);
    }
}
