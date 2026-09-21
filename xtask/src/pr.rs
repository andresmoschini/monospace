//! `cargo xtask pr`: prepare and open the pull request this branch is for.
//!
//! The repository has three shapes of change and a body for each
//! ([`.github/PULL_REQUEST_TEMPLATE/`](../../.github/PULL_REQUEST_TEMPLATE/README.md)), and GitHub
//! offers only one of the three by default. `body` writes the right one to `target/pr-body.md`
//! together with the closing keyword the stage calls for; `open` checks it was filled in, pushes
//! the branch and calls `gh pr create` with it.
//!
//! # Design notes
//!
//! **The shape comes from the branch, not from the issue's label.** Both answer, and they agree,
//! but the label says where the issue stands while the branch says what is about to be proposed —
//! and a tooling change has a branch and no state label at all. Reading the branch also costs no
//! network, which keeps `body` usable with nothing fetched.
//!
//! **It is two verbs because a body has to be filled between them.** One verb would either submit
//! a template with its prompts unanswered, which is the failure the three templates exist to
//! prevent, or open an editor, which a session cannot answer. Splitting them also gives an agent
//! the same contract a person gets: run `body`, write into the file, run `open`.
//!
//! **`open` refuses a body whose sections are all still empty**, by the same reasoning that makes
//! `spec stage build` read the decision sheet rather than its name: the artifact that merges is the
//! handoff, and an unfilled section is as easy to push as a filled one. A section is empty when it
//! holds nothing but headings, HTML comments and the keyword line — the check is textual, and it
//! cannot tell a thoughtful paragraph from a careless one.
//!
//! **The keyword is appended rather than left to the author.** Which stage closes the issue and
//! which only references it is a rule of the flow, written in `CONTRIBUTING.md`, and every pull
//! request had to restate it correctly from memory. `--refs` is the escape hatch for the case the
//! rule does not cover: a change that belongs to an umbrella issue it does not finish.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::process::{capture, ensure_gh_ready, run_visible};

/// Where `body` writes, and where `open` reads from when no path is given.
const DEFAULT_BODY_PATH: &str = "target/pr-body.md";

/// One of the three shapes of change the repository has, each with its own pull request body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// Stage one of a feature: the spec, the research and the answered decision sheet.
    Deciding,
    /// Stage two of a feature: the design, the tasks, the code and the tests.
    Building,
    /// A change to the repository's own tooling, documents or rules. No spec directory, no stage.
    Tooling,
}

impl Shape {
    /// The template this shape's body is copied from, relative to the workspace root.
    fn template(self) -> &'static str {
        match self {
            Shape::Deciding => ".github/PULL_REQUEST_TEMPLATE/deciding.md",
            Shape::Building => ".github/PULL_REQUEST_TEMPLATE/building.md",
            // The tooling body is GitHub's default and lives at the path GitHub reads, rather than
            // in the directory beside the other two; the directory's README says why.
            Shape::Tooling => ".github/pull_request_template.md",
        }
    }

    /// The keyword this shape's body carries: only the last pull request of a feature closes the
    /// issue, and a tooling change has exactly one.
    fn keyword(self) -> &'static str {
        match self {
            Shape::Deciding => "Refs",
            Shape::Building | Shape::Tooling => "Closes",
        }
    }

    /// What this shape prefixes a derived title with, where the title comes from the issue.
    fn title_prefix(self) -> Option<&'static str> {
        match self {
            Shape::Deciding => Some("Decide"),
            Shape::Building => Some("Build"),
            Shape::Tooling => None,
        }
    }

    /// How this shape reads in a message to the operator.
    fn name(self) -> &'static str {
        match self {
            Shape::Deciding => "deciding",
            Shape::Building => "building",
            Shape::Tooling => "tooling",
        }
    }
}

/// Runs the `pr` command: dispatches to `body` or `open` from the remaining arguments.
pub fn run(mut args: impl Iterator<Item = String>) -> ExitCode {
    let rest: Vec<String> = args.by_ref().collect();
    match rest.first().map(String::as_str) {
        Some("body") => to_exit_code(write_body(&crate::workspace_root(), &rest[1..])),
        Some("open") => to_exit_code(open_pull_request(&crate::workspace_root(), &rest[1..])),
        None | Some("help" | "--help" | "-h") => {
            print_usage();
            ExitCode::SUCCESS
        }
        Some(unknown) => {
            eprintln!("xtask: unknown `pr` verb `{unknown}`\n");
            print_usage();
            ExitCode::FAILURE
        }
    }
}

