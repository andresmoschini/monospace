//! A gallery of what a diagram adds over the figures, and the one completeness guarantee the
//! core's cannot have.
//!
//! A `Shape` here is a closed set of three kinds, so [`kind_of`] matches over it with no arm for
//! anything else — and **that match is the guarantee**. A fourth kind added and drawn compiles the
//! library and stops the build here, at the line that has to name it. Verified by adding a variant,
//! implementing its `draw`, and watching the library succeed and this file fail.
//!
//! Two of the three files earn their place by showing something the core's gallery cannot. The
//! order is the only part of `docs/diagram-model.md` that leaves a trace in a buffer, and moving
//! `#1` one place toward the front is the only operation in the project that changes a picture
//! without changing a figure. The other is the arm terminal meeting a box border, which reads `├`
//! and is written by neither figure.
//!
//! The cost is one place the rule from the core's gallery does not hold. A `Diagram` keeps its
//! shapes in a private `Vec<Placed>` and offers one query by an identity, with no listing of what
//! it holds, so there is nothing to derive a label from and the block says which shapes it holds
//! in a string written by hand. ADR-0030 withdrew `Extent` for the same reason — a shape with no
//! reader — and the reader that has since arrived is that one query, not the read iterator that
//! record declined and the listing it declined beside.
//!
//! [ADR-0064](../../../docs/decisions/0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md)
//! records why a figure's picture needs this carrier and not a `<!-- render: -->` marker.

use std::fmt::Write as _;

use monospace_core::{
    Arm, Buffer, Cell, Direction, Glyph, GlyphCatalog, Pos, Size, Stroke, render,
};

use crate::{Diagram, Endpoint, Shape, ShapeId};

/// One description for every file, so the prose is written once rather than per level.
const WHAT: &str = concat!(
    "A gallery: a chosen set of examples, one block each, for reading rather than for coverage. ",
    "Each label lists the shapes in the order they were added, each one derived from its own ",
    "Debug, so a label cannot disagree with the values it names; what was done between two ",
    "drawings of one diagram is the second line, and it is named for the change rather than for ",
    "the order, which 080 could make and a displacement does not. It is written by hand, because ",
    "a diagram offers one query by an identity and no listing of the shapes it holds. Under each ",
    "picture is the surface that picture came from: a count of the positions the diagram wrote, ",
    "then a row for each of them, saying what that position renders, what each of its four arms ",
    "holds, and the base stroke every Set arm is drawn in. A Set arm names the stroke; Unset and ",
    "Closed are the model's own two words for the other two. A position left unwritten is a hole ",
    "in the picture rather than a row here, and the count is how many there are. A literal has no ",
    "arms, so its arm columns are empty. A block moving means a figure or an order changed, and ",
    "what moved is the thing to look at — the same acceptance a characterization gets, without its ",
    "claim of covering a range.",
);

const COLUMNS: [&str; 7] = ["at", "glyph", "top", "right", "bottom", "left", "base"];

fn arm_text(arm: &Arm) -> String {
    match arm {
        Arm::Set(stroke) => stroke.0.clone(),
        Arm::Closed => "Closed".to_string(),
        Arm::Unset => "Unset".to_string(),
    }
}

fn at(x: i32, y: i32) -> Pos {
    Pos { x, y }
}

fn light() -> Stroke {
    Stroke::from("light")
}

fn glyph(text: &str) -> Glyph {
    Glyph::new(text).expect("one glyph")
}

fn window(width: u32, height: u32) -> Size {
    Size { width, height }
}

/// The kind's own name, by a `match` with no arm for anything else. **This is the check.** The
/// core has no equivalent and cannot have one: its figures are an open trait, so there is no set
/// to be exhaustive over. Here the set is closed, so adding a fourth kind makes this function
/// stop compiling until it is given a name — and a name is what a catalog entry needs.
fn kind_of(shape: &Shape) -> &'static str {
    match shape {
        Shape::Box { .. } => "Box",
        Shape::Line { .. } => "Line",
        Shape::Connector { .. } => "Connector",
    }
}

/// A box, a line and a connector: the closed set, each named by [`kind_of`]. A factory rather than
/// a list of values, because the closed set is then three calls rather than three values and the
/// three clones they would need. `Shape` had no `Clone` when this was written and the factory was
/// the only way to have the same figure twice; it is `Clone` now, so the choice is free rather
/// than forced, and what it buys is that each kind is constructed in one place.
fn the_kinds() -> Vec<(&'static str, Shape)> {
    vec![
        (
            kind_of(&box_shape(at(0, 0), None)),
            box_shape(at(0, 0), None),
        ),
        (kind_of(&line_shape(at(0, 0), 6)), line_shape(at(0, 0), 6)),
        (
            kind_of(&connector_shape(at(0, 0), at(6, 0))),
            connector_shape(at(0, 0), at(6, 0)),
        ),
    ]
}

fn box_shape(at_: Pos, fill: Option<&str>) -> Shape {
    Shape::Box {
        at: at_,
        size: window(6, 3),
        stroke: light(),
        fill: fill.map(glyph),
    }
}

fn line_shape(at_: Pos, len: u32) -> Shape {
    Shape::Line {
        at: at_,
        len,
        orientation: monospace_core::Orientation::Horizontal,
        stroke: light(),
    }
}

