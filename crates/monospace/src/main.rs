//! The interactive application: the loop, and the terminal it runs on.
//!
//! [`docs/application-model.md`](../../../docs/application-model.md) is the design this crate
//! implements, and
//! [`specs/174-a-full-screen-application-whose-menu-hol.md`](../../../specs/174-a-full-screen-application-whose-menu-hol.md)
//! is the specification it was built against.
//!
//! # Design notes
//!
//! **The loop is four lines and holds nothing.** Draw a frame, ask for an event, turn the event
//! into an action, carry the action out. Everything else — what a control is, where it was drawn,
//! where the keyboard is, which region holds it — belongs to [`chrome`] or [`screen`], and the
//! loop is the one place where none of that is decided. This is the shape the spike wrote in both
//! directions in about forty lines, and it is the shape the specification says this application
//! already has.
//!
//! **A frame is drawn before the first event is read**, so the screen the events arrive against is
//! the one the user can see. The alternative reads first and draws second, which on a slow
//! connection is an application that looks like it did not start.
//!
//! **The terminal guard is held by `main` and not by the loop**, and that is the whole of rules 2
//! and 3: a panic unwinding out of [`run`] passes through the binding here and drops it, so the
//! terminal is given back whether the loop returned, whether the way out was taken, or whether
//! something inside it fell over. The loop itself says nothing about giving anything back, which is
//! what makes `a_panic_through_the_loop_gives_the_terminal_back_without_a_line` a test rather
//! than a claim.

mod app;
mod chrome;
mod event;
#[cfg(test)]
mod pictures;
mod screen;
mod terminal;

use std::{
    io::{self, stdout},
    process::ExitCode,
};

use crossterm::event::Event;
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    app::App,
    event::{CrosstermEvents, EventSource, action_of},
    terminal::{Crossterm, TerminalGuard},
};

fn main() -> ExitCode {
    let mut terminal = match Terminal::new(CrosstermBackend::new(stdout())) {
        Ok(terminal) => terminal,
        Err(error) => {
            eprintln!("monospace: the terminal could not be opened: {error}");
            return ExitCode::FAILURE;
        }
    };

    // Bound here and dropped on every way out of `main`, including one that arrives as a panic
    // unwinding through `run` below.
    let guard = match TerminalGuard::take(Box::new(Crossterm)) {
        Ok(guard) => guard,
        Err(error) => {
            eprintln!("monospace: the terminal could not be taken: {error}");
            return ExitCode::FAILURE;
        }
    };

    let outcome = run(&mut terminal, &mut App::default(), &mut CrosstermEvents);
    drop(guard);

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("monospace: {error}");
            ExitCode::FAILURE
        }
    }
}

/// The error `run` answers with, so that one loop serves every backend.
///
/// **`CrosstermBackend`'s error is already `io::Error` and `TestBackend`'s cannot happen at all**,
/// which is the whole of the difference: a test that renders the screen has no terminal to fail
/// against. A trait is what lets [`run`] stay one function rather than one per backend, and the
/// bound it replaces (`io::Error: From<B::Error>`) is not satisfied by the second of the two.
trait IntoTerminalError {
    /// This error, as the one the loop reports.
    fn into_terminal_error(self) -> io::Error;
}

impl IntoTerminalError for io::Error {
    fn into_terminal_error(self) -> io::Error {
        self
    }
}

impl IntoTerminalError for std::convert::Infallible {
    /// The only value of this type is the one that says there was none, and there is no error to
    /// build.
    fn into_terminal_error(self) -> io::Error {
        match self {}
    }
}

/// The loop, from a whole screen to the way out.
///
/// Generic over the backend and over the event source so that both can be stood in for: the
/// backend is what turns the screen into a buffer with no terminal involved, and the source is
/// what lets a test hand the loop a list of events — or a panic.
fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    events: &mut impl EventSource,
) -> io::Result<()>
where
    B::Error: IntoTerminalError,
{
    loop {
        terminal
            .draw(|frame| screen::draw(frame, app.chrome_mut()))
            .map_err(B::Error::into_terminal_error)?;
        let event = events.next()?;
        offer(app, &event);
        if !app.running() {
            return Ok(());
        }
    }
}