/// Converts a `Result` into the `ExitCode` `main` returns, printing the error if there is one.
fn to_exit_code(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

/// Writes the body for this branch's shape to `target/pr-body.md`, keyword included.
fn write_body(root: &Path, args: &[String]) -> Result<(), String> {
    let refs_only = args.iter().any(|argument| argument == "--refs");
    let issue_argument = parse_issue_argument(args)?;

    let branch = current_branch(root)?;
    let shape = shape_from_branch(&branch);
    let issue = resolve_issue(&branch, shape, issue_argument)?;

    let template = root.join(shape.template());
    let contents = fs::read_to_string(&template)
        .map_err(|error| format!("xtask: could not read {}: {error}", template.display()))?;

    let keyword = if refs_only { "Refs" } else { shape.keyword() };
    let body = format!("{}\n{keyword} #{issue}\n", contents.trim_end());

    let path = root.join(DEFAULT_BODY_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("xtask: could not create {}: {error}", parent.display()))?;
    }
    fs::write(&path, body)
        .map_err(|error| format!("xtask: could not write {}: {error}", path.display()))?;

    println!("Shape: {} (from branch {branch})", shape.name());
    println!("Template: {}", shape.template());
    println!("Keyword: {keyword} #{issue}");
    println!("Body: {DEFAULT_BODY_PATH}");
    println!("Next: fill every section, then run `cargo xtask pr open`");
    Ok(())
}

/// Pushes the branch and opens the pull request with the body prepared for it.
fn open_pull_request(root: &Path, args: &[String]) -> Result<(), String> {
    let title_argument = parse_flag(args, "--title")?;
    let body_argument = parse_flag(args, "--body-file")?;

    let branch = current_branch(root)?;
    let shape = shape_from_branch(&branch);

    let dirty = capture(root, "git", &["status", "--porcelain"])?;
    if !dirty.is_empty() {
        return Err(format!(
            "xtask: the working tree is not clean, so the pull request would not hold everything \
             you have:\n{dirty}"
        ));
    }

    let body_path = body_argument.map_or_else(|| root.join(DEFAULT_BODY_PATH), PathBuf::from);
    let body = fs::read_to_string(&body_path).map_err(|error| {
        format!(
            "xtask: could not read {}: {error}. `cargo xtask pr body` writes it.",
            body_path.display()
        )
    })?;

    let empty = empty_sections(&body);
    if !empty.is_empty() {
        return Err(format!(
            "xtask: {} of the body is still empty: {}. A section with nothing to say says \"None.\"",
            if empty.len() == 1 {
                "a section"
            } else {
                "part"
            },
            empty.join(", ")
        ));
    }

    ensure_gh_ready(root)?;

    let title = match title_argument {
        Some(title) => title,
        None => derive_title(root, &branch, shape)?,
    };

    run_visible(root, "git", &["push", "--set-upstream", "origin", &branch])?;

    let body_string = body_path.to_string_lossy().to_string();
    run_visible(
        root,
        "gh",
        &[
            "pr",
            "create",
            "--base",
            "main",
            "--head",
            &branch,
            "--title",
            &title,
            "--body-file",
            &body_string,
        ],
    )
}

/// The title to open with when none was given: the issue's own title behind the stage's verb, or
/// for a tooling change the subject of the first commit the branch added.
fn derive_title(root: &Path, branch: &str, shape: Shape) -> Result<String, String> {
    if let (Some(prefix), Some(issue)) = (shape.title_prefix(), issue_from_branch(branch)) {
        let issue_str = issue.to_string();
        let title = capture(
            root,
            "gh",
            &[
                "issue", "view", &issue_str, "--json", "title", "--jq", ".title",
            ],
        )?;
        return Ok(format!("{prefix}: {title}"));
    }

    let log = capture(
        root,
        "git",
        &["log", "--reverse", "--format=%s", "origin/main..HEAD"],
    )?;
    log.lines().next().map(str::to_string).ok_or_else(|| {
        "xtask: this branch adds no commit to origin/main, so there is nothing to propose and no          title to derive. `--title` overrides."
            .to_string()
    })
}

