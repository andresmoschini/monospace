//! The interactive application: a full-screen program whose menu holds nothing but the way out.
//!
//! **What this slice proves is that the terminal comes back.** A full-screen loop that hands the
//! terminal back is the part of a TUI that can be wrong on a machine, and nothing above it is worth
//! building until it is known to work, so nothing is drawn here: the screen holds a menu, one leaf,
//! and the leaf is the way out. The crate holds no domain logic and reaches no other crate in the
//! workspace, which is what [`README.md`](../../../README.md) promises of the TUI when it explains
//! why the CLI comes first.
//!
//! The rules this program follows are in [`docs/application-model.md`](../../../docs/application-model.md),
//! and the layers below it are in [`docs/diagram-model.md`](../../../docs/diagram-model.md) and
//! [`docs/model.md`](../../../docs/model.md).
//!
//! # Design notes
//!
//! **The terminal is given back by a guard rather than by a line at the end of the loop.** There are
//! four ways out of this program — the one the way out names, the one an `io::Error` takes, a run
//! whose events run out, and the way a panic unwinds — and a line at the end of the loop is reached
//! by three of them. That is the whole reason [`screen::GivenBack`] owns the screen rather than
//! [`app::run`] holding a reference to it, and it is what
//! `a_panic_ends_the_application_with_the_terminal_as_it_was_found` pins.
//!
//! **A screen is a trait, and the reason is raw mode.** Two of the rules are about what the terminal
//! was asked rather than about what it holds, and raw mode is a request to the operating system
//! rather than a sequence of bytes: a screen that recorded the bytes written to it could show that
//! the alternate screen was entered and could not show that the terminal was put into raw mode at
//! all. A terminal answers both by being one.
//!
//! **The size of the screen is asked at draw time rather than taken from a resize event**, which is
//! what depending on no widget framework costs: an event may have been coalesced, and a stale size
//! in a program whose clicks resolve to shapes is a wrong hit-test rather than a wrong picture.

mod app;
mod screen;

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut terminal = screen::Crossterm::stdout();

    match app::run(&mut terminal) {
        Ok(()) => ExitCode::SUCCESS,
        // **The terminal has already been given back by the time an error reaches here**, because
        // `run` borrows it into a guard that is dropped on the way out — so the message lands on the
        // shell's screen rather than inside the alternate one, where it would vanish along with it.
        Err(error) => {
            eprintln!("monospace: {error}");
            ExitCode::FAILURE
        }
    }
}
