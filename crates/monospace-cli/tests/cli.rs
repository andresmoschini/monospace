//! End-to-end checks that run the built binary as a subprocess.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};

/// Writes `contents` to a fresh file under the system temp directory, named after the calling
/// test so parallel tests never collide.
fn write_description(name: &str, contents: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "monospace-cli-test-{name}-{}.json",
        std::process::id()
    ));
    std::fs::File::create(&path)
        .and_then(|mut file| file.write_all(contents.as_bytes()))
        .expect("temp description file should be writable");
    path
}

/// Runs the built binary with `args` and returns its captured output.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_monospace-cli"))
        .args(args)
        .output()
        .expect("the monospace-cli binary should be runnable")
}

/// The shipped demonstration's path, relative to the workspace root, so a test can pass it
/// explicitly the same way a user would.
const DEMO_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/demo.json");

/// The first of the five captioned pictures the no-argument run prints, with its caption line and
/// the blank line after it stripped. Pins no caption's wording.
///
/// The four pictures below it are appended rather than inserted, so the coordinates `char_at`
/// reads in the tests further down still name the same cells of the first picture — which is why
/// this function needs no change of its own when a picture is added.
///
/// Only the demonstration is captioned. A file's run is one picture and nothing else, so a test
/// that passes a path compares the whole of stdout.
fn first_demonstrated_picture(output: &str) -> String {
    let (first_block, _rest) = output
        .split_once("\n\n")
        .expect("two captioned pictures separated by a blank line");
    let (_caption, picture) = first_block
        .split_once('\n')
        .expect("a caption line precedes the picture");
    format!("{picture}\n")
}

/// Two entries carrying the same `id` are **both** read. Both shapes are drawn, the run exits
/// successfully, and nothing is written to stderr.
///
/// **Pinned as an accepted cost rather than as a bug**, so a later slice that decides to report a
/// repeated identity has to say so rather than discover it. The diagram-level half — both held,
/// `get` returning the first — is `add_under_does_not_move_the_counter_and_does_not_check_the_name`
/// in `monospace-diagram`; this is the same cost arriving through a wire, where a reader might
/// plausibly have checked it and did not.
#[test]
fn two_entries_carrying_one_identity_are_both_read_and_both_drawn() {
    let path = write_description(
        "repeated-id",
        r##"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 4 } },
            "next_id": 2,
            "shapes": [
                { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light" },
                { "kind": "box", "id": "#1", "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light" }
            ]
        }"##,
    );

    let output = run(&[path.to_str().expect("temp path should be valid UTF-8")]);

    assert!(output.status.success(), "exited with {}", output.status);
    assert!(output.stderr.is_empty(), "wrote to stderr");
    // Both boxes draw: the first entry's border and the second one's, overlapping. A reader that
    // refused a repeated identity would have drawn one of them, and one that read the first and
    // dropped the second would draw a different picture than two of them do.
    let picture = String::from_utf8_lossy(&output.stdout);
    assert!(
        picture.contains("┌──┐") && picture.contains("└──┘") && picture.lines().count() >= 4,
        "both entries must draw, and a 6x4 window is four rows:\n{picture}"
    );
}

/// With no arguments, the binary prints the shipped demonstration and exits successfully, with
/// nothing on stderr.
#[test]
fn no_arguments_prints_the_demonstration_and_exits_successfully() {
    let output = run(&[]);

    assert!(output.status.success(), "exited with {}", output.status);
    assert!(!output.stdout.is_empty(), "printed nothing");
    assert!(output.stderr.is_empty(), "wrote to stderr");
}

