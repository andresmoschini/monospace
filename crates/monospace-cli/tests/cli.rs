//! End-to-end checks that run the built binary as a subprocess.

use std::process::Command;

/// The binary must print the single 4x3 box from spec 0001's own "A box" example, a blank line,
/// then the same box stamped twice — at `(0, 0)` and at `(2, 1)` — through `monospace-core`'s
/// public API, and nothing else.
///
/// In the pair, `┴` and `┤` open toward the first box: the second box abstains on the sides
/// facing outward, so the first box's stroke survives on the side facing away from the second,
/// per spec 0002's rule 3.
#[test]
fn prints_the_box_then_the_crossed_pair() {
    let output = Command::new(env!("CARGO_BIN_EXE_monospace-cli"))
        .output()
        .expect("the monospace-cli binary should be runnable");

    assert!(output.status.success(), "exited with {}", output.status);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).into_owned(),
        "┌──┐\n│  │\n└──┘\n\n┌──┐  \n│ ┌┴─┐\n└─┤┘ │\n  └──┘\n"
    );
    assert!(output.stderr.is_empty(), "wrote to stderr");
}
