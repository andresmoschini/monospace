//! The open dialog: the framework's file browser.
//!
//! **This is the spike's version of the dialog.** The slice before it typed a path into one field;
//! this one hands the choice to `rat-widget`'s `FileDialog`, which brings the roots, the directory
//! and file lists, a path field and a `New` button. What the application keeps is the one thing the
//! framework does not know: what to do with the chosen path, and how to say why it failed.
//!
//! **The dialog chooses a path and nothing else.** Reading and parsing are `crate::load`'s, and the
//! action that carries them out is `callbacks::apply`'s, so the dialog does not know what a
//! description is. The chosen path is held in `pending` until the action takes it, because
//! `Action::Load` stays a meaning with no payload, as `Action::Delete` is.
//!
//! **The roots are the application's.** The framework would derive them from the platform, which
//! is neither deterministic nor testable; the dialog is given one `Start` root at the directory it
//! opens in, and the path field still reaches anywhere. A later slice can decide whether the
//! default roots belong in the product.
//!
//! **A failure keeps the browser up.** The reason is drawn on one row below it, so the person can
//! correct the choice without opening the menu again. The framework's dialog has no error slot, so
//! the row is the application's.

use std::path::{Path, PathBuf};

use rat_theme4::WidgetStyle;
use rat_theme4::theme::SalsaTheme;
use rat_widget::event::{Dialog, FileOutcome, HandleEvent};
use rat_widget::file_dialog::{FileDialog, FileDialogState};
use rat_widget::text::HasScreenCursor;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::paragraph::{Paragraph, Wrap};

use crate::vocabulary::AppError;

/// The largest the dialog grows: enough for the three lists and the buttons, and no more than the
/// region leaves around it.
const MAX_WIDTH: u16 = 72;

/// The largest the dialog grows, in rows.
const MAX_HEIGHT: u16 = 20;

/// The title of the dialog.
const TITLE: &str = "Open a description";

/// The state of the open dialog: the framework's browser, the path it chose, and what went wrong.
#[derive(Default)]
pub(crate) struct OpenDialogState {
    /// The framework's browser: the roots, the two lists, the path field and the buttons.
    files: FileDialogState,

    /// The path the browser chose, waiting for `Action::Load` to take it.
    pending: Option<PathBuf>,

    /// Why the last attempt failed, or `None` before one failed.
    error: Option<String>,
}

/// What an event means to the open dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpenOutcome {
    /// The dialog took no interest in the event.
    Continue,

    /// The dialog changed, and the frame is owed.
    Changed,

    /// A file was chosen, and the path is to be read.
    Ok,
}

impl OpenDialogState {
    /// Opens the dialog at the directory the application was started in.
    ///
    /// **A directory that cannot be read is the dialog's own failure**, not the load's: the browser
    /// opens anyway, with the reason on the row below it, so the path field is still reachable.
    pub(crate) fn open(&mut self) {
        self.error = None;
        self.pending = None;

        let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if let Err(error) = self.open_at(&start) {
            self.error = Some(format!("{}: {error}", start.display()));
        }
    }

    /// Opens the dialog at `dir`, with `dir` as its only root.
    ///
    /// **The roots are cleared and rebuilt every time** so a second open does not accumulate them,
    /// and so the root is the directory the person is looking at rather than a platform guess.
    fn open_at(&mut self, dir: &Path) -> Result<(), AppError> {
        self.files.no_default_roots();
        self.files.clear_roots();
        self.files.add_root("Start", dir);
        self.files.open_dialog(dir)?;

        Ok(())
    }

    /// Whether the dialog is up.
    pub(crate) fn active(&self) -> bool {
        self.files.active()
    }

    /// The path the browser chose, if one was chosen and the action has not taken it yet.
    pub(crate) fn take_pending(&mut self) -> Option<PathBuf> {
        self.pending.take()
    }

    /// Why the last attempt failed, or `None` before one failed.
    #[cfg(test)]
    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Says why the attempt failed.
    ///
    /// **The browser keeps the keyboard**, because a failure reopens it at the path's directory;
    /// the row below the browser is the only thing the error changes.
    pub(crate) fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// Reopens the browser at `dir`, keeping the reason the last attempt failed.
    ///
    /// **The error is not cleared here**: the caller has just set it, and reopening is what makes
    /// the corrected choice possible.
    pub(crate) fn reopen_at(&mut self, dir: &Path) {
        if let Err(error) = self.open_at(dir) {
            self.error = Some(format!("{}: {error}", dir.display()));
        }
    }

