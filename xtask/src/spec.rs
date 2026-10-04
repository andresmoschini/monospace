//! `cargo xtask spec`: open a change's branch, and check the specs the gate reads.
//!
//! A change is one GitHub issue, and where it stands is in its branch's name rather than in a label:
//! `NNN-slug` carries a change of one stage, and `NNN-slug-deciding` with `NNN-slug-building` carry
//! a change of two. `new` opens the deciding branch and links it to the issue, `use` puts a clone on
//! whichever of the two branches the issue already has, and `check` is the gate's `specs` step.
//!
//! Only `check` is on the quality gate's call graph. `cargo xtask check` has no notion of `git`
//! branches or GitHub pull requests, so the two verbs are not part of what "green" means.
//!
//! # Design notes
//!
//! **The precondition for `building` reads the decision table rather than only its name.** Every
//! other precondition here asks "does this file exist in `origin/main`", because a merged file is
//! the handoff. The handoff is an *agreed* spec, and a sheet whose rows still read `_pending_`
//! merges exactly as easily as one that is answered. So the check opens the file and refuses on any
//! line carrying that marker, and the marker is one string rather than a rule per field: the
//! frontmatter's `decided` and every row's `Answer` both use it, and a spec nobody wrote at all is
//! caught by the file being absent instead.
//!
//! **The slug is read back from the spec's name, never re-derived from the issue's title.** It is
//! derived once, when the deciding branch is opened, and every later invocation recovers it from
//! `specs/NNN-*.md` in `origin/main`. Re-deriving it per invocation looks equivalent and is not:
//! renaming an issue then renames the branch `use` looks for, and the change becomes unreachable
//! by number.
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

use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

use crate::process::{capture, capture_untrimmed, ensure_gh_ready, run_visible};

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

/// The marker a decision row carries until it is answered. Three places grep for this string — the
/// gate, `verify_deciding_merged` below, and `pr`'s refusal to open a `-building` pull request — so
/// changing it is changing the flow, not editing a document.
const PENDING: &str = "_pending_";

pub(crate) const DECIDING: &str = "deciding";
/// The suffix the building branch carries.
pub(crate) const BUILDING: &str = "building";

/// One of the two stages a two-stage change crosses, in order.
///
/// A change of one stage has no suffix at all, which is what makes `NNN-slug` distinguishable from
/// a branch that never went through this command: it opens neither stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    /// The spec is being written; branch `NNN-slug-deciding`.
    Deciding,
    /// The spec is agreed and merged, and the code follows it; branch `NNN-slug-building`.
    Building,
}

impl Stage {
    /// The word this stage is named by, which is also its branch suffix.
    fn suffix(self) -> &'static str {
        match self {
            Stage::Deciding => DECIDING,
            Stage::Building => BUILDING,
        }
    }

    /// Both stages, in the order a change crosses them.
    const ALL: [Stage; 2] = [Stage::Deciding, Stage::Building];

    /// What to do once this stage's branch is checked out.
    ///
    /// This is printed output rather than documentation, so it says what to do under this flow
    /// rather than naming commands that no longer exist.
    fn next_step(self) -> &'static str {
        match self {
            Stage::Deciding => concat!(
                "write `specs/NNN-slug.md` on this branch, filling `## The decision` until no row ",
                "reads `_pending_`, then open the pull request with `cargo xtask pr body`"
            ),
            Stage::Building => concat!(
                "build against the merged spec, complete `## What proves it` with the tests that ",
                "hold each rule, then open the pull request with `cargo xtask pr body`"
            ),
        }
    }
}

/// Runs the `spec` command: dispatches to `new`, `use` or `check` from the remaining arguments.
pub fn run(mut args: impl Iterator<Item = String>) -> ExitCode {
    match args.next().as_deref() {
        Some("new") => run_new(args),
        Some("use") => run_use(args),
        Some("check") => to_exit_code(check(&crate::workspace_root()).map(|_| ())),
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
        Ok(issue) => to_exit_code(new_change(&crate::workspace_root(), issue)),
        Err(message) => fail(&message),
    }
}

/// Parses `spec use`'s argument and runs it.
fn run_use(mut args: impl Iterator<Item = String>) -> ExitCode {
    match parse_issue(args.next()) {
        Ok(issue) => to_exit_code(use_change(&crate::workspace_root(), issue)),
        Err(message) => fail(&message),
    }
}

