//! `cargo xtask pr`: prepare and open the pull request this branch is for.
//!
//! The repository has one pull request body, and GitHub offers one of its own by default.
//! `body` writes that one to `target/pr-body.md` together with the closing keyword the branch's
//! shape calls for; `open` checks it was filled in, refuses a branch whose precondition is unmet,
//! pushes, and calls `gh pr create` with it.
//!
//! # Design notes
//!
//! **The shape comes from the branch, and the number is read before the suffix.** A branch either
//! opens with digits or it does not, and that question comes first: under this flow the common case
//! is a one-stage change on a bare `NNN-slug`, and reading the suffix first sent every one of them
//! down the tooling arm — the same body, the wrong keyword, and no number where the issue's was
//! wanted. Reading the digits first is what makes the three-arm match total.
//!
//! **There are two forms of change and three branch names, and that is not a contradiction.** A
//! one-stage change is `NNN-slug` with no suffix; a two-stage change has *both* `-deciding` and
//! `-building`. There is deliberately no suffix meaning "no decision to take": its absence is what
//! says that, so "is a decision pending?" is a question about the name that can be answered, and a
//! `-building` with no `-deciding` merged is a detectable mistake.
//!
//! **It is two verbs because a body has to be filled between them.** One verb would either submit
//! a template with its prompts unanswered, which is the failure the section check exists to prevent,
//! or open an editor, which a session cannot answer. Splitting them also gives an agent the same
//! contract a person gets: run `body`, write into the file, run `open`.
//!
//! **`open` refuses a body whose sections are all still empty**, by the same reasoning that makes
//! the building stage read the decision table rather than its name: the artifact that merges is the
//! handoff, and an unfilled section is as easy to push as a filled one. A section is empty when it
//! holds nothing but headings, HTML comments and the keyword line — the check is textual, and it
//! cannot tell a thoughtful paragraph from a careless one.
//!
//! **The section check knows about fences.** A `` ``` `` opens a block whose lines are text, so a
//! `<!--` inside one is two characters rather than a comment that never closes, and a `## ` inside
//! one is a heading in an example rather than a section of the body. The old check knew neither, and
//! a body carrying a generated picture — which is most of them — is exactly the case where both
//! happen. It also refused nothing at all for a body with no `## ` heading, which no template
//! produces and a hurried author might.
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

/// The one body every change opens with, relative to the workspace root.
///
/// It lives at the path GitHub reads rather than in a directory beside nothing, because there is one
/// template now and a directory holding one file is a directory with nothing to say.
const TEMPLATE: &str = ".github/pull_request_template.md";

/// The stage suffix `cargo xtask spec` gives the deciding branch.
const DECIDING: &str = crate::spec::DECIDING;
/// The stage suffix `cargo xtask spec` gives the building branch.
const BUILDING: &str = crate::spec::BUILDING;

/// What a branch says about the change it carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    /// The spec alone, before the code: branch `NNN-slug-deciding`.
    Deciding,
    /// The code, against a spec already agreed and merged: branch `NNN-slug-building`.
    Building,
    /// The whole change in one pull request: branch `NNN-slug`, no suffix.
    Single,
    /// A branch carrying no issue number, so it cannot be an issue's change at all.
    Unnumbered,
}

