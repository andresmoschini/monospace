//! Rewriting the line endings of every file Git would stage.
//!
//! `.gitattributes` sets one rule for everything Git considers text — `* text=auto eol=lf` — and
//! `core.safecrlf` is on, so a file written with CRLF is one `git add` refuses rather than
//! converts. The Speckit CLI writes CRLF on Windows and rewrites several files on every `specify
//! update`, which is what puts the two rules in conflict
//! ([ADR-0059](../../docs/decisions/0059-normalize-what-the-speckit-cli-writes-or-git-refuses-it.md)).
//!
//! # Design notes
//!
//! **Git answers both questions this step has to ask.** `git ls-files --eol` reports the endings
//! in the worktree copy of every tracked and every untracked-but-not-ignored file, and the
//! attributes `.gitattributes` gives it. Whether those bytes are text at all is Git's own decision,
//! spelled `-text`; which files are wanted in CRLF is `*.bat` and `*.cmd` and nothing else. A list
//! of extensions written here would be a second thing to keep in step with the one Git enforces, and
//! keeping it in step is exactly the work this step exists to remove.
//!
//! **The decision is made from what Git reported, not from what was read.** A copy reported as
//! `crlf` or `mixed`, whose attributes do not ask for CRLF, is rewritten; the run after that reports
//! it `lf` and finds nothing to do. Idempotence is a consequence of the decision rather than
//! something a test has to notice.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::process::capture_untrimmed;

/// The pair of bytes a CRLF line ending is made of.
const CRLF: &[u8] = b"\r\n";
/// The byte a line ends with when there is nothing else to it.
const LF: u8 = b'\n';

/// What `git ls-files --eol` reports about one file.
#[derive(Debug)]
struct Reported<'a> {
    /// The path relative to the workspace root, in Git's own spelling.
    relative: &'a str,
    /// What Git found in the worktree copy.
    endings: &'a str,
    /// Whether `.gitattributes` asks for CRLF, which is the one place it is the right answer.
    wants_crlf: bool,
}

/// `cargo xtask fix`'s step: rewrites every file Git would stage that carries CRLF the repository
/// does not ask for.
///
/// It sits beside `editorconfig` rather than inside it, and last rather than first:
/// `editorconfig-checker` reads the same rule and is configured to skip `.specify/`, where a CRLF
/// file is one `git add` refuses with no command to fix it by; and every step before this one
/// writes, so the ending of a line is the last thing a byte should be decided on.
pub fn fix(root: &Path) -> bool {
    crate::report_failure(rewrite(root))
}

/// Rewrites every worktree copy that needs it, and reports which those were.
fn rewrite(root: &Path) -> Result<(), String> {
    let reported = reported(root)?;
    let mut rewritten = Vec::new();

    for record in reported.split('\0').filter(|record| !record.is_empty()) {
        let file = parse(record)?;
        if !needs_rewrite(file.endings, file.wants_crlf) {
            continue;
        }
        if let Some(lines) = rewrite_one(root, &file)? {
            rewritten.push(format!(
                "  {}: {lines} line(s) now end in LF",
                file.relative
            ));
        }
    }

    if rewritten.is_empty() {
        println!("every file Git would stage already ends its lines the way .gitattributes asks");
        return Ok(());
    }

    for line in &rewritten {
        println!("{line}");
    }
    println!("{} file(s) rewritten", rewritten.len());

    Ok(())
}

/// Asks Git for every file it would stage, and what it found in each worktree copy.
///
/// `--cached --others --exclude-standard` is the tracked files plus the new ones that are not
/// ignored: the set `editorconfig-checker` and `cspell` read, so a file this step never sees is a
/// file no step in the gate has an opinion about either.
fn reported(root: &Path) -> Result<String, String> {
    capture_untrimmed(
        root,
        "git",
        &[
            "ls-files",
            "-z",
            "--eol",
            "--cached",
            "--others",
            "--exclude-standard",
        ],
    )
}

/// Reads one NUL-separated record of `git ls-files --eol` output.
///
/// A record is three space-padded fields, a tab, and the path, and `-z` is what keeps a path holding
/// a space, a quote or a newline intact between the two:
///
/// ```text
/// i/lf    w/crlf  attr/text=auto eol=lf<TAB>.specify/integration.json
/// ```
fn parse(record: &str) -> Result<Reported<'_>, String> {
    let Some((fields, relative)) = record.split_once('\t') else {
        return Err(format!(
            "xtask: `git ls-files --eol` printed a record with no path in it: {record}\n\n    The \
             fields and the path are separated by a tab, and this one has none."
        ));
    };

    // The worktree field is the one that decides anything, and it is found by its own prefix rather
    // than by position: the index field is empty for a file nothing has staged yet, and the
    // attribute field holds however many attributes the file has.
    let mut endings = None;
    let mut wants_crlf = false;

    for field in fields.split_whitespace() {
        if let Some(value) = field.strip_prefix("w/") {
            endings = Some(value);
        } else if field == "eol=crlf" {
            wants_crlf = true;
        }
    }

    Ok(Reported {
        relative,
        endings: endings.unwrap_or_default(),
        wants_crlf,
    })
}

/// Whether a worktree copy has to be rewritten to the endings this repository asks for.
///
/// Git spells the endings `lf`, `crlf`, `mixed`, `none` or `-text`, and `-text` is its answer to
/// "are these bytes text", so a file it calls binary is never rewritten and nothing here has to
/// decide that again. `crlf` and `mixed` are the two that carry CRLF; a mixed file has no one ending
/// left to keep, and under `eol=lf` the only right answer is LF throughout.
fn needs_rewrite(endings: &str, wants_crlf: bool) -> bool {
    (endings == "crlf" || endings == "mixed") && !wants_crlf
}

