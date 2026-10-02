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

use crate::{Anchor, Delta, Diagram, Endpoint, Position, Reference, Shape, ShapeId};

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
            at: from_at.into(),
            leaving: Direction::Right,
            terminal: monospace_core::Terminal::Glyph {
                glyph: glyph("◄")
            },
        },
        to: Endpoint {
            at: to_at.into(),
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
            at: from_at.into(),
            leaving: Direction::Right,
            terminal: monospace_core::Terminal::Arm,
        },
        to: Endpoint {
            at: to_at.into(),
            leaving: Direction::Left,
            terminal: monospace_core::Terminal::Arm,
        },
        stroke: light(),
    }
}

// ------------------------------------------ an endpoint hanging from a figure's side

/// The connector of _an endpoint hangs from a side_ with its `from` named as a **reference** to the
/// box's right side rather than as the point that side resolves to.
fn hanging_connector(box_id: ShapeId) -> Shape {
    Shape::Connector {
        from: Endpoint {
            at: Position::Reference(Reference {
                id: box_id,
                anchor: Anchor::Right,
                offset: Delta { dx: 0, dy: 0 },
            }),
            leaving: Direction::Right,
            terminal: monospace_core::Terminal::Arm,
        },
        to: Endpoint {
            at: at(7, 1).into(),
            leaving: Direction::Left,
            terminal: monospace_core::Terminal::Arm,
        },
        stroke: light(),
    }
}

