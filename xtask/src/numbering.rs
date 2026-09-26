//! Rejecting two entries that claim one number.
//!
//! A decision record and a feature are each named after the number that identifies them — ADR
//! `NNNN`, feature directory `specs/NNN-slug/` — and nothing stopped two of them from taking one
//! number. Every citation in this repository is made by number, so a repetition makes a reference
//! ambiguous rather than wrong, which is the harder kind to notice
//! ([issue #26](https://github.com/andresmoschini/monospace/issues/26)).
//!
//! # Design notes
//!
//! **The two trees are separate sequences, checked separately.** A feature's number is its GitHub
//! issue's, and a record's is the next unused number in its own directory; the two are numbered by
//! different people for different reasons, so `specs/006-` and `docs/decisions/0006-` are not a
//! collision. `docs/specs/` is deliberately outside: it is frozen and its numbering does not
//! continue, per the constitution's _The previous spec home is history_.
//!
//! **An entry is the first path segment under the tree, and its number is the digits it opens
//! with.** One rule reads a file and a directory alike, and a name that opens with anything else
//! claims no number — which is what exempts `docs/decisions/README.md`,
//! `adr-template.md` and `constitution-history.md` without listing them anywhere.
//!
//! **Two spellings of one number are one number.** The padding comes off before entries are
//! grouped, so `6-` and `0006-` answer each other; a step comparing the digits as written would
//! report them as two numbers, and a hand-written entry is exactly what this step exists to catch.
//! The message prints the number without its padding while the entries below carry the spelling, so
//! two names disagreeing about it are visible rather than smoothed over.
//!
//! **The list comes from Git, like the rest of the gate.** It is the same set `editorconfig` and
//! `cspell` read, with nothing ignored quietly — and it is read the way `render` reads it, so a
//! directory nothing has staged yet claims no number until it does.

use std::collections::BTreeMap;
use std::path::Path;

use crate::process::capture;

/// The trees whose entries are named after a number, each with what a collision there needs.
///
/// The two are independent: the second field says what to do about a repetition in that tree, which
/// is not the same answer twice.
const NUMBERED: [(&str, &str); 2] = [
    (
        "docs/decisions",
        "Rename one of them, and add its row to the index in docs/decisions/README.md.",
    ),
    (
        "specs",
        "`cargo xtask spec new` derives this name from the issue, so a second entry under one \
         number was not made by it. Remove the one that should not be there, or rename it.",
    ),
];

/// The gate's step: reports every number more than one entry claims, and returns whether there
/// were none.
pub fn check(root: &Path) -> bool {
    crate::report_failure(walk(root))
}

/// Visits both trees and either reports the numbers claimed twice or says what it looked at.
fn walk(root: &Path) -> Result<(), String> {
    let mut counted = 0usize;
    let mut clashes = Vec::new();

    for (tree, remedy) in NUMBERED {
        let entries = entries(root, tree)?;
        counted += entries.len();
        clashes.extend(
            claimed_twice(&entries)
                .into_iter()
                .map(|clash| report(tree, remedy, &clash)),
        );
    }

    if !clashes.is_empty() {
        return Err(format!(
            "{}\n{} of {counted} entries claim a number another one claims.",
            clashes.join("\n"),
            clashes.len()
        ));
    }

    println!("{counted} entries checked, no number claimed twice");
    Ok(())
}

/// Every entry directly under `tree`, as Git tracks it: the first path segment of each tracked file
/// below it, deduplicated.
///
/// A tree of directories arrives one file at a time — `git ls-files` reports paths, not the
/// directories holding them — so the directory is recovered from the paths of the files in it. A
/// directory holding nothing tracked is not reported, which is the same limit `render` has.
fn entries(root: &Path, tree: &str) -> Result<Vec<String>, String> {
    let mut entries: Vec<String> = capture(root, "git", &["ls-files", "--", tree])?
        .lines()
        .filter_map(|path| path.strip_prefix(tree)?.strip_prefix('/'))
        .filter_map(|below| below.split('/').next())
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect();

    entries.sort();
    entries.dedup();
    Ok(entries)
}

/// The numbers more than one of `entries` claims, with every entry claiming each.
///
/// A `BTreeMap` is what makes the report the same on every run: the order entries arrive in is
/// Git's, and a map keyed by the number sorts them without a comparison of its own.
fn claimed_twice(entries: &[String]) -> Vec<Clash<'_>> {
    let mut groups: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    for entry in entries {
        if let Some(number) = number_of(entry) {
            groups.entry(padded_out(number)).or_default().push(entry);
        }
    }

    groups
        .into_iter()
        .filter(|(_, claiming)| claiming.len() > 1)
        .map(|(number, claiming)| Clash {
            number,
            claiming: claiming.into_iter().map(str::to_string).collect(),
        })
        .collect()
}

/// The digits `entry` opens with, or `None` when it opens with anything else.
///
/// A name that is nothing but digits claims all of itself, which is why the length of the run comes
/// from `unwrap_or` rather than from a second pass over the string.
fn number_of(entry: &str) -> Option<&str> {
    let end = entry
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(entry.len());

    (end > 0).then_some(&entry[..end])
}

/// The number that two spellings of it agree on: the leading digits with the padding taken off.
///
/// It stays a string rather than becoming a number, because a name carrying more digits than any
/// integer holds has to keep claiming one instead of quietly claiming none. A run of nothing but
/// zeros is `0` rather than empty, so `0000-` and `0-` still answer each other.
fn padded_out(number: &str) -> &str {
    let digits = number.trim_start_matches('0');

    if digits.is_empty() { "0" } else { digits }
}

