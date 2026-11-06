//! The text editor: a box's text in the framework's `TextArea`, inside a `DialogFrame`.
//!
//! **The framework's widget and the application's protocol.** `TextArea` brings selection,
//! movement by word and by line, local undo, scrolling, the pointer and wide-glyph columns;
//! `DialogFrame` brings the box, the two buttons and the focus order. This module says what the
//! buffer is seeded with, what a key means, what is stored and where the cursor goes.
//!
//! **While it is up, it owns every event.** The editor is an overlay: an event it does not use is
//! `Consumed`, not `Ignored`, so `Ctrl+Q`, `Ctrl+S`, `Ctrl+O` and `Ctrl+N` do nothing and a click
//! outside it reaches nothing. `Ctrl+C`, `Ctrl+X` and `Ctrl+V` are the buffer's own, as they are
//! in any field: the application binds none of them (D3 of #304), and the forced exit is
//! `Ctrl+Shift+Q`, answered before the overlay.
//!
//! **The buffer is normalized at this boundary and stored literally.** `Text` keeps refusing
//! control characters; the editor turns CRLF and CR into `\n`, each tab into four spaces and
//! drops every other control character, so a paste cannot trap a person in a modal with no way to
//! succeed. Whitespace only is `None`: an invisible text is not a thing a person can see or take
//! back.

use monospace_core::Glyph;
use monospace_diagram::{ShapeId, Text};
use monospace_editing::Command;
use rat_theme4::WidgetStyle;
use rat_theme4::theme::SalsaTheme;
use rat_widget::dialog_frame::{DialogFrame, DialogFrameState};
use rat_widget::event::{ButtonOutcome, HandleEvent, Outcome, Regular, TextOutcome};
use rat_widget::focus::{Focus, FocusBuilder, Navigation};
use rat_widget::text::HasScreenCursor;
use rat_widget::textarea::{TextArea, TextAreaState, TextWrap};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Rect};
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui_widgets::block::Block;

use crate::overlay::Overlay;
use crate::vocabulary::{Action, Reply};

/// The largest the dialog grows: wide enough for a label, short enough to leave the region around
/// it visible.
const MAX_WIDTH: u16 = 56;

/// The largest the dialog grows, in rows. It includes the border and the row of buttons.
const MAX_HEIGHT: u16 = 14;

/// The state of the text editor: the box it edits, its frame and its buffer.
#[derive(Debug)]
pub(crate) struct TextEditorState {
    /// The box being edited, for the title.
    id: ShapeId,

    /// The frame and its two buttons.
    frame: DialogFrameState,

    /// The buffer, its selection and its local undo.
    text: TextAreaState,
}

