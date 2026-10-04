//! `cargo xtask feature`: opening the branch a change is worked on, from the issue that asks for
//! it.
//!
//! A change is one GitHub issue crossing one or two branches. Which one is settled by its **rung**,
//! the answer to the question
//! [CONTRIBUTING.md](https://github.com/andresmoschini/monospace/blob/main/CONTRIBUTING.md#starting-work)
//! asks — *is there a sentence in the model that this change would make false?* — and a rung decides
//! the branch suffix, the label, and the body of the pull request that closes it:
//!
//! ```text
//!   fix    NNN-slug              labels: wish -> building
//!   slice  NNN-slug-building     labels: wish -> building
//!   decide NNN-slug-deciding     labels: wish -> deciding
//!          NNN-slug-building            -> building
//! ```
//!
//! `new` opens the first branch, `build` opens the second after checking against `origin/main` that
//! the first one merged, and `use` puts a clone on whichever of them exists. None of this is part of
//! the quality gate: `cargo xtask check` has no notion of `git` branches or GitHub labels, and this
//! module is not on its call graph.
//!
//! Every subprocess call lives behind a thin wrapper (`run_visible`, `capture`) so the logic that
//! decides *what* to run — slugging a title, naming a branch, reading a rung off a label set — stays
//! in plain functions that take values and return `Result`, and can be unit tested with no network
//! and no repository.
//!
//! # Design notes
//!
//! **The rung is an argument, not a lookup.** The obvious way to keep this tool small would be to
//! read `docs/model.md` and decide the rung itself. That is the one thing it must not do: whether a
//! sentence becomes false is a judgement about the domain, made by whoever read the sentence, and a
//! tool that cannot be wrong in that judgement can only get it wrong silently. So the rung arrives
//! as an argument, defaulted from the issue's labels because that is right often enough to be worth
//! a keystroke, and always printed with the reason — so a default taken on faith is visible as one.
//!
//! **There is no artifact directory, so the precondition changed what it reads.** The building stage
//! used to open `specs/NNN-slug/decisions.md` and refuse on any line still marked `_pending_`, which
//! was a real check: a sheet whose entries are unanswered merges exactly as easily as one that is
//! answered. The handoff that check protected has moved — it is now the merged deciding pull
//! request, and `CONTRIBUTING.md` says so — so the check asks `gh` whether that pull request merged
//! rather than reading a file that no longer exists. The check survives; only its subject changed,
//! which is the whole argument for having had one.
//!
//! **The slug is read from the issue every time, because deleting `specs/` removed the only place it
//! was recorded.** There is no directory to list and no `find_slug` to disagree with itself. The
//! consequence is stated rather than designed around: renaming an issue changes the branch this tool
//! hands out, so the branch is what pins a slug, and `use` prints the branch it chose for that
//! reason.
//!
//! **`use` tries suffixes in order instead of storing which rung opened the branch.** `building` is
//! one label for two rungs, so the label cannot name the suffix: a slice is `-building` and a fix is
//! nothing. Asking which branch exists is one `ls-remote` and it cannot disagree with the label,
//! which a stored rung eventually would.
//!
//! **The label vocabulary is unchanged.** `wish`, `deciding` and `building` already exist on the
//! repository with descriptions that say what they mean, and they are not this module's to rename.
//! A fix and a slice skip `deciding` and land on `building`, which is what those labels already mean
//! for a change nobody has to agree on first.

use std::path::Path;
use std::process::{Command, ExitCode};

use crate::process::{capture, ensure_gh_ready, run_visible, to_exit_code};

/// How much of the flow a change is on, which is what its branch suffix and label follow from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rung {
    /// The model says what to do and the code disagrees: one branch, no suffix, the default body.
    Fix,
    /// No sentence of the model becomes false, and what is wanted is still missing: one branch,
    /// `-building`.
    Slice,
    /// More than one answer is defensible, so there is a decision to agree before any code:
    /// `-deciding`, then `-building`. Never the default, always asked for.
    Decide,
}

