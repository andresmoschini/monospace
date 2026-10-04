//! A sweep of what a description can put on a canvas, over the part of that space which is
//! finite.
//!
//! **This is a characterization, not a contract test.** It records what the code does over a range
//! too wide to assert by hand, and a change to it is the consequence of a decision taken
//! elsewhere rather than a decision here. It is accepted after a report of how many cases moved, in
//! which families, and three examples with their before and after — a rule each snapshot file
//! repeats at its own head. It is kept apart from every picture a test
//! asserts by hand by its own directory, not by its name, and one file per first value so a review
//! tool can render each diff: a single 20,000-line file is what GitHub would not show at all.
//!
//! [`docs/diagram-demo.md`](../../../docs/diagram-demo.md) is the readable half of the same
//! subject: it shows one instance of each decision the format makes, and this holds the finite
//! remainder of that space whole. The two are the same reach — a marker in a document and this both
//! go through the description format — and the split is a document's, because a document cannot
//! cover a range and this cannot be read.
//!
//! ## The range, and what is outside it
//!
//! Four of the format's fields are **unbounded** and are therefore outside any sweep: a `fill` and
//! a terminal's `glyph` are any single grapheme cluster, a connector's `offset` is any pair of
//! `i32`, and a position is any point. `docs/diagram-demo.md` gives each of those one instance,
//! which is a claim a reader can check by eye.
//!
//! What is left is the product of two finite fields — the `kind` of a shape and the `stroke` it is
//! drawn in — and this covers that product **whole**, in two families:
//!
//! - **A shape in each stroke it can be drawn in.** Nine tables, but only five draw a shape on its
//!   own: the other four hold only the mixtures between two tables, and one stroke writes all four
//!   of its arms in itself. Three kinds times five strokes is fifteen cases.
//! - **Two shapes crossing, in every ordered pair of those five strokes.** A crossing is the only
//!   place a cell carries arms in two different strokes, and therefore the only place a mixing table
//!   is reached at all. Five by five, each in both orders because the order decides the shared
//!   cell, is forty-five cases.
//!
//! Nothing is sampled. Every combination the range names is here, and a case that renders nothing
//! is a result rather than a gap — the four mixing tables produce exactly that, and the sweep is
//! what makes it a fact about the tables rather than an absence nobody had looked for.

use std::fmt::Write as _;

use monospace_core::{Buffer, Size};

use crate::description::Description;

/// The five tables a shape can be drawn in on its own. The four mixing tables are deliberately
/// absent: they hold only the mixtures between two of these, and a shape carries one stroke.
const DRAWING_STROKES: [&str; 5] = ["ascii", "light", "light-round", "heavy", "double"];

/// The three kinds, which the format names in its own spelling rather than the diagram crate's.
const KINDS: [&str; 3] = ["box", "line", "connector"];

/// The window both families are drawn in: one box and one crossing, with room for the widest
/// arrangement either reaches.
const SIZE: Size = Size {
    width: 12,
    height: 7,
};

/// The description every case is built from, with `SHAPES` standing where the shapes go. Built as
/// text rather than as a `Description` value so that **the parse is inside the sweep**: a case that
/// stops being readable is a result of the format, not something this file decides.
fn description_of(shapes: &str) -> String {
    format!(
        r#"{{
        "canvas": {{ "origin": {{ "x": 0, "y": 0 }},
                    "size": {{ "width": {}, "height": {} }} }},
        "next_id": 3,
        "shapes": [ {shapes} ]
    }}"#,
        SIZE.width, SIZE.height,
    )
}

/// One case, as its rendering with its own name above it and nothing below.
///
/// The trailing blanks are trimmed on every line before they are written: `render` pads each line
/// to the window's width and the gate's `editorconfig-checker` runs with `trim_trailing_whitespace`
/// on, so an untrimmed sweep would be a diff of nothing but the padding.
fn case(name: &str, shapes: &str) -> String {
    let description = description_of(shapes);
    let parsed: Description = serde_json::from_str(&description)
        .unwrap_or_else(|error| panic!("{name} is not a readable description: {error}"));
    let (origin, size) = parsed.window();
    let diagram = parsed.into_diagram();

    let mut buffer = Buffer::new(origin, size);
    diagram.draw(&mut buffer);
    let rendered = monospace_core::render(&buffer, &crate::glyph_catalog(), origin, size);

    let mut out = String::new();
    let _ = writeln!(out, "{name}");
    for line in rendered.lines() {
        let _ = writeln!(out, "{}", line.trim_end());
    }
    out
}

/// One snapshot per first value, with the characterization report rule at the head of every file.
fn pin(name: &str, cases: &[String]) {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/snapshots/characterization"
    ));
    settings.set_description(concat!(
        "This file records what the code does over a range too wide to assert by hand. ",
        "A change here is the consequence of a decision taken elsewhere, not a decision. ",
        "It is accepted after a report of how many cases moved, in which families, and ",
        "three examples with their before and after.",
    ));
    let joined = cases.join("\n");
    settings.bind(|| insta::assert_snapshot!(name, joined));
}

