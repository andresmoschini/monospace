//! Regenerating the pictures a tracked Markdown file carries.
//!
//! The constitution's _Show the rendering_ asks that every picture in a tracked file be either
//! generated from a description the file carries or labelled hypothetical. This is the half that
//! makes the first kind mechanical: `cargo xtask render` rewrites each fence from its description,
//! and the same code run in checking mode fails the gate when the two have parted
//! ([ADR-0052](../../docs/decisions/0052-show-the-rendering.md)).
//!
//! # Design notes
//!
//! **The description lives in the file, not beside it.** A marker carries its own JSON rather than
//! a path to it, so a reader sees what produced the picture without opening anything, and
//! principle VIII's exemption — a picture "and the description it comes from" costs an artifact
//! nothing — has something to exempt.
//!
//! **A picture is written with its trailing blanks trimmed.** A rendering is padded to the
//! window's width, and the gate runs `editorconfig-checker` with `trim_trailing_whitespace` over
//! everything Git tracks. [ADR-0045](../../docs/decisions/0045-pin-every-arrow-arrangement-as-a-reviewed-snapshot.md)
//! met this first for the arrow sweep and answered it the same way; comparing trimmed against
//! trimmed keeps one answer rather than two.
//!
//! **The grammar is strict and says so when it is broken.** A half-written marker is reported by
//! file and line instead of being skipped, because a marker silently ignored is a picture nothing
//! checks — which is the state this exists to end.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::process::{capture, capture_untrimmed};

/// The line that opens a marker. The description follows it, one or more lines of JSON.
const OPEN: &str = "<!-- render:";
/// The line that closes the description.
const DESCRIPTION_END: &str = "-->";
/// The fence a generated picture sits in.
const FENCE: &str = "```text";
/// The line that closes a marker, after the fence.
const CLOSE: &str = "<!-- /render -->";

/// One marker: the description it carries, and where its picture sits in the file.
#[derive(Debug)]
struct Marker {
    /// One-based line of the `<!-- render:` that opened it, for reporting.
    line: usize,
    /// The description, as the JSON text between the marker and its `-->`.
    description: String,
    /// Index of the fence's opening line. The picture is everything after it up to `fence_close`.
    fence_open: usize,
    /// Index of the fence's closing line.
    fence_close: usize,
}

/// Runs `cargo xtask render`.
pub fn run(mut args: impl Iterator<Item = String>) -> ExitCode {
    let checking = match args.next().as_deref() {
        None => false,
        Some("--check") => true,
        Some("help" | "--help" | "-h") => {
            print_usage();
            return ExitCode::SUCCESS;
        }
        Some(unknown) => {
            eprintln!("xtask: unknown option `{unknown}`\n");
            print_usage();
            return ExitCode::FAILURE;
        }
    };

    let root = crate::workspace_root();
    match walk(&root, checking) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

/// Visits every tracked Markdown file and either rewrites its pictures or reports the stale ones.
fn walk(root: &Path, checking: bool) -> Result<(), String> {
    let binary = build_cli(root)?;
    let mut stale = Vec::new();
    let mut rewritten = 0usize;
    let mut markers = 0usize;

    for relative in tracked_markdown(root)? {
        let path = root.join(&relative);
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("xtask: could not read {relative}: {error}"))?;
        let lines: Vec<&str> = text.split('\n').collect();
        let found = parse(&lines, &relative)?;
        markers += found.len();

        let mut wanted: Vec<(usize, String)> = Vec::new();
        for marker in &found {
            let picture = render_one(root, &binary, &marker.description, &relative, marker.line)?;
            let current = lines[marker.fence_open + 1..marker.fence_close].join("\n");
            if current != picture {
                if checking {
                    stale.push(report(&relative, marker.line, &current, &picture));
                } else {
                    wanted.push((marker.fence_open, picture));
                }
            }
        }

        if !wanted.is_empty() {
            rewrite(&path, &relative, &lines, &found, &wanted)?;
            rewritten += wanted.len();
            println!("  {relative}: {} picture(s) rewritten", wanted.len());
        }
    }

    if !stale.is_empty() {
        return Err(format!(
            "{}\n{} of {markers} picture(s) have parted from the description beside them. Run \
             `cargo xtask render`.",
            stale.join("\n"),
            stale.len()
        ));
    }

    if checking {
        println!("{markers} generated picture(s) match their descriptions");
    } else if rewritten == 0 {
        println!("{markers} generated picture(s) were already up to date");
    }
    Ok(())
}