/// Parses an issue number argument, which is required and must be a positive integer.
fn parse_issue(raw: Option<String>) -> Result<u32, String> {
    let raw = raw.ok_or_else(|| "xtask: spec requires an issue number".to_string())?;
    raw.parse::<u32>()
        .map_err(|_| format!("xtask: `{raw}` is not a valid issue number"))
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

/// Opens the deciding stage for `issue`: fetches, reads the issue's title, derives its slug, links
/// the branch to the issue, and checks it out.
///
/// It writes no spec file. The spec is the deciding pull request's own output, and a template
/// emitted here would be a file nothing had agreed to yet — which is the same thing as a `draft`
/// that nobody is writing.
fn new_change(root: &Path, issue: u32) -> Result<(), String> {
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
    let stage = Stage::Deciding;
    let branch = branch_name(&issue_number, &slug, stage);

    ensure_branch(root, issue, &branch)?;

    println!("Spec: {}", spec_path(&issue_number, &slug));
    println!("Branch: {branch}");
    println!("Stage: {}", stage.suffix());
    println!("Next: {}", stage.next_step());
    Ok(())
}

/// Puts the working copy on whichever stage of `issue` has a branch, local before remote, deciding
/// before building.
///
/// A one-stage change has no branch from this command and is checked out by hand like any other
/// branch; `use` only knows the two names it is the one to have written.
fn use_change(root: &Path, issue: u32) -> Result<(), String> {
    ensure_gh_ready(root)?;
    run_visible(root, "git", &["fetch", "origin", "--prune"])?;

    let issue_number = format_issue_number(issue);
    let slug = find_slug(&list_specs(root)?, &issue_number)?;

    for stage in Stage::ALL {
        let branch = branch_name(&issue_number, &slug, stage);
        if local_branch_exists(root, &branch)? {
            return checkout_branch(root, &branch)
                .map(|()| announce(&issue_number, &slug, stage, &branch));
        }
        if remote_branch_exists(root, &branch)? {
            return checkout_branch(root, &branch)
                .map(|()| announce(&issue_number, &slug, stage, &branch));
        }
    }

    let tried = Stage::ALL
        .iter()
        .map(|stage| branch_name(&issue_number, &slug, *stage))
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "xtask: no branch for issue {issue_number} was found locally or on origin: {tried}. \
         `cargo xtask spec new {issue_number}` opens the deciding stage."
    ))
}

/// Prints what `use` settled on.
fn announce(issue_number: &str, slug: &str, stage: Stage, branch: &str) {
    println!("Spec: {}", spec_path(issue_number, slug));
    println!("Branch: {branch}");
    println!("Stage: {}", stage.suffix());
    println!("Next: {}", stage.next_step());
}

/// The spec file for `issue_number` and `slug`.
pub(crate) fn spec_path(issue_number: &str, slug: &str) -> String {
    format!("specs/{issue_number}-{slug}.md")
}

/// Lists the entries of `specs/` in `origin/main`, one per line, as
/// `git ls-tree --name-only origin/main specs/` reports them.
pub(crate) fn list_specs(root: &Path) -> Result<Vec<String>, String> {
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

/// Finds the single entry in `entries` whose name is `specs/{issue_number}-*.md`, and returns the
/// slug between the number and the extension.
///
/// # Errors
///
/// Names what was found when there is no match or more than one.
pub(crate) fn find_slug(entries: &[String], issue_number: &str) -> Result<String, String> {
    let prefix = format!("specs/{issue_number}-");
    let matches: Vec<&String> = entries
        .iter()
        .filter(|entry| entry.starts_with(&prefix))
        .collect();

    match matches.as_slice() {
        [single] => Ok(single[prefix.len()..]
            .strip_suffix(".md")
            .unwrap_or(&single[prefix.len()..])
            .to_string()),
        [] => Err(format!(
            "xtask: no spec matching `specs/{issue_number}-*.md` was found in origin/main"
        )),
        multiple => {
            let found = multiple
                .iter()
                .map(|entry| entry.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!(
                "xtask: expected exactly one spec matching `specs/{issue_number}-*.md` in \
                 origin/main, found {}: {found}",
                multiple.len()
            ))
        }
    }
}

/// Checks, against `origin/main`, that the deciding stage has merged: the spec is there, and no row
/// of its decision table is still unanswered.
///
/// # Errors
///
/// Names the missing file and says explicitly that the deciding pull request is not merged, or
/// quotes the lines on which the spec is still pending.
pub(crate) fn verify_deciding_merged(
    root: &Path,
    issue_number: &str,
    slug: &str,
) -> Result<(), String> {
    let path = spec_path(issue_number, slug);
    if !file_exists_in_origin_main(root, &path)? {
        return Err(format!(
            "xtask: the deciding pull request is not merged: `{path}` does not exist in origin/main"
        ));
    }

    let object = format!("origin/main:{path}");
    let spec = capture_untrimmed(root, "git", &["show", &object])?;
    let pending = pending_lines(&spec);

    if pending.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "xtask: `{path}` is not answered: {}. The building stage runs against an agreed spec, \
             so answer it on the deciding branch and merge that first.",
            describe_pending(&pending)
        ))
    }
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
/// whatever else is left. Three is enough to recognize which rows they are; the file itself is
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

/// Ensures `branch` exists, checking it out. Uses the remote branch if `gh issue develop` (or an
/// earlier run of this command) already created it; otherwise creates it linked to `issue`.
///
/// This is the only thing that ties a branch to an issue, and `gh issue develop` is what does it —
/// the branch name alone is a name.
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

/// Zero-pads `issue` to at least three digits; an issue number with more digits keeps all of them.
pub(crate) fn format_issue_number(issue: u32) -> String {
    format!("{issue:03}")
}