    /// Chooses `path` without walking the browser, for the tests that prove the load.
    #[cfg(test)]
    pub(crate) fn choose(&mut self, path: PathBuf) {
        self.files.active = false;
        self.pending = Some(path);
    }
}

/// What an event means to the dialog.
///
/// **The browser consumes everything while it is up**, as the About dialog does: an event it does
/// not use returns [`OpenOutcome::Continue`], and `callbacks` answers that with nothing, so nothing
/// behind the overlay is asked. `Ok` does not load by itself: the chosen path waits in `pending`,
/// and a failure reopens the browser with the reason below it.
pub(crate) fn handle(event: &Event, state: &mut OpenDialogState) -> OpenOutcome {
    if !state.active() {
        return OpenOutcome::Continue;
    }

    match state.files.handle(event, Dialog) {
        Ok(FileOutcome::Ok(path)) => {
            state.pending = Some(path);
            state.error = None;
            OpenOutcome::Ok
        }
        Ok(FileOutcome::Cancel) => {
            state.pending = None;
            state.error = None;
            OpenOutcome::Changed
        }
        Ok(FileOutcome::Changed) => OpenOutcome::Changed,
        Ok(_) => OpenOutcome::Continue,
        Err(error) => {
            state.error = Some(error.to_string());
            OpenOutcome::Changed
        }
    }
}

