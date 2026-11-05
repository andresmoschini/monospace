//! The save dialog: a path typed by hand, an `Ok` and a `Cancel`.
//!
//! **The framework's frame and buttons, and the application's field.** `rat-widget`'s
//! `DialogFrame` draws the box, the two buttons and the row they sit in, and its `TextInput` is the
//! one field: the application says what the dialog is called and what the field holds. The dialog
//! is an overlay, after the message dialog, and it takes every key and every control while it is
//! up, the same way (D6 of #189, D6 of #270).
//!
//! **The dialog says a path and nothing else.** Serialization and writing are `crate::save`'s,
//! and the action that carries them out is `callbacks::apply`'s, so the dialog does not know how
//! a diagram is written out and a later slice can replace the field with the framework's
//! `FileDialog` without touching either (D1 of #270).
//!
//! **The focus runs inside the dialog.** The field starts focused, `Tab` moves to the buttons, and
//! the focused button is drawn in reverse video, which is the cue the message dialog and the menu bar
//! already use (D3 of #189, D1 of #270).

use rat_theme4::WidgetStyle;
use rat_theme4::theme::SalsaTheme;
use rat_widget::dialog_frame::{DialogFrame, DialogFrameState};
use rat_widget::event::{ButtonOutcome, HandleEvent, Outcome, Regular};
use rat_widget::focus::{Focus, FocusBuilder, HasFocus};
use rat_widget::text::HasScreenCursor;
use rat_widget::text_input::{TextInput, TextInputState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui_widgets::block::Block;
use ratatui_widgets::paragraph::Paragraph;

use crate::vocabulary::{Action, Reply};

/// The largest the dialog grows: wide enough for a path, short enough to leave the chrome around
/// it visible.
const MAX_WIDTH: u16 = 44;

/// The largest the dialog grows, in rows.
const MAX_HEIGHT: u16 = 7;

/// The label above the field.
const LABEL: &str = "File name";

/// The title of the dialog.
const TITLE: &str = "Save description";

/// The state of the save dialog: whether it is up, what has been typed, and its frame.
#[derive(Debug, Default)]
pub(crate) struct SaveDialogState {
    /// Whether the dialog is up. The overlay holds the keyboard only while this is true.
    active: bool,

    /// The frame and its two buttons.
    frame: DialogFrameState,

    /// The path being typed.
    input: TextInputState,
}

impl SaveDialogState {
    /// Opens the dialog with an empty field.
    ///
    /// **The field holds the focus when the dialog opens** (D4 of #189, D1 of #270): a person opens
    /// it to type, so the first key is the first character of the path rather than a `Tab`.
    pub(crate) fn open(&mut self) {
        self.active = true;
        self.input.set_value("");
        self.input.focus.set(true);
        self.frame.ok.focus.set(false);
        self.frame.cancel.focus.set(false);
    }

    /// Whether the dialog is up.
    #[cfg(test)]
    pub(crate) fn active(&self) -> bool {
        self.active
    }

    /// The path as it was typed, without the spaces around it.
    pub(crate) fn path(&self) -> &str {
        self.input.text().trim()
    }

    /// Closes the dialog and forgets what was typed.
    pub(crate) fn close(&mut self) {
        self.active = false;
        self.input.set_value("");
        self.input.focus.set(false);
        self.frame.ok.focus.set(false);
        self.frame.cancel.focus.set(false);
    }

    /// Where the terminal cursor goes, if the field is active and focused.
    pub(crate) fn screen_cursor(&self) -> Option<(u16, u16)> {
        if self.active && self.input.is_focused() {
            self.input.screen_cursor()
        } else {
            None
        }
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

/// What an event means to the save dialog.
///
/// **The dialog consumes everything while it is up**: an event it does not use returns
/// [`Reply::Consumed`], so nothing behind the overlay is asked. `Cancel` and `Esc` close it and
/// answer [`Reply::Repaint`]. `Ok` and `Enter` close it and answer
/// `Reply::Act(Action::SaveFile(path))`.
pub(crate) fn handle(event: &Event, state: &mut SaveDialogState) -> Reply {
    if !state.active {
        return Reply::Ignored;
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
        return Reply::Repaint;
    }

    if state.frame.ok.handle(event, Regular) == ButtonOutcome::Pressed {
        let path = state.path().to_owned();
        state.close();
        return Reply::Act(Action::SaveFile(path));
    }

    if is_escape(event) {
        state.close();
        return Reply::Repaint;
    }

    // **`Enter` accepts from the field as well as from `Ok`**: a path typed and entered is the
    // shortest way through, and the field is where the dialog opens. A focused `Cancel` answered
    // above, so `Enter` on it still cancels.
    if is_enter(event) {
        let path = state.path().to_owned();
        state.close();
        return Reply::Act(Action::SaveFile(path));
    }

    if changed {
        Reply::Repaint
    } else {
        Reply::Consumed
    }
}

/// Paints the dialog, centered in `area`, and says where the terminal's cursor goes.
///
/// **The rectangle is the layout's**, as the message dialog's is: `centered` clamps the maximum
/// to the area it is given, so a region smaller than the maximum shrinks the pop-up rather than
/// the frame around it.
pub(crate) fn draw(
    area: Rect,
    buf: &mut Buffer,
    state: &mut SaveDialogState,
    theme: &SalsaTheme,
) -> Option<(u16, u16)> {
    if !state.active {
        return None;
    }

    let pop_up = area.centered(Constraint::Max(MAX_WIDTH), Constraint::Max(MAX_HEIGHT));

    DialogFrame::new()
        .block(Block::bordered().title(TITLE))
        .styles(crate::dialog_style::style(theme))
        .left(Constraint::Length(2))
        .right(Constraint::Length(2))
        .top(Constraint::Length(0))
        .bottom(Constraint::Length(0))
        .render(pop_up, buf, &mut state.frame);

    let content = state.frame.widget_area;
    let [label, input] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(content);

    Paragraph::new(LABEL).render(label, buf);

    TextInput::new()
        .styles(theme.style(WidgetStyle::TEXT))
        .render(input, buf, &mut state.input);

    state.screen_cursor()
}

fn is_escape(event: &Event) -> bool {
    matches!(
        event,
        Event::Key(KeyEvent {
            code: KeyCode::Esc,
            kind: KeyEventKind::Press,
            ..
        })
    )
}

fn is_enter(event: &Event) -> bool {
    matches!(
        event,
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            kind: KeyEventKind::Press,
            modifiers: KeyModifiers::NONE,
            ..
        })
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rat_widget::focus::HasFocus;
    use ratatui_core::style::Modifier;
    use ratatui_crossterm::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// The region the dialog is drawn into: the same size the frame gives it on a 40×10 screen.
    const REGION: Rect = Rect::new(0, 1, 40, 8);

    fn theme() -> SalsaTheme {
        crate::config::Config::default().theme
    }

    fn drawn() -> (SaveDialogState, Buffer) {
        let mut state = SaveDialogState::default();
        state.open();

        let mut buffer = Buffer::empty(REGION);
        let _ = draw(REGION, &mut buffer, &mut state, &theme());

        (state, buffer)
    }

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

    fn reversed(buffer: &Buffer, column: u16, line: u16) -> bool {
        buffer[(column, line)]
            .style()
            .add_modifier
            .contains(Modifier::REVERSED)
    }

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn the_save_dialog_holds_a_field_and_two_buttons() {
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

    #[test]
    fn tab_moves_from_the_field_to_the_ok() {
        let (mut state, _) = drawn();

        assert!(
            state.input.is_focused(),
            "the field did not start focused, so the test proves nothing"
        );

        let outcome = handle(&key(KeyCode::Tab), &mut state);

        assert_eq!(outcome, Reply::Repaint, "Tab changed nothing");
        assert!(!state.input.is_focused(), "Tab left the focus in the field");
        assert!(state.frame.ok.is_focused(), "Tab did not reach the Ok");
    }

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

    #[test]
    fn typing_reaches_the_field() {
        let (mut state, _) = drawn();

        for character in "out.json".chars() {
            let _ = handle(&key(KeyCode::Char(character)), &mut state);
        }

        assert_eq!(state.path(), "out.json", "the field did not take the keys");
    }

    #[test]
    fn the_field_says_where_the_cursor_goes() {
        let (mut state, _) = drawn();
        state.set_path("out.json");

        let mut buffer = Buffer::empty(REGION);
        let cursor = draw(REGION, &mut buffer, &mut state, &theme());

        assert!(cursor.is_some(), "the field has no cursor");
        assert!(
            REGION.contains(cursor.expect("the cursor is set").into()),
            "the cursor is outside the region"
        );
    }

    #[test]
    fn escape_closes_the_save_dialog() {
        let (mut state, _) = drawn();

        assert_eq!(handle(&key(KeyCode::Esc), &mut state), Reply::Repaint);
        assert!(!state.active(), "escape did not close the dialog");
    }

    #[test]
    fn enter_closes_the_dialog_and_mints_save_file() {
        let (mut state, _) = drawn();
        state.set_path("out.json");

        assert_eq!(
            handle(&key(KeyCode::Enter), &mut state),
            Reply::Act(Action::SaveFile("out.json".to_owned())),
            "Enter in the field did not accept"
        );
        assert!(!state.active(), "the dialog remained open after accept");
    }

    #[test]
    fn an_inactive_dialog_ignores_events() {
        let mut state = SaveDialogState::default();
        assert_eq!(handle(&key(KeyCode::Enter), &mut state), Reply::Ignored);
        assert_eq!(handle(&key(KeyCode::Esc), &mut state), Reply::Ignored);
    }

    #[test]
    fn an_unused_event_is_consumed_while_active() {
        let (mut state, _) = drawn();
        assert_eq!(handle(&key(KeyCode::Right), &mut state), Reply::Consumed);
    }
}