/// The message a stale picture produces: where it is, what the file holds, and what the
/// description renders.
///
/// Both pictures are shown whole. The subject of this failure is something the project draws, so
/// prose describing the difference would be the thing _Show the rendering_ exists to remove.
fn report(relative: &str, line: usize, current: &str, wanted: &str) -> String {
    format!(
        "\n{relative}:{line}: the picture is not what its description renders.\n\n  in the file:\n\
         {}\n\n  from the description:\n{}",
        indent(current),
        indent(wanted)
    )
}

/// Indents a picture by four spaces so it reads as a block inside a message.
fn indent(picture: &str) -> String {
    picture
        .lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Writes `path` back with each marker's picture replaced.
///
/// `wanted` holds the new picture for each fence that moved, keyed by the fence's opening line.
fn rewrite(
    path: &Path,
    relative: &str,
    lines: &[&str],
    markers: &[Marker],
    wanted: &[(usize, String)],
) -> Result<(), String> {
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut index = 0usize;

    while index < lines.len() {
        out.push(lines[index].to_string());

        if let Some(marker) = markers.iter().find(|marker| marker.fence_open == index) {
            match wanted.iter().find(|(open, _)| *open == index) {
                Some((_, picture)) => out.extend(picture.split('\n').map(str::to_string)),
                None => out.extend(
                    lines[marker.fence_open + 1..marker.fence_close]
                        .iter()
                        .map(|line| (*line).to_string()),
                ),
            }
            index = marker.fence_close;
        } else {
            index += 1;
        }
    }

    std::fs::write(path, out.join("\n"))
        .map_err(|error| format!("xtask: could not write {relative}: {error}"))
}

/// Finds every marker in `lines`, or names the first one that is not written the way the grammar
/// asks for.
fn parse(lines: &[&str], relative: &str) -> Result<Vec<Marker>, String> {
    let mut markers = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        // A marker shown inside a fence is an illustration of the grammar, not an instance of it.
        // The decision sheet's template and CONTRIBUTING.md both show one, and a longer fence is
        // how Markdown already spells "the fence inside this is content".
        if let Some(width) = fence_width(lines[index]) {
            index += 1;
            while index < lines.len() && fence_width(lines[index]).is_none_or(|w| w < width) {
                index += 1;
            }
            index += 1;
            continue;
        }

        if lines[index].trim() != OPEN {
            index += 1;
            continue;
        }

        let line = index + 1;
        let opened = index;
        index += 1;

        let description_start = index;
        while index < lines.len() && lines[index].trim() != DESCRIPTION_END {
            index += 1;
        }
        if index == lines.len() {
            return Err(expected(relative, line, "a `-->` closing the description"));
        }
        let description = lines[description_start..index].join("\n");
        index += 1;

        while index < lines.len() && lines[index].trim().is_empty() {
            index += 1;
        }
        if index == lines.len() || lines[index].trim() != FENCE {
            return Err(expected(relative, line, "a ````text` fence"));
        }
        let fence_open = index;
        index += 1;

        while index < lines.len() && lines[index].trim() != "```" {
            index += 1;
        }
        if index == lines.len() {
            return Err(expected(relative, line, "a ``` closing the fence"));
        }
        let fence_close = index;
        index += 1;

        while index < lines.len() && lines[index].trim().is_empty() {
            index += 1;
        }
        if index == lines.len() || lines[index].trim() != CLOSE {
            return Err(expected(
                relative,
                line,
                "a `<!-- /render -->` closing the marker",
            ));
        }
        index += 1;

        debug_assert!(opened < fence_open);
        markers.push(Marker {
            line,
            description,
            fence_open,
            fence_close,
        });
    }

    Ok(markers)
}

/// How many backticks open or close a fence on this line, if it is a fence line at all.
///
/// The `CommonMark` rule: a fence of `n` backticks is closed only by one of `n` or more, which is
/// what lets a four-backtick block hold a three-backtick fence as content.
fn fence_width(line: &str) -> Option<usize> {
    let backticks = line.trim_start().chars().take_while(|c| *c == '`').count();
    (backticks >= 3).then_some(backticks)
}