impl Rung {
    /// The word this rung is named by, as `new` takes it and prints it.
    fn name(self) -> &'static str {
        match self {
            Rung::Fix => "fix",
            Rung::Slice => "slice",
            Rung::Decide => "decide",
        }
    }

    /// The suffix the first branch of this rung carries, hyphen included.
    ///
    /// It is not the label with a hyphen in front, because **fix** is where the two part company: a
    /// fix has no suffix at all and is still labelled `building`. A branch nobody can tell from a
    /// slice by its name is the point of the rung, so the empty suffix stays empty.
    fn suffix(self) -> &'static str {
        match self {
            Rung::Fix => "",
            Rung::Slice => "-building",
            Rung::Decide => "-deciding",
        }
    }

    /// The label this rung moves its issue to when `new` opens it.
    fn label(self) -> &'static str {
        match self {
            Rung::Decide => "deciding",
            Rung::Fix | Rung::Slice => "building",
        }
    }

    /// The answer to the question that puts a change on this rung, in the words the operator used.
    fn reason(self) -> &'static str {
        match self {
            Rung::Fix => "the model says what to do and the code disagrees with it",
            Rung::Slice => "no sentence of the model changes, and what is wanted is still missing",
            Rung::Decide => {
                "more than one answer is defensible — a sentence of the model becomes false, or \
                 something a caller observes changes shape"
            }
        }
    }

    /// What `new` prints once the branch exists: the rung, why it is that one, and how to change it.
    fn report(self, from_labels: bool) -> String {
        let chosen = format!("Rung: {} — {}", self.name(), self.reason());
        if from_labels {
            format!("{chosen}\nRead off the labels; override with --rung fix|slice|decide")
        } else {
            chosen
        }
    }

    /// The rung `raw` names.
    ///
    /// # Errors
    ///
    /// Names the three that exist, because the argument is a judgement and a wrong one is worth
    /// correcting rather than defaulting past.
    fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "fix" => Ok(Rung::Fix),
            "slice" => Ok(Rung::Slice),
            "decide" => Ok(Rung::Decide),
            other => Err(format!(
                "xtask: `{other}` is not a rung; it is fix, slice or decide"
            )),
        }
    }

    /// The rung an issue's labels suggest, used only as the default for `--rung`.
    ///
    /// **There is deliberately no label that defaults to `decide`.** It is the only rung where a
    /// wrong answer costs something — a proposal agreed on the wrong question — and every label set
    /// this can read is compatible with more than one defensible answer. Asking is the whole cost.
    ///
    /// # Errors
    ///
    /// An issue carrying neither `bug` nor a capability label is a question rather than a default:
    /// guessing here would put the rung in the branch name on evidence nobody read.
    fn from_labels(labels: &[String]) -> Result<Self, String> {
        let has = |name: &str| labels.iter().any(|label| label == name);

        if has("bug") {
            Ok(Rung::Fix)
        } else if has("capability") || has("wish") {
            Ok(Rung::Slice)
        } else {
            Err(format!(
                "xtask: the issue's labels suggest no rung ({}); pass --rung fix|slice|decide. \
                 `bug` is a fix and a capability or a wish is a slice; `decide` is never a default, \
                 because reading the model is what puts a change there.",
                if labels.is_empty() {
                    "it carries no labels at all".to_string()
                } else {
                    format!("it carries: {}", labels.join(", "))
                }
            ))
        }
    }
}

/// Runs the `feature` command: dispatches to `new`, `build` or `use` from the remaining arguments.
pub fn run(mut args: impl Iterator<Item = String>) -> ExitCode {
    let rest: Vec<String> = args.by_ref().collect();
    match rest.first().map(String::as_str) {
        Some("new") => to_exit_code(new_change(&crate::workspace_root(), &rest[1..])),
        Some("build") => to_exit_code(build_change(&crate::workspace_root(), &rest[1..])),
        Some("use") => to_exit_code(use_change(&crate::workspace_root(), &rest[1..])),
        None | Some("help" | "--help" | "-h") => {
            print_usage();
            ExitCode::SUCCESS
        }
        Some(unknown) => {
            eprintln!("xtask: unknown `feature` verb `{unknown}`\n");
            print_usage();
            ExitCode::FAILURE
        }
    }
}

/// The label every rung replaces when `new` opens a change: an issue nobody has worked yet.
///
/// It is the same for all three, which is why it is not a method on `Rung`. A model rung replaces
/// it with `deciding` and its second branch replaces *that* with `building`; a fix or a slice skips
/// `deciding` and lands on `building` from here.
const WISH_LABEL: &str = "wish";