/// Running with no arguments from a different working directory prints the same diagram, since
/// the demonstration is embedded in the binary rather than read from a path relative to the
/// working directory.
#[test]
fn running_from_a_different_working_directory_prints_the_same_diagram() {
    let from_temp_dir = Command::new(env!("CARGO_BIN_EXE_monospace-cli"))
        .current_dir(std::env::temp_dir())
        .output()
        .expect("the monospace-cli binary should be runnable from the temp directory");
    let from_manifest_dir = Command::new(env!("CARGO_BIN_EXE_monospace-cli"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("the monospace-cli binary should be runnable from its manifest directory");

    assert_eq!(from_temp_dir.stdout, from_manifest_dir.stdout);
}

/// The shipped demonstration's own path, passed explicitly, prints the picture the no-argument
/// run prints first — and stops there.
///
/// Byte-identical output from both runs was asked for once. It no longer holds, deliberately: the
/// caption and the moved shape belong to the demonstration, not to a description someone hands
/// the binary. What survives is that both parse the same text and draw it the same way.
#[test]
fn the_demo_path_passed_explicitly_prints_the_demonstrations_first_picture() {
    let no_arguments = run(&[]);
    let explicit_path = run(&[DEMO_PATH]);

    assert!(no_arguments.status.success());
    assert!(explicit_path.status.success());
    assert_eq!(
        String::from_utf8_lossy(&explicit_path.stdout),
        first_demonstrated_picture(&String::from_utf8_lossy(&no_arguments.stdout))
    );
}

/// An explicit path to a hand-written single-box file prints exactly that box.
#[test]
fn an_explicit_path_prints_the_hand_written_box() {
    let path = write_description(
        "one-box",
        r##"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 2,
            "shapes": [
                { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░" }
            ]
        }"##,
    );

    let output = run(&[path.to_str().expect("temp path should be valid UTF-8")]);

    assert!(output.status.success(), "exited with {}", output.status);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "┌──┐\n│░░│\n└──┘\n"
    );
    assert!(output.stderr.is_empty(), "wrote to stderr");
}

/// A box, a line and a connector at stated positions print all three composed.
#[test]
fn a_file_with_a_box_a_line_and_a_connector_prints_all_three_composed() {
    let path = write_description(
        "three-shapes",
        r##"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 10, "height": 5 } },
            "next_id": 4,
            "shapes": [
                { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░" },
                { "kind": "line", "id": "#2", "at": { "x": 0, "y": 4 }, "len": 4, "orientation": "horizontal",
                  "stroke": "light" },
                { "kind": "connector", "id": "#3",
                  "from": { "at": { "kind": "point", "x": 5, "y": 0 }, "leaving": "right",
                            "terminal": { "kind": "glyph", "glyph": ">" } },
                  "to": { "at": { "kind": "point", "x": 9, "y": 2 }, "leaving": "down",
                          "terminal": { "kind": "glyph", "glyph": "v" } },
                  "stroke": "light" }
            ]
        }"##,
    );

    let output = run(&[path.to_str().expect("temp path should be valid UTF-8")]);

    assert!(output.status.success(), "exited with {}", output.status);
    // Corrected on feature 099: this connector's route was a genuine tie under _The route of a
    // connector_ (fewest bends, same closeness), which feature 039 broke arbitrarily by comparing
    // waypoint coordinates. The model's own tie-break prefers the route that turns at the middle
    // of the route rectangle (x = 7), which is what this picture now pins.
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "┌──┐ >─┐  \n│░░│   │  \n└──┘   │ v\n       └─┘\n────      \n"
    );
    assert!(output.stderr.is_empty(), "wrote to stderr");
}

/// A box at the origin, four by three, so its right side centre is `{3, 1}`. The figure the three
/// wire cases below hang an endpoint from.
const A_BOX: &str = r##"{ "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
        "stroke": "light" }"##;

/// Runs a one-connector description over `at` and returns the whole of stdout, which for a path is
/// one picture and nothing else — nothing else because a picture bound for a Markdown fence cannot
/// arrive wrapped in prose, and `cargo xtask render` embeds this output in one.
///
/// `label` names the temp file, so it is the caller's short handle and not the JSON: a description
/// is full of slashes and quotes and makes for no filename at all.
fn picture_of_a_connector_hanging_from(label: &str, at: &str) -> String {
    let path = write_description(
        label,
        &format!(
            r##"{{
            "canvas": {{ "origin": {{ "x": 0, "y": 0 }}, "size": {{ "width": 9, "height": 3 }} }},
            "next_id": 3,
            "shapes": [
                {A_BOX},
                {{ "kind": "connector", "id": "#2",
                  "from": {{ "at": {at}, "leaving": "right",
                             "terminal": {{ "kind": "glyph", "glyph": ">" }} }},
                  "to": {{ "at": {{ "kind": "point", "x": 8, "y": 1 }}, "leaving": "left",
                           "terminal": {{ "kind": "arm" }} }},
                  "stroke": "light" }}
            ]
        }}"##
        ),
    );

    let output = run(&[path.to_str().expect("temp path should be valid UTF-8")]);
    assert!(output.status.success(), "exited with {}", output.status);
    assert!(output.stderr.is_empty(), "wrote to stderr");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// A file may name a side of another figure and an offset, and the picture it draws is the