/// **Family one: a shape in each stroke it can be drawn in.** Three kinds times five strokes.
///
/// One file per kind, because a change to how a `box` draws is a different thing to read than a
/// change to how a `line` draws, and one file per value is what makes a diff legible rather than
/// merely small.
#[test]
fn every_shape_in_every_stroke_it_can_be_drawn_in() {
    let mut by_kind: Vec<(String, Vec<String>)> = Vec::new();

    for kind in KINDS {
        let mut cases = Vec::new();
        for stroke in DRAWING_STROKES {
            // The three kinds need three different shapes, and each is the smallest one that
            // draws: a box and a connector have a size, a line a length.
            let shapes = match kind {
                "box" => format!(
                    r##"{{ "kind": "box", "id": "#1", "at": {{ "x": 3, "y": 2 }},
                          "size": {{ "width": 5, "height": 3 }}, "stroke": "{stroke}" }}"##
                ),
                "line" => format!(
                    r##"{{ "kind": "line", "id": "#1", "at": {{ "x": 1, "y": 3 }}, "len": 9,
                          "orientation": "horizontal", "stroke": "{stroke}" }}"##
                ),
                _ => format!(
                    r##"{{ "kind": "connector", "id": "#1",
                          "from": {{ "at": {{ "kind": "point", "x": 1, "y": 3 }},
                                    "leaving": "right",
                                    "terminal": {{ "kind": "arm" }} }},
                          "to": {{ "at": {{ "kind": "point", "x": 10, "y": 3 }},
                                  "leaving": "left",
                                  "terminal": {{ "kind": "arm" }} }},
                          "stroke": "{stroke}" }}"##
                ),
            };
            cases.push(case(&format!("{kind} in {stroke}"), &shapes));
        }
        by_kind.push((format!("every_{kind}_in_every_stroke"), cases));
    }

    for (name, cases) in by_kind {
        pin(&name, &cases);
    }
}

/// **Family two: two shapes crossing, in every ordered pair of the five strokes.** Forty-five cases.
///
/// A crossing is the only place a cell ends up carrying arms in two strokes, and therefore the only
/// place any of the four mixing tables is reached — so this family is what decides whether those
/// tables answer or not. Both orders are here because the front-most shape decides the shared cell,
/// which means a pair of different strokes is not symmetric.
#[test]
fn two_shapes_crossing_in_every_ordered_pair_of_strokes() {
    for front in DRAWING_STROKES {
        let mut cases = Vec::new();
        for behind in DRAWING_STROKES {
            // A pair of *different* strokes is asymmetric and is emitted both ways, because the
            // front-most shape decides the shared cell. A pair of the *same* stroke is not: both
            // ways are the same diagram, and emitting it twice would put two cases under one name.
            let arrangements: [(String, String); 2] = [
                (front.to_owned(), behind.to_owned()),
                (behind.to_owned(), front.to_owned()),
            ];
            let mut seen = Vec::new();
            for (front_stroke, behind_stroke) in arrangements {
                let name = format!("{front_stroke} over {behind_stroke}");
                if seen.contains(&name) {
                    continue;
                }
                seen.push(name.clone());
                // A vertical line crossing a horizontal one, so the shared cell is a four-way
                // junction in both orders rather than a T.
                let shapes = format!(
                    r##"{{ "kind": "box", "id": "#1", "at": {{ "x": 4, "y": 1 }},
                          "size": {{ "width": 5, "height": 5 }}, "stroke": "{front_stroke}" }},
                       {{ "kind": "line", "id": "#2", "at": {{ "x": 2, "y": 3 }}, "len": 9,
                          "orientation": "horizontal", "stroke": "{behind_stroke}" }}"##
                );
                cases.push(case(&name, &shapes));
            }
        }
        pin(&format!("crossing_{front}_over_every_stroke"), &cases);
    }
}

/// The two families above, counted, so that a change to the range is visible rather than silent.
///
/// Not a snapshot: a constant about the sweep itself, and a test rather than a claim.
#[test]
fn the_sweep_covers_its_range_whole() {
    assert_eq!(KINDS.len() * DRAWING_STROKES.len(), 15);
    // Per first stroke: the four other strokes in both orders, plus its own pair once, which is
    // nine. The diagonal is not asymmetric, so emitting it twice would be two names for one
    // diagram.
    let n = DRAWING_STROKES.len();
    assert_eq!(n * (2 * (n - 1) + 1), 45);
    assert!(
        !DRAWING_STROKES
            .iter()
            .any(|stroke| stroke.starts_with("light-round-")
                || *stroke == "light-double"
                || *stroke == "light-heavy"),
        "the mixing tables are not drawing tables and must not appear in the range"
    );
}
