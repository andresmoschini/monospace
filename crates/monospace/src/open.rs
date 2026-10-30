//! The open dialog: a path typed by hand, an `Ok` and a `Cancel`.
//!
//! **The framework's frame and buttons, and the application's field.** `rat-widget`'s
//! `DialogFrame` draws the box, the two buttons and the row they sit in, and its `TextInput` is the
//! one field: the application says what the dialog is called and what the field holds. The dialog
//! is the second overlay, after the About one, and it takes every key and every control while it is
//! up, the same way (D6 of #189).
//!
//! **The dialog says a path and nothing else.** Reading and parsing are `crate::load`'s, and the
//! action that carries them out is `callbacks::apply`'s, so the dialog does not know what a
//! description is and a later slice can replace the field with the framework's `FileDialog`
//! without touching either.
//!
//! **The focus runs inside the dialog.** The field starts focused, `Tab` moves to the buttons, and
//! the focused button is drawn in reverse video, which is the cue the About dialog and the menu bar
//! already use (D3 of #189).

use rat_theme4::WidgetStyle;
use rat_theme4::theme::SalsaTheme;
use rat_widget::dialog_frame::{DialogFrame, DialogFrameState, DialogFrameStyle};
use rat_widget::event::{ButtonOutcome, HandleEvent, Outcome, Regular};
use rat_widget::focus::{Focus, FocusBuilder};
use rat_widget::text::HasScreenCursor;
use rat_widget::text_input::{TextInput, TextInputState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui_widgets::block::Block;
use ratatui_widgets::paragraph::{Paragraph, Wrap};

/// The largest the dialog grows: wide enough for a path, short enough to leave the chrome around
/// it visible.
const MAX_WIDTH: u16 = 44;

/// The largest the dialog grows, in rows.
const MAX_HEIGHT: u16 = 10;

/// The label above the field.
const LABEL: &str = "File name";

/// The title of the dialog.
const TITLE: &str = "Open a description";

/// The state of the open dialog: whether it is up, what has been typed, and what went wrong.
#[derive(Debug, Default)]
pub(crate) struct OpenDialogState {
    /// Whether the dialog is up. The overlay holds the keyboard only while this is true.
    active: bool,

    /// The frame and its two buttons.
    frame: DialogFrameState,

    /// The path being typed.
    input: TextInputState,

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

    /// `Ok` was pressed, and the path is to be read.
    Ok,
}

impl OpenDialogState {
    /// Opens the dialog with an empty field and no error.
    ///
    /// **The field holds the focus when the dialog opens** (D4 of #189): a person opens it to type,
    /// so the first key is the first character of the path rather than a `Tab`.
    pub(crate) fn open(&mut self) {
        self.active = true;
        self.error = None;
        self.input.set_value("");
        self.input.focus.set(true);
        self.frame.ok.focus.set(false);
        self.frame.cancel.focus.set(false);
    }

    /// Whether the dialog is up.
    pub(crate) fn active(&self) -> bool {
        self.active
    }

    /// The path as it was typed, without the spaces around it.
    pub(crate) fn path(&self) -> &str {
        self.input.text().trim()
    }

    /// Why the last attempt failed, or `None` before one failed.
    #[cfg(test)]
    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Says why the attempt failed, and gives the field back the focus so it can be corrected.
    pub(crate) fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.input.focus.set(true);
        self.frame.ok.focus.set(false);
        self.frame.cancel.focus.set(false);
    }

    /// Closes the dialog and forgets what was typed.
    fn close(&mut self) {
        self.active = false;
        self.error = None;
        self.input.set_value("");
        self.input.focus.set(false);
        self.frame.ok.focus.set(false);
        self.frame.cancel.focus.set(false);
    }

    /// The focus order: the field, then `Ok`, then `Cancel`.
    ///
    /// **It is rebuilt on every event rather than kept** because that is what the framework's
    /// dialog does: the flags live in the states, and the builder only says the order.
    fn build_focus(&self) -> Focus {
        let mut builder = FocusBuilder::default();
        builder
            .widget(&self.input)
            .widget(&self.frame.ok)
            .widget(&self.frame.cancel);
        builder.build()
    }

    /// Types `path` into the field, for the tests that would otherwise walk it one key at a time.
    #[cfg(test)]
    pub(crate) fn set_path(&mut self, path: &str) {
        self.input.set_value(path);
    }
}

