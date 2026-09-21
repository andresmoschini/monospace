//! `cargo xtask spec`: the feature branch lifecycle described in
//! [ADR-0051](../../docs/decisions/0051-stop-at-the-decision-sheet-and-merge-three-stages-into-two.md),
//! [ADR-0033](../../docs/decisions/0033-keep-the-flow-state-in-labels-on-one-issue.md) and
//! [ADR-0034](../../docs/decisions/0034-let-xtask-own-the-feature-branch.md).
//!
//! A feature is one GitHub issue, crossing two branches (`NNN-slug-deciding`, `NNN-slug-building`),
//! each with its own label (`deciding`, `building`) and its own precondition checked against
//! `origin/main`. `new` opens the first branch, `stage <issue> build` opens the second after
//! checking its precondition, and `use` puts a clone on whichever branch the issue's labels say is
//! current. None of this is part of the quality gate: `cargo xtask check` has no notion of `git`
//! branches or GitHub labels, and this module is not on its call graph.
//!
//! Every subprocess call lives behind a thin wrapper (`run_visible`, `capture`) so the logic that
//! decides *what* to run — slugging a title, naming a branch, reading a precondition, mapping
//! labels to a stage — stays in plain functions that take values and return `Result`, and can be
//! unit tested with no network and no repository.
//!
//! # Design notes
//!
//! **The precondition for `building` reads the decision sheet rather than only its name.** Every
//! other precondition here asks "does this file exist in `origin/main`", because a merged file is
//! the handoff. The handoff ADR-0051 names is an *answered* sheet, and a sheet whose entries still
//! read `_pending_` merges exactly as easily as one that is answered. So the check opens the file
//! and refuses on any line carrying that marker: the header's `**Answered**` field and every
//! entry's `**Answer**` field both use it, so neither needs a rule of its own, and a sheet the
//! template never touched is caught by `decisions.md` being absent instead.

use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

/// One of the two stages a feature crosses, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    /// The spec, the research and the decision sheet are being written; branch `NNN-slug-deciding`,
    /// label `deciding`.
    Deciding,
    /// The sheet is answered and merged, and the design, the tasks and the code follow it; branch
    /// `NNN-slug-building`, label `building`.
    Building,
}

impl Stage {
    /// The word this stage is named by, which is both its branch suffix and its label.
    ///
    /// The two differed under the three stages ADR-0051 replaced — the branch was `-impl` and the
    /// label was `doing` — and the rename brought them together.
    fn name(self) -> &'static str {
        match self {
            Stage::Deciding => "deciding",
            Stage::Building => "building",
        }
    }

    /// The label the previous stage left behind, which moving into this stage removes.
    fn label_to_remove(self) -> &'static str {
        match self {
            Stage::Deciding => "wish",
            Stage::Building => "deciding",
        }
    }

    /// The Spec Kit commands to run once this stage's branch is checked out, one per session.
    fn next_step(self) -> &'static str {
        match self {
            Stage::Deciding => concat!(
                "run `/speckit-specify`, then `/speckit-clarify` if the spec leaves open ",
                "questions, then `/speckit-plan` part one, which stops at `decisions.md`"
            ),
            Stage::Building => {
                "run `/speckit-plan` part two, then `/speckit-tasks`, then `/speckit-implement`"
            }
        }
    }
}

/// Runs the `spec` command: dispatches to `new`, `stage` or `use` from the remaining arguments.
pub fn run(mut args: impl Iterator<Item = String>) -> ExitCode {
    match args.next().as_deref() {
        Some("new") => run_new(args),
        Some("stage") => run_stage(args),
        Some("use") => run_use(args),
        None | Some("help" | "--help" | "-h") => {
            print_usage();
            ExitCode::SUCCESS
        }
        Some(unknown) => {
            eprintln!("xtask: unknown `spec` verb `{unknown}`\n");
            print_usage();
            ExitCode::FAILURE
        }
    }
}

