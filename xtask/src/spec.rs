//! The specification document.
//!
//! The format is in `specs/README.md`, and the gate's `specs` step is what holds every spec to it: it
//! reads the frontmatter and the set of `## ` headings, and reports every way every file fails. There
//! is no command of its own: `cargo xtask check` is where this runs, and the two things `change`
//! reads out of a spec — a name and an answer — are the functions below.
//!
//! The change a spec belongs to is `change`'s business: a change is one issue, one branch and one
//! file, and where it stands is in the branch's name rather than in anything read here.
//!
//! # Design notes
//!
//! **`check` counts frontmatter, not prose.** It reads the status and the pull request references,
//! and the set of `## ` headings. It does not read section order and it does not read the body,
//! because a heading someone reformatted is a thing people learn to work around, and a rule that
//! can be satisfied by renaming a line stops being a rule. Every problem in every file is collected
//! before anything is reported, so one run says everything.
//!
//! **The listing is a `:(glob)` pathspec and the magic word is load-bearing.** A bare `specs/*.md`
//! is an ordinary git pathspec, where `*` crosses `/` and therefore matches every `spec.md` that
//! ever lived under a feature directory — which is what the tree used to hold. There were 1,994
//! files down there and the step reported all of them as broken specs. `:(glob)` says `*` stops at
//! the separator. A regression test asserts the nested file is invisible, and it was checked by
//! reverting the prefix: without it the test fails.
//!
//! **The marker is grepped, not parsed.** One string, in the frontmatter's `decided` and in every
//! decision's answer alike, so where a decision puts it is the spec's business. A spec that reads
//! `_pending_` in a sentence *about* the marker is refused, which is the price of a rule that can be
//! satisfied by moving a line, paid in both directions.

use std::fs;
use std::path::Path;

use crate::process::capture;

/// The `## ` headings every spec carries. Nine of them, which is the count the flow's format was
/// arrived at by deleting eight files; a tenth heading is a tenth thing to keep in step.
const SECTIONS: [&str; 9] = [
    "Why now",
    "Scope",
    "The decision",
    "Model slice",
    "Public surface",
    "Behavior",
    "Examples",
    "What proves it",
    "Open questions",
];

/// The four values `status` may take.
const STATUSES: [&str; 4] = ["draft", "agreed", "implemented", "abandoned"];

/// The marker an answer carries until it is given. Two places read this string — the gate below and
/// `change`'s refusal to cut a building branch — so changing it is changing the flow, not editing a
/// document. It is grepped rather than parsed, so where a decision puts it is the spec's business,
/// not this module's.
const PENDING: &str = "_pending_";

/// The file in `specs/` that is not a spec: the format's own documentation.
///
/// It is named rather than recognized by a shape, because a shape test would quietly stop
/// excluding it the day somebody gave it frontmatter, and because the one rule this step has that
/// is not about a spec's content is that this path is not one.
const NOT_A_SPEC: &str = "specs/README.md";