/// Opens the first branch of `issue`'s change: fetches, reads the issue's title, derives its slug,
/// settles the rung, opens the branch, and moves the label to match.
fn new_change(root: &Path, args: &[String]) -> Result<(), String> {
    let issue = parse_issue(args)?;
    ensure_gh_ready(root)?;
    run_visible(root, "git", &["fetch", "origin", "--prune"])?;

    let labels = current_labels(root, issue)?;
    let (rung, from_labels) = match rung_argument(args)? {
        Some(raw) => (Rung::parse(&raw)?, false),
        None => (Rung::from_labels(&labels)?, true),
    };

    let slug = slugify(&issue_title(root, issue)?)?;
    let branch = format!("{}-{slug}{}", format_issue_number(issue), rung.suffix());

    ensure_branch(root, issue, &branch)?;
    move_label(root, issue, WISH_LABEL, rung.label())?;

    println!("{}", rung.report(from_labels));
    println!("Branch: {branch}");
    println!("Next: {}", next_step(rung));
    Ok(())
}

/// Opens the building branch of `issue`, after checking that the deciding pull request merged.
///
/// Only the **decide** rung has a second branch. For the other two this says so and names the branch
/// `new` already opened, rather than creating a third name for one change.
fn build_change(root: &Path, args: &[String]) -> Result<(), String> {
    let issue = parse_issue(args)?;
    ensure_gh_ready(root)?;
    run_visible(root, "git", &["fetch", "origin", "--prune"])?;

    let labels = current_labels(root, issue)?;
    if !labels.iter().any(|label| label == "deciding") {
        return Err(format!(
            "xtask: issue #{issue} is not on a deciding stage ({}). Only the model rung has one; a \
             fix or a slice is built on the branch `cargo xtask feature new {issue}` opened.",
            describe(&labels)
        ));
    }

    let slug = slugify(&issue_title(root, issue)?)?;
    let number = format_issue_number(issue);
    let deciding = format!("{number}-{slug}-deciding");

    let Some(merged) = merged_pull_request(root, &deciding)? else {
        return Err(not_merged_message(root, issue, &deciding));
    };

    let branch = format!("{number}-{slug}-building");
    ensure_branch(root, issue, &branch)?;
    move_label(root, issue, "deciding", "building")?;

    println!(
        "Rung: decide — some sentence of docs/model.md or docs/diagram-model.md becomes false"
    );
    println!("Decided by: #{merged}, merged");
    println!("Branch: {branch}");
    println!("Next: {}", next_step(Rung::Decide));
    Ok(())
}

/// Puts the working copy on the branch of `issue`'s change, with no side effects on labels or remote
/// branches.
fn use_change(root: &Path, args: &[String]) -> Result<(), String> {
    let issue = parse_issue(args)?;
    ensure_gh_ready(root)?;
    run_visible(root, "git", &["fetch", "origin", "--prune"])?;

    let labels = current_labels(root, issue)?;
    let slug = slugify(&issue_title(root, issue)?)?;
    let number = format_issue_number(issue);

    let Some(suffix) = find_suffix(root, issue, &labels, &number, &slug)? else {
        return Err(format!(
            "xtask: no branch for issue #{issue} exists yet; `cargo xtask feature new {issue}` \
             opens one."
        ));
    };

    let branch = format!("{number}-{slug}{suffix}");
    checkout_branch(root, &branch)?;

    println!("Branch: {branch}");
    println!("Labels: {}", describe(&labels));
    println!("Next: {}", next_step(Rung::Decide));
    Ok(())
}

/// The first branch suffix of `issue` whose branch exists, preferring local over remote.
///
/// Returns `None` when neither does, which is the answer `use` turns into the command that creates
/// one. The suffixes come from the labels rather than from a stored rung, because `building` is one
/// label for two rungs: see the module's design notes.
fn find_suffix(
    root: &Path,
    issue: u32,
    labels: &[String],
    number: &str,
    slug: &str,
) -> Result<Option<&'static str>, String> {
    let suffixes = candidate_suffixes(labels, issue)?;

    for suffix in suffixes {
        let branch = format!("{number}-{slug}{suffix}");
        if local_branch_exists(root, &branch)? || remote_branch_exists(root, &branch)? {
            return Ok(Some(suffix));
        }
    }

    Ok(None)
}