/// Parses `spec new`'s argument and runs it.
fn run_new(mut args: impl Iterator<Item = String>) -> ExitCode {
    match parse_issue(args.next()) {
        Ok(issue) => to_exit_code(new_feature(&crate::workspace_root(), issue)),
        Err(message) => fail(&message),
    }
}

/// Parses `spec stage`'s arguments and runs it.
fn run_stage(mut args: impl Iterator<Item = String>) -> ExitCode {
    let issue = match parse_issue(args.next()) {
        Ok(issue) => issue,
        Err(message) => return fail(&message),
    };
    if let Err(message) = parse_stage(args.next().as_deref()) {
        return fail(&message);
    }
    to_exit_code(build_feature(&crate::workspace_root(), issue))
}

/// Parses `spec use`'s argument and runs it.
fn run_use(mut args: impl Iterator<Item = String>) -> ExitCode {
    match parse_issue(args.next()) {
        Ok(issue) => to_exit_code(use_feature(&crate::workspace_root(), issue)),
        Err(message) => fail(&message),
    }
}

/// Parses an issue number argument, which is required and must be a positive integer.
fn parse_issue(raw: Option<String>) -> Result<u32, String> {
    let raw = raw.ok_or_else(|| "xtask: spec requires an issue number".to_string())?;
    raw.parse::<u32>()
        .map_err(|_| format!("xtask: `{raw}` is not a valid issue number"))
}

/// Parses `stage`'s second argument, which must be `build`.
///
/// `build` is the only stage `stage` can move into: `new` opens the deciding stage, and there is no
/// third one. The verb still takes the word, so the command says which stage it opens rather than
/// leaving a reader of the shell history to remember that there is only one.
fn parse_stage(raw: Option<&str>) -> Result<(), String> {
    match raw {
        Some("build") => Ok(()),
        Some(other) => Err(format!("xtask: stage must be `build`, not `{other}`")),
        None => Err("xtask: spec stage requires `build`".to_string()),
    }
}

/// Prints `message` to standard error and reports failure.
fn fail(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::FAILURE
}

/// Converts a `Result` into the `ExitCode` `main` returns, printing the error if there is one.
fn to_exit_code(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => fail(&message),
    }
}

/// Opens the deciding stage for `issue`: fetches, reads the issue's title, derives its slug, opens
/// `NNN-slug-deciding`, moves the label to `deciding`, and writes `.specify/feature.json`.
fn new_feature(root: &Path, issue: u32) -> Result<(), String> {
    ensure_gh_ready(root)?;
    run_visible(root, "git", &["fetch", "origin", "--prune"])?;

    let issue_str = issue.to_string();
    let title = capture(
        root,
        "gh",
        &[
            "issue", "view", &issue_str, "--json", "title", "--jq", ".title",
        ],
    )?;
    let slug = slugify(&title)?;
    let issue_number = format_issue_number(issue);
    let branch = branch_name(&issue_number, &slug, Stage::Deciding);

    ensure_branch(root, issue, &branch)?;
    move_label(root, issue, Stage::Deciding)?;

    let feature_directory = format!("specs/{issue_number}-{slug}");
    write_feature_json(root, &feature_directory)?;

    println!("Feature directory: {feature_directory}");
    println!("Branch: {branch}");
    println!("Next: {}", Stage::Deciding.next_step());
    Ok(())
}

/// Opens the building stage for `issue`, after checking against `origin/main` that the deciding
/// stage merged and that the decision sheet it merged is answered.
fn build_feature(root: &Path, issue: u32) -> Result<(), String> {
    ensure_gh_ready(root)?;
    run_visible(root, "git", &["fetch", "origin", "--prune"])?;

    let issue_number = format_issue_number(issue);
    let entries = list_spec_directories(root)?;
    let slug = find_slug(&entries, &issue_number)?;

    verify_deciding_merged(root, &issue_number, &slug)?;

    let branch = branch_name(&issue_number, &slug, Stage::Building);
    ensure_branch(root, issue, &branch)?;
    move_label(root, issue, Stage::Building)?;

    let feature_directory = format!("specs/{issue_number}-{slug}");
    write_feature_json(root, &feature_directory)?;

    println!("Feature directory: {feature_directory}");
    println!("Branch: {branch}");
    println!("Next: {}", Stage::Building.next_step());
    Ok(())
}

