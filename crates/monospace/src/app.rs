//! The application as the framework sees it: the global state, the event type, and the four
//! callbacks `run_tui` drives.
//!
//! # Design notes
//!
//! **The loop is `run_tui`'s and not this module's.** What used to be here — the `while` that
//! drew, read an event, parsed it and updated the model — is now a call in `main`, and the four
//! functions below are what the loop calls back into. **What the framework took with it:** the
//! terminal guard, the redraw flag and the running state. `Control::Quit` is the first,
//! `Control::Changed` and `Control::Unchanged` are the second, and the third was the loop's own
//! question. **What breaks if it changes:** a `Global` that stops being `Default` forces `main`
//! to build it by hand, and the four callbacks stop being plain `fn` pointers the day one of them
//! needs a parameter — `run_tui` takes four of those, and a closure is not one.
//!
//! **`Message` stays the application's vocabulary and `Control` stays the framework's.** The
//! first says what a key means to this application; the second says what the loop does about it.
//! Folding the two together would make the event handler the only place a key is named, and the
//! tests in `message.rs` are the reason it is not.

use std::error::Error;

use crossterm::event::Event;
use rat_salsa::event::RenderedEvent;
use rat_salsa::{Control, SalsaAppContext, SalsaContext};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::message::Message;
use crate::view;

/// What the application can hand back, which is anything `std` can name.
///
/// `Box<dyn Error>` rather than a local enum: `run_tui` asks for `From<io::Error>` and
/// `RunConfig::default()` for `From<crossbeam_channel::TryRecvError>`, and the boxed error covers
/// both through `std`'s blanket conversion — a local enum would restate two conversions for a name
/// this application has nowhere to keep.
///
/// **Named `AppError` and not `Error`**, because `Error` is already the trait `std` defines, and a
/// module cannot hold both under one name — the alias would shadow the very trait it wraps.
pub(crate) type AppError = Box<dyn Error>;

/// Globally accessible state: the salsa context, and what the chrome will add to it.
pub(crate) struct Global {
    ctx: SalsaAppContext<AppEvent, AppError>,
}

impl Default for Global {
    /// The global state the loop starts from, which is the context and nothing else yet.
    fn default() -> Self {
        Self {
            ctx: SalsaAppContext::default(),
        }
    }
}

impl SalsaContext<AppEvent, AppError> for Global {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<AppEvent, AppError>) {
        self.ctx = app_ctx;
    }

    fn salsa_ctx(&self) -> &SalsaAppContext<AppEvent, AppError> {
        &self.ctx
    }
}

/// What the loop distributes: a terminal event, and the notice that a frame went out.
#[derive(Debug)]
pub(crate) enum AppEvent {
    /// One event from the terminal.
    Event(Event),
    /// The frame was rendered; widget state that follows the area can relocate now.
    Rendered,
}

impl From<Event> for AppEvent {
    fn from(value: Event) -> Self {
        Self::Event(value)
    }
}

impl From<RenderedEvent> for AppEvent {
    fn from(_: RenderedEvent) -> Self {
        Self::Rendered
    }
}

/// Called once before the first frame, to give the state its focus.
///
/// **The `Result` is the framework's signature and not this function's.** `run_tui` takes four
/// `fn` pointers that return `Result`, so a callback that cannot fail still names the type — and
/// the day one of them can, the signature is already what the loop expects.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn init(_state: &mut (), _ctx: &mut Global) -> Result<(), AppError> {
    Ok(())
}

/// Draws one frame into the buffer the loop hands over.
///
/// **The `Result` is the framework's signature and not this function's**, as it is in `init`.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn render(
    area: Rect,
    buf: &mut Buffer,
    _state: &mut (),
    _ctx: &mut Global,
) -> Result<(), AppError> {
    view::draw(area, buf);
    Ok(())
}

/// Reads one event as a control decision, which is what the loop does next.
///
/// **The `Result` is the framework's signature and not this function's**, as it is in `init`.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn event(
    event: &AppEvent,
    _state: &mut (),
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, AppError> {
    if let AppEvent::Event(event) = event
        && let Some(message) = Message::parse(event)
    {
        return Ok(match message {
            Message::Quit => Control::Quit,
            Message::Resize => Control::Changed,
        });
    }

    Ok(Control::Continue)
}

/// Hands a failure back, which ends the loop the way a failure in `event` would.
pub(crate) fn error(
    error: AppError,
    _state: &mut (),
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, AppError> {
    Err(error)
}