/// Rewrites one worktree copy, answering how many lines it changed, or `None` when there was
/// nothing in it to change.
fn rewrite_one(root: &Path, file: &Reported<'_>) -> Result<Option<usize>, String> {
    let path = root.join(file.relative);

    // A file the worktree no longer has is reported with the endings in the index, and staging the
    // deletion is not this step's job.
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!("xtask: could not read {}: {error}", file.relative));
        }
    };

    let (fixed, lines) = to_lf(&bytes);
    if lines == 0 {
        return Ok(None);
    }

    fs::write(&path, fixed)
        .map_err(|error| format!("xtask: could not write {}: {error}", file.relative))?;

    Ok(Some(lines))
}

/// Replaces every CRLF with a bare LF, answering how many there were.
///
/// A lone CR is left where it is: nothing here produces one, `.gitattributes` has no rule for one,
/// and a file carrying one is a file whose bytes belong to something this step cannot see.
fn to_lf(bytes: &[u8]) -> (Vec<u8>, usize) {
    let mut fixed = Vec::with_capacity(bytes.len());
    let mut lines = 0;
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index..].starts_with(CRLF) {
            fixed.push(LF);
            index += CRLF.len();
            lines += 1;
        } else {
            fixed.push(bytes[index]);
            index += 1;
        }
    }

    (fixed, lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tracked file whose worktree copy has been written on Windows is rewritten: Git reports
    /// `crlf`, and the rule asks for `lf`.
    #[test]
    fn a_crlf_file_is_rewritten() {
        assert!(needs_rewrite("crlf", false));
    }

    /// A file Git read and called binary is left alone, whatever bytes it holds.
    #[test]
    fn a_binary_file_is_left_alone() {
        assert!(!needs_rewrite("-text", false));
    }

    /// A file already in LF has nothing to do, and neither does one with no line ending at all.
    #[test]
    fn a_file_with_nothing_to_normalize_is_left_alone() {
        assert!(!needs_rewrite("lf", false));
        assert!(!needs_rewrite("none", false));
    }

    /// `*.bat` and `*.cmd` are the one place CRLF is correct, and the Windows batch interpreter
    /// needs it. This is the answer this step would give without asking Git.
    #[test]
    fn a_file_gitattributes_wants_in_crlf_is_left_alone() {
        assert!(!needs_rewrite("crlf", true));
        assert!(!needs_rewrite("mixed", true));
    }

    /// A file carrying both endings has no one ending left to keep, and LF throughout is the only
    /// answer `eol=lf` allows.
    #[test]
    fn a_mixed_file_is_rewritten() {
        assert!(needs_rewrite("mixed", false));
    }

    /// The record is read by its own prefixes, because the index field is empty for a file nothing
    /// has staged yet and the attribute field holds however many attributes there are.
    #[test]
    fn a_record_is_read_by_prefix_rather_than_by_position() {
        let file = parse("i/      w/mixed attr/text=auto eol=lf\t.specify/integration.json")
            .expect("a record Git would print");

        assert_eq!(file.relative, ".specify/integration.json");
        assert_eq!(file.endings, "mixed");
        assert!(!file.wants_crlf);
    }

    /// A batch file arrives with `eol=crlf` among its attributes, and that is what saves it.
    #[test]
    fn a_batch_file_is_recognized_by_its_attribute() {
        let file = parse("i/      w/crlf  attr/text eol=crlf\ttools/build.cmd")
            .expect("a record Git would print");

        assert_eq!(file.endings, "crlf");
        assert!(file.wants_crlf);
        assert!(!needs_rewrite(file.endings, file.wants_crlf));
    }

    /// A path holding a space or a tab survives `-z`, and the tab between the fields and the path
    /// is the first one there is.
    #[test]
    fn a_path_holding_a_space_and_a_tab_is_read_whole() {
        let file = parse("i/lf    w/lf    attr/text=auto eol=lf\ta file\twith a tab.md")
            .expect("a record Git would print");

        assert_eq!(file.relative, "a file\twith a tab.md");
    }

    /// A record with no tab in it is Git's format having changed under this step, and it is
    /// reported rather than skipped: a record nobody reads is a file nobody fixes.
    #[test]
    fn a_record_with_no_path_in_it_is_reported() {
        let error = parse("i/lf w/lf attr/text=auto eol=lf").expect_err("there is no tab in it");

        assert!(error.contains("no path"), "{error}");
    }

    /// The whole content is replaced, and the count is what a report prints.
    #[test]
    fn every_crlf_becomes_a_bare_lf() {
        let (fixed, lines) = to_lf(b"alpha\r\nbeta\r\n");

        assert_eq!(fixed, b"alpha\nbeta\n");
        assert_eq!(lines, 2);
    }

    /// A file with nothing to replace comes back with the same bytes, which is what makes a second
    /// run find no work.
    #[test]
    fn a_file_with_no_crlf_comes_back_unchanged() {
        let (fixed, lines) = to_lf(b"alpha\nbeta\n");

        assert_eq!(fixed, b"alpha\nbeta\n");
        assert_eq!(lines, 0);
    }

    /// A CRLF split across a read boundary is still one line ending: the scan looks at both bytes
    /// rather than at one.
    #[test]
    fn a_lone_carriage_return_is_not_a_line_ending() {
        let (fixed, lines) = to_lf(b"alpha\r\r\nbeta");

        assert_eq!(fixed, b"alpha\r\nbeta");
        assert_eq!(lines, 1);
    }

    /// An empty file stays empty.
    #[test]
    fn an_empty_file_stays_empty() {
        assert_eq!(to_lf(b""), (Vec::new(), 0));
    }
}
