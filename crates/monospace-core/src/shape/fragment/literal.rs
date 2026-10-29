//! The literal fragment: one chosen glyph, spelled out and answered by no set. See _A cell can be
//! a literal instead_ in [`docs/model.md`](../../../../../docs/model.md).

use crate::{Cell, Glyph, Pos, Shape, Surface};

/// A literal: one cell written as a glyph the caller spelled out.
///
/// A fragment in the sense of _Complete and fragment_ in
/// [`docs/model.md`](../../../../../docs/model.md).
#[derive(Debug)]
pub(crate) struct Literal {
    pub(crate) at: Pos,
    pub(crate) glyph: Glyph,
}

impl Shape for Literal {
    fn draw(&self, surface: &mut dyn Surface) {
        surface.stamp(self.at, Cell::Literal(self.glyph.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::Literal;
    use crate::{
        Buffer, Cell, CellReader, FigureLayer, Glyph, Pos, Shape, Size, StampMode, Window,
    };

    #[test]
    fn a_literal_writes_its_glyph_as_a_literal_cell() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        let mut layer = FigureLayer::new(&mut buffer, StampMode::Above);
        let glyph = Glyph::new("►").expect("\"►\" is one glyph");

        Literal {
            at: Pos { x: 0, y: 0 },
            glyph: glyph.clone(),
        }
        .draw(&mut layer);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&Cell::Literal(glyph)));
    }
}