/// What an event means to the dialog.
///
/// **The dialog consumes everything while it is up**: a key it does not use returns
/// [`OpenOutcome::Continue`], and `callbacks` answers that with nothing, so nothing behind the
/// overlay is asked. `Ok` does not close the dialog: the path is read first, and a failure leaves
/// the dialog up with the reason in it.
pub(crate) fn handle(event: &Event, state: &mut OpenDialogState) -> OpenOutcome {
    if !state.active {
        return OpenOutcome::Continue;
    }

    let mut changed = false;

    // **The focus is asked first** (rule 11 of #189): `Tab` moves between the field and the
    // buttons before any of them is handed the key, and a press moves the focus to the control
    // under the pointer.
    if state.build_focus().handle(event, Regular) == Outcome::Changed {
        changed = true;
    }

    let input: Outcome = state.input.handle(event, Regular).into();
    if input == Outcome::Changed {
        changed = true;
    }

    if state.frame.cancel.handle(event, Regular) == ButtonOutcome::Pressed {
        state.close();
        return OpenOutcome::Changed;
    }

    if state.frame.ok.handle(event, Regular) == ButtonOutcome::Pressed {
        return OpenOutcome::Ok;
    }

    if is_escape(event) {
        state.close();
        return OpenOutcome::Changed;
    }

    // **`Enter` accepts from the field as well as from `Ok`**: a path typed and entered is the
    // shortest way through, and the field is where the dialog opens. A focused `Cancel` answered
    // above, so `Enter` on it still cancels.
    if is_enter(event) {
        return OpenOutcome::Ok;
    }

    if changed {
        OpenOutcome::Changed
    } else {
        OpenOutcome::Continue
    }
}

/// Paints the dialog, centered in `area`, and says where the terminal's cursor goes.
///
/// **The rectangle is the layout's**, as the About dialog's is: `centered` clamps the maximum to
/// the area it is given, so a region smaller than the maximum shrinks the pop-up rather than the
/// frame around it.
pub(crate) fn draw(
    area: Rect,
    buf: &mut Buffer,
    state: &mut OpenDialogState,
    theme: &SalsaTheme,
) -> Option<(u16, u16)> {
    if !state.active {
        return None;
    }

    let pop_up = area.centered(Constraint::Max(MAX_WIDTH), Constraint::Max(MAX_HEIGHT));

    DialogFrame::new()
        .block(Block::bordered().title(TITLE))
        .styles(dialog_style(theme))
        .left(Constraint::Length(2))
        .right(Constraint::Length(2))
        .top(Constraint::Length(0))
        .bottom(Constraint::Length(0))
        .render(pop_up, buf, &mut state.frame);

    let content = state.frame.widget_area;
    let [label, input, reason] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(content);

    Paragraph::new(LABEL).render(label, buf);

    TextInput::new()
        .styles(theme.style(WidgetStyle::TEXT))
        .render(input, buf, &mut state.input);

    if let Some(error) = &state.error {
        Paragraph::new(error.as_str())
            .style(theme.p.red(3))
            .wrap(Wrap { trim: true })
            .render(reason, buf);
    }

    state.input.screen_cursor()
}

/// The dialog's styles, with the buttons' focus cue replaced by reverse video.
///
/// **Every state that is not the pointer is a state** (D3 of #189): the focused button and the
/// pressed one are both reversed, so pressing an `Ok` the keyboard is not on cannot look like a
/// second kind of focus. `hover` is cleared because a control under the pointer is not a state.
fn dialog_style(theme: &SalsaTheme) -> DialogFrameStyle {
    let mut style: DialogFrameStyle = theme.style(WidgetStyle::DIALOG_FRAME);

    if let Some(button) = style.button_style.as_mut() {
        button.focus = Some(Style::new().reversed());
        button.armed = Some(Style::new().reversed());
        button.hover = None;
    }

    style
}

/// Whether the event is an `Esc` press, with no modifier on it and no kind but a press.
fn is_escape(event: &Event) -> bool {
    matches!(
        event,
        Event::Key(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            ..
        })
    )
}

