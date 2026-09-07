//! Minimal non-interactive command-line front end for Monospace.
//!
//! It holds no domain logic of its own: everything it draws comes from `monospace-core`.

use monospace_core::{Arm, Buffer, Cell, GlyphCatalog, Pos, Size, Stroke, render};

fn main() {
    let mut buffer = Buffer::new(
        Pos { x: 0, y: 0 },
        Size {
            width: 4,
            height: 3,
        },
    );
    let light = || Stroke::from("light");
    let corner = |top, right, bottom, left| Cell {
        base: light(),
        top,
        right,
        bottom,
        left,
    };

    buffer.stamp(
        Pos { x: 0, y: 0 },
        corner(Arm::Closed, Arm::Set, Arm::Set, Arm::Closed),
    );
    buffer.stamp(
        Pos { x: 1, y: 0 },
        corner(Arm::Closed, Arm::Set, Arm::Closed, Arm::Set),
    );
    buffer.stamp(
        Pos { x: 2, y: 0 },
        corner(Arm::Closed, Arm::Set, Arm::Closed, Arm::Set),
    );
    buffer.stamp(
        Pos { x: 3, y: 0 },
        corner(Arm::Closed, Arm::Closed, Arm::Set, Arm::Set),
    );
    buffer.stamp(
        Pos { x: 0, y: 1 },
        corner(Arm::Set, Arm::Closed, Arm::Set, Arm::Closed),
    );
    buffer.stamp(
        Pos { x: 3, y: 1 },
        corner(Arm::Set, Arm::Closed, Arm::Set, Arm::Closed),
    );
    buffer.stamp(
        Pos { x: 0, y: 2 },
        corner(Arm::Set, Arm::Set, Arm::Closed, Arm::Closed),
    );
    buffer.stamp(
        Pos { x: 1, y: 2 },
        corner(Arm::Closed, Arm::Set, Arm::Closed, Arm::Set),
    );
    buffer.stamp(
        Pos { x: 2, y: 2 },
        corner(Arm::Closed, Arm::Set, Arm::Closed, Arm::Set),
    );
    buffer.stamp(
        Pos { x: 3, y: 2 },
        corner(Arm::Set, Arm::Closed, Arm::Closed, Arm::Set),
    );

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
