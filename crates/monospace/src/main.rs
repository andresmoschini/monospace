//! The interactive application: the loop, and the terminal it runs on.
//!
//! The only place the two halves meet. `app` holds what the application is; `framework` holds
//! everything that depends on `rat_salsa` or `ratatui`.

mod about_dialog;
mod callbacks;
mod config;
mod context;
mod event;
mod menu;
mod paint;
mod state;
mod vocabulary;

use std::process::ExitCode;

use monospace_diagram::Diagram;
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

fn run() -> Result<(), AppError> {
    let mut global = AppGlobal::new(Config::default());
    let initial_diagram = Diagram::new();
    let mut state = AppState::new(initial_diagram);
    let cfg = RunConfig::default()?.poll(PollCrossterm).poll(PollRendered);

    run_tui(
        callbacks::init,
        callbacks::render,
        callbacks::event,
        callbacks::error,
        &mut global,
        &mut state,
        cfg,
    )?;

    Ok(())
}
