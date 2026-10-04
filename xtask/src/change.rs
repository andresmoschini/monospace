//! `cargo xtask change`: the branch a change stands on.
//!
//! A change is one GitHub issue, and where it stands is in its branch's name rather than in a label:
//! `NNN-slug` carries a change of one stage, and `NNN-slug-deciding` with `NNN-slug-building` carry
//! a change of two. Those three names are the flow's whole vocabulary for where a change is, and each
//! command here opens or finds one of them.
//!
//! `open` cuts the branch a change's shape calls for and leaves the clone on it, `use` puts a clone
//! on a branch that already exists, and `status` says where a change stands without cutting
//! anything. `open` and `use` print the same block, because the next step is what an operator or an
//! agent needs from both, and which step that is follows from the stage rather than from the verb.
//!
//! # Design notes
//!
//! **The stage argument is the branch suffix, and leaving it out is an answer.** `change open 170
//! building` opens `170-<slug>-building`, and `change open 170` opens `170-<slug>` — the change of
//! one stage, which the flow writes as the two-stage form with the first stage collapsed. There is
//! deliberately no word meaning "no decision to take": its absence is what says that, which is the
//! rule the branch names already state and the reason the argument can have a default at all.
//!
//! **`use` without a stage reports an ambiguity rather than resolving it.** It has to choose between
//! a change of two stages' deciding and building branches, and after the deciding merge the deciding
//! branch is still there — so the moment somebody most wants the building one is the moment a fixed
//! order is most likely to be wrong. A command that cannot know answers by asking, and here asking
//! costs one word.
//!
//! **The branches are found by asking git, not by deriving names.** `use` and `status` read
//! `refs/heads/NNN-*` and `refs/remotes/origin/NNN-*` rather than assembling a name from the issue
//! number and a slug, which is what makes them work for a change of one stage: that change has no
//! spec in `origin/main` to read a slug back from, and the branch is the only place its name is
//! written down.
//!
//! **Both sides of "does this branch exist" are answered from the tracking refs.** Every verb here
//! starts by fetching, so they are current by the time anything asks, and both answers come back the
//! same shape: a name per line. Asking the remote as well would be a second code path, and a second
//! failure mode, for a question this module has already answered.
//!
//! **A branch is tied to its issue by `gh issue develop`, for all three names.** It is the only thing
//! that ties one, and a branch name alone is a name: without the link the issue's page cannot list
//! the branch, and which issue it carries is a thing somebody has to remember.
//!
//! **`open` refuses to cut a building branch before the deciding stage merged.** It is the check
//! `pr open` already made, called one step earlier, and the reader of it is the same function: a
//! building branch cut against a spec that has not merged is a branch carrying the wrong promise, and
//! finding out at the pull request costs a push to undo.
//!
//! **The number is read before the suffix.** A branch either opens with digits or it does not, and
//! that question comes first: under this flow the common case is a change of one stage on a bare
//! `NNN-slug`, and reading the suffix first sends every one of them down the arm for branches that
//! carry no issue at all.
//!
//! **`gh`'s answers are read as text.** This crate parses no JSON. `title_of_issue` asks for one
//! title and `pull_requests` for one list, and the second is printed as it comes — `number state` a
//! line per pull request, nothing at all where there are none — because it is a line of a person's
//! `status` output and a value first would buy nothing.

use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

use crate::process::{capture, capture_untrimmed, ensure_gh_ready, run_visible};
use crate::spec;

/// The suffix the deciding branch carries.
pub(crate) const DECIDING: &str = "deciding";
/// The suffix the building branch carries.
pub(crate) const BUILDING: &str = "building";

/// The three branches a change can stand on.
///
/// A branch with no issue number is not a stage of anything, so it is `None` rather than a fourth
/// variant: `pr` is the only thing that has to reason about one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
    /// The spec is being written; branch `NNN-slug-deciding`.
    Deciding,
    /// The spec is agreed and merged, and the code follows it; branch `NNN-slug-building`.
    Building,
    /// The whole change in one pull request; branch `NNN-slug`, with no suffix.
    Single,
}