/// Paints the browser, centered in `area`, and says where the terminal's cursor goes.
///
/// **The error row is taken out of the pop-up's height**, so the browser and the reason never
/// overlap, and a dialog with nothing to report is exactly the height it would have been.
pub(crate) fn draw(
    area: Rect,
    buf: &mut Buffer,
    state: &mut OpenDialogState,
    theme: &SalsaTheme,
) -> Option<(u16, u16)> {
    if !state.active() {
        return None;
    }

    let pop_up = area.centered(Constraint::Max(MAX_WIDTH), Constraint::Max(MAX_HEIGHT));
    let reason_height = u16::from(state.error.is_some());
    let [dialog, reason] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(reason_height)]).areas(pop_up);

    FileDialog::new()
        .styles(theme.style(WidgetStyle::FILE_DIALOG))
        .block(Block::bordered().title(TITLE))
        .render(dialog, buf, &mut state.files);

    if let Some(error) = &state.error {
        Paragraph::new(error.as_str())
            .style(theme.p.red(3))
            .wrap(Wrap { trim: true })
            .render(reason, buf);
    }

    state.files.screen_cursor()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use ratatui_crossterm::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// The region the dialog is drawn into: larger than the smallest terminal, so the three lists
    /// and the buttons all fit.
    const REGION: Rect = Rect::new(0, 0, 70, 20);

    /// The theme the dialog is drawn in.
    fn theme() -> SalsaTheme {
        Config::default().theme
    }

    /// A fresh directory under the system temp directory, holding one `demo.json` with `contents`.
    ///
    /// **The directory is the test's own**, named after it, so a parallel test's files are never
    /// in the list the browser reads.
    fn a_directory_with(name: &str, contents: &str) -> (PathBuf, PathBuf) {
        use std::io::Write as _;

        let dir = std::env::temp_dir().join(format!(
            "monospace-open-dialog-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("the test's directory should be writable");

        let path = dir.join("demo.json");
        std::fs::File::create(&path)
            .and_then(|mut file| file.write_all(contents.as_bytes()))
            .expect("the test's description should be writable");

        (dir, path)
    }

    /// The dialog, opened at `dir` and painted once.
    fn drawn_at(dir: &Path) -> (OpenDialogState, Buffer) {
        let mut state = OpenDialogState::default();
        state
            .open_at(dir)
            .expect("the test's directory should be readable");

        let mut buffer = Buffer::empty(REGION);
        let _ = draw(REGION, &mut buffer, &mut state, &theme());

        (state, buffer)
    }

    /// The picture of the dialog, as text.
    fn picture(buffer: &Buffer) -> String {
        (buffer.area.top()..buffer.area.bottom())
            .map(|line| {
                (buffer.area.left()..buffer.area.right())
                    .map(|column| buffer[(column, line)].symbol().to_string())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// The event the terminal reports for a key.
    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    /// **The browser is a title, the file list, and `New`, `Ok` and `Cancel`**, and the file the
    /// directory holds is listed.
    #[test]
    fn the_open_dialog_lists_the_directory_it_was_given() {
        let (dir, _) = a_directory_with("listed", "{}");
        let (_, buffer) = drawn_at(&dir);
        let frame = picture(&buffer);

        assert!(frame.contains(TITLE), "the dialog has no title: {frame:?}");
        assert!(frame.contains("Ok"), "the dialog has no Ok: {frame:?}");
        assert!(
            frame.contains("Cancel"),
            "the dialog has no Cancel: {frame:?}"
        );
        assert!(frame.contains("New"), "the dialog has no New: {frame:?}");
        assert!(
            frame.contains("demo.json"),
            "the directory's file is not listed: {frame:?}"
        );
    }

    /// **`Enter` on the file list chooses the selected file**, and the path waits for the load
    /// action rather than being read here.
    #[test]
    fn enter_chooses_the_selected_file() {
        let (dir, path) = a_directory_with("chosen", "{}");
        let mut state = OpenDialogState::default();
        state
            .open_at(&dir)
            .expect("the test's directory should be readable");

        let outcome = handle(&key(KeyCode::Enter), &mut state);

        assert_eq!(outcome, OpenOutcome::Ok, "Enter did not choose the file");
        assert_eq!(
            state.take_pending(),
            Some(path),
            "the chosen path is not the file's"
        );
        assert!(!state.active(), "the dialog stayed up after choosing");
    }

    /// **`Esc` closes the dialog**, and the dialog is not left holding the keyboard.
    #[test]
    fn escape_closes_the_dialog() {
        let (dir, _) = a_directory_with("escape", "{}");
        let (mut state, _) = drawn_at(&dir);

        assert_eq!(handle(&key(KeyCode::Esc), &mut state), OpenOutcome::Changed);
        assert!(!state.active(), "escape did not close the dialog");
    }

    /// **An error is shown below the browser**, and the dialog stays up so another file can be
    /// chosen.
    #[test]
    fn an_error_is_shown_and_the_dialog_stays_up() {
        let (dir, _) = a_directory_with("error", "{}");
        let (mut state, _) = drawn_at(&dir);
        state.set_error("missing.json: the file was not found".to_owned());

        let mut buffer = Buffer::empty(REGION);
        let _ = draw(REGION, &mut buffer, &mut state, &theme());

        let frame = picture(&buffer);
        assert!(
            frame.contains("missing.json"),
            "the error is not on the frame: {frame:?}"
        );
        assert!(state.active(), "the error closed the dialog");
    }

    /// **`F4` reaches the path field**, and the browser says where the terminal's cursor goes.
    #[test]
    fn the_path_field_says_where_the_cursor_goes() {
        let (dir, _) = a_directory_with("cursor", "{}");
        let (mut state, _) = drawn_at(&dir);

        let _ = handle(&key(KeyCode::F(4)), &mut state);

        let mut buffer = Buffer::empty(REGION);
        let cursor = draw(REGION, &mut buffer, &mut state, &theme());

        assert!(cursor.is_some(), "the path field has no cursor");
        assert!(
            REGION.contains(cursor.expect("the cursor is set").into()),
            "the cursor is outside the region"
        );
    }

    /// **A small region still draws the browser**, because the frame clamps the pop-up to the
    /// region it is given rather than to its preferred size.
    #[test]
    fn a_small_region_still_draws_the_dialog() {
        let (dir, _) = a_directory_with("small", "{}");
        let mut state = OpenDialogState::default();
        state
            .open_at(&dir)
            .expect("the test's directory should be readable");

        let small = Rect::new(0, 0, 40, 8);
        let mut buffer = Buffer::empty(small);
        let _ = draw(small, &mut buffer, &mut state, &theme());

        let frame = picture(&buffer);
        assert!(
            frame.contains("Ok"),
            "the small dialog has no Ok: {frame:?}"
        );
    }
}
