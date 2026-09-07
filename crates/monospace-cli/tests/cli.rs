//! End-to-end checks that run the built binary as a subprocess.

use std::process::Command;

/// The binary must print exactly the 4x3 box from spec 0001's own "A box" example, drawn cell by
/// cell through `monospace-core`'s public API, and nothing else.
#[test]
fn prints_the_box_from_the_public_api() {
    let output = Command::new(env!("CARGO_BIN_EXE_monospace-cli"))
        .output()
        .expect("the monospace-cli binary should be runnable");

    assert!(output.status.success(), "exited with {}", output.status);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).into_owned(),
        "┌──┐\n│  │\n└──┘\n"
    );
    assert!(output.stderr.is_empty(), "wrote to stderr");
}