impl Stage {
    /// The word this stage is named by, which is also its branch suffix.
    ///
    /// `None` for the change of one stage: its branch has no suffix, and that absence is what says
    /// there is no decision to take.
    fn suffix(self) -> Option<&'static str> {
        match self {
            Stage::Deciding => Some(DECIDING),
            Stage::Building => Some(BUILDING),
            Stage::Single => None,
        }
    }

    /// What `branch` says about the change it carries, or `None` where it carries no issue.
    ///
    /// The number comes first on purpose; see the design notes.
    pub(crate) fn from_branch(branch: &str) -> Option<Stage> {
        if !branch.starts_with(|character: char| character.is_ascii_digit()) {
            return None;
        }
        if branch.ends_with(DECIDING) {
            return Some(Stage::Deciding);
        }
        if branch.ends_with(BUILDING) {
            return Some(Stage::Building);
        }
        Some(Stage::Single)
    }

    /// How this stage reads in a message to the operator.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Stage::Deciding => DECIDING,
            Stage::Building => BUILDING,
            Stage::Single => "single",
        }
    }

    /// Whether a change at this stage has a spec file at all.
    ///
    /// The change of one stage has none: there was no decision to record, so there is nothing for the
    /// deciding pull request to have produced, and printing a path that will never exist would be
    /// worse than printing no line.
    fn has_spec(self) -> bool {
        self.suffix().is_some()
    }

    /// What to do once this stage's branch is checked out.
    ///
    /// This is printed output rather than documentation, so it says what to do under this flow rather
    /// than naming commands that no longer exist.
    fn next_step(self) -> &'static str {
        match self {
            Stage::Deciding => {
                "write `specs/NNN-slug.md` on this branch, filling `## The decision` until no \
                 answer reads `_pending_`, then open the pull request with `cargo xtask pr body`"
            }
            Stage::Building => concat!(
                "build against the merged spec, complete `## What proves it` with the tests that ",
                "hold each rule, then open the pull request with `cargo xtask pr body`"
            ),
            Stage::Single => concat!(
                "build the change on this branch, then open the pull request with `cargo xtask pr ",
                "body`"
            ),
        }
    }
}

/// Runs the `change` command: dispatches to `open`, `use` or `status` from the remaining arguments.
pub fn run(mut args: impl Iterator<Item = String>) -> ExitCode {
    let rest: Vec<String> = args.by_ref().collect();
    let root = crate::workspace_root();

    match rest.first().map(String::as_str) {
        Some("open") => crate::to_exit_code(open(&root, &rest[1..])),
        Some("use") => crate::to_exit_code(switch(&root, &rest[1..])),
        Some("status") => crate::to_exit_code(status(&root, &rest[1..])),
        None | Some("help" | "--help" | "-h") => {
            print_usage();
            ExitCode::SUCCESS
        }
        Some(unknown) => {
            eprintln!("xtask: unknown `change` verb `{unknown}`\n");
            print_usage();
            ExitCode::FAILURE
        }
    }
}

/// Cuts the branch a change's shape calls for and leaves the clone on it.
///
/// No stage at all is the change of one stage, which the flow writes as the two-stage form with the
/// first stage collapsed; `deciding` opens the spec's stage and `building` the code's.
///
/// # Errors
///
/// Names what could not be read, what is not there yet that the stage needs, or what already exists
/// that the stage contradicts.
fn open(root: &Path, args: &[String]) -> Result<(), String> {
    let (issue, stage) = parse_open_arguments(args)?;
    ensure_gh_ready(root)?;
    run_visible(root, "git", &["fetch", "origin", "--prune"])?;

    let issue_number = format_issue_number(issue);

    let slug = match stage {
        // The building branch's slug was derived and committed when the deciding branch was opened,
        // so it is read back out of the merged spec rather than derived from a title that may have
        // been renamed since.
        Stage::Building => merged_slug(root, &issue_number)?,
        // These two begin something, and neither has a committed name to read it back from.
        Stage::Deciding | Stage::Single => {
            let slug = slug_of_title(root, issue)?;
            if stage == Stage::Single {
                refuse_two_stage(root, &issue_number)?;
            }
            slug
        }
    };

    let branch = branch_name(&issue_number, &slug, stage);
    if stage == Stage::Building {
        verify_deciding_merged(root, &branch)?;
    }
    ensure_branch(root, issue, &branch)?;

    announce(&issue_number, &slug, stage, &branch);
    Ok(())
}