/// The branch name for `stage` of the change identified by `issue_number` and `slug`.
fn branch_name(issue_number: &str, slug: &str, stage: Stage) -> String {
    format!("{issue_number}-{slug}-{}", stage.suffix())
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

/// The file in `specs/` that is not a spec: the format's own documentation.
///
/// It is named rather than recognized by a shape, because a shape test would quietly stop
/// excluding it the day somebody gave it frontmatter, and because the one rule this step has that
/// is not about a spec's content is that this path is not one.
const NOT_A_SPEC: &str = "specs/README.md";

/// The gate's `specs` step: reports every way every spec fails the format, and returns whether
/// there were none.
///
/// # Errors
///
/// One message naming every problem in every file, so one run says everything rather than one
/// problem per run.
pub fn check(root: &Path) -> Result<bool, String> {
    let listed = capture(root, "git", &["ls-files", ":(glob)specs/*.md"])?;
    let paths: Vec<&str> = listed
        .lines()
        .map(str::trim)
        .filter(|path| !path.is_empty() && *path != NOT_A_SPEC)
        .collect();

    if paths.is_empty() {
        return Ok(false);
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
        return Ok(true);
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
             no unanswered row",
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

/// Prints usage for `cargo xtask spec`.
fn print_usage() {
    println!("Open a change's branch, and check the specs the gate reads.");
    println!();
    println!("Usage: cargo xtask spec <verb> [args]");
    println!();
    println!("Verbs:");
    println!("  new <issue>   Open the deciding stage for an issue, linked to it");
    println!("  use <issue>   Check out the branch of an issue's open stage");
    println!("  check         Report every spec that fails the format (the gate's step)");
    println!("  help          Show this message");
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

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
    fn a_spec_is_named_after_its_issue_and_its_slug() {
        assert_eq!(
            spec_path("163", "light-spec-driven"),
            "specs/163-light-spec-driven.md"
        );
    }

    #[test]
    fn find_slug_reads_the_name_back_and_leaves_the_extension_off() {
        let entries = vec![
            "specs/006-give-a-glyph-a-type.md".to_string(),
            "specs/163-light-spec-driven.md".to_string(),
        ];
        assert_eq!(find_slug(&entries, "163").unwrap(), "light-spec-driven");
    }

    #[test]
    fn find_slug_errors_on_no_match() {
        let entries = vec!["specs/006-give-a-glyph-a-type.md".to_string()];
        let error = find_slug(&entries, "999").unwrap_err();
        assert!(
            error.contains("999"),
            "error did not name the missing issue: {error}"
        );
    }

    #[test]
    fn find_slug_errors_on_two_matches() {
        let entries = vec![
            "specs/163-light.md".to_string(),
            "specs/163-light-spec-driven.md".to_string(),
        ];
        let error = find_slug(&entries, "163").unwrap_err();
        assert!(
            error.contains("specs/163-light.md"),
            "error did not name what was found: {error}"
        );
        assert!(
            error.contains("specs/163-light-spec-driven.md"),
            "error did not name what was found: {error}"
        );
    }

    /// A spec shaped like the one `specs/README.md` describes, with `agreed` deciding whether the
    /// decision table is answered and the spec therefore names the pull request that agreed it.
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
             | # | Question | Answer | Why not the alternative | Answered by |\n\
             | --- | --- | --- | --- | --- |\n\
             | D1 | Which? | {answer} | The other one. | maintainer |\n\
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
            .map(|number| (number, "| D1 | Which? | _pending_ |".to_string()))
            .collect();
        let described = describe_pending(&pending);
        assert!(
            described.starts_with("line 1 reads `| D1 | Which? | _pending_ |`"),
            "the first pending line was not quoted: {described}"
        );
        assert!(
            described.ends_with("and 2 more"),
            "the rest were not counted: {described}"
        );
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
    fn a_repository_with_no_specs_directory_is_not_a_failure() {
        let scratch = Scratch::new();
        scratch.write("README.md", "# Nothing here\n");

        assert_eq!(check(&scratch.path), Ok(false));
    }

    #[test]
    fn an_agreed_spec_in_the_right_shape_passes() {
        let scratch = Scratch::new();
        scratch.write("specs/163-light.md", &spec(true));

        assert_eq!(check(&scratch.path), Ok(true));
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
    fn an_agreed_spec_with_a_pending_row_is_reported() {
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

        assert_eq!(check(&scratch.path), Ok(true));
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

        assert_eq!(check(&scratch.path), Ok(true));

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

        assert_eq!(check(&scratch.path), Ok(true));
    }

    /// `specs/README.md` documents the format and is not a spec. It sits in the same directory because
    /// that is where a reader looks for it, so the step names it rather than looking for a shape it
    /// would stop recognizing the day somebody gave it frontmatter.
    #[test]
    fn the_formats_own_readme_is_not_checked_as_a_spec() {
        let scratch = Scratch::new();
        scratch.write(NOT_A_SPEC, "# Specifications\n\nNo frontmatter here.\n");

        assert_eq!(check(&scratch.path), Ok(false));

        scratch.write("specs/163-light.md", &spec(true));
        assert_eq!(check(&scratch.path), Ok(true));
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
