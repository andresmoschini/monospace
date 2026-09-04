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
use std::time::SystemTime;

/// The npm executable.
///
/// On Windows it has to be named with its extension: npm ships as a shell script plus `.cmd` and
/// `.ps1` shims, and `Command::new` does not apply `PATHEXT` the way a shell does, so a bare `npm`
/// is simply not found.
#[cfg(windows)]
const NPM: &str = "npm.cmd";
/// The npm executable.
#[cfg(not(windows))]
const NPM: &str = "npm";

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
        name: "prettier",
        // Invoked from node_modules/.bin rather than through npx, which costs about a second per
        // call for nothing. Formatting and line width for Markdown and JSON are decided here;
        // .editorconfig supplies indentation and line endings, so the editor and this step read the
        // same source.
        program: "node_modules/.bin/prettier",
        args: &["--check", "."],
    },
    Step {
        name: "markdownlint",
        // Globs and ignores live in .markdownlint-cli2.jsonc, so this takes no arguments and the
        // configuration has one home. It checks structure only; prettier owns formatting.
        program: "node_modules/.bin/markdownlint-cli2",
        args: &[],
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
        Some("setup") => run_setup(),
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

    if let Err(problem) = node_tooling_state(&root) {
        eprintln!("{problem}");
        eprintln!();
        eprintln!("    Run `cargo xtask setup` and try again.");
        eprintln!();
        eprintln!(
            "Nothing was checked. Running only the Rust steps would print a passing summary for \
             half a gate, which is worse than refusing to start."
        );
        return ExitCode::FAILURE;
    }

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
    let program = program_path(root, step.program);

    match Command::new(&program)
        .args(step.args)
        .current_dir(root)
        .status()
    {
        Ok(status) => status.success(),
        Err(error) => {
            eprintln!("xtask: could not run `{}`: {error}", program.display());
            false
        }
    }
}

/// Resolves a step's executable, following the rule a shell already uses: a bare name is looked up
/// on `PATH`, and anything containing a separator is a path relative to the workspace root.
///
/// The relative form is how the npm-installed tools under `node_modules/.bin` are reached. On
/// Windows npm ships those as a shell script alongside `.cmd` and `.ps1` shims, and `Command::new`
/// does not apply `PATHEXT` to an explicit path the way it would to a bare name, so the `.cmd` has
/// to be spelled out.
fn program_path(root: &Path, program: &str) -> PathBuf {
    if !program.contains('/') {
        return PathBuf::from(program);
    }

    let path = root.join(program);
    if cfg!(windows) {
        path.with_extension("cmd")
    } else {
        path
    }
}

/// Installs the Node tooling exactly as `package-lock.json` describes it.
fn run_setup() -> ExitCode {
    let root = workspace_root();
    let step = Step {
        name: "setup",
        program: NPM,
        // `npm ci` installs strictly from the lockfile and deletes anything that does not belong,
        // so two machines end up with the same tree. `npm install` would quietly rewrite the
        // lockfile instead, which is the opposite of what a pinned setup wants.
        args: &["ci"],
    };

    println!("--- {} ---", step.name);
    if !run(&root, &step) {
        eprintln!(
            "\nxtask: `{NPM} ci` failed. If it was not found at all, install Node first; the \
             version this project expects is in .nvmrc."
        );
        return ExitCode::FAILURE;
    }

    println!("\nNode tooling installed");
    ExitCode::SUCCESS
}

/// Checks that the Node tooling is installed and not older than `package-lock.json`.
///
/// npm records the tree it installed in `node_modules/.package-lock.json`, so comparing that file's
/// timestamp against the lockfile is enough to notice a stale install. It costs two `stat` calls,
/// which is why the gate can afford it on every run: a no-op `npm install` was measured at roughly
/// two seconds, more than every Rust step put together.
fn node_tooling_state(root: &Path) -> Result<(), String> {
    let lockfile = root.join("package-lock.json");
    let installed = root.join("node_modules").join(".package-lock.json");

    if !installed.exists() {
        return Err(format!(
            "xtask: the Node tooling is not installed.\n\n    {} does not exist.",
            installed.display()
        ));
    }

    if modified_time(&lockfile)? > modified_time(&installed)? {
        return Err(format!(
            "xtask: the installed Node tooling is out of date.\n\n    {} is newer than {}.",
            lockfile.display(),
            installed.display()
        ));
    }

    Ok(())
}

/// Last modification time of `path`, or a message naming the file that could not be read.
fn modified_time(path: &Path) -> Result<SystemTime, String> {
    path.metadata()
        .and_then(|metadata| metadata.modified())
        .map_err(|error| format!("xtask: could not read {}: {error}", path.display()))
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
    println!("  setup    Install the Node tooling the gate needs, from package-lock.json");
    println!("  help     Show this message");
}
