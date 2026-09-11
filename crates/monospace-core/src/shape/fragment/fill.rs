//! The fill fragment: a rectangle written as one chosen glyph. See _Pieces_ in
//! [`docs/model.md`](../../../../../docs/model.md) and
//! [ADR-0028](../../../../../docs/decisions/0028-give-each-fragment-its-own-cell-rule.md).

use super::rect;
use crate::{Cell, Glyph, Pos, Shape, Size, Surface};

/// A rectangle filled with one chosen glyph, hiding whatever a figure behind it would show.
///
/// A fragment in the sense of _Complete and fragment_ in
/// [`docs/model.md`](../../../../../docs/model.md): the glyph is a literal, per _A cell can be a
/// literal instead_, so nothing connects into it.
pub(crate) struct Fill {
    pub(crate) at: Pos,
    pub(crate) size: Size,
    pub(crate) glyph: Glyph,
}

impl Shape for Fill {
    fn draw(&self, surface: &mut dyn Surface) {
        for at in rect(self.at, self.size) {
            surface.stamp(at, Cell::Literal(self.glyph.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Fill;
    use crate::{Buffer, Cell, Glyph, Layer, Pos, Shape, Size, StampMode};

    #[test]
    fn a_fill_writes_the_glyph_at_every_position_of_its_rectangle() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 2,
                height: 2,
            },
        );
        let mut layer = Layer::new(&mut buffer, StampMode::Above);
        let glyph = Glyph::new("░").expect("\"░\" is one glyph");

        Fill {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 2,
                height: 2,
            },
            glyph: glyph.clone(),
        }
        .draw(&mut layer);

        for x in 0..2 {
            for y in 0..2 {
                assert_eq!(
                    buffer.cell(Pos { x, y }),
                    Some(&Cell::Literal(glyph.clone())),
                    "position ({x}, {y})"
                );
            }
        }
    }
}