/// Puts the working copy on the branch of whatever stage `issue`'s labels say is current, with no
/// side effects on labels or remote branches.
fn use_feature(root: &Path, issue: u32) -> Result<(), String> {
    ensure_gh_ready(root)?;
    run_visible(root, "git", &["fetch", "origin", "--prune"])?;

    let issue_number = format_issue_number(issue);
    let entries = list_spec_directories(root)?;
    let slug = find_slug(&entries, &issue_number)?;

    let labels = current_labels(root, issue)?;
    let stage = stage_from_labels(&labels)?;

    let branch = branch_name(&issue_number, &slug, stage);
    checkout_branch(root, &branch)?;

    let feature_directory = format!("specs/{issue_number}-{slug}");
    write_feature_json(root, &feature_directory)?;

    println!("Feature directory: {feature_directory}");
    println!("Branch: {branch}");
    println!("Stage: {}", stage.name());
    println!("Next: {}", stage.next_step());
    Ok(())
}

/// Lists the immediate entries of `specs/` in `origin/main`, one per line, as
/// `git ls-tree --name-only origin/main specs/` reports them.
fn list_spec_directories(root: &Path) -> Result<Vec<String>, String> {
    let listing = capture(
        root,
        "git",
        &["ls-tree", "--name-only", "origin/main", "specs/"],
    )?;
    Ok(listing
        .lines()
        .map(str::to_string)
        .filter(|line| !line.is_empty())
        .collect())
}

/// Checks, against `origin/main`, that the deciding stage has merged: `spec.md` and `decisions.md`
/// are there, and no entry of the sheet is still unanswered.
///
/// # Errors
///
/// Names the missing file and says explicitly that the deciding pull request is not merged, or
/// quotes the lines on which the sheet is still pending.
fn verify_deciding_merged(root: &Path, issue_number: &str, slug: &str) -> Result<(), String> {
    for file in ["spec.md", "decisions.md"] {
        let path = format!("specs/{issue_number}-{slug}/{file}");
        if !file_exists_in_origin_main(root, &path)? {
            return Err(format!(
                "xtask: the deciding pull request is not merged: `{path}` does not exist in \
                 origin/main"
            ));
        }
    }

    let path = format!("specs/{issue_number}-{slug}/decisions.md");
    let object = format!("origin/main:{path}");
    let sheet = capture_untrimmed(root, "git", &["show", &object])?;
    let pending = pending_lines(&sheet);

    if pending.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "xtask: `{path}` is not answered: {}. The building stage runs against an answered \
             sheet, so answer it on the deciding branch and merge that first.",
            describe_pending(&pending)
        ))
    }
}

/// The lines of a decision sheet that still carry the template's `_pending_` marker, as
/// one-based line numbers paired with the trimmed text of the line.
fn pending_lines(sheet: &str) -> Vec<(usize, String)> {
    sheet
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("_pending_"))
        .map(|(index, line)| (index + 1, line.trim().to_string()))
        .collect()
}