/// One event's whole trip: it becomes an action, and the action is the only thing that reaches the
/// state.
///
/// **It is here and not on `App`, and private, and both matter.** An `App` that took an `Event`
/// would name `crossterm`'s type in its own API, and the application's state would depend on the
/// terminal library — the one thing `App` is not. And the two ends are already separated:
/// `action_of` turns a foreign event into one of the chrome's actions, and `apply` carries an
/// action out, each with its own tests in its own module. **What is left here is the join**, and
/// the join is the one thing neither of them can prove alone, which is why this is where a click
/// and its key are shown arriving at the same state.
///
/// **Private because it has exactly one caller.** `run` is the loop and this is its second line of
/// work; anything else reaching for it would be a caller the trip was not written for.
fn offer(app: &mut App, event: &Event) {
    if let Some(action) = action_of(app.chrome(), event) {
        app.apply(action);
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{
        Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };
    use ratatui::{Terminal, backend::TestBackend};

    use super::{offer, run, screen};
    use crate::{
        app::App,
        chrome::{ControlId, Focus},
        event::EventSource,
        terminal::{TerminalGuard, TerminalIo},
    };
    use std::{cell::RefCell, io, rc::Rc};

    /// An event source that hands out a list and then stops answering.
    ///
    /// **Ending the list is an error rather than an endless stream of nothing**, so a test that
    /// forgets to take the way out fails on a reported error instead of hanging.
    struct Scripted {
        events: std::vec::IntoIter<Event>,
    }

    impl Scripted {
        fn of(events: Vec<Event>) -> Self {
            Self {
                events: events.into_iter(),
            }
        }
    }

    impl EventSource for Scripted {
        fn next(&mut self) -> io::Result<Event> {
            self.events
                .next()
                .ok_or_else(|| io::Error::other("the script ran out of events"))
        }
    }

    /// A source whose next event falls over, which is the only way to walk the loop into a panic.
    struct Falling;

    impl EventSource for Falling {
        fn next(&mut self) -> io::Result<Event> {
            panic!("the event source fell over")
        }
    }

    /// Draws one frame, which is what stores the geometry a click is tested against.
    ///
    /// **A control the frame has not drawn has no rectangle and answers no click.** That is D3
    /// holding rather than D3 failing, and it is why a test that clicks something an overlay just
    /// revealed has to draw again first: the loop does that on its own, between one event and the
    /// next, and a test calling `offer` has to do it by hand.
    fn draw(app: &mut App) {
        let mut terminal = Terminal::new(TestBackend::new(40, 10)).expect("a test backend");
        terminal
            .draw(|frame| screen::draw(frame, app.chrome_mut()))
            .expect("the frame is drawn");
    }

    /// An application with one frame drawn, which is what gives the controls a rectangle.
    fn drawn() -> App {
        let mut app = App::default();
        draw(&mut app);
        app
    }

    /// **Rule 9's join, which is the whole of what this module's `offer` exists to be proved by.**
    ///
    /// The four ways in are named by the specification and they are four different events — two
    /// keys, a click and the focused control's own key — and the claim is that all four leave the
    /// same state. Neither `event.rs` nor `app.rs` can show that: the first knows what an event
    /// becomes and the second what an action does, and "the same action, from four different
    /// events" is a claim about the pair.
    #[test]
    fn the_way_out_is_one_action_reached_by_a_key_a_click_and_the_focused_control() {
        let by_key = {
            let mut app = drawn();
            offer(&mut app, &chord(KeyModifiers::CONTROL, KeyCode::Char('q')));
            app.running()
        };
        let by_control_c = {
            let mut app = drawn();
            offer(&mut app, &chord(KeyModifiers::CONTROL, KeyCode::Char('c')));
            app.running()
        };
        // A click on the way-out control, which is the drop-down's only entry and is at the
        // coordinates the last frame drew it at.
        let by_click = {
            let mut app = drawn();
            offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::F(9)));
            draw(&mut app);
            offer(
                &mut app,
                &mouse_at(MouseEventKind::Down(MouseButton::Left), QUIT.0, QUIT.1),
            );
            offer(
                &mut app,
                &mouse_at(MouseEventKind::Up(MouseButton::Left), QUIT.0, QUIT.1),
            );
            app.running()
        };
        // And the same control's own key, while the drop-down has it selected.
        let by_its_own_key = {
            let mut app = drawn();
            offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::F(9)));
            offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::Enter));
            app.running()
        };

        assert!(!by_key, "Ctrl+Q did not reach the way out");
        assert!(!by_control_c, "Ctrl+C did not reach the way out");
        assert!(
            !by_click,
            "a click on the control did not reach the way out"
        );
        assert!(
            !by_its_own_key,
            "the focused way-out control's own key did not reach the way out"
        );
    }

    /// **The same claim for the control that is not the way out**, and it is the one that can fail
    /// quietly: a click and a key that each applied an action of their own would leave two
    /// different states that both look right.
    #[test]
    fn a_click_on_the_control_and_its_key_apply_the_same_action() {
        let mut by_click = drawn();
        for kind in [
            MouseEventKind::Down(MouseButton::Left),
            MouseEventKind::Up(MouseButton::Left),
        ] {
            offer(&mut by_click, &mouse_at(kind, FILE.0, FILE.1));
        }

        // The keyboard reaches the same control by the focus stack rather than by a key: `F9` and
        // then `Esc` leave it on `File`, and `Enter` is what a focused control answers.
        let mut by_key = drawn();
        offer(&mut by_key, &chord(KeyModifiers::NONE, KeyCode::F(9)));
        offer(&mut by_key, &chord(KeyModifiers::NONE, KeyCode::Esc));
        assert_eq!(
            by_key.chrome().focus(),
            Focus::Bar(ControlId::File),
            "`Esc` did not leave the keyboard on the control that opened the drop-down"
        );
        offer(&mut by_key, &chord(KeyModifiers::NONE, KeyCode::Enter));

        assert_eq!(
            by_click.chrome().drop_down_is_open(),
            by_key.chrome().drop_down_is_open(),
            "the click and the key reached the same control by different paths"
        );
        assert_eq!(by_click.chrome().focus(), by_key.chrome().focus());
    }

    /// **Rule 16 through the join**, which is where `F9` and `Esc` live once the application does
    /// not know what a key is.
    #[test]
    fn f9_opens_the_drop_down_and_escape_closes_it() {
        let mut app = drawn();
        assert!(!app.chrome().drop_down_is_open());

        offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::F(9)));
        assert!(app.chrome().drop_down_is_open());

        offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::Esc));
        assert!(
            !app.chrome().drop_down_is_open(),
            "Esc did not close the drop-down that F9 opened"
        );
    }

    /// A mouse button at a point on the screen.
    fn mouse_at(kind: MouseEventKind, x: u16, y: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// Where the draw pass puts the bar's controls, for a test that means to click one.
    const FILE: (u16, u16) = (1, 0);
    /// Where it puts the drop-down's only entry, once the drop-down is open.
    const QUIT: (u16, u16) = (3, 2);

    /// A terminal that only records whether it was given back, because the loop cannot be walked
    /// into a panic without one that is safe to do that to.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    struct State {
        raw: bool,
        alternate: bool,
        mouse: bool,
    }

    /// A stand-in terminal whose state a test can hold onto while the guard owns the terminal.
    struct Harmless(Rc<RefCell<State>>);

    impl TerminalIo for Harmless {
        fn take(&mut self) -> io::Result<()> {
            let mut state = self.0.borrow_mut();
            state.raw = true;
            state.alternate = true;
            Ok(())
        }
        fn give_back(&mut self) -> io::Result<()> {
            *self.0.borrow_mut() = State::default();
            Ok(())
        }
        fn take_mouse(&mut self) -> io::Result<()> {
            self.0.borrow_mut().mouse = true;
            Ok(())
        }
        fn give_up_mouse(&mut self) -> io::Result<()> {
            self.0.borrow_mut().mouse = false;
            Ok(())
        }
    }

    fn chord(modifiers: KeyModifiers, code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    /// Runs `body` with the report a panic prints kept off the test's output.
    fn unwinding_silently(body: impl FnOnce()) {
        let report = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
        std::panic::set_hook(report);
        assert!(outcome.is_err(), "the body was expected to unwind");
    }

    /// The terminal was taken by whoever holds the guard, which is what `main` does — and it is
    /// what makes the panic unwind *through* the guard rather than around one the loop made for
    /// itself.
    #[test]
    fn a_panic_through_the_loop_gives_the_terminal_back_without_a_line() {
        let terminal = Rc::new(RefCell::new(State::default()));

        unwinding_silently(|| {
            let guard = TerminalGuard::take(Box::new(Harmless(terminal.clone())))
                .expect("the terminal is taken");
            let mut screen = Terminal::new(TestBackend::new(40, 10)).expect("a test backend");
            let _ = run(&mut screen, &mut App::default(), &mut Falling);
            drop(guard);
        });

        assert_eq!(
            *terminal.borrow(),
            State::default(),
            "the loop fell over and took the terminal with it: nothing in the loop said a word \
             about giving it back, so this is the guard `main` holds doing it on the way past"
        );
    }

    /// **The two halves of the claim, which is what this test exists to hold together.**
    ///
    /// `Ctrl+C` is the same key the shell would turn into a signal, and the reason it reaches the
    /// loop as a key is that **raw mode turns `ISIG` off**. The first half is therefore that raw mode
    /// is on while the loop runs; the second is that a `Ctrl+C` key event ends the loop rather than
    /// reaching nothing. Neither alone is the claim, and the two are checked against one terminal in
    /// one test because the link between them is the terminal's input mode rather than anything this
    /// crate decides.
    ///
    /// What this cannot show is that no signal was raised: a signal is delivered to the process and not
    /// to a buffer, and a test that watched for one would be testing `crossterm` and the platform
    /// rather than this loop. What it shows is that both ends hold at once — raw mode on, and the key
    /// arriving as a key.
    #[test]
    fn control_c_reaches_the_state_as_a_key_and_not_as_a_signal() {
        let terminal = Rc::new(RefCell::new(State::default()));
        let guard =
            TerminalGuard::take(Box::new(Harmless(terminal.clone()))).expect("the terminal");

        assert!(
            terminal.borrow().raw,
            "raw mode is off, so ISIG is on and Ctrl+C would be a signal that ends this process \
         before the loop saw a key"
        );

        let mut screen = Terminal::new(TestBackend::new(40, 10)).expect("a test backend");
        let mut app = App::default();
        let mut events = Scripted::of(vec![chord(KeyModifiers::CONTROL, KeyCode::Char('c'))]);
        let outcome = run(&mut screen, &mut app, &mut events);

        assert!(
            outcome.is_ok(),
            "the loop did not end on Ctrl+C: {outcome:?}"
        );
        assert!(
            !app.running(),
            "Ctrl+C arrived as a key and reached nothing: the way out is the same action a click and \
         `Enter` reach"
        );

        drop(guard);
    }

    #[test]
    fn the_loop_ends_on_the_way_out_and_on_nothing_else() {
        let mut terminal = Terminal::new(TestBackend::new(40, 10)).expect("a test backend");
        let mut app = App::default();

        let outcome = run(
            &mut terminal,
            &mut app,
            &mut Scripted::of(vec![
                chord(KeyModifiers::NONE, KeyCode::F(9)),
                chord(KeyModifiers::NONE, KeyCode::Char('z')),
                chord(KeyModifiers::CONTROL, KeyCode::Char('q')),
            ]),
        );
        assert!(
            outcome.is_ok(),
            "the loop did not end on the way out: {outcome:?}"
        );
        assert!(!app.running());
    }

    /// **A key that names no action does not end the loop.** The script above would have had to
    /// report the exhausted source as an error to get this far, so an application that stopped on
    /// any key at all would have stopped before `Ctrl+Q` and said so here instead.
    #[test]
    fn a_key_that_names_no_action_does_not_end_the_loop() {
        let mut terminal = Terminal::new(TestBackend::new(40, 10)).expect("a test backend");
        let mut app = App::default();

        let mut events = Scripted::of(vec![chord(KeyModifiers::NONE, KeyCode::Char('z'))]);
        let outcome = run(&mut terminal, &mut app, &mut events);
        assert!(
            outcome.is_err(),
            "the loop ended on a key that names no action"
        );
        assert!(app.running());
    }
}