/// The first block is the model's §6 _Attachment_ reached with a reference where the model spells
/// the point; the second is the same diagram with the box displaced four cells right, so the arrow
/// follows the side it hangs from; the third is the arrangement as written with the **connector**
/// displaced two cells down, so the arrow slides off that side and draws as a translation of itself;
/// and the **fourth takes the box out**, so the arrow is not drawn at all — `wrote 0 of 24
/// positions`.
///
/// **That fourth block is empty, and the emptiness is the finding rather than a defect.** The
/// gallery's arrangement is one box and one connector, and the connector's `from` names the only
/// other figure, so **there is no survivor**: the block is the specification's B1.1 picture without
/// the `#2` that stands in it, which is why it is blank where B1.1's hand-drawn one is not. The
/// empty surface row says the number rather than leaving the impression, which is what makes it
/// worth having at all: it is a snapshot, so a removal that ever began drawing the route — or
/// freezing it where it had resolved — would write cells into that window and move it. **The one
/// thing that would change it is widening the arrangement to hold a second box**, so the block
/// could carry B1.1's picture rather than its degenerate case; that moves all three blocks the
/// snapshot already holds, it is **not** D2's answer, and it is named here so a maintainer can
/// raise it at review rather than find it in a snapshot.
///
/// All four are in one snapshot because what a reader is meant to see is them together, and the two
/// displacements are the two directions of one rule: displacing the figure a reference hangs from
/// carries the endpoint, displacing the figure that holds the reference moves it, and the pair is
/// what makes the difference visible rather than asserted. The third block is **drawn by
/// `displaced_by`** rather than built from the two positions the rule yields by hand, so a rule that
/// stopped holding drops this snapshot instead of leaving a picture that no longer matches the code
/// (B1.1, B3.4, §4 of the model).
///
/// **The label names three values now, because the reference holds three.** A block cannot print a
/// shape for itself — a `Diagram` offers one query by an identity and no listing — so this string
/// is written by hand, and a hand-written label that named two of a reference's three values would
/// let a reader assume a third default. `offset (0, 0)` is what says the arrow stands *on* the
/// side rather than beside it. The offset is drawn nowhere in any of the three pictures, and §6's
/// stays exactly as the model spells it.
///
/// This is no longer the only carrier in the crate that can reach a reference: since #83 a
/// `<!-- render: -->` marker reads a description whose `at` may hold one (D2, ADR-0035). The block
/// stays here rather than moving to a marker because the three blocks are one snapshot and a marker
/// renders one picture per description (ADR-0064). It is also the only carrier that can reach a
/// **displacement** at all: a marker reads a description and a description carries no field for one
/// (ADR-0035), so no marker anywhere in the repository can hold a picture of this rule.
#[test]
fn an_endpoint_hangs_from_a_side_and_follows_it() {
    let labelled =
        "[small_box(0,0,no fill), arm_connector(from = Reference(#1, Right, offset (0,0)) -> 7,1)]";
    let mut diagram = Diagram::new();
    let box_id = diagram.add(small_box(at(0, 0), None));
    diagram.add(hanging_connector(box_id.clone()));
    let first = block(labelled, "as written", window(8, 3), &diagram);

    // The box moves four cells right, so its right side center moves from `{3, 1}` to `{7, 1}`
    // and the arrow lands on the new side without anything naming where that is.
    let moved = diagram
        .get(&box_id)
        .expect("the box is in the diagram")
        .displaced_by(Delta { dx: 4, dy: 0 });
    diagram.replace(&box_id, moved);
    let second = block(
        labelled,
        "the box displaced four cells right",
        window(8, 3),
        &diagram,
    );

    // And the third displaces the **connector**, which is the other direction and the other rule:
    // the endpoint slides off the side rather than following it, and the arrow draws as a
    // translation of itself.
    //
    // **Reached from the arrangement as written rather than from the block beside it**, and a second
    // diagram in this test is what that costs — six lines rather than a `get` and a `replace`. The
    // reason is measured rather than stylistic: the second block above left the box's right side
    // centre on `{7, 1}`, which is where the connector's own free end already stood, so a
    // displacement applied to *that* diagram would have put both endpoints on `{7, 3}` and drawn
    // nothing at all — degenerate in the same way the specification's own B2.1 is, one step further
    // along. Reaching the arrangement as written is also what keeps the third block comparable with
    // the first: the two differ by the one displacement between them and by nothing else.
    let mut from_as_written = Diagram::new();
    let second_box_id = from_as_written.add(small_box(at(0, 0), None));
    let arrow_id = from_as_written.add(hanging_connector(second_box_id));
    let moved_arrow = from_as_written
        .get(&arrow_id)
        .expect("the connector is in the diagram")
        .displaced_by(Delta { dx: 0, dy: 2 });
    from_as_written.replace(&arrow_id, moved_arrow);

    // **`window(8, 4)`, and the fourth row is the reason.** The displaced connector lands on the
    // fourth row, so a three-row window would clip it out entirely and the block would measure to
    // nothing. A snapshot that grew a fifth row of nothing is the signature of the window being too
    // short, so the row is worth naming rather than discovering.
    let third = block(
        labelled,
        "the connector displaced two cells down",
        window(8, 4),
        &from_as_written,
    );

    // **And the fourth takes the box out, which is the only picture in the repository that can show
    // a removal at all.** No `<!-- render: -->` marker can reach it: a marker reads a description
    // the file carries, and a description has no field that takes a shape out (ADR-0035), which is
    // why D2's answer is a fourth block here rather than a fifth snapshot elsewhere (ADR-0064).
    //
    // **A third diagram rather than a removal on one of the two above, and that is measured rather
    // than chosen.** Both diagrams this test holds are already mutated, and the removal would be a
    // no-op on what remains of either: it writes **0 of 32** positions on `from_as_written` and
    // **0 of 24** on `diagram`. The block would then be blank for a second and different reason
    // while the comment beside it said the first, which is the one way a snapshot like this lies.
    let mut with_the_box_taken_out = Diagram::new();
    let third_box_id = with_the_box_taken_out.add(small_box(at(0, 0), None));
    with_the_box_taken_out.add(hanging_connector(third_box_id.clone()));
    with_the_box_taken_out.remove(&third_box_id);
    let fourth = block(
        labelled,
        "the box taken out",
        window(8, 3),
        &with_the_box_taken_out,
    );

    snap(
        "an_endpoint_hangs_from_a_side_and_follows_it",
        &format!("{first}\n{second}\n{third}\n{fourth}"),
    );
}