/// Puts the working copy on a branch of `issue` that already exists.
///
/// Without a `stage` it takes the branch there is only one of, and reports the ones there are more
/// of: a change of two stages has both a deciding branch and a building one, and which one somebody
/// wants after the deciding merge is not a question the shape of the branch answers.
///
/// # Errors
///
/// Names the branches it looked for and the ones it found.
fn switch(root: &Path, args: &[String]) -> Result<(), String> {
    let (issue, stage) = parse_use_arguments(args)?;
    ensure_gh_ready(root)?;
    run_visible(root, "git", &["fetch", "origin", "--prune"])?;

    let issue_number = format_issue_number(issue);
    let found = whereabouts(root, &issue_number)?;
    let candidates: Vec<&Whereabouts> = found
        .iter()
        .filter(|entry| stage.is_none_or(|wanted| entry.stage() == wanted))
        .collect();

    let chosen = match candidates.as_slice() {
        [one] => *one,
        [] => {
            return Err(match stage {
                Some(wanted) => format!(
                    "xtask: issue {issue_number} has no `{}` branch locally or on origin. {}",
                    wanted.name(),
                    what_opens(wanted, &issue_number)
                ),
                None => format!(
                    "xtask: issue {issue_number} has no branch locally or on origin. {}",
                    what_opens(Stage::Deciding, &issue_number)
                ),
            });
        }
        _many => {
            let listed = candidates
                .iter()
                .map(|entry| entry.describe())
                .collect::<Vec<_>>()
                .join(", ");
            let first = candidates[0].stage();
            let hint = if candidates.iter().all(|entry| entry.stage() == first) {
                format!(
                    "they are all `{}` stages, so check the one you want out by hand",
                    first.name()
                )
            } else {
                format!("name the stage: `cargo xtask change use {issue_number} deciding`")
            };
            return Err(format!(
                "xtask: issue {issue_number} has {} branches — {listed} — and {hint}.",
                candidates.len()
            ));
        }
    };

    checkout_branch(root, &chosen.branch)?;
    // Every candidate came out of a `{issue_number}-*` listing, so it is a change's branch and the
    // slug is between its number and its suffix. A branch with none is not a name this flow writes.
    let Some(slug) = slug_from_branch(&chosen.branch) else {
        return Err(format!(
            "xtask: `{}` carries no slug between its issue number and its stage, so it is not a \
             name this flow writes.",
            chosen.branch
        ));
    };
    announce(&issue_number, &slug, chosen.stage(), &chosen.branch);
    Ok(())
}

/// Reports where `issue`, or the change the clone is on, stands.
///
/// # Errors
///
/// Names what could not be read, and says which branch the question was asked of where the clone is
/// on one that carries no issue.
fn status(root: &Path, args: &[String]) -> Result<(), String> {
    let asked = parse_status_arguments(args)?;
    ensure_gh_ready(root)?;
    run_visible(root, "git", &["fetch", "origin", "--prune"])?;

    let issue = if let Some(issue) = asked {
        issue
    } else {
        let branch = current_branch(root)?;
        issue_from_branch(&branch).ok_or_else(|| {
            format!(
                "xtask: `{branch}` carries no issue number, so it is not a change's branch. Name \
                 the issue: `cargo xtask change status <issue>`."
            )
        })?
    };

    let issue_number = format_issue_number(issue);
    // A number nobody can see the title of is still a number worth reporting, and `gh`'s complaint
    // about an issue that does not exist is not this command's to relay.
    match title_of_issue(root, issue) {
        Ok(title) => println!("Issue: #{issue_number} — {title}"),
        Err(_) => println!("Issue: #{issue_number}"),
    }

    let found = whereabouts(root, &issue_number)?;
    for entry in &found {
        let pull = entry
            .pull
            .as_deref()
            .map_or_else(String::new, |pull| format!("; PR {pull}"));
        println!("Branch: {} ({}){pull}", entry.branch, entry.place());
    }

    // A branch is where the slug is written down, and a change with no branch left is a change whose
    // merged spec is the only place its name survives. Both are read, and neither is derived.
    let slug = match found
        .iter()
        .find_map(|entry| slug_from_branch(&entry.branch))
    {
        Some(slug) => Some(slug),
        None => find_slug(&list_specs(root)?, &issue_number)?,
    };
    let merged = slug
        .as_deref()
        .map(|slug| merged_spec(root, &issue_number, slug))
        .transpose()?
        .flatten();

    match (&slug, &merged) {
        (Some(_), Some(spec)) => println!(
            "Spec: {} — {}, decided by {}",
            spec.path,
            spec.status.as_deref().unwrap_or("no status"),
            spec.decided.as_deref().unwrap_or("nobody")
        ),
        (Some(slug), None) => println!(
            "Spec: {} — not in origin/main",
            spec_path(&issue_number, slug)
        ),
        (None, _) => {}
    }

    println!(
        "Next: {}",
        next_step(&issue_number, &found, merged.as_ref())
    );
    Ok(())
}

