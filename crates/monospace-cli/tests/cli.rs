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

/// The binary must print the single 4x3 box with its interior filled, then the same pair of
/// overlapping filled boxes stamped two ways: the second box `Above`, then `Below`, each pair
/// labelled.
///
/// In the `Above` pair, `┴` and `┤` open toward the first box: the second box abstains outward,
/// so the first box's stroke survives on the side facing away from the second — and the second
/// box's fill (`░`) covers the first box's corner, since the second box is in front. In the
/// `Below` pair the roles swap — `├` and `┬` open toward the second box, because `Below` only
/// writes the sides the first box left `Unset`, and the first box's fill survives over the second
/// box's corner instead, since the first box is in front there.
#[test]
fn prints_the_box_then_both_pairs() {
    let output = Command::new(env!("CARGO_BIN_EXE_monospace-cli"))
        .output()
        .expect("the monospace-cli binary should be runnable");

    assert!(output.status.success(), "exited with {}", output.status);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).into_owned(),
        "┌──┐\n│░░│\n└──┘\n\nAbove:\n┌──┐  \n│░┌┴─┐\n└─┤░░│\n  └──┘\n\nBelow:\n┌──┐  \n│░░├─┐\n└─┬┘░│\n  └──┘\n"
    );
    assert!(output.stderr.is_empty(), "wrote to stderr");
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