/// The branch the working copy is on.
///
/// # Errors
///
/// `main` and a detached head are both refused: a pull request from either is not a proposal.
fn current_branch(root: &Path) -> Result<String, String> {
    let branch = capture(root, "git", &["rev-parse", "--abbrev-ref", "HEAD"])?;
    match branch.as_str() {
        "main" => Err("xtask: this is `main`; a pull request is opened from a branch.".to_string()),
        "HEAD" => Err("xtask: the head is detached, so there is no branch to propose.".to_string()),
        _ => Ok(branch),
    }
}

/// Which shape of change `branch` carries, read from the suffix `cargo xtask spec` gave it.
fn shape_from_branch(branch: &str) -> Shape {
    if branch.ends_with("-deciding") {
        Shape::Deciding
    } else if branch.ends_with("-building") {
        Shape::Building
    } else {
        Shape::Tooling
    }
}

/// The issue number `branch` starts with, where it starts with one.
fn issue_from_branch(branch: &str) -> Option<u32> {
    let digits: String = branch.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// Settles which issue the body references: the branch's, the argument, or an error saying which
/// is missing or that the two disagree.
fn resolve_issue(branch: &str, shape: Shape, argument: Option<u32>) -> Result<u32, String> {
    match (issue_from_branch(branch), argument) {
        (Some(from_branch), None) => Ok(from_branch),
        (Some(from_branch), Some(given)) if from_branch == given => Ok(from_branch),
        (Some(from_branch), Some(given)) => Err(format!(
            "xtask: branch `{branch}` is feature {from_branch}, but {given} was given"
        )),
        (None, Some(given)) => Ok(given),
        (None, None) => Err(format!(
            "xtask: a {} branch carries no issue number, so this needs one: \
             `cargo xtask pr body <issue>`",
            shape.name()
        )),
    }
}

/// Parses the single positional argument, an issue number, from `args`.
fn parse_issue_argument(args: &[String]) -> Result<Option<u32>, String> {
    let positional = args.iter().find(|argument| !argument.starts_with("--"));
    match positional {
        None => Ok(None),
        Some(raw) => raw
            .parse::<u32>()
            .map(Some)
            .map_err(|_| format!("xtask: `{raw}` is not a valid issue number")),
    }
}

/// Parses `--name value` out of `args`.
fn parse_flag(args: &[String], name: &str) -> Result<Option<String>, String> {
    match args.iter().position(|argument| argument == name) {
        None => Ok(None),
        Some(index) => args
            .get(index + 1)
            .cloned()
            .map(Some)
            .ok_or_else(|| format!("xtask: `{name}` needs a value")),
    }
}

/// The headings of `body` under which nothing was written.
///
/// Headings, HTML comments and the trailing keyword line are not content: a body straight out of a
/// template is every section, and a body with one section left blank is that one.
fn empty_sections(body: &str) -> Vec<String> {
    let mut sections: Vec<(String, bool)> = Vec::new();
    let mut in_comment = false;

    for line in body.lines() {
        let text = strip_comments(line, &mut in_comment);
        let text = text.trim();

        if let Some(heading) = text.strip_prefix("## ") {
            sections.push((heading.trim().to_string(), false));
        } else if !text.is_empty()
            && !is_keyword_line(text)
            && let Some((_, filled)) = sections.last_mut()
        {
            *filled = true;
        }
    }

    sections
        .into_iter()
        .filter(|(_, filled)| !filled)
        .map(|(heading, _)| heading)
        .collect()
}

/// `line` with every HTML comment removed, carrying whether a comment is still open into the next
/// call through `in_comment`.
fn strip_comments(line: &str, in_comment: &mut bool) -> String {
    let mut kept = String::new();
    let mut rest = line;

    loop {
        if *in_comment {
            let Some(end) = rest.find("-->") else {
                return kept;
            };
            rest = &rest[end + 3..];
            *in_comment = false;
        } else {
            let Some(start) = rest.find("<!--") else {
                kept.push_str(rest);
                return kept;
            };
            kept.push_str(&rest[..start]);
            rest = &rest[start + 4..];
            *in_comment = true;
        }
    }
}

/// Whether `text` is the keyword line a body ends with, which is written by `body` rather than by
/// whoever filled the sections in.
fn is_keyword_line(text: &str) -> bool {
    let Some(rest) = text
        .strip_prefix("Refs #")
        .or_else(|| text.strip_prefix("Closes #"))
    else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|character| character.is_ascii_digit())
}

