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

/// User story 2, acceptance scenario 1: with no arguments, the binary prints the shipped
/// demonstration and exits successfully, with nothing on stderr.
#[test]
fn no_arguments_prints_the_demonstration_and_exits_successfully() {
    let output = run(&[]);

    assert!(output.status.success(), "exited with {}", output.status);
    assert!(!output.stdout.is_empty(), "printed nothing");
    assert!(output.stderr.is_empty(), "wrote to stderr");
}

/// User story 2, acceptance scenario 2, FR-022: running with no arguments from a different
/// working directory prints the same diagram, since the demonstration is embedded in the binary
/// rather than read from a path relative to the working directory.
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

/// User story 2, acceptance scenario 3: passing the shipped demonstration's own path explicitly
/// produces byte-identical output to no arguments, since both parse the same text through the
/// same `Description::render` path.
#[test]
fn the_demo_path_passed_explicitly_matches_no_arguments() {
    let no_arguments = run(&[]);
    let explicit_path = run(&[DEMO_PATH]);

    assert!(no_arguments.status.success());
    assert!(explicit_path.status.success());
    assert_eq!(no_arguments.stdout, explicit_path.stdout);
}

/// User story 1, acceptance scenario 1: an explicit path to a hand-written single-box file
/// prints exactly that box.
#[test]
fn an_explicit_path_prints_the_hand_written_box() {
    let path = write_description(
        "one-box",
        r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "shapes": [
                { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░", "mode": "above" }
            ]
        }"#,
    );

    let output = run(&[path.to_str().expect("temp path should be valid UTF-8")]);

    assert!(output.status.success(), "exited with {}", output.status);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).into_owned(),
        "┌──┐\n│░░│\n└──┘\n"
    );
    assert!(output.stderr.is_empty(), "wrote to stderr");
}

/// User story 1, acceptance scenario 2: a box, a line and an arrow at stated positions print
/// all three composed.
#[test]
fn a_file_with_a_box_a_line_and_an_arrow_prints_all_three_composed() {
    let path = write_description(
        "three-shapes",
        r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 10, "height": 5 } },
            "shapes": [
                { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░", "mode": "above" },
                { "kind": "line", "at": { "x": 0, "y": 4 }, "len": 4, "orientation": "horizontal",
                  "stroke": "light", "mode": "above" },
                { "kind": "arrow",
                  "from": { "at": { "x": 5, "y": 0 }, "leaving": "right", "head": ">" },
                  "to": { "at": { "x": 9, "y": 2 }, "leaving": "down", "head": "v" },
                  "stroke": "light", "mode": "above" }
            ]
        }"#,
    );

    let output = run(&[path.to_str().expect("temp path should be valid UTF-8")]);

    assert!(output.status.success(), "exited with {}", output.status);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).into_owned(),
        "┌──┐ >┐   \n│░░│  │   \n└──┘  │  v\n      └──┘\n────      \n"
    );
    assert!(output.stderr.is_empty(), "wrote to stderr");
}

/// User story 1, acceptance scenario 3: the same file with its shapes reordered changes which
/// one is drawn on top where they overlap.
#[test]
fn reordering_shapes_changes_which_one_is_drawn_on_top() {
    let first = write_description(
        "overlap-a-then-b",
        r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 4 } },
            "shapes": [
                { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░", "mode": "above" },
                { "kind": "box", "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "▓", "mode": "above" }
            ]
        }"#,
    );
    let second = write_description(
        "overlap-b-then-a",
        r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 4 } },
            "shapes": [
                { "kind": "box", "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "▓", "mode": "above" },
                { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░", "mode": "above" }
            ]
        }"#,
    );

    let first_output = run(&[first.to_str().expect("temp path should be valid UTF-8")]);
    let second_output = run(&[second.to_str().expect("temp path should be valid UTF-8")]);

    assert_eq!(
        String::from_utf8_lossy(&first_output.stdout).into_owned(),
        "┌──┐  \n│░┌┴─┐\n└─┤▓▓│\n  └──┘\n"
    );
    assert_eq!(
        String::from_utf8_lossy(&second_output.stdout).into_owned(),
        "┌──┐  \n│░░├─┐\n└─┬┘▓│\n  └──┘\n"
    );
    assert_ne!(first_output.stdout, second_output.stdout);
}

