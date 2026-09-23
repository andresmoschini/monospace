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
//! the shipped demonstration the way spec 080 asks for it: two captioned pictures, the second with
//! one shape moved one place toward the front.
//!
//! That split exists because moving `#1` forward says something only about the shipped
//! demonstration, whose first two entries are two partially overlapping opaque boxes. Applied to a
//! file someone hands the binary it is a demonstration's assumption imposed on their description.
//! It is also what lets `cargo xtask render` embed this output in a document
//! ([ADR-0052](../../../docs/decisions/0052-show-the-rendering.md)): a picture that goes into a
//! Markdown fence cannot arrive wrapped in prose.

mod description;

use std::process::ExitCode;

use description::Description;
use monospace_core::{Buffer, GlyphCatalog};
use monospace_diagram::ShapeId;

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

    let mut buffer = Buffer::new(origin, size);
    diagram.draw(&mut buffer);
    monospace_core::render(&buffer, &glyph_catalog(), origin, size)
}

/// Renders `description` as written, then again with its first entry moved one place toward the
/// front, each picture under a caption.
///
/// This is the shipped demonstration's output, and spec 080's FR-018 is what asks for the two
/// captions. The move it shows is meaningful only for that description, which is why a file the
/// binary is handed goes through [`render_once`] instead.
fn demonstrate(description: Description) -> String {
    let (origin, size) = description.window();
    let mut diagram = description.into_diagram();
    let catalog = glyph_catalog();

    let mut first = String::from("As written:\n");
    let mut buffer = Buffer::new(origin, size);
    diagram.draw(&mut buffer);
    first.push_str(&monospace_core::render(&buffer, &catalog, origin, size));

    // `forward` is a safe no-op when there is no such shape — e.g. an empty description (FR-016).
    diagram.forward(&ShapeId::new("#1"));

    let mut second = String::from("\nWith the back-most shape moved one place forward:\n");
    let mut buffer = Buffer::new(origin, size);
    diagram.draw(&mut buffer);
    second.push_str(&monospace_core::render(&buffer, &catalog, origin, size));

    first + &second
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
        let (first, _second) = demonstrated_pictures(one_box_json());

        assert_eq!(render_once(parse(one_box_json())), first);
    }

    /// Parses `json` as a description, which every test here writes by hand.
    fn parse(json: &str) -> Description {
        serde_json::from_str(json).expect("well-formed description")
    }

    /// The demonstration's two pictures, found by the blank line between them and returned
    /// without their captions, so nothing here pins a caption's wording.
    fn demonstrated_pictures(json: &str) -> (String, String) {
        let output = demonstrate(parse(json));
        let (first_block, rest) = output
            .split_once("\n\n")
            .expect("two captioned pictures separated by a blank line");
        let (_first_caption, first) = first_block
            .split_once('\n')
            .expect("a caption line precedes the first picture");
        let (_second_caption, second) = rest
            .split_once('\n')
            .expect("a caption line precedes the second picture");
        (format!("{first}\n"), second.to_owned())
    }

    /// TE-007: two partially overlapping opaque boxes demonstrate as two pictures — the first the
    /// two boxes in the order written, the second the two in the opposite order.
    ///
    /// This used to run the binary against a file. A file no longer demonstrates, so the claim is
    /// made here, against the function that does.
    #[test]
    fn two_overlapping_boxes_demonstrate_in_opposite_orders() {
        let (first, second) = demonstrated_pictures(
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

    /// An empty description demonstrates as two identical pictures: there is no back-most shape
    /// to move.
    #[test]
    fn an_empty_description_demonstrates_as_two_identical_pictures() {
        let (first, second) = demonstrated_pictures(
            r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "shapes": []
        }"#,
        );

        assert_eq!(first, second);
    }

    /// A description holding exactly one shape demonstrates as two identical pictures: that shape
    /// is both front-most and back-most, so moving it changes nothing.
    #[test]
    fn one_shape_demonstrates_as_two_identical_pictures() {
        let (first, second) = demonstrated_pictures(one_box_json());

        assert_eq!(first, second);
    }
}
