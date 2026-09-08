//! End-to-end checks that run the built binary as a subprocess.

use std::process::Command;

/// The binary must print the single 4x3 box, then the same pair of overlapping boxes stamped two
/// ways: the second box `Above`, then `Below`, each pair labelled.
///
/// In the `Above` pair, `┴` and `┤` open toward the first box: the second box abstains outward,
/// so the first box's stroke survives on the side facing away from the second. In the `Below`
/// pair the roles swap — `├` and `┬` open toward the second box, because `Below` only writes the
/// sides the first box left `Unset`, which are exactly the ones facing away from it.
#[test]
fn prints_the_box_then_both_pairs() {
    let output = Command::new(env!("CARGO_BIN_EXE_monospace-cli"))
        .output()
        .expect("the monospace-cli binary should be runnable");

    assert!(output.status.success(), "exited with {}", output.status);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).into_owned(),
        "┌──┐\n│  │\n└──┘\n\nAbove:\n┌──┐  \n│ ┌┴─┐\n└─┤┘ │\n  └──┘\n\nBelow:\n┌──┐  \n│ ┌├─┐\n└─┬┘ │\n  └──┘\n"
    );
    assert!(output.stderr.is_empty(), "wrote to stderr");
}