/// User story 1, acceptance scenario 4, FR-017: running the same file twice produces identical
/// output.
#[test]
fn running_the_same_file_twice_produces_identical_output() {
    let path = write_description(
        "determinism",
        r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "shapes": [
                { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░", "mode": "above" }
            ]
        }"#,
    );

    let first = run(&[path.to_str().expect("temp path should be valid UTF-8")]);
    let second = run(&[path.to_str().expect("temp path should be valid UTF-8")]);

    assert_eq!(first.stdout, second.stdout);
}

/// User story 3, acceptance scenario 1: a path that does not exist prints nothing to stdout,
/// names the path on stderr, and fails.
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

/// User story 3, acceptance scenario 2: malformed JSON prints nothing to stdout, locates the
/// problem on stderr, and fails.
#[test]
fn malformed_json_locates_the_problem_on_stderr_and_fails() {
    let path = write_description("broken", r#"{ "canvas": "#);

    let output = run(&[path.to_str().expect("temp path should be valid UTF-8")]);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "printed to stdout");
    assert!(!output.stderr.is_empty(), "wrote nothing to stderr");
}

/// User story 3, acceptance scenario 3: an unrecognized shape kind prints nothing to stdout,
/// names the kind on stderr, and fails.
#[test]
fn an_unrecognized_kind_names_it_on_stderr_and_fails() {
    let path = write_description(
        "bad-kind",
        r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "shapes": [ { "kind": "triangle", "mode": "above" } ]
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

/// Returns the character at `(x, y)` in `output`, treating each line as a row and each `char` as
/// a column, the same coordinates the demo's `canvas` uses.
fn char_at(output: &str, x: usize, y: usize) -> char {
    output
        .lines()
        .nth(y)
        .and_then(|line| line.chars().nth(x))
        .unwrap_or_else(|| panic!("no character at ({x}, {y}) in {output:?}"))
}

/// User story 3, SC-003: with no arguments, the demonstration contains both a box-drawing
/// character (from Light) and one of `+`, `-`, `|` (from ASCII).
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

/// User story 3, FR-017, SC-009: at the two crossings between an ASCII figure and a Light figure
/// added to the demo, the shared cell reads from whichever figure is in front. `(28, 2)` is where
/// an ASCII box, drawn with `mode: "above"` after a Light box already on the canvas, shares a
/// border cell with it; `(36, 2)` is where a Light box, drawn with `mode: "above"` after an ASCII
/// box, shares a border cell with it.
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
/// tables that have no mixing set loaded (only Light+ASCII and Light+Heavy are documented, and
/// neither is loaded by the CLI). Per ADR-0009, a key missing from the catalog degrades every
/// connected arm to one base stroke — whichever figure was stamped last, since both boxes use
/// `mode: "above"`. Each crossing cell below is the same position the original two crossings use:
/// where the first box's bottom border meets the second box's left side.
#[test]
fn crossings_between_the_new_tables_degrade_to_whichever_figure_is_in_front() {
    let output = run(&[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    for (x, y, expected, label) in [
        (2, 11, '╬', "Double in front of Light"),
        (10, 11, '┼', "Light in front of Double"),
        (18, 11, '╋', "Heavy in front of Light"),
        (26, 11, '┼', "Light in front of Heavy"),
        (34, 11, '┼', "Light Round in front of Light"),
        (42, 11, '╬', "Double in front of Heavy"),
    ] {
        let ch = char_at(&stdout, x, y);
        assert_eq!(ch, expected, "{label}: expected {expected:?} at ({x}, {y})");
    }
}
