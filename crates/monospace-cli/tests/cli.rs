//! End-to-end checks that run the built binary as a subprocess.

use std::process::Command;

/// The binary must print exactly what the core library returns, and nothing else.
///
/// Comparing against `monospace_core::greeting()` rather than against a literal is deliberate: the
/// point is to check that the two crates stay wired together, not to restate the greeting in a
/// second place where it could drift.
#[test]
fn prints_the_greeting_from_the_core_library() {
    let output = Command::new(env!("CARGO_BIN_EXE_monospace-cli"))
        .output()
        .expect("the monospace-cli binary should be runnable");

    assert!(output.status.success(), "exited with {}", output.status);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).into_owned(),
        format!("{}\n", monospace_core::greeting())
    );
    assert!(output.stderr.is_empty(), "wrote to stderr");
}