/// The gate's `specs` step: reports every way every spec fails the format, and answers whether it
/// passed.
///
/// There is no second answer to give. A repository with no specs is not a failure — `specs/` holds
/// only its own README until the first spec is written — and it used to answer `Ok(false)` to say
/// so, which the gate's adapter read as success because the `bool` inside the `Result` is dropped.
/// A step that reports a verdict nobody reads is a step whose verdict will be read wrong.
///
/// # Errors
///
/// One message naming every problem in every file, so one run says everything rather than one
/// problem per run.
pub fn check(root: &Path) -> Result<(), String> {
    let listed = capture(root, "git", &["ls-files", ":(glob)specs/*.md"])?;
    let paths: Vec<&str> = listed
        .lines()
        .map(str::trim)
        .filter(|path| !path.is_empty() && *path != NOT_A_SPEC)
        .collect();

    if paths.is_empty() {
        println!("no spec to check");
        return Ok(());
    }

    let mut problems = Vec::new();
    for path in &paths {
        let text = fs::read_to_string(root.join(path))
            .map_err(|error| format!("xtask: could not read {path}: {error}"))?;
        problems.extend(one_spec(path, &text));
    }

    if problems.is_empty() {
        println!(
            "{} spec(s) checked, each with its nine sections and an answerable status",
            paths.len()
        );
        return Ok(());
    }

    Err(format!(
        "xtask: {} problem(s) in {} spec(s):\n{}",
        problems.len(),
        paths.len(),
        problems
            .iter()
            .map(|problem| format!("  {problem}"))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

/// The `status` the spec in `text` declares, unquoted, where it declares one.
///
/// This is what `change status` reports for a change, read from the spec as `origin/main` has it, so
/// a spec that reached `main` still reading `draft` is visible from the command that talks about
/// changes and not only from the gate.
pub(crate) fn declared_status(text: &str) -> Option<String> {
    frontmatter_of(text)
        .and_then(|frontmatter| field(frontmatter, "status"))
        .map(unquoted)
        .map(str::to_string)
}

/// The pull request the spec in `text` names as the one that decided it, unquoted.
///
/// A draft names none, which is the normal case and not a defect.
pub(crate) fn decided_by(text: &str) -> Option<String> {
    frontmatter_of(text)
        .and_then(|frontmatter| field(frontmatter, "decided"))
        .map(unquoted)
        .map(str::to_string)
}

/// The lines of a spec that still carry the `_pending_` marker, as one-based line numbers paired
/// with the trimmed text of the line.
pub(crate) fn pending_lines(spec: &str) -> Vec<(usize, String)> {
    spec.lines()
        .enumerate()
        .filter(|(_, line)| line.contains(PENDING))
        .map(|(index, line)| (index + 1, line.trim().to_string()))
        .collect()
}

/// Renders what `pending_lines` found for the operator: the first three, quoted, and a count of
/// whatever else is left. Three is enough to recognize which answers they are; the file itself is
/// where they get read.
pub(crate) fn describe_pending(pending: &[(usize, String)]) -> String {
    let shown = pending
        .iter()
        .take(3)
        .map(|(number, text)| format!("line {number} reads `{text}`"))
        .collect::<Vec<_>>()
        .join(", ");

    match pending.len().saturating_sub(3) {
        0 => shown,
        rest => format!("{shown}, and {rest} more"),
    }
}

/// Every way `text`, the spec at `path`, fails the format.
fn one_spec(path: &str, text: &str) -> Vec<String> {
    let mut problems = Vec::new();

    if !text.lines().any(|line| line.starts_with("# ")) {
        problems.push(format!("{path}: no `# ` title"));
    }

    let Some(frontmatter) = frontmatter_of(text) else {
        problems.push(format!("{path}: no frontmatter; a spec starts with `---`"));
        return problems;
    };

    problems.extend(frontmatter_problems(path, frontmatter));
    problems.extend(section_problems(path, text));

    let status = field(frontmatter, "status").map(unquoted);
    if matches!(status, Some("agreed" | "implemented")) && text.contains(PENDING) {
        problems.push(format!(
            "{path}: `status` is `{}` and the spec still reads `{PENDING}`; an agreed spec has \
             no unanswered decision",
            status.unwrap_or_default()
        ));
    }

    problems
}

/// The lines between the `---` that opens a spec's frontmatter and the `---` that closes it.
///
/// `None` when the file does not open with the fence, or never closes it — both of which are a
/// spec with no frontmatter as far as anything reading it is concerned.
fn frontmatter_of(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

/// Every way `frontmatter`, a spec's field block, fails the field table.
fn frontmatter_problems(path: &str, frontmatter: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let status = field(frontmatter, "status").map(unquoted);

    match status {
        None => problems.push(format!("{path}: frontmatter has no `status`")),
        Some(given) if !STATUSES.contains(&given) => problems.push(format!(
            "{path}: `status` reads `{given}`, which is not one of {}",
            STATUSES.join(", ")
        )),
        Some(_) => {}
    }

    if field(frontmatter, "date").is_none() {
        problems.push(format!("{path}: frontmatter has no `date`"));
    }

    // `decided` is what makes an agreed spec citable, so it is required exactly once there is a
    // pull request that agreed it. A draft has none, which is the normal case and not a defect.
    if matches!(status, Some("agreed" | "implemented")) {
        problems.extend(reference_problems(
            path,
            frontmatter,
            "decided",
            "`status` is agreed or implemented",
        ));
    }

    if status == Some("implemented") {
        problems.extend(reference_problems(
            path,
            frontmatter,
            "implemented",
            "`status` is implemented",
        ));
    }

    problems
}

/// The problem with a `name: value` field of `frontmatter`, if it is required and does not name a
/// pull request.
fn reference_problems(path: &str, frontmatter: &str, name: &str, why: &str) -> Vec<String> {
    match field(frontmatter, name) {
        None => vec![format!(
            "{path}: frontmatter has no `{name}`, which it needs because {why}"
        )],
        Some(value) if names_a_pull_request(unquoted(value)) => Vec::new(),
        Some(value) => vec![format!(
            "{path}: `{name}` reads `{}`, which is not a pull request number like `#163`",
            unquoted(value)
        )],
    }
}

/// The `name: value` field called `name` in a frontmatter block, value untrimmed.
fn field<'a>(frontmatter: &'a str, name: &str) -> Option<&'a str> {
    frontmatter.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        (key.trim() == name).then_some(value)
    })
}