/// What to do next, given everything `status` knows about a change.
///
/// It is one question with four answers, and the order is what makes it total: a change that is over
/// says so before anything looks for work, a change with a deciding branch and no merged spec is the
/// one stage that is still open, and a change with nothing at all is one that has not been opened.
fn next_step(issue_number: &str, found: &[Whereabouts], merged: Option<&MergedSpec>) -> String {
    if let Some(spec) = merged {
        match spec.status.as_deref() {
            Some("abandoned") => return format!("{} is `abandoned`; nothing to open.", spec.path),
            Some("implemented") => {
                return format!("{} is `implemented`; nothing to open.", spec.path);
            }
            _ => {}
        }
    }

    let branch_of = |stage| found.iter().find(|entry| entry.stage() == stage);
    let build = |branch: &str| {
        format!(
            "build against the merged spec on `{branch}`, then `cargo xtask pr body` and \
             `cargo xtask pr open`"
        )
    };

    if let Some(entry) = branch_of(Stage::Building) {
        return build(&entry.branch);
    }
    if let Some(entry) = branch_of(Stage::Single) {
        return format!(
            "build the change on `{}`, then `cargo xtask pr body` and `cargo xtask pr open`",
            entry.branch
        );
    }
    if let Some(entry) = branch_of(Stage::Deciding)
        && !entry.is_merged()
    {
        let slug = slug_from_branch(&entry.branch).unwrap_or_default();
        return format!(
            "write {}, filling `## The decision` until no answer reads `_pending_`, then \
             `cargo xtask pr body` and `cargo xtask pr open`",
            spec_path(issue_number, &slug)
        );
    }
    if merged.is_some() {
        return format!(
            "the deciding stage is merged and no branch is left: `cargo xtask change open \
             {issue_number} building`"
        );
    }
    format!("nothing is open for it yet: `cargo xtask change open {issue_number}`")
}

/// What a change of `stage` is opened with, as a sentence a refusal can end with.
fn what_opens(stage: Stage, issue_number: &str) -> String {
    match stage {
        Stage::Single => {
            format!("`cargo xtask change open {issue_number}` opens one for a change of one stage.")
        }
        _ => format!(
            "`cargo xtask change open {issue_number} {}` opens one.",
            stage.name()
        ),
    }
}

/// Refuses a branch with no stage for an issue that already has a deciding one.
///
/// The deciding branch is what says the change is of two stages, and opening a bare `NNN-slug` beside
/// it would leave the two halves of one change on two branches that say nothing about each other.
///
/// It looks for that branch by listing rather than by assembling its name from the slug it just
/// derived, because the two are not the same string when the issue was renamed after the deciding
/// branch was cut — and the rule this module follows everywhere else is that a name is read back,
/// never re-derived.
fn refuse_two_stage(root: &Path, issue_number: &str) -> Result<(), String> {
    let found = whereabouts(root, issue_number)?;
    let Some(deciding) = found.iter().find(|entry| entry.stage() == Stage::Deciding) else {
        return Ok(());
    };
    Err(format!(
        "xtask: issue {issue_number} already has a deciding stage on `{}`, so this is a change of \
         two stages. `cargo xtask change open {issue_number} deciding` opens that, and \
         `cargo xtask change open {issue_number} building` opens the one after it merges.",
        deciding.branch
    ))
}

/// Ensures `branch` exists, checking it out. Uses the branch on `origin` if this command or a person
/// already created it; otherwise creates it on `origin` linked to `issue`.
///
/// This is the only thing that ties a branch to an issue, and `gh issue develop` is what does it —
/// the branch name alone is a name.
fn ensure_branch(root: &Path, issue: u32, branch: &str) -> Result<(), String> {
    if branch_exists(root, branch)? {
        return checkout_branch(root, branch);
    }
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

/// What is known about one branch of a change: where it is, and what has been proposed from it.
struct Whereabouts {
    /// The branch name, which is also where the slug is read from.
    branch: String,
    /// Whether the branch is checked out in this clone.
    local: bool,
    /// Whether `origin` has it.
    remote: bool,
    /// The pull requests on the branch with their states, where there are any.
    pull: Option<String>,
}

impl Whereabouts {
    /// The stage this branch's name says.
    fn stage(&self) -> Stage {
        Stage::from_branch(&self.branch).unwrap_or(Stage::Single)
    }

    /// Where the branch is, as one phrase: `local, origin`, `origin`, or an em dash.
    fn place(&self) -> &'static str {
        match (self.local, self.remote) {
            (true, true) => "local, origin",
            (true, false) => "local",
            (false, true) => "origin",
            (false, false) => "—",
        }
    }

    /// Whether a merged pull request has come out of this branch.
    fn is_merged(&self) -> bool {
        self.pull
            .as_deref()
            .is_some_and(|pull| pull.lines().any(|line| line.ends_with(" MERGED")))
    }

    /// The branch as an ambiguity names it: the name, then where it is.
    fn describe(&self) -> String {
        format!("`{}` ({})", self.branch, self.place())
    }
}