fn connector_shape(from_at: Pos, to_at: Pos) -> Shape {
    Shape::Connector {
        from: Endpoint {
            at: from_at,
            leaving: Direction::Right,
            terminal: monospace_core::Terminal::Glyph {
                glyph: glyph("◄")
            },
        },
        to: Endpoint {
            at: to_at,
            leaving: Direction::Left,
            terminal: monospace_core::Terminal::Glyph {
                glyph: glyph("►")
            },
        },
        stroke: light(),
    }
}

fn small_box(at_: Pos, fill: Option<&str>) -> Shape {
    Shape::Box {
        at: at_,
        size: window(4, 3),
        stroke: light(),
        fill: fill.map(glyph),
    }
}

/// Draws `diagram` into a fresh window of `size` and returns the picture and the surface.
fn draw(size: Size, diagram: &Diagram) -> (String, String) {
    let origin = at(0, 0);
    let mut buffer = Buffer::new(origin, size);
    diagram.draw(&mut buffer);
    let picture = render(&buffer, &GlyphCatalog::light(), origin, size);
    (picture, surface(&buffer, size))
}

fn surface(buffer: &Buffer, size: Size) -> String {
    let catalog = GlyphCatalog::light();
    let mut rows: Vec<[String; 7]> = Vec::new();
    let total = size.width as usize * size.height as usize;

    for y in 0..size.height.cast_signed() {
        for x in 0..size.width.cast_signed() {
            let cell = buffer.cell(at(x, y));
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

/// One block: what the diagram holds, then what was done between the two drawings, then the
/// picture, then the surface. `shapes` is a label rather than the values, because a `Diagram` offers
/// one query by an identity and no listing of the shapes it holds, so there is nothing here to
/// derive a label from. `change` is written by hand for the same reason, and is named for the
/// change rather than for the order because a figure's displacement is a change too.
fn block(shapes: &str, change: &str, size: Size, diagram: &Diagram) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "  shapes: {shapes}");
    let _ = writeln!(out, "  change: {change}");

    let (picture, surface) = draw(size, diagram);
    for line in picture.lines() {
        let _ = writeln!(out, "{}", format!("  {line}").trim_end());
    }
    let _ = write!(out, "{surface}");
    out
}

/// Writes one snapshot. The path is set rather than left to `insta`'s default, which puts a
/// snapshot beside the source that wrote it — the characterization does the same, and the two
/// kinds are kept apart by directory.
fn snap(name: &str, body: &str) {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/snapshots/gallery"
    ));
    settings.set_description(WHAT);
    settings.bind(|| insta::assert_snapshot!(name, body));
}

// ------------------------------------------------------------ the three kinds, one each

#[test]
fn the_three_kinds() {
    let mut out = String::new();
    for (kind, shape) in the_kinds() {
        let label = format!("Shape::{kind} {shape:?}");
        let mut diagram = Diagram::new();
        diagram.add(shape);
        let _ = writeln!(out, "{label}");
        let (picture, surface) = draw(window(7, 3), &diagram);
        for line in picture.lines() {
            let _ = writeln!(out, "{}", format!("  {line}").trim_end());
        }
        let _ = write!(out, "{surface}");
        let _ = writeln!(out);
    }
    snap("the_three_kinds", &out);
}

// ----------------------------------------------------------------- what the order is for

#[test]
fn the_order_decides_a_shared_cell() {
    // As written: `#1` behind, `#2` in front, so `▓` covers where they share. Rebuilt rather than
    // cloned: `Shape` is `Clone` now, and rebuilding from the factory is what keeps both blocks
    // drawing the two boxes as written rather than one of them drawing a second pair.
    let labelled = "[small_box(0,0,fill=░), small_box(2,1,fill=▓)]";
    let mut diagram = Diagram::new();
    diagram.add(small_box(at(0, 0), Some("░")));
    diagram.add(small_box(at(2, 1), Some("▓")));
    let first = block(labelled, "as written", window(6, 4), &diagram);

    // One place toward the front, which puts `░` over `▓` and changes three cells.
    diagram.forward(&ShapeId::new("#1"));
    let moved = block(
        labelled,
        "#1 moved one place toward the front",
        window(6, 4),
        &diagram,
    );

    snap(
        "the_order_decides_a_shared_cell",
        &format!("{first}\n{moved}"),
    );
}

// ------------------------------------------------- an arm terminal composing with a border

#[test]
fn an_arm_terminal_composes_where_a_border_is() {
    let labelled = "[small_box(0,0,no fill), arm_connector(3,1 -> 7,1)]";
    let mut diagram = Diagram::new();
    diagram.add(small_box(at(0, 0), None));
    diagram.add(arm_connector(at(3, 1), at(7, 1)));
    snap(
        "an_arm_terminal_composes_where_a_border_is",
        &block(labelled, "as written", window(8, 3), &diagram),
    );
}

fn arm_connector(from_at: Pos, to_at: Pos) -> Shape {
    Shape::Connector {
        from: Endpoint {
            at: from_at,
            leaving: Direction::Right,
            terminal: monospace_core::Terminal::Arm,
        },
        to: Endpoint {
            at: to_at,
            leaving: Direction::Left,
            terminal: monospace_core::Terminal::Arm,
        },
        stroke: light(),
    }
}
