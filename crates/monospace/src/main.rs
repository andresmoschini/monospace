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
mod event;
mod frame;
mod menu;
mod shortcuts;
mod state;
mod status;
mod vocabulary;

use std::process::ExitCode;

use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::{RunConfig, run_tui};

use crate::config::Config;
use crate::context::AppGlobal;
use crate::state::AppState;
use crate::vocabulary::AppError;

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
    let mut global = AppGlobal::new(Config::default());
    let mut state = AppState::new();
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