/// Whether the event is an `Enter` press, with no modifier on it and no kind but a press.
fn is_enter(event: &Event) -> bool {
    matches!(
        event,
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            ..
        })
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use rat_widget::focus::HasFocus;
    use ratatui_core::style::Modifier;

    /// The region the dialog is drawn into: the same size the frame gives it on a 40x10 screen.
    const REGION: Rect = Rect::new(0, 1, 40, 8);

    /// The theme the dialog is drawn in.
    fn theme() -> SalsaTheme {
        Config::default().theme
    }

    /// The dialog, opened and painted once.
    fn drawn() -> (OpenDialogState, Buffer) {
        let mut state = OpenDialogState::default();
        state.open();

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

    /// Whether the cell at a frame position is painted in reverse video.
    fn reversed(buffer: &Buffer, column: u16, line: u16) -> bool {
        buffer[(column, line)]
            .style()
            .add_modifier
            .contains(Modifier::REVERSED)
    }

    /// The event the terminal reports for a key.
    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    /// **The dialog is a title, a field, an `Ok` and a `Cancel`**, and nothing else.
    #[test]
    fn the_open_dialog_holds_a_field_and_two_buttons() {
        let (_, buffer) = drawn();
        let frame = picture(&buffer);

        assert!(frame.contains(TITLE), "the dialog has no title: {frame:?}");
        assert!(
            frame.contains(LABEL),
            "the dialog has no field label: {frame:?}"
        );
        assert!(frame.contains("Ok"), "the dialog has no Ok: {frame:?}");
        assert!(
            frame.contains("Cancel"),
            "the dialog has no Cancel: {frame:?}"
        );
    }

    /// **The field holds the focus when the dialog opens**, and `Tab` moves it to `Ok`.
    #[test]
    fn tab_moves_from_the_field_to_the_ok() {
        let (mut state, _) = drawn();

        assert!(
            state.input.is_focused(),
            "the field did not start focused, so the test proves nothing"
        );

        let outcome = handle(&key(KeyCode::Tab), &mut state);

        assert_eq!(outcome, OpenOutcome::Changed, "Tab changed nothing");
        assert!(!state.input.is_focused(), "Tab left the focus in the field");
        assert!(state.frame.ok.is_focused(), "Tab did not reach the Ok");
    }

    /// **The focused button is reverse video**, so the cue is the one the menu bar and the About
    /// dialog already use.
    #[test]
    fn the_focused_ok_is_reversed() {
        let (mut state, _) = drawn();
        let _ = handle(&key(KeyCode::Tab), &mut state);

        let mut buffer = Buffer::empty(REGION);
        let _ = draw(REGION, &mut buffer, &mut state, &theme());

        let (column, line) = crate::frame::cell_of(&buffer, "Ok");
        assert!(
            reversed(&buffer, column, line),
            "the focused Ok is not reversed"
        );
    }

    /// **Typing reaches the field**, and the path is what was typed.
    #[test]
    fn typing_reaches_the_field() {
        let (mut state, _) = drawn();

        for character in "demo.json".chars() {
            let _ = handle(&key(KeyCode::Char(character)), &mut state);
        }

        assert_eq!(state.path(), "demo.json", "the field did not take the keys");
    }

    /// **`Enter` accepts from the field**, which is where the dialog opens.
    #[test]
    fn enter_accepts_from_the_field() {
        let (mut state, _) = drawn();
        state.set_path("demo.json");

        assert_eq!(
            handle(&key(KeyCode::Enter), &mut state),
            OpenOutcome::Ok,
            "Enter in the field did not accept"
        );
        assert!(state.active(), "the dialog closed before the path was read");
    }

    /// **`Esc` closes the dialog**, and the dialog is not left holding the keyboard.
    #[test]
    fn escape_closes_the_dialog() {
        let (mut state, _) = drawn();

        assert_eq!(handle(&key(KeyCode::Esc), &mut state), OpenOutcome::Changed);
        assert!(!state.active(), "escape did not close the dialog");
    }

    /// **An error is shown in the dialog**, and the dialog stays up so the path can be corrected.
    #[test]
    fn an_error_is_shown_and_the_dialog_stays_up() {
        let (mut state, _) = drawn();
        state.set_error("missing.json: the file was not found".to_owned());

        let mut buffer = Buffer::empty(REGION);
        let _ = draw(REGION, &mut buffer, &mut state, &theme());

        let frame = picture(&buffer);
        assert!(
            frame.contains("missing.json"),
            "the error is not on the frame: {frame:?}"
        );
        assert!(state.active(), "the error closed the dialog");
        assert!(
            state.input.is_focused(),
            "the error did not give the field the focus"
        );
    }

    /// **`Ok` and `Cancel` are the framework's buttons**, and the dialog's cursor is the field's.
    #[test]
    fn the_field_says_where_the_cursor_goes() {
        let (mut state, _) = drawn();
        state.set_path("demo.json");

        let mut buffer = Buffer::empty(REGION);
        let cursor = draw(REGION, &mut buffer, &mut state, &theme());

        assert!(cursor.is_some(), "the field has no cursor");
        assert!(
            REGION.contains(cursor.expect("the cursor is set").into()),
            "the cursor is outside the region"
        );
    }
}
