//! The subprocess wrappers the repository's automation shares.
//!
//! `spec` and `pr` both drive `git` and `gh`, and both want the same two shapes: a command whose
//! progress the operator watches, and a command whose output is read back. Keeping them here is
//! what lets the logic that decides *what* to run stay in plain functions that take values and
//! return `Result`, in the module that owns the decision.

use std::path::Path;
use std::process::Command;

/// Confirms `gh` is installed and authenticated, distinguishing the two failure modes so the
/// fix is never ambiguous.
pub(crate) fn ensure_gh_ready(root: &Path) -> Result<(), String> {
    if Command::new("gh")
        .arg("--version")
        .current_dir(root)
        .output()
        .is_err()
    {
        return Err(
            "xtask: `gh` is not installed. Install it from https://cli.github.com and try again."
                .to_string(),
        );
    }

    let authenticated = Command::new("gh")
        .args(["auth", "status"])
        .current_dir(root)
        .output()
        .is_ok_and(|output| output.status.success());

    if authenticated {
        Ok(())
    } else {
        Err(
            "xtask: `gh` is installed but not authenticated. Run `gh auth login` and try again."
                .to_string(),
        )
    }
}

/// Runs `program` with `args` from `root`, inheriting the child's stdio so its progress streams to
/// the operator as it happens. Used for the commands worth watching live: `git fetch`,
/// `gh issue develop`, `git checkout`.
pub(crate) fn run_visible(root: &Path, program: &str, args: &[&str]) -> Result<(), String> {
    match Command::new(program).args(args).current_dir(root).status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!(
            "xtask: `{program} {}` exited with {status}",
            args.join(" ")
        )),
        Err(error) => Err(format!("xtask: could not run `{program}`: {error}")),
    }
}

/// Runs `program` with `args` from `root` and returns its trimmed standard output.
///
/// # Errors
///
/// Names the program, its arguments and the captured standard error when the command cannot be
/// spawned or exits with a failure status.
pub(crate) fn capture(root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    capture_untrimmed(root, program, args).map(|output| output.trim().to_string())
}

/// Runs `program` with `args` from `root` and returns its standard output as it came.
///
/// Reading a file out of `origin/main` is the one caller that needs this: trimming would drop a
/// leading blank line and shift every line number reported back to the operator by one.
///
/// # Errors
///
/// As `capture`.
pub(crate) fn capture_untrimmed(
    root: &Path,
    program: &str,
    args: &[&str],
) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("xtask: could not run `{program}`: {error}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!(
            "xtask: `{program} {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}