/// picture the resolved point draws.
#[test]
fn a_file_naming_a_reference_draws_what_the_point_it_resolves_to_draws() {
    let named = picture_of_a_connector_hanging_from(
        "with-offset",
        r##"{ "kind": "reference", "shape": "#1", "anchor": "right", "offset": { "dx": 2, "dy": 0 } }"##,
    );
    let spelled = picture_of_a_connector_hanging_from(
        "with-offset-point",
        r#"{ "kind": "point", "x": 5, "y": 1 }"#,
    );

    assert_eq!(named, spelled);
    assert!(
        !named.trim().is_empty(),
        "a reference to a box the file holds must draw the connector"
    );
}

/// A reference carrying **no** `offset` at all draws exactly what a description naming the point
/// it resolves to draws, byte for byte.
///
/// The case `{"kind": "reference", "shape": "#1", "anchor": "right"}` exists for: `offset` is
/// optional and absent is zero, so a reference on the side itself is three fields rather than five,
/// and it has to draw the same thing a file spelling `{3, 1}` outright does.
#[test]
fn a_file_naming_a_reference_with_no_offset_draws_the_point_it_stands_on() {
    let named = picture_of_a_connector_hanging_from(
        "no-offset",
        r##"{ "kind": "reference", "shape": "#1", "anchor": "right" }"##,
    );
    let spelled = picture_of_a_connector_hanging_from(
        "no-offset-point",
        r#"{ "kind": "point", "x": 3, "y": 1 }"#,
    );

    assert_eq!(named, spelled);
}

/// A file naming a shape it does not hold draws **a box and no connector at all**, and the run
/// succeeds.
///
/// This is a hole that was silent below the wire arriving through it — a position that cannot be
/// resolved is not drawn and nothing reports that it was skipped — and the offset is what makes it
/// a test rather than a comment: a large offset is the case where an implementation might clamp,
/// fall back or report, and none of the three happens. Nothing on stderr, a successful exit, and
/// the picture is the box alone.
#[test]
fn a_file_naming_a_shape_it_does_not_hold_draws_the_box_and_no_connector_and_succeeds() {
    let drawn = picture_of_a_connector_hanging_from(
        "foreign-shape",
        r##"{ "kind": "reference", "shape": "#7", "anchor": "right", "offset": { "dx": 2, "dy": 0 } }"##,
    );
    let box_alone = {
        let path = write_description(
            "just-the-box",
            &format!(
                r#"{{
                "canvas": {{ "origin": {{ "x": 0, "y": 0 }}, "size": {{ "width": 9, "height": 3 }} }},
                "next_id": 2,
                "shapes": [ {A_BOX} ]
            }}"#
            ),
        );
        let output = run(&[path.to_str().expect("temp path should be valid UTF-8")]);
        assert!(output.status.success(), "exited with {}", output.status);
        assert!(output.stderr.is_empty(), "wrote to stderr");
        String::from_utf8_lossy(&output.stdout).into_owned()
    };

    assert_eq!(
        drawn, box_alone,
        "a reference to nothing must draw the box and nothing else"
    );
}

