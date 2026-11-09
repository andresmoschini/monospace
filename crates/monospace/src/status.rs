//! The contextual status and command bar along the bottom.
//!
//! **Two rows without box borders of their own** (D2 of #279): row 0 renders the contextual
//! status blocks, and row 1 renders the available command shortcuts in the current context.
//!
//! **In IDLE**, the upper row reports the application name and version, the drawing's footprint,
//! the count of shapes, and the viewport origin; the lower row offers
//! `CREATE: [b] Box [l] Line [c] Connector · SELECT: [Tab] Next [Shift+Tab] Prev [Click] Pick`
//! when the diagram holds at least one figure and `CREATE: [b] Box [l] Line [c] Connector` when it
//! has no figures.
//!
//! **When a shape is selected**, the upper row reports its identity and properties (`Box`, `Line`,
//! or `Connector`). For a box or line, the lower row offers `MOVE: ↑↓←→`, with `RESIZE` between
//! `MOVE` and `TEXT` or `STROKE` offering the cursor keys the figure can use — all four for a box
//! (`Shift+↑↓←→`), the row's two for a horizontal line (`Shift+←→`), and the column's two for a
//! vertical line (`Shift+↑↓`). `TEXT: [t] Edit` is offered only for a box. For a connector,
//! whole-shape movement and resizing are absent, and the lower row offers
//! `ENDPOINTS: [1] From [2] To · INVERT: [i]` instead. All selected figures conclude with
//! `STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect`.
//!
//! **In `STROKE_SELECT`**, the upper row reports the figure and the stroke it carries, and the
//! lower row offers the five catalog strokes and `[Esc] Cancel`. **In `ORDER_MODE`**, the upper
//! row reports the figure's place in the order and the lower row offers the four order moves and
//! `[Esc] Done`. **In `ENDPOINT_EDIT`**, the upper row inspects the focused endpoint, and the lower
//! row offers offset/standoff adjustment or detached movement, resetting, detachment, terminal
//! selection, endpoint switching, and `[Esc] Done`. **In `TERMINAL_SELECT`**, the lower row offers
//! the five terminal styles and `[Esc] Cancel`. **In `LEAVING_SELECT`**, the lower row offers the
//! four compass departure directions and `[Esc] Cancel`.

use monospace_core::{Direction, Footprint, Orientation, Pos, ShapeId, Terminal};
use monospace_diagram::{Anchor, Diagram, Endpoint, EndpointAt, Shape};
use rat_theme4::StyleName;
use rat_theme4::theme::SalsaTheme;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;

use crate::diagram_state::{DiagramState, End, SubMode};

/// The name the application answers to.
pub(crate) const NAME: &str = "monospace";

/// The version of the application.
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

/// What the bar says about a drawing that has reached nothing yet.
const NOTHING_DRAWN: &str = "nothing drawn";

/// The separator between contextual blocks.
const SEPARATOR: &str = " · ";

/// How far the drawing reaches, as the bar says it: `46×13 at 0,0` or `4×3 at -4,-3`.
///
/// **The width and height are derived here rather than on [`Footprint`]**, because the span does not
/// fit a [`Size`] when a drawing reaches across the whole plane and only a caller that knows what
/// to do about it can decide. The arithmetic is `i32`: a `Pos` is `i16` and `max - min + 1` is
/// `i32` at most, so no corner of the plane overflows the sum.
///
/// [`Size`]: monospace_core::Size
fn footprint_text(footprint: Option<Footprint>) -> String {
    let Some(footprint) = footprint else {
        return NOTHING_DRAWN.to_owned();
    };

    let width = i32::from(footprint.max.x) - i32::from(footprint.min.x) + 1;
    let height = i32::from(footprint.max.y) - i32::from(footprint.min.y) + 1;

    format!(
        "{width}×{height} at {x},{y}",
        x = footprint.min.x,
        y = footprint.min.y
    )
}

fn connector_endpoint(shape: &Shape, end: End) -> Option<(u8, &'static str, &Endpoint)> {
    match (shape, end) {
        (Shape::Connector { from, .. }, End::From) => Some((1, "From", from)),
        (Shape::Connector { to, .. }, End::To) => Some((2, "To", to)),
        _ => None,
    }
}

fn anchor_text(anchor: Anchor) -> &'static str {
    match anchor {
        Anchor::Top => "top",
        Anchor::Right => "right",
        Anchor::Bottom => "bottom",
        Anchor::Left => "left",
    }
}

fn direction_text(direction: Direction) -> &'static str {
    match direction {
        Direction::Up => "up",
        Direction::Right => "right",
        Direction::Down => "down",
        Direction::Left => "left",
    }
}

fn orientation_text(orientation: Orientation) -> &'static str {
    match orientation {
        Orientation::Horizontal => "Horizontal",
        Orientation::Vertical => "Vertical",
    }
}