/// The branch suffixes an issue's labels allow, in the order to try them.
///
/// # Errors
///
/// An issue still carrying `wish` has had no branch opened for it, and the error names the command
/// rather than reporting an empty candidate list. Any other state names the labels it does carry,
/// which is what the operator has to look at to get out of it.
fn candidate_suffixes(labels: &[String], issue: u32) -> Result<&'static [&'static str], String> {
    let has = |name: &str| labels.iter().any(|label| label == name);

    if has("building") {
        // A slice is `-building` and a fix is nothing, and both carry this label.
        Ok(&["-building", ""])
    } else if has("deciding") {
        Ok(&["-deciding"])
    } else if has("wish") {
        Err(format!(
            "xtask: issue #{issue} is still a wish, so no branch has been opened for it; \
             `cargo xtask feature new {issue}` opens one."
        ))
    } else {
        Err(format!(
            "xtask: issue #{issue} carries neither the deciding nor the building label ({}).",
            describe(labels)
        ))
    }
}

/// What to do next on `rung`, in one line, naming the thing rather than a command to type.
fn next_step(rung: Rung) -> &'static str {
    match rung {
        Rung::Fix => "the fix: a failing test, then the code, then `cargo xtask pr body`",
        Rung::Slice => "the code and its tests, then `cargo xtask pr body`",
        Rung::Decide => "the proposal in `deciding.md` — decide first, code after it merges",
    }
}

/// The message `build` refuses with when the deciding pull request has not merged, distinguishing
/// the three states it can be in so the fix is never ambiguous.
fn not_merged_message(root: &Path, issue: u32, deciding: &str) -> String {
    match open_pull_request(root, deciding) {
        Ok(Some(number)) => format!(
            "xtask: the deciding pull request #{number} for issue #{issue} is open, not merged. \
             The building branch runs against a merged proposal; merge #{number} first."
        ),
        Ok(None) => format!(
            "xtask: no deciding pull request was ever opened for issue #{issue} (`{deciding}`). \
             Build it on the deciding branch: `cargo xtask pr body` and `cargo xtask pr open`."
        ),
        Err(problem) => problem,
    }
}

/// Ensures `branch` exists, checking it out. Uses the remote branch if it already exists there —
/// which is what happens when a pull request was opened from it and the local branch was deleted
/// with it; otherwise creates it linked to `issue`.
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

/// The title of `issue`, which is where the branch's slug comes from.
fn issue_title(root: &Path, issue: u32) -> Result<String, String> {
    let issue_str = issue.to_string();
    capture(
        root,
        "gh",
        &[
            "issue", "view", &issue_str, "--json", "title", "--jq", ".title",
        ],
    )
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

/// The number of the merged pull request whose head is `branch`, if there is one.
fn merged_pull_request(root: &Path, branch: &str) -> Result<Option<u32>, String> {
    pull_request_number(root, branch, "merged")
}

/// The number of the open pull request whose head is `branch`, if there is one.
fn open_pull_request(root: &Path, branch: &str) -> Result<Option<u32>, String> {
    pull_request_number(root, branch, "open")
}

/// The number of the pull request from `branch` in `state`, as the lowest number if a branch was
/// proposed more than once.
fn pull_request_number(root: &Path, branch: &str, state: &str) -> Result<Option<u32>, String> {
    let output = capture(
        root,
        "gh",
        &[
            "pr",
            "list",
            "--state",
            state,
            "--head",
            branch,
            "--json",
            "number",
            "--jq",
            ".[].number",
        ],
    )?;
    Ok(output
        .lines()
        .filter_map(|line| line.trim().parse::<u32>().ok())
        .min())
}

/// Moves `issue`'s labels by removing `remove` and adding `add`, calling `gh issue edit` only when
/// at least one of those is actually needed.
fn move_label(root: &Path, issue: u32, remove: &str, add: &str) -> Result<(), String> {
    let labels = current_labels(root, issue)?;
    let issue_str = issue.to_string();
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
        println!("Labels: already {} ({})", add, describe(&labels));
        Ok(())
    }
}

/// Renders a label set for a message, saying so plainly when there is nothing to list.
fn describe(labels: &[String]) -> String {
    if labels.is_empty() {
        "no labels at all".to_string()
    } else {
        labels.join(", ")
    }
}