/// Renders what `pending_lines` found for the operator: the first three, quoted, and a count of
/// whatever else is left. Three is enough to recognize which entries they are; the file itself is
/// where they get read.
fn describe_pending(pending: &[(usize, String)]) -> String {
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

/// Ensures `branch` exists, checking it out. Uses the remote branch if `gh issue develop` (or an
/// earlier run of this command) already created it; otherwise creates it linked to `issue`.
fn ensure_branch(root: &Path, issue: u32, branch: &str) -> Result<(), String> {
    if remote_branch_exists(root, branch)? {
        checkout_branch(root, branch)
    } else {
        let issue_str = issue.to_string();
        run_visible(
            root,
            "gh",
            &[
                "issue",
                "develop",
                &issue_str,
                "--name",
                branch,
                "--base",
                "main",
                "--checkout",
            ],
        )
    }
}

/// Checks out `branch`, creating a local tracking branch from `origin/<branch>` if there is no
/// local branch by that name yet.
fn checkout_branch(root: &Path, branch: &str) -> Result<(), String> {
    if local_branch_exists(root, branch)? {
        run_visible(root, "git", &["checkout", branch])
    } else {
        let upstream = format!("origin/{branch}");
        run_visible(
            root,
            "git",
            &["checkout", "-b", branch, "--track", &upstream],
        )
    }
}

/// Whether `branch` exists as a local branch.
fn local_branch_exists(root: &Path, branch: &str) -> Result<bool, String> {
    let reference = format!("refs/heads/{branch}");
    let status = Command::new("git")
        .args(["show-ref", "--verify", "--quiet", &reference])
        .current_dir(root)
        .status()
        .map_err(|error| format!("xtask: could not run `git show-ref`: {error}"))?;
    Ok(status.success())
}

/// Whether `branch` exists on the `origin` remote.
fn remote_branch_exists(root: &Path, branch: &str) -> Result<bool, String> {
    let output = capture(root, "git", &["ls-remote", "--heads", "origin", branch])?;
    Ok(!output.is_empty())
}

/// Whether `path` exists as a file in `origin/main`, checked with `git cat-file -e`.
fn file_exists_in_origin_main(root: &Path, path: &str) -> Result<bool, String> {
    let object = format!("origin/main:{path}");
    let status = Command::new("git")
        .args(["cat-file", "-e", &object])
        .current_dir(root)
        // A missing file is the answer this asks for, not a failure to report, and git prints its
        // own `fatal: path ... does not exist` to stderr on the way to saying so. Silencing it
        // leaves the caller's message as the only one the operator reads.
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("xtask: could not run `git cat-file`: {error}"))?;
    Ok(status.success())
}

/// The names of the labels currently on `issue`.
fn current_labels(root: &Path, issue: u32) -> Result<Vec<String>, String> {
    let issue_str = issue.to_string();
    let output = capture(
        root,
        "gh",
        &[
            "issue",
            "view",
            &issue_str,
            "--json",
            "labels",
            "--jq",
            ".labels[].name",
        ],
    )?;
    Ok(output
        .lines()
        .map(str::to_string)
        .filter(|line| !line.is_empty())
        .collect())
}

/// Moves `issue`'s label to `stage`, removing the previous stage's label and adding this one, and
/// calling `gh issue edit` only when at least one of those is actually needed.
fn move_label(root: &Path, issue: u32, stage: Stage) -> Result<(), String> {
    let labels = current_labels(root, issue)?;
    let issue_str = issue.to_string();
    let remove = stage.label_to_remove();
    let add = stage.name();

    let mut args: Vec<&str> = vec!["issue", "edit", &issue_str];
    if labels.iter().any(|label| label == remove) {
        args.push("--remove-label");
        args.push(remove);
    }
    if !labels.iter().any(|label| label == add) {
        args.push("--add-label");
        args.push(add);
    }

    if args.len() > 3 {
        run_visible(root, "gh", &args)
    } else {
        Ok(())
    }
}

/// Writes `.specify/feature.json` at the workspace root, pointing Spec Kit at `feature_directory`.
fn write_feature_json(root: &Path, feature_directory: &str) -> Result<(), String> {
    let path = root.join(".specify").join("feature.json");
    let contents = format!("{{\"feature_directory\": \"{feature_directory}\"}}\n");
    fs::write(&path, contents)
        .map_err(|error| format!("xtask: could not write {}: {error}", path.display()))
}

