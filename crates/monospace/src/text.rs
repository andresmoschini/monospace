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
use rat_theme4::WidgetStyle;
use rat_theme4::theme::SalsaTheme;
use rat_widget::dialog_frame::{DialogFrame, DialogFrameState};
use rat_widget::event::{ButtonOutcome, HandleEvent, Outcome, Regular, TextOutcome};
use rat_widget::focus::{Focus, FocusBuilder};
use rat_widget::text::HasScreenCursor;
use rat_widget::textarea::{TextArea, TextAreaState, TextWrap};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Rect};
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui_widgets::block::Block;

use crate::vocabulary::{Action, Reply};

/// The largest the dialog grows: wide enough for a label, short enough to leave the region around
/// it visible.
const MAX_WIDTH: u16 = 56;

/// The largest the dialog grows, in rows. It includes the border and the row of buttons.
const MAX_HEIGHT: u16 = 14;

/// The state of the text editor: whether it is up, the box it edits, its frame and its buffer.
#[derive(Debug, Default)]
pub(crate) struct TextEditorState {
    /// Whether the editor is up. The overlay holds the keyboard only while this is true.
    active: bool,

    /// The box being edited, for the title.
    id: Option<ShapeId>,

    /// The frame and its two buttons.
    frame: DialogFrameState,

    /// The buffer, its selection and its local undo.
    text: TextAreaState,
}

impl TextEditorState {
    /// Opens the editor over box `id`, seeded with `text` and with all of it selected.
    ///
    /// **The line ending is set explicitly to `\n`.** `TextAreaState` defaults to `\r\n` on
    /// Windows, and the model refuses a carriage return: without this line, every `Enter` on
    /// Windows would produce a buffer that `Text::new` rejects.
    ///
    /// **The wrap is `Word(1)` and not the framework's default.** `TextWrap::Shift` means no
    /// wrapping at all, which is not what D6 asks for; the spec is corrected with this commit's
    /// PR (see commit 9).
    ///
    /// **`open` is dead until the next commit**, which adds `t` and the action that calls it.
    #[expect(
        dead_code,
        reason = "the next commit opens it from `t`; the tests arrive with that commit"
    )]
    pub(crate) fn open(&mut self, id: ShapeId, text: Option<&Text>) {
        let seed = text
            .map(|text| {
                text.lines()
                    .iter()
                    .map(|line| line.iter().map(Glyph::as_str).collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();

        self.active = true;
        self.id = Some(id);
        self.frame.ok.focus.set(false);
        self.frame.cancel.focus.set(false);
        self.text.focus.set(true);
        self.text.set_newline("\n");
        self.text.set_text_wrap(TextWrap::Word(1));
        self.text.set_text(seed);
        let _ = self.text.select_all();
    }

    // **The three test accessors — `active`, `buffer` and `selected_text` — arrive with the
    // tests in commit 5.** A `#[cfg(test)]` accessor with no caller is dead code in the test
    // build too, so adding them here would make `-D warnings` fail before the tests exist.

    /// Closes the editor and forgets the buffer.
    ///
    /// **The buffer is cleared rather than kept**: the next opening seeds it from the box, and a
    /// buffer left behind would be a second copy of a text that may already have changed.
    pub(crate) fn close(&mut self) {
        self.active = false;
        self.id = None;
        self.text.focus.set(false);
        self.text.set_text("");
        self.frame.ok.focus.set(false);
        self.frame.cancel.focus.set(false);
    }

    /// The title the frame draws: the identity is what tells one box from another.
    fn title(&self) -> String {
        match self.id {
            Some(id) => format!("Text of Box #{id}"),
            None => String::from("Text of Box"),
        }
    }

    /// The focus order: the buffer, then `Ok`, then `Cancel`.
    ///
    /// **It is rebuilt on every event rather than kept**, as the open and save dialogs do: the
    /// flags live in the states and the builder only says the order.
    fn build_focus(&self) -> Focus {
        let mut builder = FocusBuilder::default();
        builder
            .widget(&self.text)
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
pub(crate) fn handle(event: &Event, state: &mut TextEditorState) -> Reply {
    if !state.active {
        return Reply::Ignored;
    }

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
        state.close();
        return Reply::Repaint;
    }

    if state.frame.ok.handle(event, Regular) == ButtonOutcome::Pressed {
        let text = stored(&state.text.text());
        state.close();
        return Reply::Act(Action::SetText(text));
    }

    if is_escape(event) {
        state.close();
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
    state: &mut TextEditorState,
    theme: &SalsaTheme,
) -> Option<(u16, u16)> {
    if !state.active {
        return None;
    }

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
fn stored(buffer: &str) -> Option<Text> {
    let mut normalized = String::with_capacity(buffer.len());

    for character in buffer.chars() {
        match character {
            '\r' => {
                normalized.push('\n');
            }
            '\t' => normalized.push_str("    "),
            '\n' => {
                if !normalized.ends_with('\n') {
                    normalized.push('\n');
                }
            }
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