/// One number claimed by more than one entry.
struct Clash<'a> {
    /// The number, as the entries claiming it spell it.
    number: &'a str,
    /// Every entry claiming it, in the order Git listed them.
    claiming: Vec<String>,
}

/// The message a collision produces: the number, the entries answering to it, and the fix that
/// tree needs.
///
/// The entries are named rather than counted, because a number on its own says nothing about which
/// file has to move.
fn report(tree: &str, remedy: &str, clash: &Clash) -> String {
    let entries = clash
        .claiming
        .iter()
        .map(|entry| format!("    {tree}/{entry}"))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "\nxtask: {n} entries under {tree} claim the number {}:\n\n{entries}\n\n    {remedy}",
        clash.number,
        n = clash.claiming.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Numbers no two entries share produce no clash, and the entries are still counted.
    #[test]
    fn one_number_per_entry_is_no_clash() {
        let entries = vec![
            "0000-first-record.md".to_string(),
            "0001-second-record.md".to_string(),
        ];

        assert!(claimed_twice(&entries).is_empty());
    }

    /// Two entries under one number are a clash, and both are named. The number is carried without
    /// its padding, because that is the form two spellings of it agree on.
    #[test]
    fn two_entries_under_one_number_clash() {
        let entries = vec![
            "0063-first-thing.md".to_string(),
            "0063-second-thing.md".to_string(),
        ];

        let clash = claimed_twice(&entries);
        assert_eq!(clash.len(), 1);
        assert_eq!(clash[0].number, "63");
        assert_eq!(clash[0].claiming.len(), 2);
    }

    /// Three entries under one number are one clash, not two.
    #[test]
    fn three_entries_under_one_number_are_one_clash() {
        let entries = vec![
            "0063-a.md".to_string(),
            "0063-b.md".to_string(),
            "0063-c.md".to_string(),
        ];

        assert_eq!(claimed_twice(&entries).len(), 1);
    }

    /// A name opening with anything else claims no number, so the two files that hold no record
    /// are exempt without being listed anywhere.
    #[test]
    fn a_name_without_leading_digits_claims_no_number() {
        let entries = vec![
            "README.md".to_string(),
            "adr-template.md".to_string(),
            "constitution-history.md".to_string(),
        ];

        assert!(claimed_twice(&entries).is_empty());
    }

    /// The message names the number, both entries and the fix, so a reader knows what to move.
    #[test]
    fn the_report_names_the_number_the_entries_and_the_fix() {
        let entries = vec!["0063-a.md".to_string(), "0063-b.md".to_string()];

        let message = report("docs/decisions", NUMBERED[0].1, &claimed_twice(&entries)[0]);

        assert!(message.contains("0063"), "{message}");
        assert!(message.contains("docs/decisions/0063-a.md"), "{message}");
        assert!(message.contains("docs/decisions/0063-b.md"), "{message}");
        assert!(message.contains("README.md"), "{message}");
    }

    /// A name of nothing but digits claims all of itself rather than none of it, so a degenerate
    /// entry cannot slip past the rule it is the whole subject of.
    #[test]
    fn a_name_of_nothing_but_digits_claims_all_of_itself() {
        assert_eq!(number_of("0063"), Some("0063"));
    }

    /// A feature directory is an entry by its first path segment, so a tree of directories and a
    /// tree of files are read by the same rule.
    #[test]
    fn a_directory_is_an_entry_by_its_first_segment() {
        assert_eq!(number_of("079-a-diagram-holds-shapes"), Some("079"));
    }

    /// Two spellings of one number are one number, so a padded name and a bare one answer each
    /// other rather than sitting in two groups.
    #[test]
    fn two_spellings_of_one_number_are_one_clash() {
        let entries = vec![
            "0006-record-the-session.md".to_string(),
            "6-the-same-number-written-differently.md".to_string(),
        ];

        let clash = claimed_twice(&entries);
        assert_eq!(clash.len(), 1);
        assert_eq!(clash[0].claiming.len(), 2);
    }

    /// The number reaches the message without its padding while the entries below carry the
    /// spelling, so two names disagreeing about it are visible rather than smoothed over.
    #[test]
    fn the_report_prints_the_bare_number_and_the_names_as_they_are() {
        let entries = vec![
            "0006-record-the-session.md".to_string(),
            "6-the-same-number-written-differently.md".to_string(),
        ];

        let message = report("docs/decisions", NUMBERED[0].1, &claimed_twice(&entries)[0]);

        assert!(message.contains("claim the number 6:"), "{message}");
        assert!(message.contains("0006-record-the-session.md"), "{message}");
        assert!(
            message.contains("6-the-same-number-written-differently.md"),
            "{message}"
        );
    }

    /// A run of nothing but zeros is the number zero rather than an empty one, or `0000-` and `0-`
    /// would answer nothing at all.
    #[test]
    fn a_run_of_zeros_is_the_number_zero() {
        assert_eq!(padded_out("0000"), "0");
        assert_eq!(padded_out("0"), "0");
    }

    /// The digits are read as written, padding included; dropping it is a second decision with a
    /// name of its own, which is what keeps the parse honest about what it did.
    #[test]
    fn the_digits_are_read_as_written() {
        assert_eq!(number_of("006-give-a-glyph-a-type"), Some("006"));
        assert_eq!(number_of("0006-record-the-claude-session.md"), Some("0006"));
    }
}
