//! The terminal the application drives, and the guard that hands it back.
//!
//! **Two things live here that are one thing each.** `Screen` is what the application asks of a
//! terminal, `Crossterm` is the terminal this process was given, and `GivenBack` is what makes the
//! asking stop mattering — because the restore happens when the guard is dropped, on every way out
//! including the one a panic takes.

use std::io::{self, StdoutLock, Write};

use crossterm::event::{self, Event};
use crossterm::style::Print;
use crossterm::terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, queue};

/// What the application asks of a terminal.
///
/// **A trait rather than a struct because two of the rules are about what the terminal was asked
/// rather than about what it holds.** Raw mode is a request to the operating system and not a
/// sequence of bytes, so a screen that recorded the bytes written to it could show that the
/// alternate screen was entered and could not show that the terminal was put into raw mode at all.
/// A terminal answers both by being one.
pub(crate) trait Screen {
    /// Takes the whole screen and puts the terminal into the mode events are read in.
    fn take(&mut self) -> io::Result<()>;

    /// The screen's own size, in columns and rows.
    ///
    /// **Asked at draw time rather than carried from an event**, so a resize that has been coalesced
    /// is answered anyway: in a program whose clicks resolve to shapes a stale size is a wrong
    /// hit-test rather than a wrong picture.
    fn size(&self) -> io::Result<(u16, u16)>;

    /// Writes `text` over the whole screen.
    fn draw(&mut self, text: &str) -> io::Result<()>;

    /// The next event, or `None` where the terminal has no further event to give.
    fn next_event(&mut self) -> io::Result<Option<Event>>;

    /// Gives back the screen, the cursor and the input mode the application found.
    fn give_back(&mut self) -> io::Result<()>;
}

/// The mode a terminal's keys arrive in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    /// A line at a time, which is the mode a shell hands a program over in.
    Cooked,
    /// One key at a time, which is the mode an event loop reads in.
    Raw,
}

/// A terminal as `crossterm` reaches it.
pub(crate) struct Crossterm {
    /// Where the screen is written.
    ///
    /// **Locked once for the life of the application** rather than locked per row, because these are
    /// the writers `execute!` and `queue!` go through and two of them at once interleave.
    out: StdoutLock<'static>,
    /// The mode the terminal's keys arrived in, which is the one they are left in.
    found: Mode,
}

impl Crossterm {
    /// The terminal this process was started from.
    pub(crate) fn stdout() -> Self {
        Self {
            out: io::stdout().lock(),
            found: Mode::Cooked,
        }
    }
}

impl Screen for Crossterm {
    fn take(&mut self) -> io::Result<()> {
        // **What was found is read before anything is changed**, which is the only order in which
        // _the input mode the application found is the one it leaves behind_ can hold: a terminal
        // already reading keys one at a time is put back into that rather than cooked.
        self.found = if terminal::is_raw_mode_enabled()? {
            Mode::Raw
        } else {
            Mode::Cooked
        };
        terminal::enable_raw_mode()?;
        execute!(
            self.out,
            EnterAlternateScreen,
            cursor::Hide,
            event::EnableMouseCapture
        )
    }

    fn size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }

    fn draw(&mut self, text: &str) -> io::Result<()> {
        // **The whole screen is cleared and written again rather than diffed against what was there.**
        // That is the cost the specification's D6 names for depending on no widget framework, and the
        // smallest thing that can be right: an ASCII canvas scrolled by a row has nearly every cell
        // changed, which is the case a double buffer exists for.
        queue!(self.out, terminal::Clear(ClearType::All))?;
        for (row, line) in text.lines().enumerate() {
            let row = u16::try_from(row).expect("a screen holds fewer rows than a `u16` counts");
            queue!(self.out, cursor::MoveTo(0, row), Print(line))?;
        }
        self.out.flush()
    }

    fn next_event(&mut self) -> io::Result<Option<Event>> {
        // **Blocking, because there is nothing to do until an event arrives** and a loop that polled
        // would spend the application's life asking. `None` is where a screen has run out of events,
        // which is a test double's answer rather than a terminal's.
        event::read().map(Some)
    }

    fn give_back(&mut self) -> io::Result<()> {
        // **The reverse of `take`, and in the same order reversed**, so a terminal that fails to come
        // back is left with less held than one that fails to be taken.
        execute!(
            self.out,
            event::DisableMouseCapture,
            cursor::Show,
            LeaveAlternateScreen
        )?;
        match self.found {
            Mode::Raw => terminal::enable_raw_mode(),
            Mode::Cooked => terminal::disable_raw_mode(),
        }
    }
}

