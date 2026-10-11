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

/// The four things rules 1 through 4 name, as flags.
///
/// **It is a model and not a recording, so that a test asks a question about state.** A stand-in
/// that merely wrote down the calls it was given would let an implementation that reported success
/// without changing anything pass every one of the tests here; this one is the terminal, as far as
/// the loop is concerned, and the assertions are about what it holds afterwards.
///
/// **It exists only for the tests**, because a rule about what a terminal is left holding has no
/// other question to ask it, and the real terminal is asked nothing at all.
#[cfg(test)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "the four flags are the four things rules 1 to 4 name, and a state with fewer \
        fields would not be the terminal"
)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TerminalState {
    /// Raw mode is on: every key arrives, and `ISIG` is off, so `Ctrl+C` is a key and not a signal.
    raw: bool,
    /// The alternate screen is up rather than the shell's own history.
    alternate: bool,
    /// The cursor is hidden rather than sitting where the last draw put it.
    cursor_hidden: bool,
    /// Mouse events are being reported.
    mouse: bool,
}

#[cfg(test)]
impl TerminalState {
    /// What a terminal holds when the application has not touched it, which is the whole of what
    /// "the terminal as it was found" means: there is no other state this application changes.
    const fn found() -> Self {
        Self {
            raw: false,
            alternate: false,
            cursor_hidden: false,
            mouse: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, io, rc::Rc};

    use super::{Crossterm, TerminalGuard, TerminalIo, TerminalState};

    /// A handle the test keeps onto a terminal it does not own.
    ///
    /// **The guard holds the terminal, so a test that cannot reach it could only assert on the
    /// guard's existence.** This is the one way both are held at once, and it is why the trait is
    /// boxed rather than generic: a `TerminalGuard<T>` would force `main` to name its own terminal
    /// type in every signature that mentions the guard.
    #[derive(Debug, Clone, Default)]
    struct Handle(Rc<RefCell<TerminalState>>);

    impl Handle {
        fn state(&self) -> TerminalState {
            *self.0.borrow()
        }
    }

    /// The terminal the tests stand in for: the model above, reached through a handle.
    #[derive(Debug, Default)]
    struct FakeTerminal {
        state: Handle,
    }

    impl TerminalIo for FakeTerminal {
        fn take(&mut self) -> io::Result<()> {
            let mut state = self.state.0.borrow_mut();
            state.raw = true;
            state.alternate = true;
            state.cursor_hidden = true;
            Ok(())
        }

        fn give_back(&mut self) -> io::Result<()> {
            *self.state.0.borrow_mut() = TerminalState::found();
            Ok(())
        }

        fn take_mouse(&mut self) -> io::Result<()> {
            self.state.0.borrow_mut().mouse = true;
            Ok(())
        }

        fn give_up_mouse(&mut self) -> io::Result<()> {
            self.state.0.borrow_mut().mouse = false;
            Ok(())
        }
    }

    /// A guard over a stand-in terminal, with the handle a test watches.
    fn take() -> (Handle, TerminalGuard) {
        let handle = Handle::default();
        let guard = TerminalGuard::take(Box::new(FakeTerminal {
            state: handle.clone(),
        }))
        .expect("the stand-in terminal is taken");
        (handle, guard)
    }

    /// Runs `body` with the report a panic prints kept off the test's output.
    ///
    /// **The panic is still a panic**: `catch_unwind` catches the unwind and the hook is only what
    /// prints it. Silencing it keeps a test that is *about* the unwind from burying its own result
    /// under a backtrace nobody asked for.
    fn unwinding_silently(body: impl FnOnce()) {
        let report = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
        std::panic::set_hook(report);
        assert!(outcome.is_err(), "the body was expected to unwind");
    }

    #[test]
    fn the_application_gives_the_whole_screen_back_when_it_ends() {
        let (terminal, guard) = take();
        let found = TerminalState::found();

        assert_ne!(
            terminal.state(),
            found,
            "the application took nothing, so there is nothing for the test to be about"
        );

        drop(guard);
        assert_eq!(
            terminal.state(),
            found,
            "the screen, the cursor, the input mode or the mouse was left as the application found it"
        );
    }

    #[test]
    fn the_input_mode_the_application_found_is_the_one_it_leaves_behind() {
        let (terminal, guard) = take();

        assert!(
            terminal.state().raw,
            "raw mode was not on while the loop ran"
        );

        drop(guard);
        assert!(
            !terminal.state().raw,
            "the input mode was left in raw mode, so the shell behind this process is unusable \
             until somebody types `reset`"
        );
    }

    #[test]
    fn the_terminal_gives_up_mouse_capture_when_the_application_ends() {
        let (terminal, guard) = take();

        assert!(
            terminal.state().mouse,
            "the mouse was never captured while the loop ran"
        );

        drop(guard);
        assert!(
            !terminal.state().mouse,
            "mouse capture outlived the application"
        );
    }

    #[test]
    fn a_panic_ends_the_application_with_the_terminal_as_it_was_found() {
        let terminal = Handle::default();
        unwinding_silently(|| {
            let _guard = TerminalGuard::take(Box::new(FakeTerminal {
                state: terminal.clone(),
            }))
            .expect("the stand-in terminal is taken");
            panic!("the loop fell over");
        });
        assert_eq!(
            terminal.state(),
            TerminalState::found(),
            "a panic took the terminal with it"
        );
    }

    /// **The mouse capture is released before the screen is given back, and the screen is given
    /// back even when the mouse cannot be.** A terminal left in the alternate screen with the
    /// mouse still reporting into a process that no longer reads it is a terminal the shell behind
    /// it has to be restarted to use.
    #[test]
    fn a_failure_to_release_the_mouse_still_gives_the_screen_back() {
        struct RefusingMouse {
            state: Handle,
        }

        impl TerminalIo for RefusingMouse {
            fn take(&mut self) -> io::Result<()> {
                let mut state = self.state.0.borrow_mut();
                state.raw = true;
                state.alternate = true;
                state.cursor_hidden = true;
                state.mouse = true;
                Ok(())
            }
            fn give_back(&mut self) -> io::Result<()> {
                *self.state.0.borrow_mut() = TerminalState::found();
                Ok(())
            }
            fn take_mouse(&mut self) -> io::Result<()> {
                Ok(())
            }
            fn give_up_mouse(&mut self) -> io::Result<()> {
                Err(io::Error::other("the mouse will not be released"))
            }
        }

        let terminal = Handle::default();
        let guard = TerminalGuard::take(Box::new(RefusingMouse {
            state: terminal.clone(),
        }))
        .expect("the stand-in terminal is taken");

        drop(guard);
        assert_eq!(
            terminal.state(),
            TerminalState::found(),
            "one failed call stopped the other two from running"
        );
    }

    /// **A terminal that was never taken is never given back.** Giving it back would write escape
    /// sequences to whatever is behind this process — a shell, or a test runner — on the strength of
    /// an attempt that never succeeded.
    #[test]
    fn a_terminal_that_could_not_be_taken_is_not_given_back() {
        struct Refusing;

        impl TerminalIo for Refusing {
            fn take(&mut self) -> io::Result<()> {
                Err(io::Error::other("no terminal here"))
            }
            fn give_back(&mut self) -> io::Result<()> {
                panic!("nothing was taken, so nothing is given back")
            }
            fn take_mouse(&mut self) -> io::Result<()> {
                panic!("the screen was never taken")
            }
            fn give_up_mouse(&mut self) -> io::Result<()> {
                panic!("nothing was taken, so nothing is released")
            }
        }

        assert!(TerminalGuard::take(Box::new(Refusing)).is_err());
    }

    /// The real terminal, where there is one.
    ///
    /// **It is run against the process's own terminal and undone, and the test asserts nothing
    /// about how it went.** A test runner has no terminal of its own, so whether this takes
    /// anything is a fact about where the tests run; what is worth holding is that the real
    /// implementation is written against the same four operations every stand-in is, so the trait
    /// is a boundary rather than a convention.
    #[test]
    fn the_real_terminal_answers_the_four_operations_the_guard_asks_for() {
        let mut terminal = Crossterm;
        if terminal.take().is_ok() {
            let _ = terminal.take_mouse();
            let _ = terminal.give_up_mouse();
            let _ = terminal.give_back();
        }
    }
}