fn selected_shape_line(id: ShapeId, shape: &Shape, diagram: &Diagram) -> String {
    match shape {
        Shape::Box {
            at,
            size,
            stroke,
            fill,
            text,
        } => {
            let label = match text {
                None => format!("Box #{id}"),
                Some(t) => {
                    let first_line = t.lines().first().map_or(&[][..], |v| &v[..]);
                    let summary = if first_line.len() <= 20 && t.lines().len() <= 1 {
                        first_line
                            .iter()
                            .map(monospace_core::Glyph::as_str)
                            .collect::<String>()
                    } else {
                        let truncated = first_line
                            .iter()
                            .take(20)
                            .map(monospace_core::Glyph::as_str)
                            .collect::<String>();
                        format!("{truncated}…")
                    };
                    format!(r#"Box #{id} "{summary}""#)
                }
            };
            let pos = format!("Pos: ({}, {})", at.x, at.y);
            let size = format!("Size: {}×{}", size.width, size.height);
            let stroke = format!("Stroke: {}", stroke.0);
            let fill = match fill {
                None => "Fill: None".to_owned(),
                Some(g) => format!("Fill: '{}'", g.as_str()),
            };
            [label, pos, size, stroke, fill].join(SEPARATOR)
        }
        Shape::Line {
            at,
            len,
            orientation,
            stroke,
        } => {
            let identity = format!("Line #{id}");
            let pos = format!("Pos: ({}, {})", at.x, at.y);
            let length = format!("Length: {len}");
            let orientation = format!("Orientation: {}", orientation_text(*orientation));
            let stroke = format!("Stroke: {}", stroke.0);
            [identity, pos, length, orientation, stroke].join(SEPARATOR)
        }
        Shape::Connector { from, to, stroke } => {
            let identity = format!("Connector #{id}");
            let stroke = format!("Stroke: {}", stroke.0);
            let from = format!("1. From: {}", endpoint_text(from, diagram));
            let to = format!("2. To: {}", endpoint_text(to, diagram));
            [identity, stroke, from, to].join(SEPARATOR)
        }
    }
}

/// Formats the contextual status row (row 0) from the diagram state.
pub(crate) fn status_line(diagram_state: &DiagramState) -> String {
    let session = diagram_state.session();
    let diagram = session.diagram();

    let selected_shape = diagram_state.selected_entry();

    if let Some(sub_mode) = diagram_state.sub_mode()
        && let Some((id, shape)) = selected_shape
    {
        let (kind, current_stroke) = match shape {
            Shape::Box { stroke, .. } => ("Box", stroke.0.as_str()),
            Shape::Line { stroke, .. } => ("Line", stroke.0.as_str()),
            Shape::Connector { stroke, .. } => ("Connector", stroke.0.as_str()),
        };

        return match sub_mode {
            SubMode::StrokeSelect => {
                format!("STROKE SELECTION for {kind} #{id} · Current: {current_stroke}")
            }
            SubMode::OrderMode => {
                let Some(place) = diagram.place_of(id) else {
                    return selected_shape_line(id, shape, diagram);
                };
                let total = diagram.shape_count();
                let position = place + 1;
                let behind = position - 1;
                let ahead = total - position;

                format!(
                    "LAYER ORDER for {kind} #{id} · Position: {position} of {total} · Behind: {behind}, Ahead: {ahead}"
                )
            }
            SubMode::Endpoint(end) => {
                let Some((n, label, endpoint)) = connector_endpoint(shape, end) else {
                    return selected_shape_line(id, shape, diagram);
                };
                let at_text = endpoint_at_text(&endpoint.at, diagram);
                let term_text = terminal_text(endpoint);
                format!("ENDPOINT {n}. {label} of Connector #{id} · {at_text} · Term: {term_text}")
            }
            SubMode::TerminalSelect(end) => {
                let Some((n, label, endpoint)) = connector_endpoint(shape, end) else {
                    return selected_shape_line(id, shape, diagram);
                };
                let term_text = terminal_text(endpoint);
                format!("TERMINAL for {n}. {label} of Connector #{id} · Current: {term_text}")
            }
            SubMode::LeavingSelect(end) => {
                let Some((n, label, endpoint)) = connector_endpoint(shape, end) else {
                    return selected_shape_line(id, shape, diagram);
                };
                let (pos_text, leaving) = match endpoint.at {
                    EndpointAt::Absolute { at, leaving } => {
                        (format!("Detached at ({}, {})", at.x, at.y), leaving)
                    }
                    EndpointAt::Reference(ref r) => {
                        let pos = endpoint.resolve(diagram).unwrap_or(Pos { x: 0, y: 0 });
                        (
                            format!("Attached at ({}, {})", pos.x, pos.y),
                            r.anchor.leaving(),
                        )
                    }
                };
                let dir = direction_text(leaving);
                format!("LEAVING for {n}. {label} of Connector #{id} · {pos_text} · Current: {dir}")
            }
        };
    }

    if let Some((id, shape)) = selected_shape {
        selected_shape_line(id, shape, diagram)
    } else {
        let name_version = format!("{NAME} {VERSION}");
        let footprint = format!("Footprint: {}", footprint_text(diagram_state.footprint()));
        let count = diagram.shapes().count();
        let count_text = if count == 1 {
            "1 shape".to_owned()
        } else {
            format!("{count} shapes")
        };
        let origin = diagram_state.origin();
        let viewport = format!("Viewport: ({}, {})", origin.x, origin.y);
        [name_version, footprint, count_text, viewport].join(SEPARATOR)
    }
}

fn endpoint_at_text(at: &EndpointAt, diagram: &Diagram) -> String {
    match at {
        EndpointAt::Reference(r) => {
            let kind = match diagram.get(r.id) {
                Some(Shape::Box { .. }) => "Box",
                Some(Shape::Line { .. }) => "Line",
                Some(Shape::Connector { .. }) => "Connector",
                None => "Shape",
            };
            format!(
                "Attached to {kind} #{id} at {anchor}, offset ({dx}, {dy}), out {out}",
                id = r.id,
                anchor = anchor_text(r.anchor),
                dx = r.offset.dx,
                dy = r.offset.dy,
                out = r.out
            )
        }
        EndpointAt::Absolute { at, leaving } => {
            format!(
                "Detached at ({}, {}), leaving {}",
                at.x,
                at.y,
                direction_text(*leaving)
            )
        }
    }
}

fn endpoint_at_compact(at: &EndpointAt, diagram: &Diagram) -> String {
    match at {
        EndpointAt::Reference(r) => {
            let kind = match diagram.get(r.id) {
                Some(Shape::Box { .. }) => "Box",
                Some(Shape::Line { .. }) => "Line",
                Some(Shape::Connector { .. }) => "Connector",
                None => "Shape",
            };
            format!(
                "{kind} #{id} [{anchor} ({dx}, {dy}) out:{out}]",
                id = r.id,
                anchor = anchor_text(r.anchor),
                dx = r.offset.dx,
                dy = r.offset.dy,
                out = r.out
            )
        }
        EndpointAt::Absolute { at, leaving } => {
            format!("({}, {}) [{}]", at.x, at.y, direction_text(*leaving))
        }
    }
}

fn endpoint_text(endpoint: &Endpoint, diagram: &Diagram) -> String {
    let at_text = endpoint_at_compact(&endpoint.at, diagram);
    let term_text = terminal_text(endpoint);
    format!("{at_text} ({term_text})")
}

static CATALOG: std::sync::LazyLock<monospace_core::GlyphCatalog> =
    std::sync::LazyLock::new(monospace_glyph_sets::all);

fn terminal_text(endpoint: &Endpoint) -> String {
    let leaving = endpoint.leaving();
    match &endpoint.terminal {
        Terminal::Arm => {
            let glyph = match leaving {
                Direction::Left | Direction::Right => "─",
                Direction::Up | Direction::Down => "│",
            };
            format!("arm {glyph}")
        }
        Terminal::Glyph { glyph } => {
            format!("glyph {}", glyph.as_str())
        }
        Terminal::Symbol { style } => {
            let pointing = match leaving {
                Direction::Up => monospace_core::Symbol::DirectionDown,
                Direction::Down => monospace_core::Symbol::DirectionUp,
                Direction::Left => monospace_core::Symbol::DirectionRight,
                Direction::Right => monospace_core::Symbol::DirectionLeft,
            };
            let key = monospace_core::SymbolCell {
                symbol: pointing,
                style: style.clone(),
            };
            if let Some(glyph) = CATALOG.symbol(&key) {
                format!("{name} {glyph}", name = style.0, glyph = glyph.as_str())
            } else {
                style.0.clone()
            }
        }
    }
}

/// Formats the contextual command row (row 1) from the diagram state.
pub(crate) fn command_line(diagram_state: &DiagramState) -> String {
    crate::context_table::format_command_line(diagram_state)
}

/// Paints the contextual status and command bar.
///
/// **Two clean rows without box borders of their own** (D2 of #279): row 0 renders the contextual
/// status and row 1 renders the available commands, both painted using the theme's status base
/// style.
pub(crate) fn draw(area: Rect, buf: &mut Buffer, diagram: &DiagramState, theme: &SalsaTheme) {
    if area.is_empty() {
        return;
    }

    let style: Style = theme.style(Style::STATUS_BASE);
    buf.set_style(area, style);

    let status = status_line(diagram);
    buf.set_stringn(area.left(), area.top(), status, area.width as usize, style);

    if area.height >= 2 {
        let command = command_line(diagram);
        buf.set_stringn(
            area.left(),
            area.top() + 1,
            command,
            area.width as usize,
            style,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use monospace_core::{Pos, ShapeId};
    use std::num::NonZeroU16;

    /// The state of a region that has drawn a frame, over the diagram `json` describes.
    fn a_state_over(json: &str) -> DiagramState {
        let diagram =
            monospace_description::parse(json).expect("the test's description is readable");
        let mut state = DiagramState::new(monospace_editing::Session::new(diagram));
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));
        crate::diagram_widget::draw(
            Rect::new(0, 0, 80, 24),
            &mut buffer,
            &mut state,
            &monospace_glyph_sets::all(),
            &Config::default().theme,
        );
        state
    }

    /// The bar says the drawing's width, height and corner, and the numbers are the footprint's:
    /// `max - min + 1` on each axis, so a one-cell drawing is `1×1` and not `0×0`.
    #[test]
    fn the_bar_says_how_far_the_drawing_reaches() {
        let text = footprint_text(Some(Footprint {
            min: Pos { x: 0, y: 0 },
            max: Pos { x: 45, y: 12 },
        }));

        assert_eq!(text, "46×13 at 0,0");
    }

    /// A drawing at a negative corner says so and measures four by three rather than counting from
    /// the origin: `min` is where the drawing starts, not how far it is from `(0, 0)`.
    #[test]
    fn the_bar_says_a_corner_before_the_origin_as_it_is() {
        let text = footprint_text(Some(Footprint {
            min: Pos { x: -4, y: -3 },
            max: Pos { x: -1, y: -1 },
        }));

        assert_eq!(text, "4×3 at -4,-3");
    }

    /// One stamp is `1×1` at its own position, which is the case a `max - min` without the `+ 1`
    /// would report as nothing at all.
    #[test]
    fn a_single_stamped_cell_is_one_by_one() {
        let at = Pos { x: 7, y: 2 };

        assert_eq!(
            footprint_text(Some(Footprint { min: at, max: at })),
            "1×1 at 7,2"
        );
    }

    /// The bar may be drawn before anything has been stamped, and says so rather than showing a
    /// rectangle at the origin that would be indistinguishable from a drawing that put one cell
    /// there.
    #[test]
    fn nothing_drawn_is_said_in_words() {
        assert_eq!(footprint_text(None), NOTHING_DRAWN);
    }

    /// **The idle bar reports the application identity, footprint, shape count and viewport.**
    #[test]
    fn idle_state_shows_version_footprint_shape_count_viewport_and_click_command() {
        let mut empty_state = DiagramState::new(monospace_editing::Session::new(
            monospace_diagram::Diagram::new(),
        ));
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));
        crate::diagram_widget::draw(
            Rect::new(0, 0, 80, 24),
            &mut buffer,
            &mut empty_state,
            &monospace_glyph_sets::all(),
            &Config::default().theme,
        );

        assert_eq!(
            status_line(&empty_state),
            format!("{NAME} {VERSION} · Footprint: nothing drawn · 0 shapes · Viewport: (0, 0)")
        );
        assert_eq!(
            command_line(&empty_state),
            "CREATE: [b] Box [l] Line [c] Connector"
        );

        let single_box = a_state_over(
            r#"{
            "next_id": 2,
            "shapes": [ { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                          "size": { "width": 46, "height": 13 }, "stroke": "light" } ]
        }"#,
        );
        assert_eq!(
            status_line(&single_box),
            format!("{NAME} {VERSION} · Footprint: 46×13 at 0,0 · 1 shape · Viewport: (0, 0)")
        );
        assert_eq!(
            command_line(&single_box),
            "CREATE: [b] Box [l] Line [c] Connector · SELECT: [Tab] Next [Shift+Tab] Prev [Click] Pick"
        );

        let two_shapes = a_state_over(
            r#"{
            "next_id": 3,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                  "size": { "width": 4, "height": 3 }, "stroke": "light" },
                { "kind": "box", "id": 2, "at": { "x": 10, "y": 0 },
                  "size": { "width": 4, "height": 3 }, "stroke": "light" }
            ]
        }"#,
        );
        assert_eq!(
            status_line(&two_shapes),
            format!("{NAME} {VERSION} · Footprint: 14×3 at 0,0 · 2 shapes · Viewport: (0, 0)")
        );
    }

    /// **A selection whose identity is not in the diagram renders as idle.**
    #[test]
    fn a_selection_not_held_by_the_diagram_renders_as_idle() {
        let mut state = a_state_over(
            r#"{
            "next_id": 2,
            "shapes": [ { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                          "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
        }"#,
        );
        let unrecorded = ShapeId::new(NonZeroU16::new(99).unwrap());
        state.select(Some(unrecorded));

        assert_eq!(
            status_line(&state),
            format!("{NAME} {VERSION} · Footprint: 4×3 at 0,0 · 1 shape · Viewport: (0, 0)")
        );
        assert_eq!(
            command_line(&state),
            "CREATE: [b] Box [l] Line [c] Connector · SELECT: [Tab] Next [Shift+Tab] Prev [Click] Pick"
        );
    }

    /// **A selected box formats its identity, optional label, pos, size, stroke, and fill.**
    #[test]
    fn selected_box_shows_identity_label_pos_size_stroke_fill_and_selection_commands() {
        let mut state = a_state_over(
            r#"{
            "next_id": 2,
            "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                          "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
        }"#,
        );
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        assert_eq!(
            status_line(&state),
            "Box #1 · Pos: (2, 1) · Size: 4×3 · Stroke: light · Fill: None"
        );
        assert_eq!(
            command_line(&state),
            "MOVE: ↑↓←→ · RESIZE: Shift+↑↓←→ · TEXT: [t] Edit · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect"
        );

        let mut state = a_state_over(
            r#"{
            "next_id": 2,
            "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                          "size": { "width": 4, "height": 3 }, "stroke": "light",
                          "fill": "░", "text": "API" } ]
        }"#,
        );
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        assert_eq!(
            status_line(&state),
            "Box #1 \"API\" · Pos: (2, 1) · Size: 4×3 · Stroke: light · Fill: '░'"
        );

        let mut state = a_state_over(
            r#"{
            "next_id": 2,
            "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                          "size": { "width": 10, "height": 4 }, "stroke": "light",
                          "text": "API\nrest" } ]
        }"#,
        );
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        assert_eq!(
            status_line(&state),
            "Box #1 \"API…\" · Pos: (2, 1) · Size: 10×4 · Stroke: light · Fill: None"
        );

        let mut state = a_state_over(
            r#"{
            "next_id": 2,
            "shapes": [ { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                          "size": { "width": 25, "height": 3 }, "stroke": "light",
                          "text": "12345678901234567890extra" } ]
        }"#,
        );
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        assert_eq!(
            status_line(&state),
            "Box #1 \"12345678901234567890…\" · Pos: (0, 0) · Size: 25×3 · Stroke: light · Fill: None"
        );
    }

    /// **A selected line formats its identity, pos, length, orientation, and stroke.**
    #[test]
    fn selected_line_shows_identity_pos_length_orientation_stroke_and_selection_commands() {
        let mut state = a_state_over(
            r#"{
            "next_id": 2,
            "shapes": [ { "kind": "line", "id": 1, "at": { "x": 0, "y": 5 },
                          "len": 8, "orientation": "horizontal", "stroke": "light" } ]
        }"#,
        );
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        assert_eq!(
            status_line(&state),
            "Line #1 · Pos: (0, 5) · Length: 8 · Orientation: Horizontal · Stroke: light"
        );
        assert_eq!(
            command_line(&state),
            "MOVE: ↑↓←→ · RESIZE: Shift+←→ · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect"
        );
    }

    /// **A selected vertical line offers the vertical resize** (D1 of #301).
    #[test]
    fn selected_vertical_line_offers_the_vertical_resize() {
        let mut state = a_state_over(
            r#"{
            "next_id": 2,
            "shapes": [ { "kind": "line", "id": 1, "at": { "x": 0, "y": 5 },
                          "len": 8, "orientation": "vertical", "stroke": "light" } ]
        }"#,
        );
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        assert_eq!(
            status_line(&state),
            "Line #1 · Pos: (0, 5) · Length: 8 · Orientation: Vertical · Stroke: light"
        );
        assert_eq!(
            command_line(&state),
            "MOVE: ↑↓←→ · RESIZE: Shift+↑↓ · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect"
        );
    }

    /// **A selected connector formats identity, stroke before both endpoints, and commands.**
    #[test]
    fn selected_connector_shows_stroke_before_both_endpoints_and_selection_commands() {
        let mut state = a_state_over(
            r#"{
            "next_id": 4,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                  "size": { "width": 4, "height": 3 }, "stroke": "light" },
                { "kind": "box", "id": 2, "at": { "x": 10, "y": 0 },
                  "size": { "width": 4, "height": 3 }, "stroke": "light" },
                { "kind": "connector", "id": 3,
                  "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right", "offset": { "dx": 0, "dy": 0 }, "out": 0 },
                            "terminal": { "kind": "arm" } },
                  "to":   { "at": { "kind": "reference", "shape": 2, "anchor": "bottom", "offset": { "dx": 1, "dy": 0 }, "out": 1 },
                            "terminal": { "kind": "arm" } },
                  "stroke": "light" }
            ]
        }"#,
        );
        state.select(Some(ShapeId::new(NonZeroU16::new(3).unwrap())));
        assert_eq!(
            status_line(&state),
            "Connector #3 · Stroke: light · 1. From: Box #1 [right (0, 0) out:0] (arm ─) · 2. To: Box #2 [bottom (1, 0) out:1] (arm │)"
        );
        assert_eq!(
            command_line(&state),
            "ENDPOINTS: [1] From [2] To · INVERT: [i] · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect"
        );

        let mut state = a_state_over(
            r#"{
            "next_id": 2,
            "shapes": [
                { "kind": "connector", "id": 1,
                  "from": { "at": { "kind": "point", "x": 1, "y": 2 }, "leaving": "right",
                            "terminal": { "kind": "arm" } },
                  "to":   { "at": { "kind": "point", "x": 5, "y": 6 }, "leaving": "left",
                            "terminal": { "kind": "arm" } },
                  "stroke": "light" }
            ]
        }"#,
        );
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        assert_eq!(
            status_line(&state),
            "Connector #1 · Stroke: light · 1. From: (1, 2) [right] (arm ─) · 2. To: (5, 6) [left] (arm ─)"
        );

        let mut state = a_state_over(
            r#"{
            "next_id": 4,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 3 }, "stroke": "light" },
                { "kind": "box", "id": 2, "at": { "x": 20, "y": 0 }, "size": { "width": 6, "height": 3 }, "stroke": "light" },
                { "kind": "connector", "id": 3,
                  "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right", "offset": { "dx": 0, "dy": 0 }, "out": 0 },
                            "terminal": { "kind": "arm" } },
                  "to":   { "at": { "kind": "reference", "shape": 2, "anchor": "left", "offset": { "dx": 0, "dy": 0 }, "out": 0 },
                            "terminal": { "kind": "symbol", "style": "solid" } },
                  "stroke": "light" }
            ]
        }"#,
        );
        state.select(Some(ShapeId::new(NonZeroU16::new(3).unwrap())));
        assert_eq!(
            status_line(&state),
            "Connector #3 · Stroke: light · 1. From: Box #1 [right (0, 0) out:0] (arm ─) · 2. To: Box #2 [left (0, 0) out:0] (solid ▶)"
        );
    }

    #[test]
    fn selected_connector_omits_move_and_starts_with_endpoints() {
        let text = r#"{
            "next_id": 2,
            "shapes": [
                { "kind": "connector", "id": 1,
                  "from": { "at": { "kind": "point", "x": 1, "y": 2 }, "leaving": "right",
                            "terminal": { "kind": "arm" } },
                  "to":   { "at": { "kind": "point", "x": 5, "y": 6 }, "leaving": "left",
                            "terminal": { "kind": "arm" } },
                  "stroke": "light" }
            ]
        }"#;

        let mut state = a_state_over(text);
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        assert_eq!(
            command_line(&state),
            "ENDPOINTS: [1] From [2] To · INVERT: [i] · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect"
        );
    }

    /// **The status bar paints status on the top row and commands on the bottom row.**
    #[test]
    fn status_bar_paints_status_on_top_row_and_commands_on_bottom_row() {
        let state = a_state_over(
            r#"{
            "next_id": 2,
            "shapes": [ { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                          "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
        }"#,
        );
        let area = Rect::new(0, 0, 100, 2);
        let mut buffer = Buffer::empty(area);
        draw(area, &mut buffer, &state, &Config::default().theme);

        let row0: String = (0..area.width)
            .map(|col| buffer[(col, 0)].symbol().to_string())
            .collect();
        let row1: String = (0..area.width)
            .map(|col| buffer[(col, 1)].symbol().to_string())
            .collect();

        assert!(row0.starts_with(&status_line(&state)));
        assert!(row1.starts_with(&command_line(&state)));

        let narrow = Rect::new(0, 0, 10, 2);
        let mut narrow_buf = Buffer::empty(narrow);
        draw(narrow, &mut narrow_buf, &state, &Config::default().theme);

        let narrow0: String = (0..narrow.width)
            .map(|col| narrow_buf[(col, 0)].symbol().to_string())
            .collect();
        assert_eq!(narrow0.len(), 10);
        assert_eq!(narrow0, status_line(&state)[..10]);
    }

    /// **The command row offers RESIZE for a box and a line** (rule 10 of #284, D1 of #301).
    #[test]
    fn the_command_row_offers_resize_for_a_box_and_a_line() {
        let text = r#"{
            "next_id": 5,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                  "size": { "width": 4, "height": 3 }, "stroke": "light" },
                { "kind": "line", "id": 2, "at": { "x": 0, "y": 5 },
                  "len": 8, "orientation": "horizontal", "stroke": "light" },
                { "kind": "line", "id": 3, "at": { "x": 5, "y": 0 },
                  "len": 6, "orientation": "vertical", "stroke": "light" },
                { "kind": "connector", "id": 4,
                  "from": { "at": { "kind": "point", "x": 1, "y": 2 }, "leaving": "right",
                            "terminal": { "kind": "arm" } },
                  "to":   { "at": { "kind": "point", "x": 5, "y": 6 }, "leaving": "left",
                            "terminal": { "kind": "arm" } },
                  "stroke": "light" }
            ]
        }"#;

        let mut state = a_state_over(text);
        assert_eq!(
            command_line(&state),
            "CREATE: [b] Box [l] Line [c] Connector · SELECT: [Tab] Next [Shift+Tab] Prev [Click] Pick"
        );

        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        assert_eq!(
            command_line(&state),
            "MOVE: ↑↓←→ · RESIZE: Shift+↑↓←→ · TEXT: [t] Edit · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect"
        );

        state.select(Some(ShapeId::new(NonZeroU16::new(2).unwrap())));
        assert_eq!(
            command_line(&state),
            "MOVE: ↑↓←→ · RESIZE: Shift+←→ · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect"
        );

        state.select(Some(ShapeId::new(NonZeroU16::new(3).unwrap())));
        assert_eq!(
            command_line(&state),
            "MOVE: ↑↓←→ · RESIZE: Shift+↑↓ · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect"
        );

        state.select(Some(ShapeId::new(NonZeroU16::new(4).unwrap())));
        assert_eq!(
            command_line(&state),
            "ENDPOINTS: [1] From [2] To · INVERT: [i] · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect"
        );
    }

    #[test]
    fn stroke_selection_sub_mode_shows_title_and_stroke_options() {
        let mut state = a_state_over(
            r#"{
            "next_id": 2,
            "shapes": [ { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 },
                          "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
        }"#,
        );
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        state.set_sub_mode(Some(crate::diagram_state::SubMode::StrokeSelect));

        assert_eq!(
            status_line(&state),
            "STROKE SELECTION for Box #1 · Current: light"
        );
        assert_eq!(
            command_line(&state),
            "STROKE: [1] light ┌─┐ [2] double ╔═╗ [3] light-round ╭─╮ [4] heavy ┏━┓ [5] ascii +-+ · [Esc] Cancel"
        );
    }

    /// **`ORDER_MODE` reports the layer position and offers the four moves** (D4 of #305).
    #[test]
    fn order_mode_shows_the_layer_position_and_the_order_commands() {
        let mut state = a_state_over(
            r#"{
            "next_id": 4,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                  "size": { "width": 4, "height": 3 }, "stroke": "light" },
                { "kind": "box", "id": 2, "at": { "x": 2, "y": 0 },
                  "size": { "width": 4, "height": 3 }, "stroke": "light" },
                { "kind": "box", "id": 3, "at": { "x": 4, "y": 0 },
                  "size": { "width": 4, "height": 3 }, "stroke": "light" }
            ]
        }"#,
        );
        state.select(Some(ShapeId::new(NonZeroU16::new(2).unwrap())));
        state.set_sub_mode(Some(SubMode::OrderMode));

        assert_eq!(
            status_line(&state),
            "LAYER ORDER for Box #2 · Position: 2 of 3 · Behind: 1, Ahead: 1"
        );
        assert_eq!(
            command_line(&state),
            "ORDER: [Home] Front [End] Back [PgUp] Forward [PgDn] Backward · [Esc] Done"
        );
    }

    #[test]
    fn endpoint_sub_mode_shows_focused_endpoint_inspector_and_switch_commands() {
        let mut state = a_state_over(
            r#"{
            "next_id": 3,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 3 }, "stroke": "light" },
                { "kind": "connector", "id": 2,
                  "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right", "offset": { "dx": 0, "dy": 0 }, "out": 0 },
                            "terminal": { "kind": "arm" } },
                  "to":   { "at": { "kind": "point", "x": 20, "y": 5 }, "leaving": "left",
                            "terminal": { "kind": "symbol", "style": "solid" } },
                  "stroke": "light" }
            ]
        }"#,
        );
        state.select(Some(ShapeId::new(NonZeroU16::new(2).unwrap())));

        state.set_sub_mode(Some(SubMode::Endpoint(End::From)));
        assert_eq!(
            status_line(&state),
            "ENDPOINT 1. From of Connector #2 · Attached to Box #1 at right, offset (0, 0), out 0 · Term: arm ─"
        );
        assert_eq!(
            command_line(&state),
            "OFFSET: ↑↓←→ · OUT: [+] [-] · [0] Detach · TERMINAL: [t] · [2] To · [Esc] Done"
        );

        state.set_sub_mode(Some(SubMode::Endpoint(End::To)));
        assert_eq!(
            status_line(&state),
            "ENDPOINT 2. To of Connector #2 · Detached at (20, 5), leaving left · Term: solid ▶"
        );
        assert_eq!(
            command_line(&state),
            "MOVE: ↑↓←→ · LEAVING: [d] · TERMINAL: [t] · [1] From · [Esc] Done"
        );
    }

    #[test]
    fn endpoint_sub_mode_offers_offset_out_and_conditional_reset() {
        let text = r#"{
            "next_id": 4,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 4 }, "stroke": "light" },
                { "kind": "box", "id": 2, "at": { "x": 20, "y": 0 }, "size": { "width": 8, "height": 4 }, "stroke": "light" },
                { "kind": "connector", "id": 3,
                  "from": { "at": { "kind": "reference", "shape": 1, "anchor": "top", "offset": { "dx": 0, "dy": 0 }, "out": 0 },
                            "terminal": { "kind": "arm" } },
                  "to":   { "at": { "kind": "reference", "shape": 2, "anchor": "left", "offset": { "dx": 0, "dy": 1 }, "out": 2 },
                            "terminal": { "kind": "symbol", "style": "solid" } },
                  "stroke": "light" }
            ]
        }"#;

        let mut state = a_state_over(text);
        state.select(Some(ShapeId::new(NonZeroU16::new(3).unwrap())));

        // From: Top anchor, offset 0, out 0 -> OFFSET: ↑↓←→ · OUT: [+] [-] · [0] Detach · TERMINAL: [t] · [2] To · [Esc] Done (no reset)
        state.set_sub_mode(Some(SubMode::Endpoint(End::From)));
        assert_eq!(
            command_line(&state),
            "OFFSET: ↑↓←→ · OUT: [+] [-] · [0] Detach · TERMINAL: [t] · [2] To · [Esc] Done"
        );

        // To: Left anchor, offset 1, out 2 -> OFFSET: ↑↓←→ · OUT: [+] [-] · [r] Reset · [0] Detach · TERMINAL: [t] · [1] From · [Esc] Done
        state.set_sub_mode(Some(SubMode::Endpoint(End::To)));
        assert_eq!(
            command_line(&state),
            "OFFSET: ↑↓←→ · OUT: [+] [-] · [r] Reset · [0] Detach · TERMINAL: [t] · [1] From · [Esc] Done"
        );
    }

    #[test]
    fn terminal_select_sub_mode_inspects_active_endpoint_and_current_terminal() {
        let text = r#"{
            "next_id": 3,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 3 }, "stroke": "light" },
                { "kind": "connector", "id": 2,
                  "from": { "at": { "kind": "reference", "shape": 1, "anchor": "top", "offset": { "dx": 0, "dy": 0 }, "out": 0 },
                            "terminal": { "kind": "symbol", "style": "solid" } },
                  "to":   { "at": { "kind": "point", "x": 20, "y": 5 }, "leaving": "left",
                            "terminal": { "kind": "glyph", "glyph": "◆" } },
                  "stroke": "light" }
            ]
        }"#;

        let mut state = a_state_over(text);
        state.select(Some(ShapeId::new(NonZeroU16::new(2).unwrap())));

        // From: anchor is Top -> leaving is Up -> pointing is Down -> "solid ▼"
        state.set_sub_mode(Some(SubMode::TerminalSelect(End::From)));
        assert_eq!(
            status_line(&state),
            "TERMINAL for 1. From of Connector #2 · Current: solid ▼"
        );

        // To: glyph ◆ -> "glyph ◆"
        state.set_sub_mode(Some(SubMode::TerminalSelect(End::To)));
        assert_eq!(
            status_line(&state),
            "TERMINAL for 2. To of Connector #2 · Current: glyph ◆"
        );
    }

    #[test]
    fn terminal_select_sub_mode_offers_one_to_five_palette_and_cancel() {
        let text = r#"{
            "next_id": 3,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 3 }, "stroke": "light" },
                { "kind": "connector", "id": 2,
                  "from": { "at": { "kind": "reference", "shape": 1, "anchor": "top", "offset": { "dx": 0, "dy": 0 }, "out": 0 },
                            "terminal": { "kind": "arm" } },
                  "to":   { "at": { "kind": "point", "x": 20, "y": 5 }, "leaving": "left",
                            "terminal": { "kind": "symbol", "style": "solid" } },
                  "stroke": "light" }
            ]
        }"#;

        let mut state = a_state_over(text);
        state.select(Some(ShapeId::new(NonZeroU16::new(2).unwrap())));

        state.set_sub_mode(Some(SubMode::TerminalSelect(End::From)));
        assert_eq!(
            command_line(&state),
            "TERMINAL: [1] arm ─ [2] solid ▶ [3] outline ▷ [4] ascii > [5] diamond ◆ · [Esc] Cancel"
        );

        state.set_sub_mode(Some(SubMode::TerminalSelect(End::To)));
        assert_eq!(
            command_line(&state),
            "TERMINAL: [1] arm ─ [2] solid ▶ [3] outline ▷ [4] ascii > [5] diamond ◆ · [Esc] Cancel"
        );
    }

    #[test]
    fn endpoint_sub_mode_offers_detach_when_attached_and_leaving_when_detached() {
        let text = r#"{
            "next_id": 3,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 4 }, "stroke": "light" },
                { "kind": "connector", "id": 2,
                  "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right", "offset": { "dx": 0, "dy": 0 }, "out": 0 },
                            "terminal": { "kind": "arm" } },
                  "to":   { "at": { "kind": "point", "x": 20, "y": 5 }, "leaving": "left",
                            "terminal": { "kind": "symbol", "style": "solid" } },
                  "stroke": "light" }
            ]
        }"#;

        let mut state = a_state_over(text);
        state.select(Some(ShapeId::new(NonZeroU16::new(2).unwrap())));

        // From (attached): offers [0] Detach, not LEAVING
        state.set_sub_mode(Some(SubMode::Endpoint(End::From)));
        assert_eq!(
            command_line(&state),
            "OFFSET: ↑↓←→ · OUT: [+] [-] · [0] Detach · TERMINAL: [t] · [2] To · [Esc] Done"
        );

        // To (detached): offers LEAVING: [d], not Detach or Offset/Out
        state.set_sub_mode(Some(SubMode::Endpoint(End::To)));
        assert_eq!(
            command_line(&state),
            "MOVE: ↑↓←→ · LEAVING: [d] · TERMINAL: [t] · [1] From · [Esc] Done"
        );
    }

    #[test]
    fn leaving_select_sub_mode_inspects_detached_endpoint_and_offers_compass_directions() {
        let text = r#"{
            "next_id": 2,
            "shapes": [
                { "kind": "connector", "id": 1,
                  "from": { "at": { "kind": "point", "x": 2, "y": 1 },
                            "leaving": "right", "terminal": { "kind": "arm" } },
                  "to":   { "at": { "kind": "point", "x": 10, "y": 5 },
                            "leaving": "up", "terminal": { "kind": "symbol", "style": "solid" } },
                  "stroke": "light" }
            ]
        }"#;

        let mut state = a_state_over(text);
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));

        state.set_sub_mode(Some(SubMode::LeavingSelect(End::From)));
        assert_eq!(
            status_line(&state),
            "LEAVING for 1. From of Connector #1 · Detached at (2, 1) · Current: right"
        );
        assert_eq!(
            command_line(&state),
            "LEAVING: [1] ↑ North [2] → East [3] ↓ South [4] ← West · [Esc] Cancel"
        );

        state.set_sub_mode(Some(SubMode::LeavingSelect(End::To)));
        assert_eq!(
            status_line(&state),
            "LEAVING for 2. To of Connector #1 · Detached at (10, 5) · Current: up"
        );
        assert_eq!(
            command_line(&state),
            "LEAVING: [1] ↑ North [2] → East [3] ↓ South [4] ← West · [Esc] Cancel"
        );
    }

    #[test]
    fn idle_command_row_offers_tab_and_shift_tab_only_when_diagram_has_shapes() {
        let empty = a_state_over(r#"{ "next_id": 1, "shapes": [] }"#);
        assert_eq!(
            command_line(&empty),
            "CREATE: [b] Box [l] Line [c] Connector"
        );

        let populated = a_state_over(
            r#"{
            "next_id": 2,
            "shapes": [ { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                          "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
        }"#,
        );
        assert_eq!(
            command_line(&populated),
            "CREATE: [b] Box [l] Line [c] Connector · SELECT: [Tab] Next [Shift+Tab] Prev [Click] Pick"
        );
    }

    #[test]
    fn selected_shapes_offer_tab_next_and_shift_tab_prev_before_delete_and_deselect() {
        let text = r#"{
            "next_id": 5,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                  "size": { "width": 4, "height": 3 }, "stroke": "light" },
                { "kind": "line", "id": 2, "at": { "x": 0, "y": 5 },
                  "len": 8, "orientation": "horizontal", "stroke": "light" },
                { "kind": "line", "id": 3, "at": { "x": 5, "y": 0 },
                  "len": 6, "orientation": "vertical", "stroke": "light" },
                { "kind": "connector", "id": 4,
                  "from": { "at": { "kind": "point", "x": 1, "y": 2 }, "leaving": "right",
                            "terminal": { "kind": "arm" } },
                  "to":   { "at": { "kind": "point", "x": 5, "y": 6 }, "leaving": "left",
                            "terminal": { "kind": "arm" } },
                  "stroke": "light" }
            ]
        }"#;

        let mut state = a_state_over(text);
        for id in 1..=4 {
            state.select(Some(ShapeId::new(NonZeroU16::new(id).unwrap())));
            assert!(
                command_line(&state)
                    .ends_with(" · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect"),
                "unexpected command row for shape #{id}: {}",
                command_line(&state)
            );
        }
    }

    #[test]
    fn inconsistent_sub_mode_renders_base_shape_line_without_panicking() {
        let text = r#"{
            "next_id": 2,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                  "size": { "width": 4, "height": 3 }, "stroke": "light" }
            ]
        }"#;
        let mut state = a_state_over(text);
        state.select(Some(ShapeId::new(NonZeroU16::new(1).unwrap())));
        let base_line = status_line(&state);

        for sub_mode in [
            SubMode::Endpoint(End::From),
            SubMode::Endpoint(End::To),
            SubMode::TerminalSelect(End::From),
            SubMode::LeavingSelect(End::From),
        ] {
            state.set_sub_mode(Some(sub_mode));
            assert_eq!(
                status_line(&state),
                base_line,
                "expected fallback to base line for endpoint sub-mode on a box"
            );
        }
    }
}