/// A terminal as taken, which hands itself back when it is dropped.
///
/// **The reason the application has no line saying it gives the terminal back**, and so the reason
/// a panic ends the program with the terminal as it was found: a line is not reached when the stack
/// unwinds through it, and `Drop` is.
pub(crate) struct GivenBack<'a, S: Screen> {
    screen: &'a mut S,
    taken: bool,
}

impl<'a, S: Screen> GivenBack<'a, S> {
    /// Wraps a screen that has not been taken yet.
    pub(crate) fn of(screen: &'a mut S) -> Self {
        Self {
            screen,
            taken: false,
        }
    }

    /// Takes the screen, and remembers that it was taken however the taking went.
    pub(crate) fn take(&mut self) -> io::Result<()> {
        let taken = self.screen.take();
        self.taken = taken.is_ok();
        taken
    }

    /// The screen as taken.
    pub(crate) fn screen(&mut self) -> &mut S {
        self.screen
    }
}

impl<S: Screen> Drop for GivenBack<'_, S> {
    fn drop(&mut self) {
        // **A screen that was not taken is not given back**, so a terminal the application failed to
        // enter is left as it was found rather than half left: restoring what was never taken takes
        // away the mode the user had.
        if self.taken {
            // **And a failure to restore is dropped rather than reported**, because a `Drop` has
            // nowhere to report it to. A terminal left reading keys one at a time is recoverable with
            // `reset`, where a message the user never sees is not.
            let _ = self.screen.give_back();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

    use super::{Asked, Mode, Recorded, State};
    use crate::app::run;

    /// Every way out of the application, and what the terminal was asked on each of them: taken,
    /// put into the mode events are read in, hidden, given its mouse; and then all four of those
    /// given back in the reverse order.
    const A_WHOLE_RUN: [Asked; 9] = [
        Asked::TakeInputMode,
        Asked::TakeScreen,
        Asked::HideCursor,
        Asked::TakeMouse,
        Asked::Draw,
        Asked::ReleaseMouse,
        Asked::ShowCursor,
        Asked::ReleaseScreen,
        Asked::ReleaseInputMode,
    ];

    /// The same run on a terminal that refused to be written to: everything but the draw, which was
    /// asked for and did not happen.
    const A_WHOLE_RUN_WITHOUT_THE_DRAW: [Asked; 8] = [
        Asked::TakeInputMode,
        Asked::TakeScreen,
        Asked::HideCursor,
        Asked::TakeMouse,
        Asked::ReleaseMouse,
        Asked::ShowCursor,
        Asked::ReleaseScreen,
        Asked::ReleaseInputMode,
    ];

    /// The way out as a key, which is what every run here ends on.
    fn the_way_out() -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE))
    }

    /// The application gives the whole screen back when it ends.
    ///
    /// **The whole transcript is pinned rather than the fact that something was released**, because
    /// a program that gave the screen back and kept the mouse, or gave the mouse back twice, would
    /// satisfy the weaker claim: the list is in the order the questions were asked, which is the only
    /// way it says the restore is the reverse of the taking.
    #[test]
    fn the_application_gives_the_whole_screen_back_when_it_ends() {
        let mut screen = Recorded::scripted([the_way_out()]);

        run(&mut screen).expect("a run over a script of events does not fail");

        assert_eq!(
            screen.asked, A_WHOLE_RUN,
            "the whole screen, and everything taken with it"
        );
    }

    /// A panic ends the application with the terminal as it was found.
    ///
    /// **The panic is caught rather than left to end the test**, because what is under test is what
    /// the terminal was asked on the way out of it, and a test that died there would report nothing
    /// about that. `AssertUnwindSafe` is not a claim that this is unwind-safe: the screen is the
    /// thing being unwound through, and the alternative is a witness nothing can read afterwards.
    #[test]
    fn a_panic_ends_the_application_with_the_terminal_as_it_was_found() {
        let mut screen = Recorded::panicking_on_read();

        let unwound = catch_unwind(AssertUnwindSafe(|| run(&mut screen)));

        assert!(
            unwound.is_err(),
            "the run unwound, which is the way out under test"
        );
        assert_eq!(
            screen.asked, A_WHOLE_RUN,
            "the terminal was given back on the way a panic takes, which a line at the end of the \
             loop would not be reached by"
        );
        assert_eq!(
            screen.input, screen.found,
            "and it is reading keys in the mode it was found in"
        );
        assert!(!screen.mouse, "with the mouse the terminal's own again");
    }

    /// The input mode the application found is the one it leaves behind.
    ///
    /// **Both ways round, and the second is the one a test written against the common case would
    /// miss.** A terminal that was already reading keys one at a time is put back into that, which
    /// is what the read below `take` is for; a program that disabled raw mode on the way out would
    /// satisfy the common case and fail this one. And the mode read in is asserted as well, because
    /// a terminal left as it was found because nothing had changed it would satisfy the second half
    /// of the rule without the application ever having read a key.
    #[test]
    fn the_input_mode_the_application_found_is_the_one_it_leaves_behind() {
        for found in [Mode::Cooked, Mode::Raw] {
            let mut screen = Recorded::scripted([the_way_out()]).found_in(found);

            run(&mut screen).expect("a run over a script of events does not fail");

            assert_eq!(
                screen.read_in,
                [State {
                    input: Mode::Raw,
                    mouse: true
                }],
                "the key was read in the mode the application put the terminal in, whatever it \
                 was found in"
            );
            assert_eq!(
                screen.input, found,
                "and the terminal is left in the mode it was found in"
            );
        }
    }

    /// The terminal gives up mouse capture when the application ends.
    ///
    /// **The cost of holding it is named in the model's layer rather than here**: while the
    /// application is up the terminal's own text selection does not work, so a user who needs to
    /// select and copy with the mouse has the keyboard's selection instead, or quits. Nothing routes
    /// a click yet, and this rule is why that costs something rather than nothing.
    #[test]
    fn the_terminal_gives_up_mouse_capture_when_the_application_ends() {
        let mut screen = Recorded::scripted([the_way_out()]);

        run(&mut screen).expect("a run over a script of events does not fail");

        assert_eq!(
            screen.asked.first(),
            Some(&Asked::TakeInputMode),
            "the transcript is in the order the questions were asked"
        );
        assert!(
            screen.asked.contains(&Asked::TakeMouse),
            "the mouse was the application's while it was up: {:?}",
            screen.asked
        );
        assert!(
            !screen.mouse,
            "and the terminal's own when it ended, which is what rule 2 gives back"
        );
    }

    /// A terminal the application could not take is not given back.
    ///
    /// **Not in the specification's acceptance list, and here because of what it protects.** The
    /// restore is reached only for a screen that was taken, so a terminal whose taking failed half
    /// way is not asked to give back what it never held — which would take away the mode the user
    /// had, in a program whose only promise about the terminal is that it leaves it as it found it.
    #[test]
    fn a_screen_the_application_could_not_take_is_not_given_back() {
        let mut screen = Recorded::failing_to_take();

        let failed = run(&mut screen);

        assert!(
            failed.is_err(),
            "the failure to take the screen is the run's answer"
        );
        assert!(
            screen.asked.is_empty(),
            "and nothing of it was given back: {:?}",
            screen.asked
        );
    }

    /// A screen that answers a draw with an error is given back on that way out too.
    ///
    /// **The `io::Error` way out, which the transcript alone cannot tell from the way out the action
    /// names.** The two end in the same place and the same list, and this is what says the third one
    /// reaches it as well. The one difference is the draw that never happened: the terminal was asked
    /// to write a screen and said no, so the transcript has no `Draw` in it.
    #[test]
    fn an_error_is_also_a_way_out_that_gives_the_terminal_back() {
        let mut screen = Recorded::failing_to_draw();

        let failed = run(&mut screen);

        assert!(failed.is_err(), "the failure to draw is the run's answer");
        assert_eq!(
            screen.asked, A_WHOLE_RUN_WITHOUT_THE_DRAW,
            "and the terminal was given back on that way out as well"
        );
    }
}