/// Confirms `gh` is installed and authenticated, distinguishing the two failure modes so the
/// fix is never ambiguous.
fn ensure_gh_ready(root: &Path) -> Result<(), String> {
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
fn run_visible(root: &Path, program: &str, args: &[&str]) -> Result<(), String> {
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
fn capture(root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
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
fn capture_untrimmed(root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
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

/// Zero-pads `issue` to at least three digits; an issue number with more digits keeps all of them.
fn format_issue_number(issue: u32) -> String {
    format!("{issue:03}")
}

/// The branch name for `stage` of the feature identified by `issue_number` and `slug`.
fn branch_name(issue_number: &str, slug: &str, stage: Stage) -> String {
    format!("{issue_number}-{slug}-{}", stage.name())
}

/// Finds the single entry in `entries` (as `git ls-tree --name-only origin/main specs/` reports
/// them) whose name is `specs/{issue_number}-*`, and returns the slug that follows the prefix.
///
/// # Errors
///
/// Names what was found when there is no match or more than one.
fn find_slug(entries: &[String], issue_number: &str) -> Result<String, String> {
    let prefix = format!("specs/{issue_number}-");
    let matches: Vec<&String> = entries
        .iter()
        .filter(|entry| entry.starts_with(&prefix))
        .collect();

    match matches.as_slice() {
        [single] => Ok(single[prefix.len()..].to_string()),
        [] => Err(format!(
            "xtask: no directory matching `specs/{issue_number}-*` was found in origin/main"
        )),
        multiple => {
            let found = multiple
                .iter()
                .map(|entry| entry.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!(
                "xtask: expected exactly one directory matching `specs/{issue_number}-*` in \
                 origin/main, found {}: {found}",
                multiple.len()
            ))
        }
    }
}

/// Maps an issue's current labels to the stage they say is active.
///
/// # Errors
///
/// A `wish` label says the deciding stage has not been opened yet and names the command that opens
/// it, rather than failing obscurely. Neither state label names which labels the issue does carry
/// instead.
fn stage_from_labels(labels: &[String]) -> Result<Stage, String> {
    let has = |name: &str| labels.iter().any(|label| label == name);

    if has("building") {
        Ok(Stage::Building)
    } else if has("deciding") {
        Ok(Stage::Deciding)
    } else if has("wish") {
        Err(
            "xtask: the deciding stage has not been opened yet; `cargo xtask spec new <issue>` \
             opens it."
                .to_string(),
        )
    } else if labels.is_empty() {
        Err(
            "xtask: the issue carries neither the deciding nor the building label; it carries no \
             labels at all"
                .to_string(),
        )
    } else {
        Err(format!(
            "xtask: the issue carries neither the deciding nor the building label; it carries: {}",
            labels.join(", ")
        ))
    }
}

/// Derives a slug from an issue title: lowercase, accented Latin-1 letters folded to ASCII, any run
/// of characters that are not `[a-z0-9]` collapsed to one hyphen, truncated to 40 characters with
/// no trailing hyphen.
///
/// # Errors
///
/// A title with nothing left after folding (punctuation only) is an error.
fn slugify(title: &str) -> Result<String, String> {
    /// Latin-1 accented letters (both cases) folded to their plain ASCII letter.
    const ACCENTED: &[(char, char)] = &[
        ('á', 'a'),
        ('é', 'e'),
        ('í', 'i'),
        ('ó', 'o'),
        ('ú', 'u'),
        ('à', 'a'),
        ('è', 'e'),
        ('ì', 'i'),
        ('ò', 'o'),
        ('ù', 'u'),
        ('ä', 'a'),
        ('ë', 'e'),
        ('ï', 'i'),
        ('ö', 'o'),
        ('ü', 'u'),
        ('â', 'a'),
        ('ê', 'e'),
        ('î', 'i'),
        ('ô', 'o'),
        ('û', 'u'),
        ('ã', 'a'),
        ('õ', 'o'),
        ('ñ', 'n'),
        ('ç', 'c'),
        ('Á', 'a'),
        ('É', 'e'),
        ('Í', 'i'),
        ('Ó', 'o'),
        ('Ú', 'u'),
        ('À', 'a'),
        ('È', 'e'),
        ('Ì', 'i'),
        ('Ò', 'o'),
        ('Ù', 'u'),
        ('Ä', 'a'),
        ('Ë', 'e'),
        ('Ï', 'i'),
        ('Ö', 'o'),
        ('Ü', 'u'),
        ('Â', 'a'),
        ('Ê', 'e'),
        ('Î', 'i'),
        ('Ô', 'o'),
        ('Û', 'u'),
        ('Ã', 'a'),
        ('Õ', 'o'),
        ('Ñ', 'n'),
        ('Ç', 'c'),
    ];

    let mut slug = String::new();
    let mut pending_hyphen = false;

    for ch in title.chars() {
        let kept = ACCENTED
            .iter()
            .find(|(from, _)| *from == ch)
            .map(|(_, to)| *to)
            .or_else(|| ch.is_ascii_alphanumeric().then(|| ch.to_ascii_lowercase()));

        match kept {
            Some(c) => {
                if pending_hyphen && !slug.is_empty() {
                    slug.push('-');
                }
                slug.push(c);
                pending_hyphen = false;
            }
            // Neither one of the accented letters above nor plain ASCII: punctuation, whitespace,
            // or any other non-ASCII character. It joins the run that becomes a single hyphen,
            // rather than being kept or turned into a hyphen of its own.
            None => pending_hyphen = true,
        }
    }

    if slug.chars().count() > 40 {
        slug = slug.chars().take(40).collect();
        while slug.ends_with('-') {
            slug.pop();
        }
    }

    if slug.is_empty() {
        return Err(format!(
            "xtask: could not derive a slug from title `{title}`"
        ));
    }

    Ok(slug)
}

/// Prints usage for `cargo xtask spec`.
fn print_usage() {
    println!("Feature branch lifecycle for Monospace's two-stage Spec Kit workflow.");
    println!();
    println!("Usage: cargo xtask spec <verb> [args]");
    println!();
    println!("Verbs:");
    println!("  new <issue>           Open the deciding stage for an issue");
    println!("  stage <issue> build   Open the building stage, once the answered sheet has merged");
    println!("  use <issue>           Check out the branch of an issue's active stage");
    println!("  help                  Show this message");
}

#[cfg(test)]
mod tests {
    use super::{
        Stage, branch_name, describe_pending, find_slug, format_issue_number, parse_stage,
        pending_lines, slugify, stage_from_labels,
    };

    #[test]
    fn slugify_plain_title() {
        assert_eq!(
            slugify("Draw shapes instead of individual cells").unwrap(),
            "draw-shapes-instead-of-individual-cells"
        );
    }

    #[test]
    fn slugify_folds_accented_letters() {
        assert_eq!(
            slugify("A naïve café soirée, jalapeño").unwrap(),
            "a-naive-cafe-soiree-jalapeno"
        );
    }

    #[test]
    fn slugify_collapses_punctuation_and_spaces() {
        assert_eq!(slugify("Fix   bug!!  (urgent)").unwrap(), "fix-bug-urgent");
    }

    #[test]
    fn slugify_truncates_to_forty_characters_with_no_trailing_hyphen() {
        let title = "This is a very long issue title that exceeds forty characters easily";
        let slug = slugify(title).unwrap();
        assert!(
            slug.chars().count() <= 40,
            "slug longer than 40 characters: {slug}"
        );
        assert!(!slug.ends_with('-'), "slug ends in a hyphen: {slug}");
    }

    #[test]
    fn slugify_rejects_punctuation_only_title() {
        assert!(slugify("!!! ??? ---").is_err());
    }

    #[test]
    fn format_issue_number_pads_to_three_digits() {
        assert_eq!(format_issue_number(39), "039");
        assert_eq!(format_issue_number(6), "006");
    }

    #[test]
    fn format_issue_number_keeps_extra_digits() {
        assert_eq!(format_issue_number(1234), "1234");
    }

    #[test]
    fn branch_name_for_each_stage() {
        assert_eq!(
            branch_name("039", "draw-shapes", Stage::Deciding),
            "039-draw-shapes-deciding"
        );
        assert_eq!(
            branch_name("039", "draw-shapes", Stage::Building),
            "039-draw-shapes-building"
        );
    }

    #[test]
    fn parse_stage_takes_build_and_nothing_else() {
        assert!(parse_stage(Some("build")).is_ok());

        let error = parse_stage(Some("plan")).unwrap_err();
        assert!(
            error.contains("`build`"),
            "error did not name the only stage: {error}"
        );

        let error = parse_stage(None).unwrap_err();
        assert!(
            error.contains("`build`"),
            "error did not name the only stage: {error}"
        );
    }

    #[test]
    fn find_slug_picks_the_single_match() {
        let entries = vec!["specs/006-foo".to_string(), "specs/012-bar".to_string()];
        assert_eq!(find_slug(&entries, "012").unwrap(), "bar");
    }

    #[test]
    fn find_slug_errors_on_no_match() {
        let entries = vec!["specs/006-foo".to_string()];
        let error = find_slug(&entries, "999").unwrap_err();
        assert!(
            error.contains("999"),
            "error did not name the missing issue: {error}"
        );
    }

    #[test]
    fn find_slug_errors_on_two_matches() {
        let entries = vec!["specs/012-bar".to_string(), "specs/012-baz".to_string()];
        let error = find_slug(&entries, "012").unwrap_err();
        assert!(
            error.contains("specs/012-bar"),
            "error did not name what was found: {error}"
        );
        assert!(
            error.contains("specs/012-baz"),
            "error did not name what was found: {error}"
        );
    }

    #[test]
    fn stage_from_labels_maps_each_state_label() {
        assert_eq!(
            stage_from_labels(&["deciding".to_string()]).unwrap(),
            Stage::Deciding
        );
        assert_eq!(
            stage_from_labels(&["building".to_string()]).unwrap(),
            Stage::Building
        );
    }

    #[test]
    fn stage_from_labels_names_wish_explicitly() {
        let error = stage_from_labels(&["wish".to_string()]).unwrap_err();
        assert!(
            error.contains("spec new"),
            "error did not point at `spec new`: {error}"
        );
    }

    #[test]
    fn stage_from_labels_errors_naming_what_it_carries() {
        let error = stage_from_labels(&["triage".to_string()]).unwrap_err();
        assert!(
            error.contains("triage"),
            "error did not name the labels found: {error}"
        );
    }

    /// A sheet shaped like `.specify/templates/decisions-template.md`, with `answered` deciding
    /// whether its three `_pending_` markers are still there.
    fn sheet(answered: bool) -> String {
        let (header, first, second) = if answered {
            (
                "2026-09-21",
                "Yes, the ranking stays.",
                "Module, in arrow.rs.",
            )
        } else {
            ("_pending_", "_pending_", "_pending_")
        };

        format!(
            "# Decisions: Route an arrow\n\
             \n\
             **Feature**: `103-route` | **Written**: 2026-09-20 | **Answered**: {header}\n\
             \n\
             ## D1 — Which route wins a tie?\n\
             \n\
             - **Answer**: {first}\n\
             \n\
             ## D2 — Where does the rule live?\n\
             \n\
             - **Answer**: {second}\n"
        )
    }

    #[test]
    fn pending_lines_finds_nothing_in_an_answered_sheet() {
        assert!(pending_lines(&sheet(true)).is_empty());
    }

    #[test]
    fn pending_lines_finds_the_header_and_every_entry() {
        let pending = pending_lines(&sheet(false));
        let numbers: Vec<usize> = pending.iter().map(|(number, _)| *number).collect();
        assert_eq!(numbers, vec![3, 7, 11]);
        assert_eq!(pending[1].1, "- **Answer**: _pending_");
    }

    #[test]
    fn describe_pending_quotes_three_and_counts_the_rest() {
        let pending: Vec<(usize, String)> = (1..=5)
            .map(|number| (number, "- **Answer**: _pending_".to_string()))
            .collect();
        let described = describe_pending(&pending);
        assert!(
            described.starts_with("line 1 reads `- **Answer**: _pending_`"),
            "the first pending line was not quoted: {described}"
        );
        assert!(
            described.ends_with("and 2 more"),
            "the rest were not counted: {described}"
        );
    }
}