impl Stage {
    /// The keyword this stage's body carries: only the deciding pull request of a two-stage change
    /// references its issue rather than closing it.
    fn keyword(self) -> &'static str {
        match self {
            Stage::Deciding => "Refs",
            Stage::Building | Stage::Single | Stage::Unnumbered => "Closes",
        }
    }

    /// What this stage prefixes a derived title with, where the title comes from the issue.
    ///
    /// `None` is only for a branch with no issue to take a title from, and it means the other
    /// answer: the subject of the first commit the branch added. A one-stage change prefixes
    /// nothing, so its title is the issue's own.
    fn title_prefix(self) -> Option<&'static str> {
        match self {
            Stage::Deciding => Some("Decide: "),
            Stage::Building => Some("Build: "),
            Stage::Single => Some(""),
            Stage::Unnumbered => None,
        }
    }

    /// How this stage reads in a message to the operator.
    fn name(self) -> &'static str {
        match self {
            Stage::Deciding => "deciding",
            Stage::Building => "building",
            Stage::Single => "single-stage",
            Stage::Unnumbered => "unnumbered",
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

/// Writes the body for this branch's stage to `target/pr-body.md`, keyword included.
fn write_body(root: &Path, args: &[String]) -> Result<(), String> {
    let refs_only = args.iter().any(|argument| argument == "--refs");
    let issue_argument = parse_issue_argument(args)?;

    let branch = current_branch(root)?;
    let stage = stage_from_branch(&branch);
    let issue = resolve_issue(&branch, stage, issue_argument)?;

    let template = root.join(TEMPLATE);
    let contents = fs::read_to_string(&template)
        .map_err(|error| format!("xtask: could not read {}: {error}", template.display()))?;

    let keyword = if refs_only { "Refs" } else { stage.keyword() };
    let body = format!("{}\n{keyword} #{issue}\n", contents.trim_end());

    let path = root.join(DEFAULT_BODY_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("xtask: could not create {}: {error}", parent.display()))?;
    }
    fs::write(&path, body)
        .map_err(|error| format!("xtask: could not write {}: {error}", path.display()))?;

    println!("Stage: {} (from branch {branch})", stage.name());
    println!("Template: {TEMPLATE}");
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
    let stage = stage_from_branch(&branch);

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
            "xtask: {} still empty: {}. A section with nothing to say says \"None.\"",
            if empty.len() == 1 {
                "a section of the body is"
            } else {
                "parts of the body are"
            },
            empty.join(", ")
        ));
    }

    ensure_gh_ready(root)?;

    if stage == Stage::Building {
        verify_deciding_stage_merged(root, &branch)?;
    }

    let title = match title_argument {
        Some(title) => title,
        None => derive_title(root, &branch, stage)?,
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

/// Refuses a building branch whose deciding pull request has not merged, or whose spec is merged
/// with a row still unanswered.
///
/// This is the flow's only mechanical precondition, and it is the one the deciding stage exists
/// for: without it, `-building` is a name anybody can type and the agreement it is supposed to rest
/// on is a memory.
fn verify_deciding_stage_merged(root: &Path, branch: &str) -> Result<(), String> {
    // The answer is a question about `origin/main`, so origin has to be current for it to be one.
    run_visible(root, "git", &["fetch", "origin", "--prune"])?;

    let issue = issue_from_branch(branch)
        .ok_or_else(|| format!("xtask: `{branch}` carries no issue number"))?;
    let issue_number = crate::spec::format_issue_number(issue);
    let slug = crate::spec::find_slug(&crate::spec::list_specs(root)?, &issue_number)?;

    let deciding = format!("{issue_number}-{slug}-{DECIDING}");
    let merged = capture(
        root,
        "gh",
        &[
            "pr", "list", "--state", "merged", "--head", &deciding, "--json", "number",
        ],
    )?;
    if !lists_a_pull_request(&merged) {
        return Err(format!(
            "xtask: `{deciding}` has no merged pull request, so there is no agreed spec to build \
             against. Merge it first, or open this change as one stage on `{issue_number}-{slug}`."
        ));
    }

    crate::spec::verify_deciding_merged(root, &issue_number, &slug)
}

/// Whether `gh`'s answer to `pr list --json number` holds a pull request.
///
/// It is JSON this crate does not parse, and the whole question is whether the array is empty, so
/// the test is textual on purpose: `[]` is what `gh` prints for nothing, and anything carrying a
/// `number` key is a pull request.
fn lists_a_pull_request(answer: &str) -> bool {
    let answer = answer.trim();
    !answer.is_empty() && answer != "[]"
}

/// The title to open with when none was given: the issue's own title behind the stage's verb, or
/// for a branch carrying no issue the subject of the first commit the branch added.
fn derive_title(root: &Path, branch: &str, stage: Stage) -> Result<String, String> {
    if let Some(prefix) = stage.title_prefix()
        && let Some(issue) = issue_from_branch(branch)
    {
        let issue_str = issue.to_string();
        let title = capture(
            root,
            "gh",
            &[
                "issue", "view", &issue_str, "--json", "title", "--jq", ".title",
            ],
        )?;
        return Ok(if prefix.is_empty() {
            title
        } else {
            format!("{prefix}{title}")
        });
    }

    let log = capture(
        root,
        "git",
        &["log", "--reverse", "--format=%s", "origin/main..HEAD"],
    )?;
    log.lines().next().map(str::to_string).ok_or_else(|| {
        "xtask: this branch adds no commit to origin/main, so there is nothing to propose and no \
         title to derive. `--title` overrides."
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

/// Which stage `branch` carries: a leading digit first, the suffix second.
///
/// The order is the whole function. Reading the suffix first made every branch without one a
/// change with no issue, and under this flow the branch without a suffix is the ordinary case.
fn stage_from_branch(branch: &str) -> Stage {
    if !branch.starts_with(|character: char| character.is_ascii_digit()) {
        return Stage::Unnumbered;
    }
    if branch.ends_with(DECIDING) {
        return Stage::Deciding;
    }
    if branch.ends_with(BUILDING) {
        return Stage::Building;
    }
    Stage::Single
}

/// The issue number `branch` starts with, where it starts with one.
fn issue_from_branch(branch: &str) -> Option<u32> {
    let digits: String = branch.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// Settles which issue the body references: the branch's, the argument, or an error saying which
/// is missing or that the two disagree.
fn resolve_issue(branch: &str, stage: Stage, argument: Option<u32>) -> Result<u32, String> {
    match (issue_from_branch(branch), argument) {
        (Some(from_branch), None) => Ok(from_branch),
        (Some(from_branch), Some(given)) if from_branch == given => Ok(from_branch),
        (Some(from_branch), Some(given)) => Err(format!(
            "xtask: branch `{branch}` is change {from_branch}, but {given} was given"
        )),
        (None, Some(given)) => Ok(given),
        (None, None) => Err(format!(
            "xtask: a {} branch carries no issue number, so this needs one: \
             `cargo xtask pr body <issue>`",
            stage.name()
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
///
/// A body with no `## ` heading at all is reported as one unnamed gap rather than as nothing to
/// report. It used to return empty for that case, which is how a body of prose and a title passed.
fn empty_sections(body: &str) -> Vec<String> {
    let mut sections: Vec<(String, bool)> = Vec::new();
    let mut in_comment = false;
    let mut open_fence: Option<usize> = None;

    for line in body.lines() {
        let trimmed = line.trim_start();
        let indented_by = line.len().saturating_sub(trimmed.len());

        // A fence is a fence whatever sits inside it. Only a fence at the left margin opens one,
        // and only a bare fence closes it — the same rule CommonMark states and markdownlint
        // applies, because it is what decides where a picture in an example ends.
        if indented_by <= 3 && trimmed.starts_with("```") {
            let after = &trimmed[3..];
            match open_fence {
                Some(width) if width <= after.len() && after.trim().is_empty() => open_fence = None,
                Some(_) => {}
                None => open_fence = Some(after.len()),
            }
            fill_last(&mut sections);
            continue;
        }

        // Everything between the fences is text. A comment marker in it is two characters rather
        // than the start of a comment that never closes, and a `## ` in it belongs to an example
        // rather than to the body.
        if open_fence.is_some() {
            fill_last(&mut sections);
            continue;
        }

        let text = strip_comments(line, &mut in_comment);
        let text = text.trim();

        if let Some(heading) = text.strip_prefix("## ") {
            sections.push((heading.trim().to_string(), false));
        } else if !text.is_empty() && !is_keyword_line(text) {
            fill_last(&mut sections);
        }
    }

    if in_comment {
        // An unclosed comment makes everything after it invisible, so the sections found so far
        // cannot be reported honestly — a body with one would look filled. Report the cause.
        return vec!["the body has an HTML comment that is never closed".to_string()];
    }

    if sections.is_empty() {
        return vec!["the body has no `## ` section at all".to_string()];
    }

    sections
        .into_iter()
        .filter(|(_, filled)| !filled)
        .map(|(heading, _)| heading)
        .collect()
}

/// Marks the section being read as holding something.
fn fill_last(sections: &mut [(String, bool)]) {
    if let Some((_, filled)) = sections.last_mut() {
        *filled = true;
    }
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
    println!("Prepare and open the pull request this branch is for.");
    println!();
    println!("Usage: cargo xtask pr <verb> [args]");
    println!();
    println!("Verbs:");
    println!("  body [<issue>] [--refs]   Write target/pr-body.md from the one template");
    println!("  open [--title <text>] [--body-file <path>]");
    println!("                            Push the branch and open the pull request with it");
    println!("  help                      Show this message");
    println!();
    println!(
        "The stage comes from the branch: a leading number says it carries an issue, and then"
    );
    println!(
        "`-deciding`, `-building` or neither says which stage. A branch with no number carries"
    );
    println!("no issue, so `body` needs one as an argument.");
}

#[cfg(test)]
mod tests {
    use super::{
        Stage, empty_sections, is_keyword_line, issue_from_branch, lists_a_pull_request,
        resolve_issue, stage_from_branch,
    };

    #[test]
    fn a_branch_with_a_number_and_no_suffix_is_the_whole_change() {
        assert_eq!(stage_from_branch("163-light-spec-driven"), Stage::Single);
        assert_eq!(stage_from_branch("023-read-a-description"), Stage::Single);
    }

    #[test]
    fn each_two_stage_suffix_is_its_own_stage() {
        assert_eq!(
            stage_from_branch("023-read-a-description-deciding"),
            Stage::Deciding
        );
        assert_eq!(
            stage_from_branch("023-read-a-description-building"),
            Stage::Building
        );
    }

    /// The number is read before the suffix, so a branch with no suffix and a number in it is a
    /// change rather than a branch without an issue. Reading the suffix first sent the common case
    /// down the wrong arm.
    #[test]
    fn a_number_beats_the_absence_of_a_suffix() {
        assert_ne!(
            stage_from_branch("163-light-spec-driven"),
            Stage::Unnumbered
        );
    }

    #[test]
    fn a_branch_opening_with_no_digit_carries_no_issue() {
        assert_eq!(stage_from_branch("readme-typo"), Stage::Unnumbered);
        assert_eq!(stage_from_branch("main"), Stage::Unnumbered);
        assert_eq!(stage_from_branch("readme-typo-deciding"), Stage::Unnumbered);
    }

    #[test]
    fn only_the_deciding_stage_references_its_issue() {
        assert_eq!(Stage::Deciding.keyword(), "Refs");
        assert_eq!(Stage::Building.keyword(), "Closes");
        assert_eq!(Stage::Single.keyword(), "Closes");
        assert_eq!(Stage::Unnumbered.keyword(), "Closes");
    }

    #[test]
    fn a_one_stage_change_prefixes_its_title_with_nothing() {
        assert_eq!(Stage::Single.title_prefix(), Some(""));
        assert_eq!(Stage::Deciding.title_prefix(), Some("Decide: "));
        assert_eq!(Stage::Building.title_prefix(), Some("Build: "));
        assert_eq!(Stage::Unnumbered.title_prefix(), None);
    }

    #[test]
    fn issue_from_branch_reads_the_leading_digits_only() {
        assert_eq!(
            issue_from_branch("023-read-a-description-deciding"),
            Some(23)
        );
        assert_eq!(issue_from_branch("1234-something-building"), Some(1234));
        assert_eq!(issue_from_branch("163-light-spec-driven"), Some(163));
        assert_eq!(issue_from_branch("readme-typo"), None);
    }

    #[test]
    fn resolve_issue_prefers_the_branch_and_catches_a_disagreement() {
        assert_eq!(
            resolve_issue("023-foo-deciding", Stage::Deciding, None).unwrap(),
            23
        );
        assert_eq!(
            resolve_issue("023-foo-deciding", Stage::Deciding, Some(23)).unwrap(),
            23
        );

        let error = resolve_issue("023-foo-deciding", Stage::Deciding, Some(24)).unwrap_err();
        assert!(
            error.contains("23") && error.contains("24"),
            "error did not name both numbers: {error}"
        );

        let error = resolve_issue("readme-typo", Stage::Unnumbered, None).unwrap_err();
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

    /// A body with no `## ` heading at all has nothing that could have been filled in, and used to
    /// pass: the loop never pushed a section, the filter had nothing to drop, and `pr open` said
    /// yes to a body that was a title and a paragraph.
    #[test]
    fn a_body_with_no_section_at_all_is_reported() {
        let body = "# A title\n\nThree paragraphs of prose and no section.\n\nCloses #34\n";
        assert_eq!(
            empty_sections(body),
            vec!["the body has no `## ` section at all"]
        );
    }

    /// A `<!--` inside a fence has no closing `-->` in it, and the comment stripper used to carry
    /// that open for the rest of the document: every section after the fence was invisible and the
    /// check passed. A body carrying a generated picture is exactly this shape.
    #[test]
    fn an_unclosed_comment_inside_a_fence_does_not_swallow_the_rest() {
        let body = "## What changes\n\
                    \n\
                    ```markdown\n\
                    <!-- render:\n\
                    ```\n\
                    \n\
                    ## Why now\n\
                    \n\
                    None.\n";
        assert!(empty_sections(body).is_empty());
    }

    /// A `## ` inside a fence is part of an example, not a section of the body. Counting it would
    /// make a body with a tenth heading look like a body with one more section to fill, and the
    /// section before the fence would look filled by text that belongs to neither.
    #[test]
    fn a_heading_inside_a_fence_is_not_a_section() {
        let body = "## What changes\n\
                    \n\
                    ## Why now\n\
                    \n\
                    ```text\n\
                    ## not a section\n\
                    ```\n\
                    \n\
                    None.\n";
        assert_eq!(empty_sections(body), vec!["What changes"]);
    }

    /// A picture is content: the section it sits in was written, and `cargo xtask render` is what
    /// fills the fence.
    #[test]
    fn a_picture_in_a_fence_fills_its_section() {
        let body = "## Before / after\n\
                    \n\
                    ```text\n\
                    +---+\n\
                    |   |\n\
                    +---+\n\
                    ```\n\
                    \n\
                    ## Why now\n\
                    \n\
                    None.\n";
        assert!(empty_sections(body).is_empty());
    }

    /// A longer fence closes only on a fence at least as long, which is the rule that keeps a
    /// nested example from ending its own block.
    #[test]
    fn a_fence_is_closed_only_by_one_at_least_as_long() {
        let body = "## What changes\n\
                    \n\
                    ````markdown\n\
                    ```text\n\
                    +---+\n\
                    ```\n\
                    ````\n\
                    \n\
                    ## Why now\n\
                    \n\
                    None.\n";
        assert!(empty_sections(body).is_empty());
    }

    /// A comment marker in ordinary prose does open a comment, and one that is never closed swallows
    /// every section after it. That made a body with an unfilled section read as filled, so the
    /// unclosed comment is reported instead of the sections it hid.
    #[test]
    fn an_unclosed_comment_in_prose_is_reported_rather_than_swallowing_the_rest() {
        let body = "## What changes\n\
                    \n\
                    A `<!--` in prose is a comment marker.\n\
                    \n\
                    ## Why now\n\
                    \n\
                    None.\n";
        assert_eq!(
            empty_sections(body),
            vec!["the body has an HTML comment that is never closed"]
        );
    }

    #[test]
    fn the_keyword_line_is_not_content() {
        assert!(is_keyword_line("Refs #23"));
        assert!(is_keyword_line("Closes #1234"));
        assert!(!is_keyword_line("Closes #"));
        assert!(!is_keyword_line("Closes the loop on #23"));
    }

    #[test]
    fn an_empty_gh_answer_is_no_pull_request() {
        assert!(!lists_a_pull_request("[]"));
        assert!(!lists_a_pull_request(""));
        assert!(lists_a_pull_request("[{\"number\":170}]"));
    }
}