/// One entry per branch `issue_number` has, wherever it is: local first, remote for the rest.
fn whereabouts(root: &Path, issue_number: &str) -> Result<Vec<Whereabouts>, String> {
    let pattern = format!("{issue_number}-*");
    let mut found: Vec<Whereabouts> = branch_listing(root, &[], &pattern)?
        .into_iter()
        .map(|branch| Whereabouts {
            branch,
            local: true,
            remote: false,
            pull: None,
        })
        .collect();

    for branch in branch_listing(root, &["-r"], &pattern)? {
        match found.iter_mut().find(|entry| entry.branch == branch) {
            Some(entry) => entry.remote = true,
            None => found.push(Whereabouts {
                branch,
                local: false,
                remote: true,
                pull: None,
            }),
        }
    }

    for entry in &mut found {
        entry.pull = pull_requests(root, &entry.branch)?;
    }
    found.sort_by(|one, other| one.branch.cmp(&other.branch));
    Ok(found)
}

/// The branches matching `pattern`, one per line.
///
/// `flags` is `["-r"]` for the remote-tracking refs, which carry an `origin/` prefix a caller does
/// not want to hold.
///
/// # Errors
///
/// Names the command when git cannot be run.
fn branch_listing(root: &Path, flags: &[&str], pattern: &str) -> Result<Vec<String>, String> {
    let mut args = vec!["branch"];
    args.extend_from_slice(flags);
    args.extend_from_slice(&["--format=%(refname:short)", "--list", pattern]);

    let mut names: Vec<String> = capture(root, "git", &args)?
        .lines()
        .map(|line| match line.strip_prefix("origin/") {
            Some(without) => without.to_string(),
            None => line.trim().to_string(),
        })
        .filter(|name| !name.is_empty())
        .collect();
    names.sort();
    names.dedup();
    Ok(names)
}

/// Whether `branch` exists in this clone.
fn local_branch_exists(root: &Path, branch: &str) -> Result<bool, String> {
    Ok(!branch_listing(root, &[], branch)?.is_empty())
}

/// Whether `origin` has `branch`, read from the tracking refs the fetch at the top of every verb
/// brought up to date.
fn remote_branch_exists(root: &Path, branch: &str) -> Result<bool, String> {
    let pattern = format!("origin/{branch}");
    Ok(!branch_listing(root, &["-r"], &pattern)?.is_empty())
}

/// Whether `branch` exists here or on `origin`.
fn branch_exists(root: &Path, branch: &str) -> Result<bool, String> {
    Ok(local_branch_exists(root, branch)? || remote_branch_exists(root, branch)?)
}

/// The pull requests opened on `branch` with their states, one per line, and `None` where there are
/// none.
///
/// It is `gh`'s own formatting rather than a value this crate reads: the answer is a line of a
/// person's `status` output, and the question is whether anything was printed at all.
fn pull_requests(root: &Path, branch: &str) -> Result<Option<String>, String> {
    let answer = capture(
        root,
        "gh",
        &[
            "pr",
            "list",
            "--state",
            "all",
            "--head",
            branch,
            "--json",
            "number,state",
            "--jq",
            r#".[] | "\(.number) \(.state)""#,
        ],
    )?;
    Ok((!answer.is_empty()).then_some(answer))
}

/// The branch the working copy is on.
///
/// # Errors
///
/// `main` and a detached head are both refused: a change is a branch, and there is no issue to report
/// on either.
fn current_branch(root: &Path) -> Result<String, String> {
    let branch = capture(root, "git", &["rev-parse", "--abbrev-ref", "HEAD"])?;
    match branch.as_str() {
        "main" => Err("xtask: this is `main`; name the issue to report on".to_string()),
        "HEAD" => {
            Err("xtask: the head is detached, so there is no branch to report on".to_string())
        }
        _ => Ok(branch),
    }
}

/// The title of `issue`, which is where a slug this crate derives comes from.
///
/// # Errors
///
/// Names what `gh` said when it could not be asked.
pub(crate) fn title_of_issue(root: &Path, issue: u32) -> Result<String, String> {
    let issue_str = issue.to_string();
    capture(
        root,
        "gh",
        &[
            "issue", "view", &issue_str, "--json", "title", "--jq", ".title",
        ],
    )
}

