//! Minimal non-interactive command-line front end for Monospace.
//!
//! It holds no domain logic of its own: everything it draws comes from `monospace-core`.

use monospace_core::{
    Arm, Buffer, Cell, Glyph, GlyphCatalog, Pos, Size, StampMode, Stroke, StrokeCell, render,
};

/// Stamps the 4x3 box spec 0002 settled, with `origin` as its top-left corner and `mode` as the
/// stamp mode: `Set` along the border, `Closed` facing the interior, and `Unset` facing outward
/// so a later figure can join it rather than being refused by a border that means nothing by it.
/// The two interior positions are stamped with a literal fill, so a figure behind the box no
/// longer shows through it.
fn stamp_box(buffer: &mut Buffer, origin: Pos, mode: StampMode) {
    let light = || Stroke::from("light");
    let cell = |top, right, bottom, left| -> Cell {
        StrokeCell {
            base: light(),
            top,
            right,
            bottom,
            left,
        }
        .into()
    };
    let fill = || Cell::Literal(Glyph::new("░").expect("\"░\" is one glyph"));
    let at = |dx: i32, dy: i32| Pos {
        x: origin.x + dx,
        y: origin.y + dy,
    };

    buffer.stamp(
        at(0, 0),
        cell(Arm::Unset, Arm::Set, Arm::Set, Arm::Unset),
        mode,
    );
    buffer.stamp(
        at(1, 0),
        cell(Arm::Unset, Arm::Set, Arm::Closed, Arm::Set),
        mode,
    );
    buffer.stamp(
        at(2, 0),
        cell(Arm::Unset, Arm::Set, Arm::Closed, Arm::Set),
        mode,
    );
    buffer.stamp(
        at(3, 0),
        cell(Arm::Unset, Arm::Unset, Arm::Set, Arm::Set),
        mode,
    );
    buffer.stamp(
        at(0, 1),
        cell(Arm::Set, Arm::Closed, Arm::Set, Arm::Unset),
        mode,
    );
    buffer.stamp(at(1, 1), fill(), mode);
    buffer.stamp(at(2, 1), fill(), mode);
    buffer.stamp(
        at(3, 1),
        cell(Arm::Set, Arm::Unset, Arm::Set, Arm::Closed),
        mode,
    );
    buffer.stamp(
        at(0, 2),
        cell(Arm::Set, Arm::Set, Arm::Unset, Arm::Unset),
        mode,
    );
    buffer.stamp(
        at(1, 2),
        cell(Arm::Closed, Arm::Set, Arm::Unset, Arm::Set),
        mode,
    );
    buffer.stamp(
        at(2, 2),
        cell(Arm::Closed, Arm::Set, Arm::Unset, Arm::Set),
        mode,
    );
    buffer.stamp(
        at(3, 2),
        cell(Arm::Set, Arm::Unset, Arm::Unset, Arm::Set),
        mode,
    );
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
