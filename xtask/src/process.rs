//! The subprocess wrappers the repository's automation shares.
//!
//! `spec` and `pr` both drive `git` and `gh`, and both want the same two shapes: a command whose
//! progress the operator watches, and a command whose output is read back. Keeping them here is
//! what lets the logic that decides *what* to run stay in plain functions that take values and
//! return `Result`, in the module that owns the decision.

use std::path::Path;
use std::process::Command;

/// The environment variables that name a repository other than the one at the current directory.
///
/// Git exports `GIT_DIR` and `GIT_INDEX_FILE` to every hook it runs, and `GIT_COMMON_DIR` and
/// `GIT_WORK_TREE` beside them for a linked worktree. A child process inherits all of them, and git
/// gives them precedence over the directory it was started in — so a command that means "the
/// repository at `root`" silently becomes a command about the caller's instead.
const INHERITED_GIT_ENV: [&str; 4] = [
    "GIT_DIR",
    "GIT_INDEX_FILE",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
];

/// Builds a command that runs `program` from `root` and inherits nothing that names a repository.
///
/// # Design notes
///
/// This is the one place the removal happens, because it is the one place every subprocess is built,
/// and the alternative is a rule each caller has to remember. `git commit` is what makes it matter:
/// the pre-commit hook runs this crate with `GIT_DIR` and `GIT_INDEX_FILE` pointing at the repository
/// being committed to, so without this the gate reads that repository's index while `root` says it is
/// somewhere else.
///
/// Measured, both ways. With the variables inherited, the gate run from inside the hook reported
/// `could not read specs/163-one.md` for a file no repository holds, and the fixture in
/// `spec::tests` wrote its own paths into the caller's index and flipped its `core.bare` to true —
/// after which `git status` refused to run at all. None of it appears in a plain `cargo xtask check`,
/// which git launches with nothing set, which is why it went unnoticed: the defect only exists on the
/// path the hook takes.
///
/// What breaks if it changes: nothing outside this crate. `root` is the repository every caller means,
/// and a caller that ran this with the variables deliberately set would be asking for the wrong
/// repository.
pub(crate) fn command(root: &Path, program: &str) -> Command {
    let mut command = Command::new(program);
    command.current_dir(root);
    for name in INHERITED_GIT_ENV {
        command.env_remove(name);
    }
    command
}

/// Confirms `gh` is installed and authenticated, distinguishing the two failure modes so the
/// fix is never ambiguous.
pub(crate) fn ensure_gh_ready(root: &Path) -> Result<(), String> {
    if command(root, "gh").arg("--version").output().is_err() {
        return Err(
            "xtask: `gh` is not installed. Install it from https://cli.github.com and try again."
                .to_string(),
        );
    }

    let authenticated = command(root, "gh")
        .args(["auth", "status"])
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
    match command(root, program).args(args).status() {
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
    let output = command(root, program)
        .args(args)
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

#[cfg(test)]
mod tests {
    use super::{INHERITED_GIT_ENV, command};
    use std::path::Path;

    /// Every variable that would name another repository is cleared, and nothing else is.
    ///
    /// The comparison is of the two sets rather than of two lists: `Command` keeps its environment in
    /// a map, so `get_envs` answers in sorted order and a list comparison would fail on the order
    /// rather than on the decision.
    ///
    /// This is a contract test on the constructor rather than on what any caller goes on to do with
    /// it, because the consequence is invisible from outside: a command that inherits these reads a
    /// different repository and reports success.
    #[test]
    fn a_command_clears_every_variable_that_names_another_repository() {
        let mut cleared: Vec<String> = command(Path::new("."), "git")
            .get_envs()
            .filter(|&(_, value)| value.is_none())
            .map(|(name, _)| name.to_string_lossy().into_owned())
            .collect();
        cleared.sort();

        let mut expected = INHERITED_GIT_ENV.map(String::from);
        expected.sort();

        assert_eq!(
            cleared, expected,
            "every variable naming the caller's repository is cleared, and nothing else is"
        );
    }
}
