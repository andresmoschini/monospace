//! What the application does to the terminal while it runs, and the guard that undoes it.
//!
//! **The terminal is given back by a guard and not by a line**, so that a panic unwinding through
//! the loop restores it. The alternative — the loop returning normally on every exit and a line
//! after it — is the shape that leaves a terminal in raw mode with the cursor hidden and the user
//! typing into a shell that echoes nothing, and nothing about it looks like a bug until somebody
//! types `reset`. **A framework's panic hook would do the same job**, and is not used: the hook is
//! installed by `ratatui::run` and cannot be reached from a test, while the guard is an ordinary
//! value a test can hold, drop, and watch.
//!
//! **The four operations are a trait and not four `crossterm` calls**, and that is the only reason
//! the terminal is behind one. Rules 1 through 4 are all claims about state that exists only on a
//! real terminal, and the one thing that makes them checkable is standing in for it. The stand-in
//! the tests use is `TerminalState`, which models the four things the rules name — the input mode,
//! the screen, the cursor and the mouse — and holds them as flags, so a test asserts on **the
//! state the terminal was left in** rather than on the order two calls arrived in. Asserting the
//! order would pass for an implementation that reported success and changed nothing.

use std::io::{self, stdout};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

/// What the loop does to the terminal, in four operations.
///
/// **Splitting `take` from `take_mouse` is what rule 4 is**: the mouse is captured while the
/// application runs and released when it ends, which is a separate claim from the screen being
/// taken and given back, and folding the two together would make a test of one a test of the other.
pub(crate) trait TerminalIo {
    /// Takes the whole screen, hides the cursor and puts the terminal in raw mode.
    fn take(&mut self) -> io::Result<()>;

    /// Puts back the screen, the cursor and the input mode, in the inverse of [`take`](Self::take).
    fn give_back(&mut self) -> io::Result<()>;

    /// Starts reporting mouse events instead of leaving them to the terminal to interpret.
    fn take_mouse(&mut self) -> io::Result<()>;

    /// Stops reporting mouse events.
    fn give_up_mouse(&mut self) -> io::Result<()>;
}

/// The real terminal, which is `crossterm` and nothing else.
///
/// **Read directly rather than through `ratatui`'s re-export**, because two majors of `crossterm`
/// cannot exchange types even when their names match, and naming it in this crate's manifest with
/// `=` is the whole of the mitigation — which is only a mitigation if the name is one this crate
/// resolves rather than one a framework re-exports on its behalf.
pub(crate) struct Crossterm;

impl TerminalIo for Crossterm {
    fn take(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, Hide)
    }

    /// **The cursor is shown before the screen is left, and raw mode is dropped last.** A terminal
    /// that is out of raw mode before the alternate screen is left would redraw the shell over the
    /// cursor's own position, which is where `show` is trying to put it.
    fn give_back(&mut self) -> io::Result<()> {
        execute!(stdout(), Show, LeaveAlternateScreen)?;
        disable_raw_mode()
    }

    fn take_mouse(&mut self) -> io::Result<()> {
        execute!(stdout(), EnableMouseCapture)
    }

    fn give_up_mouse(&mut self) -> io::Result<()> {
        execute!(stdout(), DisableMouseCapture)
    }
}

/// Owns the terminal for as long as it is alive, and gives it back when it stops being.
///
/// **Held by the caller of the loop rather than by the loop itself**, which is what makes rule 3's
/// test possible: a panic unwinding out of the loop passes through the code that owns the guard and
/// drops it on the way, with nothing in the loop having said a word about it.
pub(crate) struct TerminalGuard {
    io: Box<dyn TerminalIo>,
    /// Whether the terminal was successfully taken, and is therefore ours to give back.
    ///
    /// **Set between the two calls rather than after them**, so that a failure to capture the mouse
    /// still gives back the screen: the terminal was already in raw mode at that point, and a guard
    /// that gave nothing back because its second call failed is the exact state rule 2 exists to
    /// prevent.
    taken: bool,
}

impl TerminalGuard {
    /// Takes the terminal and the mouse, and returns the guard that will give both back.
    ///
    /// # Errors
    ///
    /// A terminal that could not be taken was never taken, and
    /// whatever a failure left behind is given back on the way out of this function.
    pub(crate) fn take(io: Box<dyn TerminalIo>) -> io::Result<Self> {
        let mut guard = Self { io, taken: false };
        guard.io.take()?;
        guard.taken = true;
        guard.io.take_mouse()?;
        Ok(guard)
    }
}

impl Drop for TerminalGuard {
    /// **Both errors are dropped on purpose.** `Drop` has nowhere to report them to, and a panic
    /// raised while the stack is already unwinding would replace whatever the user was told first
    /// with a second failure about the same terminal.
    fn drop(&mut self) {
        if !self.taken {
            return;
        }
        let _ = self.io.give_up_mouse();
        let _ = self.io.give_back();
    }
}
