//! Minimal non-interactive command-line front end for Monospace.
//!
//! It holds no domain logic of its own: it converts its description format — documented in
//! `specs/079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md` — into a
//! `monospace_diagram::Diagram` and draws it.

mod description;

use std::process::ExitCode;

use description::Description;
use monospace_core::{Buffer, GlyphCatalog};

/// The shipped demonstration description, embedded at compile time so the no-argument run works
/// from any working directory and from a binary copied outside a checkout (FR-022, FR-023).
const DEMO: &str = include_str!("../assets/demo.json");

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

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

    print!("{}", render_description(description));
    ExitCode::SUCCESS
}

/// Renders `description` as written, then again with its back-most shape moved one place toward
/// the front, each picture under a caption (FR-013, FR-014, FR-015, FR-018, FR-019).
fn render_description(description: Description) -> String {
    let (origin, size) = description.window();
    let (mut diagram, back_most) = description.into_diagram();
    let catalog = glyph_catalog();

    let mut first = String::from("As written:\n");
    let mut buffer = Buffer::new(origin, size);
    diagram.draw(&mut buffer);
    first.push_str(&monospace_core::render(&buffer, &catalog, origin, size));

    // Moving the back-most entry forward is a demonstration-only assumption: it shows something
    // only because the shipped demonstration's first two entries are two partially overlapping
    // opaque boxes (FR-016).
    if let Some(id) = back_most {
        diagram.forward(&id);
    }

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

    use super::{Description, render_description};

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

        let output = render_description(description);
        let (first_block, _rest) = output
            .split_once("\n\n")
            .expect("two captioned pictures separated by a blank line");
        let (_caption, picture) = first_block
            .split_once('\n')
            .expect("a caption line precedes the picture");

        assert_eq!(format!("{picture}\n"), expected);
    }
}