/// The two overlapping boxes the reordering test below writes, as the `monospace_core` shapes
/// they describe, so the expected picture in each order comes from stamping them directly rather
/// than from a literal picture.
/// they describe, so the expected picture in each order comes from stamping them directly rather
/// than from a literal picture.
fn overlap_boxes() -> (monospace_core::BoxShape, monospace_core::BoxShape) {
    use monospace_core::{BoxShape, Glyph, Pos, Size, Stroke};

    (
        BoxShape {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: Stroke::from("light"),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        },
        BoxShape {
            at: Pos { x: 2, y: 1 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: Stroke::from("light"),
            fill: Some(Glyph::new("▓").expect("\"▓\" is one glyph")),
        },
    )
}

/// Renders `back` then `front`, back to front, each stamped with `StampMode::Above` — the picture
/// a description's `shapes` array in that order produces, per _The two orders are equivalent_.
fn render_back_to_front_with_above(
    back: &monospace_core::BoxShape,
    front: &monospace_core::BoxShape,
) -> String {
    use monospace_core::{Buffer, GlyphCatalog, Layer, Pos, Shape, Size, StampMode, render};

    let origin = Pos { x: 0, y: 0 };
    let size = Size {
        width: 6,
        height: 4,
    };
    let mut buffer = Buffer::new(origin, size);
    back.draw(&mut Layer::new(&mut buffer, StampMode::Above));
    front.draw(&mut Layer::new(&mut buffer, StampMode::Above));
    render(&buffer, &GlyphCatalog::light(), origin, size)
}

/// The same file with its shapes reordered changes which one is drawn on top where they overlap
/// — the last entry in `shapes` is front-most and decides the shared cells, matching the two core
/// shapes stamped back to front with `Above`.
#[test]
fn reordering_shapes_changes_which_one_is_drawn_on_top() {
    let first = write_description(
        "overlap-a-then-b",
        r##"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 4 } },
            "next_id": 3,
            "shapes": [
                { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░" },
                { "kind": "box", "id": "#2", "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "▓" }
            ]
        }"##,
    );
    let second = write_description(
        "overlap-b-then-a",
        r##"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 4 } },
            "next_id": 3,
            "shapes": [
                { "kind": "box", "id": "#1", "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "▓" },
                { "kind": "box", "id": "#2", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░" }
            ]
        }"##,
    );

    let first_output = run(&[first.to_str().expect("temp path should be valid UTF-8")]);
    let second_output = run(&[second.to_str().expect("temp path should be valid UTF-8")]);

    let (a, b) = overlap_boxes();
    assert_eq!(
        String::from_utf8_lossy(&first_output.stdout),
        render_back_to_front_with_above(&a, &b)
    );
    assert_eq!(
        String::from_utf8_lossy(&second_output.stdout),
        render_back_to_front_with_above(&b, &a)
    );
    assert_ne!(first_output.stdout, second_output.stdout);
}

/// Running the same file twice produces identical output.
#[test]
fn running_the_same_file_twice_produces_identical_output() {
    let path = write_description(
        "determinism",
        r##"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 2,
            "shapes": [
                { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░" }
            ]
        }"##,
    );

    let first = run(&[path.to_str().expect("temp path should be valid UTF-8")]);
    let second = run(&[path.to_str().expect("temp path should be valid UTF-8")]);

    assert_eq!(first.stdout, second.stdout);
}

/// A path that does not exist prints nothing to stdout, names the path on stderr, and fails.
#[test]
fn a_missing_path_names_it_on_stderr_and_fails() {
    let path = std::env::temp_dir().join(format!(
        "monospace-cli-test-does-not-exist-{}.json",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);

    let output = run(&[path.to_str().expect("temp path should be valid UTF-8")]);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "printed to stdout");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(path.to_str().expect("temp path should be valid UTF-8")),
        "{stderr}"
    );
}

/// Malformed JSON prints nothing to stdout, locates the problem on stderr, and fails.
#[test]
fn malformed_json_locates_the_problem_on_stderr_and_fails() {
    let path = write_description("broken", r#"{ "canvas": "#);

    let output = run(&[path.to_str().expect("temp path should be valid UTF-8")]);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "printed to stdout");
    assert!(!output.stderr.is_empty(), "wrote nothing to stderr");
}

/// An unrecognized shape kind prints nothing to stdout, names the kind on stderr, and fails.
#[test]
fn an_unrecognized_kind_names_it_on_stderr_and_fails() {
    let path = write_description(
        "bad-kind",
        r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 1,
            "shapes": [ { "kind": "triangle" } ]
        }"#,
    );

    let output = run(&[path.to_str().expect("temp path should be valid UTF-8")]);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "printed to stdout");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("triangle"), "{stderr}");
}

/// Edge case: more than one command-line argument prints a usage message and fails, instead of
/// being matched on as a path.
#[test]
fn more_than_one_argument_prints_usage_and_fails() {
    let output = run(&["one.json", "two.json"]);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "printed to stdout");
    assert!(!output.stderr.is_empty(), "wrote nothing to stderr");
}