/// The slug for a branch this command is about to open for the first time, from the issue's title.
fn slug_of_title(root: &Path, issue: u32) -> Result<String, String> {
    slugify(&title_of_issue(root, issue)?)
}

/// The slug `issue_number`'s change was given, read back from the spec that committed it.
///
/// # Errors
///
/// Says that no spec in `origin/main` carries the name, which is what a change that has not been
/// opened yet looks like from here.
fn merged_slug(root: &Path, issue_number: &str) -> Result<String, String> {
    find_slug(&list_specs(root)?, issue_number)?.ok_or_else(|| {
        format!("xtask: no spec matching `specs/{issue_number}-*.md` was found in origin/main")
    })
}

/// Refuses `branch` unless its change has a merged deciding stage with an answered decision.
///
/// The caller has already fetched, which is what makes this a question about `origin/main` rather
/// than about whatever was there when the session started.
///
/// # Errors
///
/// Names the deciding branch and whether it has a merged pull request, or quotes the lines of the
/// merged spec that still read unanswered.
pub(crate) fn verify_deciding_merged(root: &Path, branch: &str) -> Result<(), String> {
    let issue = issue_from_branch(branch)
        .ok_or_else(|| format!("xtask: `{branch}` carries no issue number"))?;
    let issue_number = format_issue_number(issue);
    let slug = merged_slug(root, &issue_number)?;
    let deciding = branch_name(&issue_number, &slug, Stage::Deciding);

    let merged = pull_requests(root, &deciding)?
        .is_some_and(|pull| pull.lines().any(|line| line.ends_with(" MERGED")));
    if !merged {
        return Err(format!(
            "xtask: `{deciding}` has no merged pull request, so there is no agreed spec to build \
             against. Merge it first, or open this change as one stage on `{issue_number}-{slug}`."
        ));
    }

    let spec = merged_spec(root, &issue_number, &slug)?.ok_or_else(|| {
        format!(
            "xtask: the deciding pull request is not merged: `{}` does not exist in origin/main",
            spec_path(&issue_number, &slug)
        )
    })?;
    if spec.pending.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "xtask: `{}` is not answered: {}. The building stage runs against an agreed spec, so \
             answer it on `{deciding}` and merge that first.",
            spec.path,
            spec::describe_pending(&spec.pending)
        ))
    }
}

/// The spec of a change as `origin/main` has it, or `None` where it has none.
fn merged_spec(root: &Path, issue_number: &str, slug: &str) -> Result<Option<MergedSpec>, String> {
    let path = spec_path(issue_number, slug);
    if !file_exists_in_origin_main(root, &path)? {
        return Ok(None);
    }

    let text = capture_untrimmed(root, "git", &["show", &format!("origin/main:{path}")])?;
    Ok(Some(MergedSpec {
        path,
        status: spec::declared_status(&text),
        decided: spec::decided_by(&text),
        pending: spec::pending_lines(&text),
    }))
}

/// The spec of a change as `origin/main` has it.
struct MergedSpec {
    /// Where it is, which is also how a change with no spec is told from one with a broken read.
    path: String,
    /// The status its frontmatter declares.
    status: Option<String>,
    /// The pull request that agreed it.
    decided: Option<String>,
    /// The lines still carrying the unanswered marker, by one-based line number.
    pending: Vec<(usize, String)>,
}

/// Prints what opening or switching branch settled on.
///
/// The `Spec:` line is one of the change of one stage does not get: it has no spec file, and a path
/// that will never exist is worse than no line.
fn announce(issue_number: &str, slug: &str, stage: Stage, branch: &str) {
    if stage.has_spec() {
        println!("Spec: {}", spec_path(issue_number, slug));
    }
    println!("Branch: {branch}");
    println!("Stage: {}", stage.name());
    println!("Next: {}", stage.next_step());
}

/// The spec file for `issue_number` and `slug`.
fn spec_path(issue_number: &str, slug: &str) -> String {
    format!("specs/{issue_number}-{slug}.md")
}