/// One thing the application asked of the terminal, named as it was asked.
///
/// **The transcript is flat and in the order of the asking**, because that order is the claim: the
/// restore is the reverse of the taking, and a set would not say so.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Asked {
    /// Put the terminal into the mode events are read in.
    TakeInputMode,
    /// Take the whole screen.
    TakeScreen,
    /// Hide the cursor.
    HideCursor,
    /// Take the mouse, so that clicks arrive as events.
    TakeMouse,
    /// Write a whole screen.
    Draw,
    /// Give the mouse back.
    ReleaseMouse,
    /// Give the cursor back.
    ShowCursor,
    /// Give the whole screen back.
    ReleaseScreen,
    /// Put the input mode back as it was found.
    ReleaseInputMode,
}

/// What the terminal was in each time the application read an event.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct State {
    /// The mode its keys arrived in.
    pub(crate) input: Mode,
    /// Whether the mouse belonged to the application rather than to the terminal.
    pub(crate) mouse: bool,
}

/// The size the terminal reports unless a test says otherwise: the width and height of the picture
/// in the specification's `## Examples`, so a screen a run produces can be compared with it row for
/// row.
#[cfg(test)]
pub(crate) const THE_PICTURE: (u16, u16) = (44, 7);

/// A terminal that records what the application asked of it and answers with a script of events.
///
/// **What the tests need is not a terminal but a witness.** Two of the rules are about what the
/// terminal was asked — that the input mode found is the one left behind, that the mouse is given
/// up — and neither can be seen from the bytes written to it, so this records the asking and models
/// what a terminal does with it.
#[cfg(test)]
pub(crate) struct Recorded {
    /// Everything the application asked, in the order it asked it.
    pub(crate) asked: Vec<Asked>,
    /// Every whole screen the application wrote.
    pub(crate) drawn: Vec<String>,
    /// Every event the application was given.
    pub(crate) arrived: Vec<Event>,
    /// What the terminal was in each time the application read an event.
    pub(crate) read_in: Vec<State>,
    /// The mode the terminal's keys arrive in now.
    pub(crate) input: Mode,
    /// The mode they arrived in before the application was started.
    pub(crate) found: Mode,
    /// Whether the mouse belongs to the application rather than to the terminal.
    pub(crate) mouse: bool,
    /// What the terminal reports its own size to be.
    size: (u16, u16),
    /// The events still to arrive.
    arriving: std::collections::VecDeque<Event>,
    /// What this terminal refuses to do, if anything.
    refuses: Refuses,
}

