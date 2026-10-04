//! The gate's `specs` step: every tracked spec is shaped the way
//! [the template](https://github.com/andresmoschini/monospace/blob/main/.github/spec-template.md)
//! says, and says where it came from.
//!
//! A spec is the record of what a change was decided to be, so the only question this asks is
//! whether that record is still readable as one: a status line naming the pull request that decided
//! it, the five sections the template defines and no others, and — when the status says the code
//! revised the decisions — something in the last section saying how.
//!
//! # Design notes
//!
//! **It checks shape, never truth.** Whether the code does what the spec says is the question a
//! reader answers by running it, and no amount of parsing gets there: a spec can be perfectly
//! formed and describe a program nobody wrote. What *can* be checked mechanically is the thing
//! that makes the rest checkable by a human — that each spec points at the pull request it came
//! from, and that a spec admitting it was revised has recorded the revision. A gate step that
//! could verify the content would be a second implementation of the domain, which is the mistake
//! `monospace-core` is kept free of.
//!
//! **The status line is parsed as a prefix, not as a format.** Two forms are allowed and the tail
//! after them is free text, because the sentence a writer puts there is theirs and pinning its
//! wording would make this a formatter wearing a checker's clothes. What is enforced is the part
//! that cannot be prose: that the line exists, that it names a pull request by number, and which of
//! the two forms it is.
//!
//! **A revision without a note is the failure this exists for.** A spec whose status says the code
//! changed the decisions, with an empty _What the implementation changed_, is the state
//! `AGENTS.md` calls the worst one: a document that reads as authoritative while being wrong about
//! the thing it was written to record. The check is two lines long and it is the only one here that
//! catches a mistake made *after* a merge, which is when a spec starts drifting.
//!
//! **Headings are checked as a set, not as an order.** The template's order is what a reader
//! expects, but reordering sections does not make a spec less true, and a step that fails on
//! formatting a human chose is a step people learn to work around.

use std::fs;
use std::path::Path;

use crate::process::capture;

/// The directory specs live in, relative to the workspace root. Flat: one file per feature, because
/// the directory-per-feature shape is the one that let eight files appear beside each spec.
const SPEC_DIRECTORY: &str = "specs";

/// The prefix of the template, checked so a spec that forgot its title is reported rather than read.
const TITLE_PREFIX: &str = "# ";

/// The five sections the template defines, in the order it puts them in.
const SECTIONS: &[&str] = &[
    "The decisions",
    "Public surface",
    "What this does not decide",
    "What proves it",
    "What the implementation changed",
];

/// The section a revision has to be recorded in, named so a failure can point at it.
const REVISION_SECTION: &str = "What the implementation changed";

/// The two status lines a spec may carry, and what each one promises. The second is a promise the
/// gate holds: the revision section may not be empty.
const STATUS_MATCHES_IT: &str = "the code matches it";
const STATUS_REVISED_IT: &str = "the code revised it";

/// The gate's step: reports every spec that has parted from the template, and returns whether they
/// all match.
pub fn check(root: &Path) -> bool {
    crate::report_failure(walk(root))
}

/// Reads every tracked spec and reports the first thing wrong with each, so one run says everything
/// rather than making a fix one file at a time.
fn walk(root: &Path) -> Result<(), String> {
    let directory = root.join(SPEC_DIRECTORY);
    if !directory.is_dir() {
        // No specs yet is a repository state, not a defect: the directory appears with the first
        // deciding pull request, and a gate step that fails on an absent directory would fail on
        // every fresh clone before the work that creates it.
        return Ok(());
    }

    let mut problems = Vec::new();
    for relative in tracked_specs(root)? {
        let path = root.join(&relative);
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("xtask: could not read {relative}: {error}"))?;
        problems.extend(problems_with(&relative, &text));
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{n} spec(s) do not match {TEMPLATE}:\n\n{body}\n\
             A spec is a commitment, so the gate holds the two things a reader cannot recover: \
             which pull request decided it, and whether the code still matches.",
            n = problems.len(),
            body = problems
                .iter()
                .map(|problem| format!("  {problem}"))
                .collect::<Vec<_>>()
                .join("\n"),
        ))
    }
}

