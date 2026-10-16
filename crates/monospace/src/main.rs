//! The interactive application: the loop, and the terminal it runs on.

mod app;
mod message;
mod view;

use std::process::ExitCode;

use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::{RunConfig, run_tui};

use crate::app::{AppError, Global, error, event, init, render};

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
    let mut global = Global::default();
    let mut state = ();

    run_tui(
        init,
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::default()?.poll(PollCrossterm).poll(PollRendered),
    )?;

    Ok(())
}