/// The message a malformed marker produces.
fn expected(relative: &str, line: usize, what: &str) -> String {
    format!(
        "xtask: {relative}:{line}: the marker opened here needs {what}.\n\n    A generated picture \
         is written as `{OPEN}`, the description, `{DESCRIPTION_END}`, the fence holding the \
         picture, then `{CLOSE}`."
    )
}

/// Renders one description by running the command-line application on it.
fn render_one(
    root: &Path,
    binary: &Path,
    description: &str,
    relative: &str,
    line: usize,
) -> Result<String, String> {
    let scratch = std::env::temp_dir().join(format!(
        "monospace-render-{}-{line}.json",
        std::process::id()
    ));
    std::fs::write(&scratch, description)
        .map_err(|error| format!("xtask: could not write {}: {error}", scratch.display()))?;

    let binary = binary.to_string_lossy().into_owned();
    let scratch_arg = scratch.to_string_lossy().into_owned();
    let rendered = capture_untrimmed(root, &binary, &[&scratch_arg]).map_err(|error| {
        format!("xtask: {relative}:{line}: the description did not render.\n\n    {error}")
    });
    let _ = std::fs::remove_file(&scratch);

    Ok(trim_trailing_blanks(&rendered?))
}

/// Drops each line's trailing blanks and the rendering's final newline, so a picture can sit in a
/// tracked file without `editorconfig-checker` rejecting the padding a window's width produces.
fn trim_trailing_blanks(rendered: &str) -> String {
    rendered
        .strip_suffix('\n')
        .unwrap_or(rendered)
        .split('\n')
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Builds the command-line application once and returns the path to it.
///
/// Every marker is rendered by running that binary, which costs about 10 ms. Going through
/// `cargo run` each time instead costs about 94 ms — measured over five runs of each — and the
/// whole point of this command is that markers multiply.
fn build_cli(root: &Path) -> Result<PathBuf, String> {
    crate::process::run_visible(root, "cargo", &["build", "-q", "-p", "monospace-cli"])?;

    let target =
        std::env::var_os("CARGO_TARGET_DIR").map_or_else(|| root.join("target"), PathBuf::from);
    let binary = target.join("debug").join(if cfg!(windows) {
        "monospace-cli.exe"
    } else {
        "monospace-cli"
    });

    if binary.exists() {
        Ok(binary)
    } else {
        Err(format!(
            "xtask: monospace-cli built, but is not at {}.\n\n    Set CARGO_TARGET_DIR if this \
             workspace builds somewhere else.",
            binary.display()
        ))
    }
}

/// Every Markdown file Git tracks, as paths relative to the workspace root.
///
/// Reading the list from Git is what keeps this in step with the rest of the gate: the same set
/// `editorconfig-checker` and `cspell` read, with nothing ignored quietly.
fn tracked_markdown(root: &Path) -> Result<Vec<String>, String> {
    Ok(capture(root, "git", &["ls-files", "*.md"])?
        .lines()
        .map(str::to_string)
        .filter(|path| !path.is_empty())
        .collect())
}

fn print_usage() {
    println!("Regenerate the pictures tracked Markdown files carry.");
    println!();
    println!("Usage: cargo xtask render [--check]");
    println!();
    println!("Options:");
    println!("  --check  Report the pictures that have parted from their description, and change");
    println!("           nothing. This is what `cargo xtask check` runs.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file with no marker yields no marker, and every line is left for the caller.
    #[test]
    fn a_file_with_no_marker_has_nothing_to_render() {
        let lines = vec![
            "# Title",
            "",
            "Some prose.",
            "",
            "```text",
            "hand drawn",
            "```",
        ];

        assert!(
            parse(&lines, "x.md")
                .expect("no marker is not an error")
                .is_empty()
        );
    }

    /// A well-formed marker yields its description and the bounds of its fence.
    #[test]
    fn a_well_formed_marker_carries_its_description_and_its_fence() {
        let lines = vec![
            "prose",
            "<!-- render:",
            "{ \"canvas\": 1,",
            "  \"shapes\": [] }",
            "-->",
            "",
            "```text",
            "┌┐",
            "└┘",
            "```",
            "",
            "<!-- /render -->",
            "more prose",
        ];

        let markers = parse(&lines, "x.md").expect("this marker is well formed");

        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].line, 2);
        assert_eq!(
            markers[0].description,
            "{ \"canvas\": 1,\n  \"shapes\": [] }"
        );
        assert_eq!(
            lines[markers[0].fence_open + 1..markers[0].fence_close].join("\n"),
            "┌┐\n└┘"
        );
    }

    /// Two markers in one file are both found.
    #[test]
    fn two_markers_in_one_file_are_both_found() {
        let one = vec![
            "<!-- render:",
            "{}",
            "-->",
            "",
            "```text",
            "a",
            "```",
            "",
            "<!-- /render -->",
        ];
        let lines: Vec<&str> = one.iter().chain(one.iter()).copied().collect();

        assert_eq!(
            parse(&lines, "x.md").expect("both are well formed").len(),
            2
        );
    }

    /// A marker whose fence is missing is reported rather than skipped, and the message names the
    /// line the marker opened on.
    #[test]
    fn a_marker_with_no_fence_is_reported_by_line() {
        let lines = vec!["<!-- render:", "{}", "-->", "", "not a fence"];

        let error = parse(&lines, "docs/model.md").expect_err("a marker with no fence is an error");

        assert!(error.contains("docs/model.md:1"), "{error}");
        assert!(error.contains("fence"), "{error}");
    }

    /// A marker whose description is never closed is reported.
    #[test]
    fn a_marker_with_no_description_end_is_reported() {
        let lines = vec!["<!-- render:", "{}"];

        let error = parse(&lines, "x.md").expect_err("an unclosed description is an error");

        assert!(error.contains("-->"), "{error}");
    }

    /// A marker whose closing comment is missing is reported: without it a picture's end is only
    /// the fence, and an ordinary fence below would be swallowed.
    #[test]
    fn a_marker_with_no_closing_comment_is_reported() {
        let lines = vec!["<!-- render:", "{}", "-->", "", "```text", "a", "```", ""];

        let error = parse(&lines, "x.md").expect_err("a marker with no close is an error");

        assert!(error.contains("/render"), "{error}");
    }

    /// A marker shown inside a longer fence is an illustration, not an instance: the decision
    /// sheet's template shows one, and rendering it would mean rendering `…`.
    #[test]
    fn a_marker_inside_a_longer_fence_is_not_a_marker() {
        let lines = vec![
            "````markdown",
            "<!-- render:",
            "{ \"shapes\": [ … ] }",
            "-->",
            "",
            "```text",
            "```",
            "",
            "<!-- /render -->",
            "````",
        ];

        assert!(
            parse(&lines, "template.md")
                .expect("an illustrated marker is not parsed")
                .is_empty()
        );
    }

    /// A real marker after an illustrated one is still found: skipping a fence must not swallow
    /// the rest of the file.
    #[test]
    fn a_marker_after_a_fence_is_still_found() {
        let lines = vec![
            "```text",
            "hand drawn",
            "```",
            "<!-- render:",
            "{}",
            "-->",
            "",
            "```text",
            "a",
            "```",
            "",
            "<!-- /render -->",
        ];

        assert_eq!(parse(&lines, "x.md").expect("well formed").len(), 1);
    }

    /// The fence rule is the `CommonMark` one: only a fence at least as long closes one.
    #[test]
    fn a_fence_is_closed_only_by_one_at_least_as_long() {
        assert_eq!(fence_width("```text"), Some(3));
        assert_eq!(fence_width("````markdown"), Some(4));
        assert_eq!(fence_width("``not a fence"), None);
        assert_eq!(fence_width("prose"), None);
    }

    /// Each line loses its trailing blanks and the rendering loses its final newline, so what goes
    /// into a fence is what `editorconfig-checker` accepts.
    #[test]
    fn trimming_drops_the_padding_and_the_final_newline() {
        assert_eq!(trim_trailing_blanks("┌┐  \n└┘  \n"), "┌┐\n└┘");
    }

    /// A blank line inside a picture stays a line: only its blanks go, not the line itself.
    #[test]
    fn trimming_keeps_a_blank_line_inside_a_picture() {
        assert_eq!(trim_trailing_blanks("a  \n   \nb\n"), "a\n\nb");
    }
}