impl TextEditorState {
    /// The editor over box `id`, seeded with `text` and with all of it selected.
    ///
    /// **The line ending is set explicitly to `\n`.** `TextAreaState` defaults to `\r\n` on
    /// Windows, and the model refuses a carriage return: without this line, every `Enter` on
    /// Windows would produce a buffer that `Text::new` rejects.
    ///
    /// **The wrap is `Word(1)` and not the framework's default.** `TextWrap::Shift` means no
    /// wrapping at all, which is not what D6 of #304 asks for.
    pub(crate) fn new(id: ShapeId, text: Option<&Text>) -> Self {
        let seed = text
            .map(|text| {
                text.lines()
                    .iter()
                    .map(|line| line.iter().map(Glyph::as_str).collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();

        let mut state = Self {
            id,
            frame: DialogFrameState::default(),
            text: TextAreaState::default(),
        };

        state.frame.ok.focus.set(false);
        state.frame.cancel.focus.set(false);
        state.text.focus.set(true);
        state.text.set_newline("\n");
        state.text.set_text_wrap(TextWrap::Word(1));
        state.text.set_text(seed);
        let _ = state.text.select_all();
        state
    }

    /// The text currently in the buffer, for the tests that would otherwise read the widget.
    #[cfg(test)]
    pub(crate) fn buffer(&self) -> String {
        self.text.text()
    }

    /// The selected text, for the tests that prove the buffer opens selected.
    #[cfg(test)]
    pub(crate) fn selected_text(&self) -> String {
        self.text.selected_text().into_owned()
    }

    /// The title the frame draws: the identity is what tells one box from another.
    fn title(&self) -> String {
        format!("Text of Box #{}", self.id)
    }

    /// The focus order: the buffer, then `Ok`, then `Cancel`.
    ///
    /// **It is rebuilt on every event rather than kept**, as the open and save dialogs do: the
    /// flags live in the states and the builder only says the order.
    ///
    /// `TextAreaState` defaults to [`Navigation::Reach`] which keeps `Tab` within the widget.
    /// Overriding it to [`Navigation::Regular`] allows `Tab` to cycle between the buffer and
    /// the frame buttons.
    fn build_focus(&self) -> Focus {
        let mut builder = FocusBuilder::default();
        builder
            .widget_navigate(&self.text, Navigation::Regular)
            .widget(&self.frame.ok)
            .widget(&self.frame.cancel);
        builder.build()
    }
}

/// What an event means to the text editor.
///
/// **The editor consumes everything while it is up** (rule 13 of #304): an event it does not use
/// returns [`Reply::Consumed`], so nothing behind the overlay is asked. `Ok` closes it and answers
/// `Reply::Act(Action::SetText(text))`; `Cancel` and `Esc` close it and answer `Reply::Repaint`.
/// **`Ctrl+C` is not special here**: it goes to the buffer like any other key, and the
/// application binds neither it nor the copy it performs.
pub(crate) fn handle(event: &Event, editor: &mut Overlay<TextEditorState>) -> Reply {
    let Some(state) = editor.get_mut() else {
        return Reply::Ignored;
    };

    // **The focus is asked first**, as in the open and save dialogs: `Tab` moves between the
    // buffer and the buttons before any of them is handed the key, and a press moves the focus
    // to the control under the pointer.
    let mut changed = state.build_focus().handle(event, Regular) == Outcome::Changed;

    // **`Tab` is the focus's and never the buffer's.** `TextAreaState` inserts a tab when it
    // still believes it holds the focus; the builder has just moved it, so the key stops here.
    if is_tab(event) {
        return if changed {
            Reply::Repaint
        } else {
            Reply::Consumed
        };
    }

    match state.text.handle(event, Regular) {
        TextOutcome::TextChanged | TextOutcome::Changed => changed = true,
        TextOutcome::Unchanged | TextOutcome::Continue => {}
    }

    if state.frame.cancel.handle(event, Regular) == ButtonOutcome::Pressed {
        editor.close();
        return Reply::Repaint;
    }

    if state.frame.ok.handle(event, Regular) == ButtonOutcome::Pressed {
        let text = stored(&state.text.text());
        let id = state.id;
        editor.close();
        return Reply::Act(Action::Command(Command::SetText { id, text }));
    }

    if is_escape(event) {
        editor.close();
        return Reply::Repaint;
    }

    if changed {
        Reply::Repaint
    } else {
        Reply::Consumed
    }
}

/// Paints the editor, centered in `area`, and says where the terminal's cursor goes.
///
/// **The rectangle is the layout's**, as the other dialogs': `centered` clamps the maximum to the
/// area it is given, so a region smaller than the modal shrinks it rather than the frame around
/// it. **The cursor is the buffer's own**: while a button holds the focus it is `None`, and the
/// terminal's cursor is not drawn (rule 14).
pub(crate) fn draw(
    area: Rect,
    buf: &mut Buffer,
    editor: &mut Overlay<TextEditorState>,
    theme: &SalsaTheme,
) -> Option<(u16, u16)> {
    let state = editor.get_mut()?;

    let pop_up = area.centered(Constraint::Max(MAX_WIDTH), Constraint::Max(MAX_HEIGHT));

    DialogFrame::new()
        .block(Block::bordered().title(state.title()))
        .styles(crate::dialog_style::style(theme))
        .left(Constraint::Length(2))
        .right(Constraint::Length(2))
        .top(Constraint::Length(0))
        .bottom(Constraint::Length(0))
        .render(pop_up, buf, &mut state.frame);

    TextArea::new()
        .styles(theme.style(WidgetStyle::TEXTAREA))
        .render(state.frame.widget_area, buf, &mut state.text);

    state.text.screen_cursor()
}

/// What saving the buffer stores, or `None` when it holds nothing but whitespace.
///
/// **The boundary normalizes and the model keeps refusing.** CRLF and a lone CR become `\n`, each
/// tab becomes four spaces, and every other control character is dropped. What is left is
/// handed to [`Text::new`], which answers `None` only for a control character — impossible after
/// the loop above — so a buffer with a visible character is stored literally, surrounding spaces
/// included.
///
/// **Consecutive line breaks are kept.** A blank line is part of the text and not an artifact of
/// the CRLF: only the pair is folded, so two `\n` remain two.
fn stored(buffer: &str) -> Option<Text> {
    let mut normalized = String::with_capacity(buffer.len());
    let mut characters = buffer.chars().peekable();

    while let Some(character) = characters.next() {
        match character {
            '\r' => {
                if characters.peek() == Some(&'\n') {
                    characters.next();
                }
                normalized.push('\n');
            }
            '\t' => normalized.push_str("    "),
            '\n' => normalized.push('\n'),
            other if other.is_control() => {}
            other => normalized.push(other),
        }
    }

    if normalized.chars().all(char::is_whitespace) {
        None
    } else {
        Text::new(&normalized)
    }
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

fn is_tab(event: &Event) -> bool {
    matches!(
        event,
        Event::Key(KeyEvent {
            code: KeyCode::Tab | KeyCode::BackTab,
            kind: KeyEventKind::Press,
            ..
        })
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use monospace_core::ShapeId;
    use ratatui_crossterm::crossterm::event::KeyModifiers;
    use std::num::NonZeroU16;

    fn id(ordinal: u16) -> ShapeId {
        ShapeId::new(NonZeroU16::new(ordinal).expect("no test names zero"))
    }

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn saved(reply: Reply) -> Option<Text> {
        match reply {
            Reply::Act(Action::Command(Command::SetText { text, .. })) => text,
            other => panic!("the editor did not save: {other:?}"),
        }
    }

    /// The editor that is up over a box.
    fn editor(id: ShapeId, text: Option<&Text>) -> Overlay<TextEditorState> {
        Overlay::up(TextEditorState::new(id, text))
    }

    /// The editor's own state, while it is up.
    fn state_of(editor: &Overlay<TextEditorState>) -> &TextEditorState {
        editor.get().expect("the text editor is up")
    }

    /// The editor's own state, for the test that writes into its buffer.
    fn state_of_mut(editor: &mut Overlay<TextEditorState>) -> &mut TextEditorState {
        editor.get_mut().expect("the text editor is up")
    }

    #[test]
    fn the_editor_opens_with_all_the_text_selected() {
        let state = editor(id(1), Some(&Text::new("one\ntwo").unwrap()));

        assert_eq!(state_of(&state).buffer(), "one\ntwo");
        assert_eq!(state_of(&state).selected_text(), "one\ntwo");
    }

    #[test]
    fn saving_normalizes_control_characters() {
        let mut state = editor(id(1), None);
        state_of_mut(&mut state)
            .text
            .set_text("one\r\ntwo\tthree\x07");

        let _ = handle(&key(KeyCode::Tab), &mut state);
        let text = saved(handle(&key(KeyCode::Enter), &mut state))
            .expect("the normalized text is not empty");

        assert_eq!(text, Text::new("one\ntwo    three").unwrap());
    }

    #[test]
    fn saving_keeps_the_blank_lines() {
        let mut state = editor(id(1), Some(&Text::new("one\n\ntwo").unwrap()));

        let _ = handle(&key(KeyCode::Tab), &mut state);
        assert_eq!(
            saved(handle(&key(KeyCode::Enter), &mut state)),
            Some(Text::new("one\n\ntwo").unwrap()),
            "a blank line was folded away"
        );
    }

    #[test]
    fn saving_turns_a_lone_carriage_return_into_a_line_break() {
        let mut state = editor(id(1), None);
        state_of_mut(&mut state).text.set_text("one\rtwo");

        let _ = handle(&key(KeyCode::Tab), &mut state);
        assert_eq!(
            saved(handle(&key(KeyCode::Enter), &mut state)),
            Some(Text::new("one\ntwo").unwrap())
        );
    }

    #[test]
    fn saving_whitespace_only_stores_none() {
        let mut state = editor(id(1), None);
        state_of_mut(&mut state).text.set_text("  \n\t ");

        let _ = handle(&key(KeyCode::Tab), &mut state);
        assert_eq!(saved(handle(&key(KeyCode::Enter), &mut state)), None);
    }

    #[test]
    fn a_wide_glyph_is_edited_in_two_columns_and_drawn_in_one_cell() {
        let mut state = editor(id(1), None);

        let _ = handle(&key(KeyCode::Char('漢')), &mut state);
        assert_eq!(state_of(&state).buffer(), "漢");

        let text = Text::new("漢").expect("the cluster is not a control character");
        assert_eq!(text.width(), 1, "the model counts one glyph as one cell");
    }
}