/// Returns the character at `(x, y)` in the first picture of `output`, treating each line as a
/// row and each `char` as a column, the same coordinates the demo's `canvas` uses. `+ 1` skips
/// the caption line the demonstration puts above that picture.
fn char_at(output: &str, x: usize, y: usize) -> char {
    output
        .lines()
        .nth(y + 1)
        .and_then(|line| line.chars().nth(x))
        .unwrap_or_else(|| panic!("no character at ({x}, {y}) in {output:?}"))
}

/// With no arguments, the demonstration contains both a box-drawing character (from Light) and
/// one of `+`, `-`, `|` (from ASCII).
#[test]
fn the_demonstration_contains_both_light_and_ascii_characters() {
    let output = run(&[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains(['┌', '┐', '└', '┘', '│', '─', '┬', '┴', '├', '┤', '┼']),
        "no box-drawing character in {stdout:?}"
    );
    assert!(
        stdout.contains(['+', '-', '|']),
        "no ASCII box character in {stdout:?}"
    );
}

/// At the two crossings between an ASCII figure and a Light figure added to the demo, the shared
/// cell reads from whichever figure is in front. `(28, 2)` is where
/// an ASCII box, added after a Light box already on the canvas and so front-most, shares a border
/// cell with it; `(36, 2)` is where a Light box, added after an ASCII box and so front-most,
/// shares a border cell with it.
#[test]
fn the_two_crossings_read_from_whichever_figure_is_in_front() {
    let output = run(&[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let ascii_in_front = char_at(&stdout, 28, 2);
    assert!(
        matches!(ascii_in_front, '+' | '-' | '|'),
        "expected an ASCII character at (28, 2), got {ascii_in_front:?}"
    );

    let light_in_front = char_at(&stdout, 36, 2);
    assert_eq!(
        light_in_front, '┼',
        "expected the Light crossing character at (36, 2)"
    );
}

/// Feature 056 follow-up: the demonstration also contains a box drawn with each of Double, Heavy
/// and Light Round, so a no-argument run shows all five built-in tables at once.
#[test]
fn the_demonstration_contains_double_heavy_and_light_round_characters() {
    let output = run(&[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains(['║', '╗', '╦', '╬', '╣', '╔', '╠', '═', '╩', '╝', '╚']),
        "no Double box character in {stdout:?}"
    );
    assert!(
        stdout.contains(['┃', '┓', '┳', '╋', '┫', '┏', '┣', '━', '┻', '┛', '┗']),
        "no Heavy box character in {stdout:?}"
    );
    assert!(
        stdout.contains(['╮', '╭', '╯', '╰']),
        "no Light Round corner character in {stdout:?}"
    );
}

/// Each pair of overlapping boxes below the original demonstration crosses two single-stroke
/// tables. Feature 060 loads mixing tables for two of the six pairs here — Light with Double and
/// Light with Heavy — so their crossing now draws the character the exact mixed arms produce
/// instead of degrading. The other two pairs (Light Round with Light, Double with Heavy) still
/// have no mixing table pairing them, so they still degrade every connected arm to the one base
/// stroke — the stroke of the figure that last claimed the cell, which is what makes the answer to
/// "why that character?" a single sentence, and whichever figure is added last since it is
/// front-most and decides first.
/// Each crossing cell below is the same position the original two crossings use: where the first
/// box's bottom border meets the second box's left side.
#[test]
fn crossings_between_the_new_tables_mix_or_degrade_depending_on_table_coverage() {
    let output = run(&[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    for (x, y, expected, label) in [
        (
            2,
            11,
            '╫',
            "Double in front of Light: covered by Light/Double, mixes instead of degrading",
        ),
        (
            10,
            11,
            '╪',
            "Light in front of Double: covered by Light/Double, mixes instead of degrading",
        ),
        (
            18,
            11,
            '╂',
            "Heavy in front of Light: covered by Light/Heavy, mixes instead of degrading",
        ),
        (
            26,
            11,
            '┿',
            "Light in front of Heavy: covered by Light/Heavy, mixes instead of degrading",
        ),
        (
            34,
            11,
            '┼',
            "Light Round in front of Light: no mixing table pairs them, still degrades",
        ),
        (
            42,
            11,
            '╬',
            "Double in front of Heavy: no mixing table pairs them, still degrades",
        ),
    ] {
        let ch = char_at(&stdout, x, y);
        assert_eq!(ch, expected, "{label}: expected {expected:?} at ({x}, {y})");
    }
}