/// Every tracked spec, as `git ls-files` reports it, relative to the workspace root.
///
/// Git is asked rather than the filesystem read so an untracked spec does not fail the gate: a spec
/// is only a record once it is committed, and the file being written is not one yet.
///
/// The `:(glob)` prefix is load-bearing and not decoration. A bare `specs/*.md` is a git pathspec,
/// where `*` crosses `/`: measured, it matched all 161 Markdown files under the eight-per-feature
/// directories this repository used to keep, nineteen of them nested `…/spec.md`, and the gate
/// reported 1994 problems against files that are not specs at all. `:(glob)` is the only spelling
/// where `*` stops at the separator, so the flat shape the template asks for is what is actually
/// read. Nothing catches this by accident: it only ever fires on a repository that still holds a
/// directory-based spec, which is to say the one this gate was written for.
fn tracked_specs(root: &Path) -> Result<Vec<String>, String> {
    let listing = capture(
        root,
        "git",
        &["ls-files", &format!(":(glob){SPEC_DIRECTORY}/*.md")],
    )?;
    Ok(listing
        .lines()
        .map(str::to_string)
        .filter(|line| line.to_ascii_lowercase().ends_with(".md"))
        .collect())
}

/// Everything wrong with one spec, as sentences naming the file and, where it helps, the line.
///
/// Every problem is collected rather than returned at the first one: a spec written from the
/// template and then edited usually misses the status line *and* has a section renamed, and fixing
/// them one run at a time is the round trip this step exists to avoid.
fn problems_with(relative: &str, text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let lines: Vec<&str> = text.lines().collect();

    if !lines
        .iter()
        .any(|line| line.trim_start().starts_with(TITLE_PREFIX))
    {
        problems.push(format!("{relative}: no `# ` title"));
    }

    let sections = sections_in(&lines);

    match status_line(&lines) {
        None => problems.push(format!(
            "{relative}: no status line. It is either \
             `Status: decided <date> in #<number> — {STATUS_MATCHES_IT}` or the same with \
             `{STATUS_REVISED_IT}`."
        )),
        Some(Status::Revised) => {
            // Trimmed, because a section holding nothing but blank lines is empty and a writer who
            // left the template's spacing behind should not be told they wrote a note.
            if section_body(&lines, REVISION_SECTION).trim().is_empty() {
                problems.push(format!(
                    "{relative}: the status says the code revised it, and \
                     `## {REVISION_SECTION}` is empty. Say what changed."
                ));
            }
        }
        Some(Status::Matches) => {}
    }

    for section in &sections {
        if !SECTIONS.contains(&section.as_str()) {
            problems.push(format!(
                "{relative}: `## {section}` is not one of the template's sections: {}",
                SECTIONS.join(", "),
            ));
        }
    }
    for expected in SECTIONS {
        if !sections.iter().any(|section| section == expected) {
            problems.push(format!("{relative}: no `## {expected}` section"));
        }
    }

    problems
}

/// Which of the two allowed status lines a spec carries.
enum Status {
    /// The code is what was decided.
    Matches,
    /// The code is not, and the revision has to be recorded.
    Revised,
}

/// The status line's promise, or `None` when there is no usable one.
///
/// A line that starts with `Status:` but names no pull request is reported as missing rather than
/// as malformed, because "there is no status line" and "your status line is wrong" are the same
/// instruction to whoever is writing it and one sentence carries both.
fn status_line(lines: &[&str]) -> Option<Status> {
    let line = lines
        .iter()
        .find(|line| line.trim_start().starts_with("Status:"))?;

    // The pull request number is what makes the line a reference rather than a date, so it is
    // required before either promise is honored.
    if !line.contains('#') || !line.chars().any(|character| character.is_ascii_digit()) {
        return None;
    }

    if line.contains(STATUS_REVISED_IT) {
        Some(Status::Revised)
    } else if line.contains(STATUS_MATCHES_IT) {
        Some(Status::Matches)
    } else {
        None
    }
}

/// The `## ` headings of `lines`, trimmed, in the order they appear.
fn sections_in(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .filter_map(|line| line.trim_start().strip_prefix("## "))
        .map(str::trim)
        .map(str::to_string)
        .collect()
}

/// What a spec says under `heading`, or an empty string when the heading is absent or its body holds
/// nothing but blank lines.
///
/// An HTML comment is content for this purpose, because the template's own prompts are comments and
/// a spec that kept them has not been filled in — which is a different problem from one that says
/// "None." on purpose, and one this is not trying to catch.
fn section_body(lines: &[&str], heading: &str) -> String {
    let mut body = String::new();
    let mut inside = false;

    for line in lines {
        if let Some(name) = line.trim_start().strip_prefix("## ") {
            inside = name.trim() == heading;
            continue;
        }
        if inside {
            body.push_str(line);
            body.push('\n');
        }
    }

    body
}

