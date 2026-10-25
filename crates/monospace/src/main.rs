//! The interactive application: the loop, the terminal it runs on, and the way out.
//!
//! **The only place the two halves meet.** `callbacks` holds what the application does, and
//! `frame` holds what one frame shows; this module holds the loop that runs them and the
//! terminal they run on.
//!
//! The name is the one `AGENTS.md` reserves for the interactive application, and the binary is
//! the whole of it: there is no library here, and the state is private to it.

mod callbacks;
mod config;
mod context;
mod diagram_widget;
mod event;
mod frame;
mod menu;
mod shortcuts;
mod state;
mod status;
mod vocabulary;

use std::process::ExitCode;

use monospace_editing::Session;
use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::{RunConfig, run_tui};

use crate::config::Config;
use crate::context::AppGlobal;
use crate::state::AppState;
use crate::vocabulary::AppError;

/// The application's own demonstration, embedded at compile time so the run with no path works
/// from any working directory and from a binary copied outside a checkout.
///
/// **A copy of the command line's file, and not a path to it.** The command line's demonstration
/// acts on four offsets of its exact diagram, so the two files may diverge: the application only
/// draws what it is given, and a sample chosen for a small region is a later question rather than
/// a coupling declared here.
const DEMO: &str = include_str!("../assets/demo.json");

/// What the application takes on the command line.
const USAGE: &str = "usage: monospace [path]";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("monospace: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Runs the loop, and reports whatever ended it.
fn run() -> Result<(), AppError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = match args.as_slice() {
        [] => None,
        [path] => Some(path.as_str()),
        [_, extra, ..] => {
            return Err(format!("unexpected argument `{extra}`\n{USAGE}").into());
        }
    };

    // **Read and parsed before the screen is taken**, so a path that cannot be read is reported
    // on the terminal a person is still looking at. The failure is not a frame, and no snapshot
    // can show it.
    let mut state = AppState::new(start(path)?);
    let mut global = AppGlobal::new(Config::default(), monospace_glyph_sets::all());
    let cfg = RunConfig::default()?.poll(PollCrossterm).poll(PollRendered);

    let outcome = run_tui(
        callbacks::init,
        callbacks::render,
        callbacks::event,
        callbacks::error,
        &mut global,
        &mut state,
        cfg,
    );

    // The failure the error handler took is written out here, with the terminal back: writing it
    // while the loop holds the screen would put it where the next frame paints over it.
    match global.take_failure() {
        Some(failure) => Err(failure),
        None => outcome,
    }
}

/// The session the application starts over: the description at `path`, or its own demonstration.
///
/// **The path is read here and the text parsed in one place**, so a file and the demonstration
/// take the same reader and there is no second way into the format. The diagram the parse hands
/// back goes straight into the session and is never named: the editing layer takes it, and the
/// application asks the session for it when it draws.
fn start(path: Option<&str>) -> Result<Session, AppError> {
    let text = match path {
        None => DEMO.to_owned(),
        Some(path) => std::fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?,
    };

    Ok(Session::new(monospace_description::parse(&text)?))
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;
    use std::path::PathBuf;

    use super::start;

    /// Writes `contents` to a fresh file under the system temp directory, named after the calling
    /// test so parallel tests never collide.
    fn write_description(name: &str, contents: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("monospace-test-{name}-{}.json", std::process::id()));
        std::fs::File::create(&path)
            .and_then(|mut file| file.write_all(contents.as_bytes()))
            .expect("temp description file should be writable");
        path
    }

    #[test]
    fn the_application_s_own_demonstration_is_a_description_it_reads() {
        assert!(
            start(None).is_ok(),
            "the embedded demonstration is not a description this build reads"
        );
    }

    #[test]
    fn the_description_at_the_path_is_read() {
        let path = write_description(
            "reading",
            r#"{
                "next_id": 2,
                "shapes": [ { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 },
                              "size": { "width": 4, "height": 3 }, "stroke": "light" } ]
            }"#,
        );

        assert!(
            start(Some(path.to_str().expect("the temp path is UTF-8"))).is_ok(),
            "a well-formed description at a path was refused"
        );
    }
}