/// What a [`Recorded`] terminal refuses to do.
///
/// **One field rather than a flag per way of failing**, because the ways are not independent and a
/// test that wants one of them should not have to know that the other two exist: what is refused is
/// a single thing, and a terminal that refuses everything is not a test double worth having.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Refuses {
    /// Nothing: it answers every question.
    Nothing,
    /// Being taken.
    Taking,
    /// Being written to.
    Drawing,
    /// Being read at all, which is a panic rather than an answer.
    Reading,
}

#[cfg(test)]
impl Recorded {
    /// A terminal reporting [`THE_PICTURE`]'s size, whose events arrive in the order given.
    pub(crate) fn scripted(events: impl IntoIterator<Item = Event>) -> Self {
        Self {
            asked: Vec::new(),
            drawn: Vec::new(),
            arrived: Vec::new(),
            read_in: Vec::new(),
            input: Mode::Cooked,
            found: Mode::Cooked,
            mouse: false,
            size: THE_PICTURE,
            arriving: events.into_iter().collect(),
            refuses: Refuses::Nothing,
        }
    }

    /// The same terminal, found in `input` rather than in the mode a shell hands a program over in.
    pub(crate) fn found_in(mut self, input: Mode) -> Self {
        self.input = input;
        self
    }

    /// A terminal whose next read is a panic rather than an event.
    pub(crate) fn panicking_on_read() -> Self {
        Self {
            refuses: Refuses::Reading,
            ..Self::scripted(Vec::new())
        }
    }

    /// A terminal that would not be taken.
    pub(crate) fn failing_to_take() -> Self {
        Self {
            refuses: Refuses::Taking,
            ..Self::scripted(Vec::new())
        }
    }

    /// A terminal that would not be written to.
    pub(crate) fn failing_to_draw() -> Self {
        Self {
            refuses: Refuses::Drawing,
            ..Self::scripted(Vec::new())
        }
    }
}

#[cfg(test)]
impl Screen for Recorded {
    fn take(&mut self) -> io::Result<()> {
        if self.refuses == Refuses::Taking {
            return Err(io::Error::other("this terminal would not be taken"));
        }
        // **The order is the real screen's order**, so the transcript is a claim about this
        // application rather than about a test double's idea of a terminal.
        self.asked.extend([
            Asked::TakeInputMode,
            Asked::TakeScreen,
            Asked::HideCursor,
            Asked::TakeMouse,
        ]);
        self.found = self.input;
        self.input = Mode::Raw;
        self.mouse = true;
        Ok(())
    }

    fn size(&self) -> io::Result<(u16, u16)> {
        Ok(self.size)
    }

    fn draw(&mut self, text: &str) -> io::Result<()> {
        if self.refuses == Refuses::Drawing {
            return Err(io::Error::other("this terminal would not be written to"));
        }
        self.asked.push(Asked::Draw);
        self.drawn.push(text.to_owned());
        Ok(())
    }

    fn next_event(&mut self) -> io::Result<Option<Event>> {
        assert_ne!(
            self.refuses,
            Refuses::Reading,
            "the read this terminal was asked for"
        );
        let event = self.arriving.pop_front();
        if let Some(arrived) = &event {
            self.arrived.push(arrived.clone());
            self.read_in.push(State {
                input: self.input,
                mouse: self.mouse,
            });
        }
        Ok(event)
    }

    fn give_back(&mut self) -> io::Result<()> {
        self.asked.extend([
            Asked::ReleaseMouse,
            Asked::ShowCursor,
            Asked::ReleaseScreen,
            Asked::ReleaseInputMode,
        ]);
        self.input = self.found;
        self.mouse = false;
        Ok(())
    }
}
