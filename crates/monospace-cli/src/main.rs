//! Minimal non-interactive command-line front end for Monospace.
//!
//! It holds no domain logic of its own: everything it draws comes from `monospace-core`.

use monospace_core::{Arm, Buffer, Cell, GlyphCatalog, Pos, Size, Stroke, render};

/// Stamps the 4x3 box spec 0002 settled, with `origin` as its top-left corner: `Set` along the
/// border, `Closed` facing the interior, and `Unset` facing outward so a later figure can join it
/// rather than being refused by a border that means nothing by it.
fn stamp_box(buffer: &mut Buffer, origin: Pos) {
    let light = || Stroke::from("light");
    let cell = |top, right, bottom, left| Cell {
        base: light(),
        top,
        right,
        bottom,
        left,
    };
    let at = |dx: i32, dy: i32| Pos {
        x: origin.x + dx,
        y: origin.y + dy,
    };

    buffer.stamp(at(0, 0), cell(Arm::Unset, Arm::Set, Arm::Set, Arm::Unset));
    buffer.stamp(at(1, 0), cell(Arm::Unset, Arm::Set, Arm::Closed, Arm::Set));
    buffer.stamp(at(2, 0), cell(Arm::Unset, Arm::Set, Arm::Closed, Arm::Set));
    buffer.stamp(at(3, 0), cell(Arm::Unset, Arm::Unset, Arm::Set, Arm::Set));
    buffer.stamp(at(0, 1), cell(Arm::Set, Arm::Closed, Arm::Set, Arm::Unset));
    buffer.stamp(at(3, 1), cell(Arm::Set, Arm::Unset, Arm::Set, Arm::Closed));
    buffer.stamp(at(0, 2), cell(Arm::Set, Arm::Set, Arm::Unset, Arm::Unset));
    buffer.stamp(at(1, 2), cell(Arm::Closed, Arm::Set, Arm::Unset, Arm::Set));
    buffer.stamp(at(2, 2), cell(Arm::Closed, Arm::Set, Arm::Unset, Arm::Set));
    buffer.stamp(at(3, 2), cell(Arm::Set, Arm::Unset, Arm::Unset, Arm::Set));
}

fn main() {
    let mut buffer = Buffer::new(
        Pos { x: 0, y: 0 },
        Size {
            width: 4,
            height: 3,
        },
    );
    stamp_box(&mut buffer, Pos { x: 0, y: 0 });

    let text = render(
        &buffer,
        &GlyphCatalog::light(),
        Pos { x: 0, y: 0 },
        Size {
            width: 4,
            height: 3,
        },
    );
    print!("{text}");
}