/// Prints usage for `cargo xtask pr`.
fn print_usage() {
    println!("Pull request bodies for Monospace's three shapes of change.");
    println!();
    println!("Usage: cargo xtask pr <verb> [args]");
    println!();
    println!("Verbs:");
    println!("  body [<issue>] [--refs]   Write target/pr-body.md from this branch's template");
    println!("  open [--title <text>] [--body-file <path>]");
    println!("                            Push the branch and open the pull request with it");
    println!("  help                      Show this message");
    println!();
    println!("The shape comes from the branch: `-deciding`, `-building`, or anything else, which");
    println!("is a tooling change. A tooling branch carries no issue number, so `body` needs one.");
}

#[cfg(test)]
mod tests {
    use super::{
        Shape, empty_sections, is_keyword_line, issue_from_branch, resolve_issue, shape_from_branch,
    };

    #[test]
    fn shape_reads_the_branch_suffix() {
        assert_eq!(
            shape_from_branch("023-read-a-description-deciding"),
            Shape::Deciding
        );
        assert_eq!(
            shape_from_branch("023-read-a-description-building"),
            Shape::Building
        );
        assert_eq!(shape_from_branch("sdd-v2-two-stages"), Shape::Tooling);
    }

    #[test]
    fn each_shape_carries_its_own_keyword() {
        assert_eq!(Shape::Deciding.keyword(), "Refs");
        assert_eq!(Shape::Building.keyword(), "Closes");
        assert_eq!(Shape::Tooling.keyword(), "Closes");
    }

    #[test]
    fn issue_from_branch_reads_the_leading_digits_only() {
        assert_eq!(
            issue_from_branch("023-read-a-description-deciding"),
            Some(23)
        );
        assert_eq!(issue_from_branch("1234-something-building"), Some(1234));
        assert_eq!(issue_from_branch("sdd-v2-two-stages"), None);
    }

    #[test]
    fn resolve_issue_prefers_the_branch_and_catches_a_disagreement() {
        assert_eq!(
            resolve_issue("023-foo-deciding", Shape::Deciding, None).unwrap(),
            23
        );
        assert_eq!(
            resolve_issue("023-foo-deciding", Shape::Deciding, Some(23)).unwrap(),
            23
        );

        let error = resolve_issue("023-foo-deciding", Shape::Deciding, Some(24)).unwrap_err();
        assert!(
            error.contains("23") && error.contains("24"),
            "error did not name both numbers: {error}"
        );

        let error = resolve_issue("sdd-v2-two-stages", Shape::Tooling, None).unwrap_err();
        assert!(
            error.contains("pr body <issue>"),
            "error did not say how to supply one: {error}"
        );
    }

    #[test]
    fn a_body_straight_out_of_a_template_is_every_section() {
        let body = "<!-- a note about the file -->\n\
                    \n\
                    ## What changes\n\
                    \n\
                    <!-- One or two lines. -->\n\
                    \n\
                    ## Why now\n\
                    \n\
                    <!-- What forced it,\n\
                    across two lines. -->\n\
                    \n\
                    Closes #34\n";
        assert_eq!(empty_sections(body), vec!["What changes", "Why now"]);
    }

    #[test]
    fn a_filled_body_has_no_empty_section() {
        let body = "## What changes\n\
                    \n\
                    <!-- One or two lines. -->\n\
                    \n\
                    Two stages instead of three.\n\
                    \n\
                    ## Why now\n\
                    \n\
                    None.\n\
                    \n\
                    Closes #34\n";
        assert!(empty_sections(body).is_empty());
    }

    #[test]
    fn text_after_a_comment_on_the_same_line_is_content() {
        let body = "## What changes\n<!-- prompt --> It changes this.\n";
        assert!(empty_sections(body).is_empty());
    }

    #[test]
    fn the_keyword_line_is_not_content() {
        assert!(is_keyword_line("Refs #23"));
        assert!(is_keyword_line("Closes #1234"));
        assert!(!is_keyword_line("Closes #"));
        assert!(!is_keyword_line("Closes the loop on #23"));
    }
}
