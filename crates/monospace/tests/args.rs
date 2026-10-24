//! End-to-end checks that run the built binary as a subprocess.
//!
//! **Only the failures are here.** The application takes the screen as soon as it has a session,
//! so a run that succeeds belongs to the frame's unit tests rather than to a subprocess. A path
//! that cannot be read or parsed is reported before the loop starts, which is what makes it the
//! one behavior a subprocess can observe from the outside.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};

/// Writes `contents` to a fresh file under the system temp directory, named after the calling
/// test so parallel tests never collide.
fn write_description(name: &str, contents: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("monospace-test-{name}-{}.json", std::process::id()));
    std::fs::File::create(&path)
        .and_then(|mut file| file.write_all(contents.as_bytes()))
        .expect("temp description file should be writable");
    path
}

/// Runs the built binary with `args` and returns its captured output.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_monospace"))
        .args(args)
        .output()
        .expect("the monospace binary should be runnable")
}

/// The failure the binary reported, with its prefix and both terminals back.
fn reported(output: &Output) -> String {
    assert!(
        !output.status.success(),
        "the application succeeded on a path it cannot read"
    );
    assert!(
        output.stdout.is_empty(),
        "the application painted a screen: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );

    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn a_path_that_cannot_be_read_is_reported() {
    let path = std::env::temp_dir().join(format!(
        "monospace-test-missing-{}.json",
        std::process::id()
    ));
    let named = path.to_str().expect("the temp path is UTF-8");

    let stderr = reported(&run(&[named]));

    assert!(
        stderr.starts_with("monospace: "),
        "the failure is not the application's: {stderr:?}"
    );
    assert!(
        stderr.contains(named),
        "the failure does not name the path it could not read: {stderr:?}"
    );
}

#[test]
fn a_path_that_is_not_a_description_is_reported() {
    let path = write_description("not-a-description", "{ this is not JSON }");
    let named = path.to_str().expect("the temp path is UTF-8");

    let stderr = reported(&run(&[named]));

    assert!(
        stderr.starts_with("monospace: "),
        "the failure is not the application's: {stderr:?}"
    );
    assert!(
        stderr.contains("line 1"),
        "the failure does not locate the problem: {stderr:?}"
    );
}