/// A field value with the quotes the format puts around a pull request number taken off.
fn unquoted(value: &str) -> &str {
    let value = value.trim();
    value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(value)
}

/// Whether `value` is a pull request reference: a `#` and the digits after it, and nothing else.
fn names_a_pull_request(value: &str) -> bool {
    let Some(digits) = value.strip_prefix('#') else {
        return false;
    };
    !digits.is_empty() && digits.chars().all(|character| character.is_ascii_digit())
}

/// Every way `text`'s `## ` headings fail to be the nine sections.
///
/// Order is not checked and neither is the body: a heading someone reformatted is a thing people
/// learn to work around, and a rule that can be satisfied by renaming a line stops being a rule.
fn section_problems(path: &str, text: &str) -> Vec<String> {
    let found: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .map(str::trim)
        .collect();

    let missing: Vec<&&str> = SECTIONS
        .iter()
        .filter(|section| !found.contains(section))
        .collect();
    let extra: Vec<&&str> = found
        .iter()
        .filter(|heading| !SECTIONS.contains(heading))
        .collect();

    let named = |headings: &[&&str]| {
        headings
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };

    [
        (!missing.is_empty())
            .then(|| format!("{path}: no `## ` section named {}", named(&missing))),
        (!extra.is_empty()).then(|| {
            format!(
                "{path}: `## ` section(s) not one of the nine: {}",
                named(&extra)
            )
        }),
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    /// A spec shaped like the one `specs/README.md` describes, with `agreed` deciding whether the
    /// decision is answered and the spec therefore names the pull request that agreed it.
    fn spec(agreed: bool) -> String {
        let (status, decided, answer) = if agreed {
            (
                "agreed",
                "decided: \"#170\"",
                "Yes, and the reasoning is in the module.",
            )
        } else {
            ("draft", "", "_pending_")
        };

        format!(
            "---\n\
             status: {status}\n\
             {decided}\n\
             date: 2026-10-05\n\
             ---\n\
             \n\
             # 163 — The change\n\
             \n\
             ## Why now\n\
             \n\
             Because.\n\
             \n\
             ## Scope\n\
             \n\
             ### In\n\
             \n\
             One thing.\n\
             \n\
             ### Out\n\
             \n\
             Another thing.\n\
             \n\
             ## The decision\n\
             \n\
             **D1 — Which?**\n\
             \n\
             **Answer:** {answer} **Why not** the other one. **Answered by** the maintainer.\n\
             \n\
             ## Model slice\n\
             \n\
             Section 3.\n\
             \n\
             ## Public surface\n\
             \n\
             `fn thing()`.\n\
             \n\
             ## Behavior\n\
             \n\
             1. A rule.\n\
             \n\
             ## Examples\n\
             \n\
             ```text\n\
             a\n\
             ```\n\
             \n\
             ## What proves it\n\
             \n\
             The test named after it.\n\
             \n\
             ## Open questions\n\
             \n\
             None.\n"
        )
    }

    #[test]
    fn pending_lines_finds_nothing_in_an_agreed_spec() {
        assert!(pending_lines(&spec(true)).is_empty());
    }

    #[test]
    fn pending_lines_finds_the_marker_in_a_draft() {
        let pending = pending_lines(&spec(false));
        assert_eq!(pending.len(), 1);
        assert!(pending[0].1.contains("_pending_"));
    }

    #[test]
    fn describe_pending_quotes_three_and_counts_the_rest() {
        let pending: Vec<(usize, String)> = (1..=5)
            .map(|number| (number, format!("**Answer:** _pending_ — D{number}")))
            .collect();
        let described = describe_pending(&pending);
        assert!(
            described.starts_with("line 1 reads `**Answer:** _pending_ — D1`"),
            "the first pending line was not quoted: {described}"
        );
        assert!(
            described.ends_with("and 2 more"),
            "the rest were not counted: {described}"
        );
    }

    /// What `change status` prints for a change comes from these two, so they are read the same way
    /// the gate reads them and not by a second path of their own.
    #[test]
    fn the_declared_status_and_the_pull_request_that_decided_are_read_unquoted() {
        let agreed = spec(true);
        assert_eq!(declared_status(&agreed).as_deref(), Some("agreed"));
        assert_eq!(decided_by(&agreed).as_deref(), Some("#170"));

        let draft = spec(false);
        assert_eq!(declared_status(&draft).as_deref(), Some("draft"));
        assert_eq!(decided_by(&draft), None);
    }

    /// A throwaway git repository, holding nothing, that deletes itself.
    ///
    /// This is the only fixture in the crate, and it is here for one reason: the `specs` step reads
    /// git's index rather than the working tree, so there is no way to exercise it against this
    /// repository without also staging real specs into it.
    struct Scratch {
        path: PathBuf,
    }

    impl Scratch {
        /// A new repository under the system temporary directory, named by a counter so two tests
        /// running at once cannot land on the same one.
        fn new() -> Scratch {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "monospace-spec-scratch-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("the scratch directory can be created");
            run_git(&path, &["init", "--quiet"]);
            Scratch { path }
        }

        /// Writes `contents` at `relative` and stages it, which is what puts it in the index — the
        /// only place `check` looks.
        fn write(&self, relative: &str, contents: &str) {
            let path = self.path.join(relative);
            fs::create_dir_all(path.parent().expect("a relative path has a parent"))
                .expect("the scratch directory can hold the file");
            fs::write(&path, contents).expect("the scratch file can be written");
            run_git(&self.path, &["add", "--all"]);
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    /// Runs `git` in `root`, failing loudly: a fixture that did not set itself up would otherwise
    /// make every assertion below it pass for the wrong reason.
    fn run_git(root: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(root)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("git runs");
        assert!(
            status.success(),
            "git {args:?} failed in {}",
            root.display()
        );
    }

    #[test]
    fn a_repository_with_no_specs_directory_has_nothing_to_check() {
        let scratch = Scratch::new();
        scratch.write("README.md", "# Nothing here\n");

        assert_eq!(check(&scratch.path), Ok(()));
    }

    #[test]
    fn an_agreed_spec_in_the_right_shape_passes() {
        let scratch = Scratch::new();
        scratch.write("specs/163-light.md", &spec(true));

        assert_eq!(check(&scratch.path), Ok(()));
    }

    #[test]
    fn a_renamed_heading_is_reported() {
        let scratch = Scratch::new();
        scratch.write(
            "specs/163-light.md",
            &spec(true).replace("## Examples", "## Sample"),
        );

        let error = check(&scratch.path).unwrap_err();
        assert!(error.contains("no `## ` section named Examples"), "{error}");
        assert!(
            error.contains("not one of the nine"),
            "the extra heading was not reported: {error}"
        );
    }

    #[test]
    fn a_status_outside_the_four_is_reported() {
        let scratch = Scratch::new();
        scratch.write(
            "specs/163-light.md",
            &spec(true).replace("status: agreed", "status: merged"),
        );

        let error = check(&scratch.path).unwrap_err();
        assert!(
            error.contains("`status` reads `merged`"),
            "the bad status was not reported: {error}"
        );
    }

    #[test]
    fn a_missing_date_is_reported() {
        let scratch = Scratch::new();
        scratch.write(
            "specs/163-light.md",
            &spec(true).replace("date: 2026-10-05\n", ""),
        );

        let error = check(&scratch.path).unwrap_err();
        assert!(error.contains("no `date`"), "{error}");
    }

    #[test]
    fn an_agreed_spec_with_a_pending_decision_is_reported() {
        let scratch = Scratch::new();
        let unanswered =
            spec(true).replace("Yes, and the reasoning is in the module.", "_pending_");
        scratch.write("specs/163-light.md", &unanswered);

        let error = check(&scratch.path).unwrap_err();
        assert!(error.contains("still reads `_pending_`"), "{error}");
    }

    #[test]
    fn a_draft_may_have_no_decided_and_may_still_be_pending() {
        let scratch = Scratch::new();
        scratch.write("specs/163-light.md", &spec(false));

        assert_eq!(check(&scratch.path), Ok(()));
    }

    #[test]
    fn an_agreed_spec_with_no_decided_is_reported() {
        let scratch = Scratch::new();
        let agreed = spec(false).replace("status: draft", "status: agreed");
        scratch.write("specs/163-light.md", &agreed);

        let error = check(&scratch.path).unwrap_err();
        assert!(error.contains("no `decided`"), "{error}");
    }

    #[test]
    fn an_implemented_spec_names_both_pull_requests() {
        let scratch = Scratch::new();
        let implemented = spec(true)
            .replace("status: agreed", "status: implemented")
            .replace(
                "decided: \"#170\"",
                "decided: \"#170\"\nimplemented: \"#171\"",
            );
        scratch.write("specs/163-light.md", &implemented);

        assert_eq!(check(&scratch.path), Ok(()));

        let without = spec(true).replace("status: agreed", "status: implemented");
        scratch.write("specs/164-other.md", &without);
        let error = check(&scratch.path).unwrap_err();
        assert!(error.contains("no `implemented`"), "{error}");
    }

    #[test]
    fn a_decided_field_that_names_no_pull_request_is_reported() {
        let scratch = Scratch::new();
        scratch.write(
            "specs/163-light.md",
            &spec(true).replace("decided: \"#170\"", "decided: 170"),
        );

        let error = check(&scratch.path).unwrap_err();
        assert!(error.contains("not a pull request number"), "{error}");
    }

    #[test]
    fn a_spec_without_a_title_or_frontmatter_is_reported() {
        let scratch = Scratch::new();
        scratch.write("specs/163-light.md", "## Why now\n\nNothing else.\n");

        let error = check(&scratch.path).unwrap_err();
        assert!(error.contains("no `# ` title"), "{error}");
        assert!(error.contains("no frontmatter"), "{error}");
    }

    /// The `:(glob)` is load-bearing: a bare `specs/*.md` is an ordinary pathspec where `*` crosses
    /// `/`, so it would read every `spec.md` a feature directory ever held. The old tree had 1,994
    /// of them and the step reported all of them. Reverting the prefix makes this fail.
    #[test]
    fn a_spec_nested_below_specs_is_not_a_spec() {
        let scratch = Scratch::new();
        scratch.write("specs/163-light.md", &spec(true));
        scratch.write("specs/006-an-old-feature/spec.md", "not a spec at all\n");

        assert_eq!(check(&scratch.path), Ok(()));
    }

    /// `specs/README.md` documents the format and is not a spec. It sits in the same directory because
    /// that is where a reader looks for it, so the step names it rather than looking for a shape it
    /// would stop recognizing the day somebody gave it frontmatter.
    #[test]
    fn the_formats_own_readme_is_not_checked_as_a_spec() {
        let scratch = Scratch::new();
        scratch.write(NOT_A_SPEC, "# Specifications\n\nNo frontmatter here.\n");

        assert_eq!(check(&scratch.path), Ok(()));

        scratch.write("specs/163-light.md", &spec(true));
        assert_eq!(check(&scratch.path), Ok(()));
    }

    #[test]
    fn every_problem_in_every_file_is_reported_by_one_run() {
        let scratch = Scratch::new();
        scratch.write(
            "specs/163-one.md",
            &spec(true).replace("## Behavior", "## Rules"),
        );
        scratch.write(
            "specs/164-two.md",
            &spec(true).replace("status: agreed", "status: done"),
        );

        let error = check(&scratch.path).unwrap_err();
        assert!(error.contains("specs/163-one.md"), "{error}");
        assert!(error.contains("Behavior"), "{error}");
        assert!(error.contains("specs/164-two.md"), "{error}");
        assert!(error.contains("`done`"), "{error}");
        assert!(
            error.contains("3 problem(s) in 2 spec(s)"),
            "the count did not cover both files: {error}"
        );
    }
}
