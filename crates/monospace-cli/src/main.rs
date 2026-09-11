//! Minimal non-interactive command-line front end for Monospace.
//!
//! It holds no domain logic of its own: everything it draws comes from `monospace-core`.

use monospace_core::{
    BoxShape, Buffer, Glyph, GlyphCatalog, Layer, Pos, Shape, Size, StampMode, Stroke, render,
};

mod description;

use description::Description;

/// Draws the 4x3 filled box spec 0002 settled, with `origin` as its top-left corner and `mode`
/// as the stamp mode. `BoxShape` computes the same border and fill this function used to stamp
/// by hand — the feature 039 plan checked the two byte for byte before this refactor landed.
fn stamp_box(buffer: &mut Buffer, origin: Pos, mode: StampMode) {
    BoxShape {
        at: origin,
        size: Size {
            width: 4,
            height: 3,
        },
        stroke: Stroke::from("light"),
        fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
    }
    .draw(&mut Layer::new(buffer, mode));
}

/// Draws the box at `(0, 0)` and again at `(2, 1)` into a 6x4 buffer, the second one stamped
/// under `second_mode`. The first box's mode never matters: it lands on an undefined buffer,
/// where every mode defines a cell the same way.
fn render_pair(catalog: &GlyphCatalog, origin: Pos, second_mode: StampMode) -> String {
    let size = Size {
        width: 6,
        height: 4,
    };
    let mut buffer = Buffer::new(origin, size);
    stamp_box(&mut buffer, origin, StampMode::Above);
    stamp_box(&mut buffer, Pos { x: 2, y: 1 }, second_mode);
    render(&buffer, catalog, origin, size)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let [path] = args.as_slice() {
        let text = std::fs::read_to_string(path).expect("path should be readable");
        let description: Description =
            serde_json::from_str(&text).expect("file should hold a well-formed description");
        print!("{}", description.render());
        return;
    }

    let catalog = GlyphCatalog::light();
    let origin = Pos { x: 0, y: 0 };

    let mut single = Buffer::new(
        origin,
        Size {
            width: 4,
            height: 3,
        },
    );
    stamp_box(&mut single, origin, StampMode::Above);
    let single = render(
        &single,
        &catalog,
        origin,
        Size {
            width: 4,
            height: 3,
        },
    );

    let above_pair = render_pair(&catalog, origin, StampMode::Above);
    let below_pair = render_pair(&catalog, origin, StampMode::Below);

    print!("{single}\nAbove:\n{above_pair}\nBelow:\n{below_pair}");
}
