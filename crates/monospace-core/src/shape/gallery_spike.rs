//! SPIKE v3 — not for keeping. One file per level: each entry is the value, then the
//! picture, then the surface. One arm per column, so the surface reads as a table.

use std::fmt::Write as _;

use crate::cell::Side;
use crate::shape::fragment::border::Border;
use crate::shape::fragment::corner::Corner;
use crate::shape::fragment::end::End;
use crate::shape::fragment::fill::Fill;
use crate::shape::fragment::head::Head;
use crate::shape::fragment::segment::Segment;
use crate::shape::route::Route;
use crate::stroke::Stroke;
use crate::{
    Arm, Arrow, BoxShape, Buffer, Cell, Direction, Endpoint, Glyph, GlyphCatalog, Layer, Line,
    Orientation, Pos, Size, StampMode, Surface, Terminal, render,
};

/// One description for every file, so the prose is written once rather than per level — and
/// once rather than per block, which is what the arms table made necessary.
const WHAT: &str = concat!(
    "A gallery: a chosen set of examples, one block each, for reading rather than for coverage. ",
    "Each label is derived from the value that drew the block, so it cannot disagree with it. ",
    "Under each picture is the surface that picture came from: a count of the positions the ",
    "figure wrote, then a row for each of them, saying what that position renders, what each of ",
    "its four arms holds, and the base stroke every Set arm is drawn in. A Set arm names the ",
    "stroke; Unset and Closed are the model's own two words for the other two. A position the ",
    "figure left unwritten is a hole in the picture rather than a row here, and the count is how ",
    "many there are. A literal has no arms, so its arm columns are empty. A block moving means a ",
    "figure changed, and what moved is the thing to look at — the same acceptance a ",
    "characterization gets, without its claim of covering a range.",
);

/// The columns of the arms table, in the order the model's four sides are named, then the base
/// stroke every `Set` arm is drawn in.
const COLUMNS: [&str; 7] = ["at", "glyph", "top", "right", "bottom", "left", "base"];

fn arm_text(arm: &Arm) -> String {
    match arm {
        Arm::Set(stroke) => stroke.0.clone(),
        Arm::Closed => "Closed".to_string(),
        Arm::Unset => "Unset".to_string(),
    }
}

fn draw(size: Size, body: impl FnOnce(&mut dyn Surface)) -> (String, String) {
    let origin = Pos { x: 0, y: 0 };
    let mut buffer = Buffer::new(origin, size);
    body(&mut Layer::new(&mut buffer, StampMode::Above));
    let picture = render(&buffer, &GlyphCatalog::light(), origin, size);
    (picture, surface(&buffer, size))
}

/// The surface as a Markdown table: one row per position the figure wrote, padded to the widest
/// cell in each column so the file reads as a table. A position it left unwritten is a hole in
/// the picture above, and the count says how many there were.
fn surface(buffer: &Buffer, size: Size) -> String {
    let catalog = GlyphCatalog::light();
    let mut rows: Vec<[String; 7]> = Vec::new();
    let total = size.width as usize * size.height as usize;

    for y in 0..size.height.cast_signed() {
        for x in 0..size.width.cast_signed() {
            let cell = buffer.cell(Pos { x, y });
            let (glyph, top, right, bottom, left, base) = match &cell {
                None => continue,
                Some(Cell::Literal(g)) => (
                    g.as_str().to_string(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                ),
                Some(Cell::Strokes(s)) => (
                    catalog
                        .glyph(&s.key())
                        .map_or("?".to_string(), |g| g.as_str().to_string()),
                    arm_text(&s.top),
                    arm_text(&s.right),
                    arm_text(&s.bottom),
                    arm_text(&s.left),
                    s.base.0.clone(),
                ),
            };
            rows.push([format!("{x},{y}"), glyph, top, right, bottom, left, base]);
        }
    }

    let widths: Vec<usize> = (0..COLUMNS.len())
        .map(|column| {
            rows.iter()
                .map(|row| row[column].chars().count())
                .chain(std::iter::once(COLUMNS[column].len()))
                .max()
                .unwrap_or(0)
        })
        .collect();

    let line = |cells: &[String]| {
        let padded: Vec<String> = cells
            .iter()
            .zip(&widths)
            .map(|(cell, width)| format!("{cell:<width$}"))
            .collect();
        format!("  | {} |", padded.join(" | "))
    };

    let mut out = format!("  wrote {} of {total} positions\n", rows.len());
    let header: Vec<String> = COLUMNS.iter().map(|name| (*name).to_string()).collect();
    let _ = writeln!(out, "{}", line(&header));
    let rule: Vec<String> = widths.iter().map(|width| "-".repeat(*width)).collect();
    let _ = writeln!(out, "  | {} |", rule.join(" | "));
    for row in &rows {
        let _ = writeln!(out, "{}", line(row));
    }
    out
}

/// One entry: the value, then the picture, then the surface. Every label is `format!("{value:?}")`
/// and every other line comes from the buffer or the renderer.
type Entry = (String, Size, Box<dyn Fn(&mut dyn Surface)>);

fn one<T: std::fmt::Debug + crate::Shape + 'static>(value: T, size: Size) -> Entry {
    let label = format!("{value:?}");
    (
        label,
        size,
        Box::new(move |s: &mut dyn Surface| value.draw(s)),
    )
}

