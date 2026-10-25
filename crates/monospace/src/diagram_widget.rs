//! The region the diagram goes into, and the diagram inside it.
//!
//! **The box and what it holds are two things.** The box says where the diagram's place is and
//! comes from #174; the cells inside it are the diagram's, drawn into a window the size of the
//! interior and copied here one at a time.
//!
//! **The drawing is the state's, and the picture is this module's.** The buffer is rebuilt on
//! every frame at the scroll origin and kept in `DiagramState`, because a click is answered from
//! the picture the person saw rather than from a drawing made when the click arrives. See
//! _Drawing_ and _Ownership_ in `docs/diagram-model.md` §7 and §8.

use rat_theme4::StyleName;
use rat_theme4::theme::SalsaTheme;
use ratatui_widgets::block::Block;

use monospace_core::{GlyphCatalog, Offset, Size};
use ratatui_core::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::state::DiagramState;

/// Paints the region, and the diagram the state draws inside it.
pub(crate) fn draw(
    area: Rect,
    buf: &mut Buffer,
    state: &mut DiagramState,
    catalog: &GlyphCatalog,
    theme: &SalsaTheme,
) {
    draw_region(area, buf, theme);
    let interior = inner_region(area);
    state.redraw(Size {
        width: interior.width,
        height: interior.height,
    });
    draw_diagram(interior, buf, state, catalog);
}

/// Copies the drawing into the frame, one cell at a time.
///
/// **Only the cells the drawing holds are written.** The frame clears the whole screen before it
/// draws, so a cell the diagram does not speak for keeps the space the clear painted.
///
/// **`monospace_core::render` is deliberately not used.** It hands back the picture as a `String`,
/// and the buffer behind it is what answers a click; one drawing with one window is what keeps
/// those two the same thing.
fn draw_diagram(area: Rect, buf: &mut Buffer, state: &DiagramState, catalog: &GlyphCatalog) {
    if area.is_empty() {
        return;
    }

    for y in 0..area.height {
        for x in 0..area.width {
            let glyph = state.glyph(Offset { x, y }, catalog);

            if let Some(glyph) = glyph {
                buf[(area.x + x, area.y + y)].set_symbol(glyph);
            }
        }
    }
}

/// Paints the region the diagram goes into.
///
/// A box needs two rows, one for its top and one for its bottom, and a region that is shorter
/// than that is not drawn at all: the frame clears it, and what a previous frame left in it is
/// gone.
fn draw_region(area: Rect, buf: &mut Buffer, theme: &SalsaTheme) {
    if area.height < 2 {
        return;
    }

    let border: Style = theme.style(Style::CONTAINER_BORDER_FG);
    Block::bordered().style(border).render(area, buf);
}

fn inner_region(area: Rect) -> Rect {
    let Rect {
        x,
        y,
        width,
        height,
    } = area;
    Rect {
        x: x + 1,
        y: y + 1,
        width: width.saturating_sub(2),
        height: height.saturating_sub(2),
    }
}