/// Parses the issue number every verb takes, which comes first and must be a positive integer.
///
/// # Errors
///
/// Names the argument when it is missing, and says it comes first — which is what the usage prints
/// and what lets the parser ignore the flags after it rather than mistaking `--rung`'s value for
/// the number.
fn parse_issue(args: &[String]) -> Result<u32, String> {
    let raw = args.first().ok_or_else(|| {
        "xtask: this verb needs an issue number first, as in `cargo xtask feature new 90`"
            .to_string()
    })?;
    raw.parse::<u32>()
        .map_err(|_| format!("xtask: `{raw}` is not a valid issue number"))
}

/// Parses `--rung <value>` out of `args`.
///
/// # Errors
///
/// Names the flag when it carries no value, which is otherwise an empty string reaching `parse` and
/// being reported as an unknown rung.
fn rung_argument(args: &[String]) -> Result<Option<String>, String> {
    match args.iter().position(|argument| argument == "--rung") {
        Some(index) => args
            .get(index + 1)
            .cloned()
            .map(Some)
            .ok_or_else(|| "xtask: `--rung` needs a value: fix, slice or decide".to_string()),
        None => Ok(None),
    }
}

/// Zero-pads `issue` to at least three digits; an issue number with more digits keeps all of them.
fn format_issue_number(issue: u32) -> String {
    format!("{issue:03}")
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

/// Prints usage for `cargo xtask feature`.
fn print_usage() {
    println!("Open the branch a change is worked on, from the issue that asks for it.");
    println!();
    println!("Usage: cargo xtask feature <verb> <issue> [args]");
    println!();
    println!("Verbs:");
    println!("  new <issue> [--rung <r>]   Open the first branch; --rung is fix, slice or decide");
    println!("  build <issue>              Open the building branch, once the deciding one merged");
    println!("  use <issue>                Check out whichever branch of the issue exists");
    println!("  help                       Show this message");
    println!();
    println!(
        "The rung comes from the issue's labels unless --rung says otherwise: `bug` is a fix,"
    );
    println!(
        "a capability or a wish is a slice, and a decide rung has to be asked for. A fix opens"
    );
    println!("NNN-slug, a slice NNN-slug-building, and a decide rung NNN-slug-deciding first.");
    println!();
    println!("Which rung, and why:");
    let rungs = [Rung::Fix, Rung::Slice, Rung::Decide];
    for rung in rungs {
        println!("  {:6} {}", rung.name(), rung.reason());
    }
    println!();
    println!("Nothing here is part of `cargo xtask check`: it knows nothing of git or GitHub.");
}

#[cfg(test)]
mod tests {
    use super::{
        Rung, candidate_suffixes, describe, format_issue_number, next_step, parse_issue, slugify,
    };

    fn labels(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_string()).collect()
    }

    #[test]
    fn slugify_plain_title() {
        assert_eq!(
            slugify("Draw shapes instead of individual cells").unwrap(),
            "draw-shapes-instead-of-individual-cells"
        );
    }

    #[test]
    fn slugify_folds_accents_and_collapses_runs() {
        assert_eq!(
            slugify("Render a naïve café corner — twice").unwrap(),
            "render-a-naive-cafe-corner-twice"
        );
    }

    #[test]
    fn slugify_truncates_to_forty_without_a_trailing_hyphen() {
        let slug =
            slugify("Taking a shape out leaves the figures that hung from it where they were")
                .unwrap();
        assert_eq!(slug.chars().count(), 40);
        assert!(!slug.ends_with('-'), "{slug}");
    }

    #[test]
    fn slugify_refuses_a_title_with_nothing_to_keep() {
        let error = slugify("¿¡...!?").unwrap_err();
        assert!(error.contains("slug"), "{error}");
    }

    #[test]
    fn issue_numbers_are_three_digits_and_keep_their_length() {
        assert_eq!(format_issue_number(23), "023");
        assert_eq!(format_issue_number(104), "104");
        assert_eq!(format_issue_number(1234), "1234");
    }

    #[test]
    fn a_bug_is_a_fix_and_a_capability_is_a_slice() {
        assert_eq!(Rung::from_labels(&labels(&["bug"])).unwrap(), Rung::Fix);
        assert_eq!(
            Rung::from_labels(&labels(&["capability", "wish"])).unwrap(),
            Rung::Slice
        );
    }

    #[test]
    fn labels_that_suggest_no_rung_ask_for_one() {
        let error = Rung::from_labels(&labels(&["tooling"])).unwrap_err();
        assert!(error.contains("--rung"), "{error}");
        assert!(error.contains("tooling"), "{error}");
    }

    #[test]
    fn no_labels_at_all_says_so_rather_than_listing_nothing() {
        let error = Rung::from_labels(&[]).unwrap_err();
        assert!(error.contains("no labels at all"), "{error}");
    }

    #[test]
    fn each_rung_names_its_own_suffix_and_label() {
        assert_eq!(Rung::Fix.suffix(), "");
        assert_eq!(Rung::Slice.suffix(), "-building");
        assert_eq!(Rung::Decide.suffix(), "-deciding");
        assert_eq!(Rung::Fix.label(), "building");
        assert_eq!(Rung::Slice.label(), "building");
        assert_eq!(Rung::Decide.label(), "deciding");
    }

    #[test]
    fn building_is_one_label_for_two_rungs_so_both_suffixes_are_tried() {
        assert_eq!(
            candidate_suffixes(&labels(&["building"]), 90).unwrap(),
            &["-building", ""]
        );
    }

    #[test]
    fn a_deciding_issue_offers_only_the_deciding_suffix() {
        assert_eq!(
            candidate_suffixes(&labels(&["deciding"]), 90).unwrap(),
            &["-deciding"]
        );
    }

    #[test]
    fn a_wish_names_the_command_that_would_open_a_branch() {
        let error = candidate_suffixes(&labels(&["wish"]), 90).unwrap_err();
        assert!(error.contains("feature new 90"), "{error}");
    }

    #[test]
    fn an_unlabelled_issue_names_the_labels_it_does_carry() {
        let error = candidate_suffixes(&labels(&["documentation"]), 90).unwrap_err();
        assert!(error.contains("documentation"), "{error}");
    }

    #[test]
    fn a_rung_defaulted_from_the_labels_says_it_was() {
        let defaulted = Rung::Slice.report(true);
        assert!(defaulted.contains("--rung"), "{defaulted}");
        assert!(defaulted.contains("Rung: slice"), "{defaulted}");
    }

    #[test]
    fn no_label_set_ever_defaults_to_deciding() {
        // The one rung where a wrong answer costs something is the one rung that has to be asked
        // for, so every label combination either lands on the other two or refuses.
        let combinations: [&[&str]; 6] = [
            &[],
            &["bug"],
            &["capability"],
            &["wish"],
            &["tooling", "documentation"],
            &["bug", "capability", "wish", "tooling"],
        ];
        for names in combinations {
            if let Ok(rung) = Rung::from_labels(&labels(names)) {
                assert_ne!(rung, Rung::Decide, "{names:?}");
            }
        }
    }

    #[test]
    fn a_rung_named_on_the_command_line_does_not_mention_the_default() {
        let stated = Rung::Decide.report(false);
        assert!(!stated.contains("--rung"), "{stated}");
        assert!(stated.starts_with("Rung: decide"), "{stated}");
    }

    #[test]
    fn a_wrong_rung_lists_the_three_that_exist() {
        let error = Rung::parse("feature").unwrap_err();
        assert!(error.contains("fix, slice or decide"), "{error}");
    }

    #[test]
    fn the_issue_number_comes_first_and_the_flags_after_it_are_not_mistaken_for_it() {
        assert_eq!(parse_issue(&["90".to_string()]).unwrap(), 90);
        assert_eq!(
            parse_issue(&["90".to_string(), "--rung".to_string(), "decide".to_string()]).unwrap(),
            90
        );
        assert!(parse_issue(&[]).is_err());
        assert!(parse_issue(&["--rung".to_string()]).is_err());
    }

    #[test]
    fn a_non_numeric_issue_is_reported_as_one() {
        let error = parse_issue(&["ninety".to_string()]).unwrap_err();
        assert!(error.contains("not a valid issue number"), "{error}");
    }

    #[test]
    fn an_empty_label_set_is_described_rather_than_joined() {
        assert_eq!(describe(&[]), "no labels at all");
        assert_eq!(
            describe(&labels(&["wish", "capability"])),
            "wish, capability"
        );
    }

    #[test]
    fn only_the_decide_rung_is_told_to_decide_first() {
        assert!(next_step(Rung::Decide).contains("decide first"));
        assert!(next_step(Rung::Fix).contains("failing test"));
        assert!(next_step(Rung::Slice).contains("tests"));
    }
}
