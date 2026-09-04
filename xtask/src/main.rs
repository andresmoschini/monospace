//! Repository automation for Monospace.
//!
//! This is the single entry point for the quality gate. The pre-commit hook and CI both invoke
//! `cargo xtask check` and nothing else, so there is exactly one definition of what "green" means
//! and no way for the two to drift apart.
//!
//! It deliberately has no dependencies. Orchestrating a list of subprocesses and propagating their
//! exit codes is what the standard library is for, and a tool whose job is to guard the project's
//! dependency policy should not be the first thing to bend it.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// One step of the quality gate: a label to report it by, and the command that implements it.
struct Step {
    /// Short name shown in the output and in the failure summary.
    name: &'static str,
    /// Executable to run, resolved through `PATH`.
    program: &'static str,
    /// Arguments passed to `program`.
    args: &'static [&'static str],
}

/// Every step of the quality gate, in the order they run.
///
/// Steps are added here as the gate grows. Order is presentation only: all of them run on every
/// invocation, so that one pass reports every problem rather than only the first.
///
/// A step passes or fails on its exit code alone. This is a deliberate limit rather than an
/// oversight, and it has known holes: `rustfmt` reports `can't set group_imports, unstable features
/// are only available in nightly channel` and still exits 0, and Cargo reports a missing
/// `workspace.resolver` the same way. In both cases the tool knows something is wrong, says so, and
/// the gate does not notice. Closing that would mean matching on the text the tools print, which is
/// brittle in a different and less obvious way.
const GATE: &[Step] = &[
    Step {
        name: "fmt",
        program: "cargo",
        args: &["fmt", "--all", "--check"],
    },
    Step {
        name: "clippy",
        program: "cargo",
        // The lints themselves live in [workspace.lints]; `-D warnings` is what turns the warnings
        // they produce into a failure here without making the editor shout while code is half
        // written.
        args: &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    },
    Step {
        name: "build",
        program: "cargo",
        args: &["build", "--workspace", "--all-targets"],
    },
    Step {
        name: "wasm",
        program: "cargo",
        // ADR-0001 asks the core to stay free of terminal and command-line assumptions so it can
        // back a WebAssembly build later. This is what turns that from a claim in a document into
        // something the compiler refuses to let through. The target installs itself via
        // rust-toolchain.toml, so this needs no setup.
        args: &[
            "check",
            "-p",
            "monospace-core",
            "--target",
            "wasm32-unknown-unknown",
        ],
    },
    Step {
        name: "test",
        program: "cargo",
        // This also runs the doctests, so there is no separate step for them.
        args: &["test", "--workspace"],
    },
    Step {
        name: "doc",
        program: "cargo",
        // The rustdoc lints are set to "deny" in [workspace.lints.rustdoc], so a broken intra-doc
        // link fails here on its own; unlike clippy, this step needs no -D flag.
        args: &["doc", "--workspace", "--no-deps"],
    },
];

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        Some("check") => run_gate(),
        None | Some("help" | "--help" | "-h") => {
            print_usage();
            ExitCode::SUCCESS
        }
        Some(unknown) => {
            eprintln!("xtask: unknown command `{unknown}`\n");
            print_usage();
            ExitCode::FAILURE
        }
    }
}

/// Runs every step of the gate and reports which ones failed.
///
/// All steps run even after one fails. A gate that stops at the first problem turns one broken
/// commit into several round trips, and the steps here are cheap enough that finishing is free.
fn run_gate() -> ExitCode {
    let root = workspace_root();
    let mut failed = Vec::new();

    for step in GATE {
        println!("\n--- {} ---", step.name);
        if !run(&root, step) {
            failed.push(step.name);
        }
    }

    if failed.is_empty() {
        println!("\nall {} checks passed", GATE.len());
        return ExitCode::SUCCESS;
    }

    eprintln!(
        "\n{} of {} checks failed: {}",
        failed.len(),
        GATE.len(),
        failed.join(", ")
    );
    ExitCode::FAILURE
}

/// Runs one step from the workspace root, returning whether it succeeded.
fn run(root: &Path, step: &Step) -> bool {
    match Command::new(step.program)
        .args(step.args)
        .current_dir(root)
        .status()
    {
        Ok(status) => status.success(),
        Err(error) => {
            eprintln!("xtask: could not run `{}`: {error}", step.program);
            false
        }
    }
}

/// The repository root, derived from this crate's own location rather than from the current
/// directory, so that every step runs against the same paths no matter where it was invoked from.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the xtask crate always sits one level below the workspace root")
        .to_path_buf()
}

fn print_usage() {
    println!("Repository automation for Monospace.");
    println!();
    println!("Usage: cargo xtask <command>");
    println!();
    println!("Commands:");
    println!("  check    Run every quality gate step; this is what the hook and CI run");
    println!("  help     Show this message");
}