/// The path of the template every spec is copied from, named in the failure message so the reader
/// knows what to copy from rather than only what is wrong.
const TEMPLATE: &str = ".github/spec-template.md";

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::{
        SECTIONS, SPEC_DIRECTORY, STATUS_MATCHES_IT, STATUS_REVISED_IT, problems_with,
        section_body, sections_in, status_line, tracked_specs,
    };

    /// A spec carrying every section the template defines, with a status line of the given wording.
    fn spec(status: &str) -> String {
        let mut text = format!("# 090 — a shape offers its corners\n\n{status}\n");
        for section in SECTIONS {
            let _ = write!(text, "\n## {section}\n\nNone.\n");
        }
        text
    }

    /// A spec whose last section says what building it changed, or nothing.
    fn revised_with(note: &str) -> String {
        let text = spec(&format!(
            "Status: decided 2026-10-04 in #164 — {STATUS_REVISED_IT}"
        ));
        text.replace(
            "## What the implementation changed\n\nNone.",
            &format!("## What the implementation changed\n\n{note}"),
        )
    }

    /// A scratch repository holding the given files, tracked, and removed on drop.
    ///
    /// `std::env::temp_dir` and a counter rather than a crate, which is the one place this module is
    /// allowed to want a fixture: `tracked_specs` is the only function here that needs a repository,
    /// and it is the only one whose bug is invisible from the outside. The counter and not the file
    /// list, because two fixtures with the same files must not land in the same directory — the
    /// tests run in parallel and the first one to drop would delete the other's repository.
    struct Scratch {
        root: std::path::PathBuf,
    }

    impl Scratch {
        fn with(files: &[(&str, &str)]) -> Self {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let root = std::env::temp_dir().join(format!(
                "monospace-spec-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = std::fs::remove_dir_all(&root);
            for (relative, contents) in files {
                let path = root.join(relative);
                std::fs::create_dir_all(path.parent().expect("a file has a parent"))
                    .expect("mkdir");
                std::fs::write(&path, contents).expect("write");
            }
            for args in [vec!["init", "--quiet"], vec!["add", "--all"]] {
                let ok = std::process::Command::new("git")
                    .args(&args)
                    .current_dir(&root)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .is_ok_and(|status| status.success());
                assert!(ok, "git {args:?} failed in {}", root.display());
            }
            Self { root }
        }

        fn specs(&self) -> Vec<String> {
            tracked_specs(&self.root).expect("git ls-files")
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn a_flat_spec_is_read_and_a_nested_one_is_not() {
        let flat = format!("{SPEC_DIRECTORY}/090-x.md");
        let scratch = Scratch::with(&[
            (&flat, "# 090 — x\n"),
            (&format!("{SPEC_DIRECTORY}/142-y/spec.md"), "# not a spec\n"),
            (&format!("{SPEC_DIRECTORY}/142-y/notes.md"), "# notes\n"),
            ("README.md", "# not in specs\n"),
        ]);

        // A bare `specs/*.md` pathspec matches all four of these: git's `*` crosses `/` unless the
        // `:(glob)` prefix says otherwise. That is what this asserts, and it is why the prefix is
        // load-bearing rather than a style choice.
        assert_eq!(scratch.specs(), vec![flat]);
    }

    #[test]
    fn an_empty_specs_directory_is_not_a_failure() {
        let scratch = Scratch::with(&[("README.md", "# no specs yet\n")]);
        assert!(scratch.specs().is_empty());
    }

    #[test]
    fn a_repository_without_a_specs_directory_at_all_is_not_a_failure() {
        let scratch = Scratch::with(&[("README.md", "# no specs yet\n")]);
        assert!(!scratch.root.join(SPEC_DIRECTORY).exists());
        assert!(super::walk(&scratch.root).is_ok());
    }

    #[test]
    fn a_fresh_spec_passes() {
        let text = spec(&format!(
            "Status: decided 2026-10-04 in #164 — {STATUS_MATCHES_IT}"
        ));
        assert_eq!(problems_with("specs/090-x.md", &text), Vec::<String>::new());
    }

    #[test]
    fn a_missing_status_line_is_reported_with_both_forms_named() {
        let text = "# 090 — x\n\n## The decisions\n\nNone.\n";
        let problems = problems_with("specs/090-x.md", text);
        let status = problems
            .iter()
            .find(|problem| problem.contains("no status line"))
            .unwrap_or_else(|| panic!("no status line was reported: {problems:?}"));
        assert!(status.contains(STATUS_MATCHES_IT), "{status}");
        assert!(status.contains(STATUS_REVISED_IT), "{status}");
        // The four sections this text does not carry are reported in the same run, not one per run.
        assert_eq!(problems.len(), 5, "{problems:?}");
    }

    #[test]
    fn a_status_line_naming_no_pull_request_is_treated_as_missing() {
        assert!(status_line(&["Status: decided today."]).is_none());
        assert!(status_line(&["Status: decided in a while ago."]).is_none());
    }

    #[test]
    fn a_revision_with_a_note_passes() {
        let text = revised_with("The freeze belongs on Shape, not on Diagram.");
        assert_eq!(problems_with("specs/090-x.md", &text), Vec::<String>::new());
    }

    #[test]
    fn a_revision_without_a_note_is_the_failure_this_step_is_for() {
        let problems = problems_with("specs/090-x.md", &revised_with(""));
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(
            problems[0].contains("Say what changed"),
            "{:?}",
            problems[0]
        );
    }

    #[test]
    fn a_revision_section_of_blank_lines_counts_as_empty() {
        // The template's own spacing, left behind by a writer who deleted the body and not the
        // blank lines under it, is not a note.
        let text = spec(&format!(
            "Status: decided 2026-10-04 in #164 — {STATUS_REVISED_IT}"
        ))
        .replace("None.\n", "");
        let problems = problems_with("specs/090-x.md", &text);
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("Say what changed")),
            "{problems:?}"
        );
    }

    #[test]
    fn none_is_content_in_the_revision_section_too() {
        // "None." is a real answer when the decisions held, and it is not what an unwritten section
        // looks like — a writer has to type it.
        let text = revised_with("None.");
        assert_eq!(problems_with("specs/090-x.md", &text), Vec::<String>::new());
    }

    #[test]
    fn a_renamed_section_is_reported_and_the_missing_one_too() {
        let text = spec(&format!(
            "Status: decided 2026-10-04 in #164 — {STATUS_MATCHES_IT}"
        ))
        .replace("## Public surface", "## Public API");
        let problems = problems_with("specs/090-x.md", &text);
        assert_eq!(problems.len(), 2, "{problems:?}");
        assert!(problems[0].contains("## Public API"), "{:?}", problems[0]);
        assert!(problems[0].contains("The decisions"), "{:?}", problems[0]);
        assert!(
            problems[1].contains("no `## Public surface`"),
            "{:?}",
            problems[1]
        );
    }

    #[test]
    fn a_spec_with_no_title_is_reported() {
        let text = spec(&format!(
            "Status: decided 2026-10-04 in #164 — {STATUS_MATCHES_IT}"
        ));
        let text = text.replace("# 090 — a shape offers its corners\n", "");
        assert_eq!(
            problems_with("specs/090-x.md", &text),
            vec!["specs/090-x.md: no `# ` title".to_string()]
        );
    }

    #[test]
    fn reordering_sections_is_allowed() {
        let mut text =
            String::from("# 090 — x\n\nStatus: decided 2026-10-04 in #164 — the code matches it\n");
        for section in SECTIONS.iter().rev() {
            let _ = write!(text, "\n## {section}\n\nNone.\n");
        }
        assert_eq!(problems_with("specs/090-x.md", &text), Vec::<String>::new());
    }

    #[test]
    fn every_problem_in_one_file_is_collected() {
        // A spec written by hand rather than copied misses the title, the status and five sections at
        // once, and one run has to say all of it.
        let problems = problems_with("specs/090-x.md", "Nothing else here.\n");
        assert_eq!(problems.len(), 7, "{problems:?}");
    }

    #[test]
    fn headings_are_read_trimmed_and_in_order() {
        let lines = vec!["## One", "  ## Two", "### Three", "Not a heading"];
        assert_eq!(sections_in(&lines), vec!["One", "Two"]);
    }

    #[test]
    fn a_section_body_stops_at_the_next_heading() {
        let lines = vec!["## One", "body", "## Two", "other"];
        assert_eq!(section_body(&lines, "One"), "body\n");
        assert_eq!(section_body(&lines, "Two"), "other\n");
        assert_eq!(section_body(&lines, "Three"), "");
    }
}