/// Lists the entries of `specs/` in `origin/main`, one per line, as
/// `git ls-tree --name-only origin/main specs/` reports them.
fn list_specs(root: &Path) -> Result<Vec<String>, String> {
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

/// The slug of the one entry in `entries` named `specs/{issue_number}-*.md`, or `None` where there
/// is none.
///
/// # Errors
///
/// Names what it found when there is more than one, which is a change with two names and therefore
/// unreachable by number.
fn find_slug(entries: &[String], issue_number: &str) -> Result<Option<String>, String> {
    let prefix = format!("specs/{issue_number}-");
    let matches: Vec<&String> = entries
        .iter()
        .filter(|entry| entry.starts_with(&prefix))
        .collect();

    match matches.as_slice() {
        [] => Ok(None),
        [single] => Ok(Some(
            single[prefix.len()..]
                .strip_suffix(".md")
                .unwrap_or(&single[prefix.len()..])
                .to_string(),
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

/// The issue number `branch` starts with, where it starts with one.
pub(crate) fn issue_from_branch(branch: &str) -> Option<u32> {
    let digits: String = branch.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// The slug in `branch`, where it is a change's branch: what is between the number and the stage
/// suffix.
///
/// The branch name is the only place the slug of a change of one stage is ever written down, which
/// is why `use` and `status` read it here rather than deriving it from the issue's title.
fn slug_from_branch(branch: &str) -> Option<String> {
    let digits = branch.chars().take_while(char::is_ascii_digit).count();
    let rest = branch[digits..].strip_prefix('-')?;
    let slug = match Stage::from_branch(branch)? {
        Stage::Deciding => rest.strip_suffix(DECIDING)?.trim_end_matches('-'),
        Stage::Building => rest.strip_suffix(BUILDING)?.trim_end_matches('-'),
        Stage::Single => rest,
    };
    (!slug.is_empty()).then(|| slug.to_string())
}

/// Zero-pads `issue` to at least three digits; an issue number with more digits keeps all of them.
fn format_issue_number(issue: u32) -> String {
    format!("{issue:03}")
}

/// The branch name for `stage` of the change identified by `issue_number` and `slug`.
///
/// The change of one stage has no suffix, which is what makes `NNN-slug` distinguishable from a
/// branch that never went through this command: it opens neither stage of anything.
fn branch_name(issue_number: &str, slug: &str, stage: Stage) -> String {
    match stage.suffix() {
        Some(suffix) => format!("{issue_number}-{slug}-{suffix}"),
        None => format!("{issue_number}-{slug}"),
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
        ('Á', 'A'),
        ('É', 'E'),
        ('Í', 'I'),
        ('Ó', 'O'),
        ('Ú', 'U'),
        ('À', 'A'),
        ('È', 'E'),
        ('Ì', 'I'),
        ('Ò', 'O'),
        ('Ù', 'U'),
        ('Ä', 'A'),
        ('Ë', 'E'),
        ('Ï', 'I'),
        ('Ö', 'O'),
        ('Ü', 'U'),
        ('Â', 'A'),
        ('Ê', 'E'),
        ('Î', 'I'),
        ('Ô', 'O'),
        ('Û', 'U'),
        ('Ã', 'A'),
        ('Õ', 'O'),
        ('Ñ', 'N'),
        ('Ç', 'C'),
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

/// The stage `open` and `use` are asked for: the branch's own suffix, or nothing for one stage.
fn parse_stage(raw: Option<&String>) -> Result<Stage, String> {
    match raw.map(String::as_str) {
        None => Ok(Stage::Single),
        Some(DECIDING) => Ok(Stage::Deciding),
        Some(BUILDING) => Ok(Stage::Building),
        Some(word) => Err(format!(
            "xtask: `{word}` is not a stage. The stage is the suffix the branch carries: \
             `deciding`, `building`, or nothing at all for a change of one stage."
        )),
    }
}

/// The issue number `args` names, and the stage that follows it if there is one.
///
/// # Errors
///
/// Says what is missing or what it could not read.
fn parse_open_arguments(args: &[String]) -> Result<(u32, Stage), String> {
    let issue = parse_issue(args.first().map(String::as_str), "open")?;
    Ok((issue, parse_stage(args.get(1))?))
}

/// The issue number `args` names, and the stage to look for if one was given.
///
/// # Errors
///
/// Says what is missing or what it could not read.
fn parse_use_arguments(args: &[String]) -> Result<(u32, Option<Stage>), String> {
    let issue = parse_issue(args.first().map(String::as_str), "use")?;
    let stage = args.get(1).map(|_| parse_stage(args.get(1))).transpose()?;
    Ok((issue, stage))
}

/// The issue number `raw`, which is required by every verb but `status`.
///
/// # Errors
///
/// Names the verb that needs it, and quotes what it could not read as a number.
fn parse_issue(raw: Option<&str>, verb: &str) -> Result<u32, String> {
    let raw = raw.ok_or_else(|| format!("xtask: change {verb} needs an issue number"))?;
    raw.parse::<u32>()
        .map_err(|_| format!("xtask: `{raw}` is not a valid issue number"))
}

/// The issue number `args` names, where `status` can also be asked without one.
///
/// # Errors
///
/// Says what it could not read as a number.
fn parse_status_arguments(args: &[String]) -> Result<Option<u32>, String> {
    match args.first() {
        None => Ok(None),
        Some(raw) if raw.starts_with("--") => Err(format!(
            "xtask: `{raw}` is not an issue number. `cargo xtask change status` reports on the \
             change this clone is on, or names one: `cargo xtask change status 170`."
        )),
        Some(raw) => parse_issue(Some(raw.as_str()), "status").map(Some),
    }
}

/// Prints usage for `cargo xtask change`.
fn print_usage() {
    println!("Open, switch to, and check a change's branch.");
    println!();
    println!("Usage: cargo xtask change <verb> [args]");
    println!();
    println!("Verbs:");
    println!(
        "  open <issue> [stage]   Cut the branch this change's stage calls for, and check it out"
    );
    println!("  use <issue> [stage]    Check out a branch of this change that already exists");
    println!("  status [<issue>]       Say where a change stands, and what to do next");
    println!("  help                   Show this message");
    println!();
    println!(
        "The stage is the suffix the branch carries: `deciding`, `building`, or nothing at all"
    );
    println!(
        "for a change of one stage. Which one applies is the answer to the question in step 2"
    );
    println!("of the flow, so it is given rather than guessed.");
}

#[cfg(test)]
mod tests {
    use super::{Stage, branch_name, find_slug, format_issue_number, slug_from_branch, slugify};

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

    /// A change of one stage is the two-stage form with the first stage collapsed, so its branch is
    /// the pair's common stem and nothing else. This is the whole reason `Stage::suffix` is optional.
    #[test]
    fn the_branch_of_one_stage_is_the_stem_and_no_suffix() {
        assert_eq!(
            branch_name("039", "draw-shapes", Stage::Single),
            "039-draw-shapes"
        );
    }

    #[test]
    fn a_number_and_no_suffix_is_the_whole_change() {
        assert_eq!(
            Stage::from_branch("163-light-spec-driven"),
            Some(Stage::Single)
        );
        assert_eq!(
            Stage::from_branch("023-read-a-description"),
            Some(Stage::Single)
        );
    }

    #[test]
    fn each_two_stage_suffix_is_its_own_stage() {
        assert_eq!(
            Stage::from_branch("023-read-a-description-deciding"),
            Some(Stage::Deciding)
        );
        assert_eq!(
            Stage::from_branch("023-read-a-description-building"),
            Some(Stage::Building)
        );
    }

    /// The number is read before the suffix, so a branch with no suffix and a number in it is a
    /// change rather than a branch without an issue. Reading the suffix first sent the common case
    /// down the wrong arm.
    #[test]
    fn a_number_beats_the_absence_of_a_suffix() {
        assert_ne!(
            Stage::from_branch("163-light-spec-driven"),
            Stage::from_branch("readme-typo")
        );
    }

    #[test]
    fn a_branch_opening_with_no_digit_is_not_a_change() {
        assert_eq!(Stage::from_branch("readme-typo"), None);
        assert_eq!(Stage::from_branch("main"), None);
        assert_eq!(Stage::from_branch("readme-typo-deciding"), None);
    }

    #[test]
    fn the_slug_is_read_back_out_of_the_branch_name() {
        assert_eq!(
            slug_from_branch("163-light-spec-driven-building").as_deref(),
            Some("light-spec-driven")
        );
        assert_eq!(
            slug_from_branch("163-light-spec-driven-deciding").as_deref(),
            Some("light-spec-driven")
        );
        // The change of one stage has no spec to read it back from, so the branch is the only place
        // its slug is written down at all.
        assert_eq!(
            slug_from_branch("163-light-spec-driven").as_deref(),
            Some("light-spec-driven")
        );
        assert_eq!(slug_from_branch("readme-typo"), None);
    }

    #[test]
    fn find_slug_reads_the_name_back_and_leaves_the_extension_off() {
        let entries = vec![
            "specs/006-give-a-glyph-a-type.md".to_string(),
            "specs/163-light-spec-driven.md".to_string(),
        ];
        assert_eq!(
            find_slug(&entries, "163").unwrap().as_deref(),
            Some("light-spec-driven")
        );
        // A spec of another change is not this one's, which is what makes `None` an answer rather
        // than a prefix match.
        assert_eq!(find_slug(&entries, "999").unwrap(), None);
    }

    #[test]
    fn find_slug_refuses_an_issue_with_two_specs() {
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
}