fn gallery(entries: Vec<Entry>) -> String {
    let mut out = String::new();
    for (label, size, entry) in entries {
        let _ = writeln!(out, "{label}");
        let (picture, surface) = draw(size, |s| entry(s));
        // The indent is trimmed off a line the picture left empty, or the gate's
        // `editorconfig-checker` step rejects the snapshot once it is tracked — the same
        // trailing blanks ADR-0045 met and answered the same way.
        for line in picture.lines() {
            let _ = writeln!(out, "{}", format!("  {line}").trim_end());
        }
        let _ = write!(out, "{surface}");
        let _ = writeln!(out);
    }
    out
}

fn snap(name: &str, body: &str) {
    let mut settings = insta::Settings::clone_current();
    settings.set_description(WHAT);
    settings.bind(|| insta::assert_snapshot!(name, body));
}

fn window(width: u32, height: u32) -> Size {
    Size { width, height }
}

fn light() -> Stroke {
    Stroke::from("light")
}

fn glyph(text: &str) -> Glyph {
    Glyph::new(text).expect("one glyph")
}

// ------------------------------------------------------ level 1b: the arrow and its route

/// The head glyph the model's own tests fix for a picture: opposite the leaving direction.
fn head(leaving: Direction) -> Glyph {
    glyph(match leaving {
        Direction::Up => "▼",
        Direction::Right => "◄",
        Direction::Down => "▲",
        Direction::Left => "►",
    })
}

fn at(x: i32, y: i32) -> Pos {
    Pos { x, y }
}

fn glyph_end(x: i32, y: i32, leaving: Direction) -> Endpoint {
    Endpoint {
        at: at(x, y),
        leaving,
        terminal: Terminal::Glyph {
            glyph: head(leaving),
        },
    }
}

fn arm_end(x: i32, y: i32, leaving: Direction) -> Endpoint {
    Endpoint {
        at: at(x, y),
        leaving,
        terminal: Terminal::Arm,
    }
}

fn arrow(from: Endpoint, to: Endpoint) -> Arrow {
    Arrow {
        from,
        to,
        stroke: light(),
    }
}

/// The route an arrow derived for the arrangement the test named. `from` and `to` are carried
/// because the route's outermost positions need to know which side faces the head beside them.
fn route(from: Pos, to: Pos, positions: Vec<Pos>) -> Route {
    Route {
        from,
        to,
        positions,
        stroke: light(),
    }
}

#[test]
fn level_1_arrow() {
    let entries = vec![
        one(
            arrow(
                glyph_end(0, 0, Direction::Down),
                glyph_end(4, 3, Direction::Left),
            ),
            window(5, 4),
        ),
        one(
            arrow(
                glyph_end(0, 0, Direction::Right),
                glyph_end(4, 3, Direction::Up),
            ),
            window(5, 4),
        ),
        one(
            arrow(
                glyph_end(0, 0, Direction::Right),
                glyph_end(6, 0, Direction::Left),
            ),
            window(7, 1),
        ),
        one(
            arrow(
                glyph_end(0, 0, Direction::Right),
                glyph_end(6, 2, Direction::Left),
            ),
            window(7, 3),
        ),
        one(
            arrow(
                glyph_end(2, 0, Direction::Right),
                glyph_end(8, 2, Direction::Left),
            ),
            window(9, 3),
        ),
        one(
            arrow(
                glyph_end(2, 0, Direction::Left),
                glyph_end(8, 2, Direction::Right),
            ),
            window(10, 3),
        ),
        one(
            arrow(
                glyph_end(2, 0, Direction::Left),
                glyph_end(8, 2, Direction::Down),
            ),
            window(9, 4),
        ),
        one(
            arrow(
                glyph_end(2, 0, Direction::Right),
                glyph_end(4, 1, Direction::Left),
            ),
            window(5, 2),
        ),
        one(
            arrow(
                glyph_end(2, 0, Direction::Left),
                glyph_end(3, 2, Direction::Right),
            ),
            window(5, 3),
        ),
        one(
            arrow(
                arm_end(0, 0, Direction::Right),
                arm_end(6, 0, Direction::Left),
            ),
            window(7, 1),
        ),
        one(
            arrow(
                arm_end(0, 0, Direction::Down),
                arm_end(4, 3, Direction::Left),
            ),
            window(5, 4),
        ),
        one(
            arrow(
                glyph_end(2, 1, Direction::Right),
                glyph_end(2, 1, Direction::Left),
            ),
            window(5, 3),
        ),
    ];
    snap("level_1_arrow", &gallery(entries));
}

