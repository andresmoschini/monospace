//! The head fragment: an arrow's head, one chosen glyph. See _An end is an arm; a head is a
//! glyph_ in [`docs/model.md`](../../../../../docs/model.md) and
//! [ADR-0029](../../../../../docs/decisions/0029-draw-a-line-end-as-one-arm.md).

use crate::{Cell, Glyph, Pos, Shape, Surface};

/// A head: one cell written as a chosen glyph, supplied by the caller because no glyph set holds
/// a rule that points — ADR-0029, FR-027.
///
/// A fragment in the sense of _Complete and fragment_ in
/// [`docs/model.md`](../../../../../docs/model.md).
pub(crate) struct Head {
    pub(crate) at: Pos,
    pub(crate) glyph: Glyph,
}

impl Shape for Head {
    fn draw(&self, surface: &mut dyn Surface) {
        surface.stamp(self.at, Cell::Literal(self.glyph.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::Head;
    use crate::{Buffer, Cell, Glyph, Layer, Pos, Shape, Size, StampMode};

    #[test]
    fn a_head_writes_its_glyph_as_a_literal() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        let mut layer = Layer::new(&mut buffer, StampMode::Above);
        let glyph = Glyph::new("►").expect("\"►\" is one glyph");

        Head {
            at: Pos { x: 0, y: 0 },
            glyph: glyph.clone(),
        }
        .draw(&mut layer);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&Cell::Literal(glyph)));
    }
}
