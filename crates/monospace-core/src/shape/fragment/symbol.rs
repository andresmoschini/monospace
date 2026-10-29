//! The symbol fragment: one cell naming a symbol and the style it is drawn in. See _A cell can
//! name a symbol instead_ in [`docs/model.md`](../../../../../docs/model.md).
//!
//! The name is resolved at render and not here: the fragment writes the pair it is handed and the
//! catalog answers it, so a set swapped under the drawing redraws every terminal that named it.

use crate::{Cell, Pos, Shape, Surface, SymbolCell};

/// A named symbol: one cell written as a symbol and the style it is drawn in.
///
/// A fragment in the sense of _Complete and fragment_ in
/// [`docs/model.md`](../../../../../docs/model.md).
#[derive(Debug)]
pub(crate) struct NamedSymbol {
    pub(crate) at: Pos,
    pub(crate) cell: SymbolCell,
}

impl Shape for NamedSymbol {
    fn draw(&self, surface: &mut dyn Surface) {
        surface.stamp(self.at, Cell::Symbol(self.cell.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::NamedSymbol;
    use crate::{
        Buffer, Cell, CellReader, FigureLayer, Pos, Shape, Size, StampMode, Symbol, SymbolCell,
        SymbolStyleName, Window,
    };

    #[test]
    fn a_named_symbol_writes_its_pair_as_a_symbol_cell() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        let mut layer = FigureLayer::new(&mut buffer, StampMode::Above);
        let cell = SymbolCell {
            symbol: Symbol::DirectionUp,
            style: SymbolStyleName::from("solid"),
        };

        NamedSymbol {
            at: Pos { x: 0, y: 0 },
            cell: cell.clone(),
        }
        .draw(&mut layer);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&Cell::Symbol(cell)));
    }
}