#[test]
fn level_1_route() {
    let entries = vec![
        one(
            route(
                at(0, 0),
                at(6, 0),
                vec![at(1, 0), at(2, 0), at(3, 0), at(4, 0), at(5, 0)],
            ),
            window(7, 1),
        ),
        one(
            route(
                at(0, 0),
                at(4, 3),
                vec![at(0, 1), at(0, 2), at(0, 3), at(1, 3), at(2, 3), at(3, 3)],
            ),
            window(5, 4),
        ),
        one(
            route(
                at(2, 0),
                at(8, 2),
                vec![
                    at(1, 0),
                    at(1, 1),
                    at(2, 1),
                    at(3, 1),
                    at(4, 1),
                    at(5, 1),
                    at(6, 1),
                    at(7, 1),
                    at(8, 1),
                    at(9, 1),
                    at(9, 2),
                ],
            ),
            window(10, 3),
        ),
    ];
    snap("level_1_route", &gallery(entries));
}

// ---------------------------------------------------------------- level 2: the fragments

#[test]
fn level_2_fragments() {
    let entries = vec![
        one(
            Border {
                from: Pos { x: 0, y: 0 },
                len: 3,
                side: Side::Top,
                stroke: light(),
                closes_interior: true,
            },
            window(3, 1),
        ),
        one(
            Border {
                from: Pos { x: 0, y: 0 },
                len: 3,
                side: Side::Top,
                stroke: light(),
                closes_interior: false,
            },
            window(3, 1),
        ),
        one(
            Border {
                from: Pos { x: 0, y: 0 },
                len: 3,
                side: Side::Left,
                stroke: light(),
                closes_interior: true,
            },
            window(3, 1),
        ),
        one(
            Corner {
                at: Pos { x: 1, y: 0 },
                opens: (Side::Right, Side::Bottom),
                stroke: light(),
            },
            window(3, 1),
        ),
        one(
            End {
                at: Pos { x: 1, y: 0 },
                side: Side::Right,
                stroke: light(),
            },
            window(3, 1),
        ),
        one(
            End {
                at: Pos { x: 1, y: 0 },
                side: Side::Top,
                stroke: light(),
            },
            window(3, 1),
        ),
        one(
            Segment {
                from: Pos { x: 0, y: 0 },
                len: 3,
                orientation: Orientation::Horizontal,
                stroke: light(),
            },
            window(3, 1),
        ),
        one(
            Fill {
                at: Pos { x: 0, y: 0 },
                size: window(3, 1),
                glyph: glyph("."),
            },
            window(3, 1),
        ),
        one(
            Head {
                at: Pos { x: 1, y: 0 },
                glyph: glyph(">"),
            },
            window(3, 1),
        ),
    ];
    snap("level_2_fragments", &gallery(entries));
}

// ------------------------------------------------------------ level 1: the complete shapes

#[test]
fn level_1_complete_shapes() {
    let entries = vec![
        one(
            BoxShape {
                at: Pos { x: 0, y: 0 },
                size: window(6, 3),
                stroke: light(),
                fill: None,
            },
            window(6, 3),
        ),
        one(
            BoxShape {
                at: Pos { x: 0, y: 0 },
                size: window(6, 3),
                stroke: light(),
                fill: Some(glyph(".")),
            },
            window(6, 3),
        ),
        one(
            BoxShape {
                at: Pos { x: 0, y: 0 },
                size: window(2, 2),
                stroke: light(),
                fill: None,
            },
            window(2, 2),
        ),
        one(
            Line {
                at: Pos { x: 0, y: 0 },
                len: 5,
                orientation: Orientation::Horizontal,
                stroke: light(),
            },
            window(5, 1),
        ),
        one(
            Line {
                at: Pos { x: 0, y: 0 },
                len: 0,
                orientation: Orientation::Horizontal,
                stroke: light(),
            },
            window(5, 1),
        ),
    ];
    snap("level_1_complete_shapes", &gallery(entries));
}
