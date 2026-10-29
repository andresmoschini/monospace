//! The head fragment: a connector's head, one chosen glyph. See _What a terminal writes_ in
//! [`docs/model.md`](../../../../../docs/model.md).

use crate::{Cell, Glyph, Pos, Shape, Surface};

/// A head: one cell written as a chosen glyph, supplied by the caller because a terminal names a
/// literal rather than a symbol. Pointer glyphs are not missing from the catalog — `solid-pointer`
/// answers the four directions with `▲`, `►`, `▼` and `◄` — but no terminal names one yet. An arm
/// rule cannot point, and `╾` and `╼` are already claimed by keys that mean heavy on one side and
/// light on the other.
///
/// A fragment in the sense of _Complete and fragment_ in
/// [`docs/model.md`](../../../../../docs/model.md).
#[derive(Debug)]
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
    use crate::{
        Buffer, Cell, CellReader, FigureLayer, Glyph, Pos, Shape, Size, StampMode, Window,
    };

    #[test]
    fn a_head_writes_its_glyph_as_a_literal() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        let mut layer = FigureLayer::new(&mut buffer, StampMode::Above);
        let glyph = Glyph::new("►").expect("\"►\" is one glyph");

        Head {
            at: Pos { x: 0, y: 0 },
            glyph: glyph.clone(),
        }
        .draw(&mut layer);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&Cell::Literal(glyph)));
    }
}
