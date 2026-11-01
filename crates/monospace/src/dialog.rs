//! The application's message dialog: the one pop-up, whatever it says.
//!
//! **The framework's message dialog, and nothing of ours.** `rat-widget`'s `MsgDialog` is the
//! one-button pop-up, and this module says how big it is, where it goes and which colors it is
//! drawn in. **What it says is the opener's**: `crate::about` fills it with the application's name,
//! and the `Ok` button is the framework's (D5 of #189).
//!
//! **It is drawn last of all and asked first of all.** The dialog is the overlay: the frame paints
//! it after the drop-down so that it covers everything, and `callbacks` asks it before the chrome
//! while it is up, so a key or a press behind it reaches nothing (D6).
//!
//! **The focus is reverse video over the theme's colors.** The theme gives the dialog its colors
//! and the button its three states; the application replaces the button's `focus` and `armed`
//! styles with the modifier alone and clears `hover`, so the `Ok` says "focused" the way a menu
//! label does and a pointer over it is not a state of the control (D3).
//!
//! **The style is handed over on every frame and not kept in the state**, so a message that wants
//! other colors needs the state to remember what kind of message it holds and nothing else.

use rat_theme4::WidgetStyle;
use rat_theme4::theme::SalsaTheme;
use rat_widget::event::{Dialog, HandleEvent, Outcome};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState, MsgDialogStyle};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Rect};
use ratatui_core::style::Style;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;

use crate::vocabulary::Reply;

/// The largest the pop-up grows: wide enough for a message, short enough to leave the chrome
/// around it visible.
const MAX_WIDTH: u16 = 40;

/// The rows the framework spends around the message: the border and the padding above and below,
/// the row its horizontal scroll bar keeps, and the `Ok`.
///
/// **Measured, not documented**: `MsgDialog` lays these out itself, and the test that a message
/// of several lines is shown whole is what holds this number to the framework.
const CHROME_HEIGHT: u16 = 7;

/// Paints the dialog, centered in `area`, as tall as its message.
///
/// **The dialog is given a rectangle and the caller keeps it small** (D5): `MsgDialog` lays its
/// content out inside the area it is handed but does not build a pop-up of its own, so the size
/// and place are the application's. **The rectangle is the layout's**: `centered` clamps a `Max`
/// constraint to the area it is given, so a region smaller than the message shrinks the pop-up
/// rather than the frame around it — and the framework's own scroll bar shows what is left over —
/// and the one cell an odd margin leaves over goes to the top and the left.
///
/// **The height is read off the message on every frame**, so whoever opens the dialog decides how
/// tall it is by what it says and nothing else.
pub(crate) fn draw(area: Rect, buf: &mut Buffer, state: &mut MsgDialogState, theme: &SalsaTheme) {
    let rows = state.message.borrow().lines().count().max(1);
    let height = u16::try_from(rows)
        .unwrap_or(u16::MAX)
        .saturating_add(CHROME_HEIGHT);
    let pop_up = area.centered(Constraint::Max(MAX_WIDTH), Constraint::Max(height));

    MsgDialog::new()
        .styles(dialog_style(theme))
        .render(pop_up, buf, state);
}

/// What an event means to the dialog.
///
/// **The dialog's own focus runs while it is up** (D6): the framework moves between the message and
/// the `Ok` with `Tab`, arms and presses the button, and closes on `Esc`. **`Outcome` already has
/// the three answers the router needs, and it also says whether the dialog is up**: the framework
/// answers `Continue` while the dialog is inactive, `Unchanged` when it kept the event without
/// changing the screen, and `Changed` when it owes a frame. One call is therefore enough — there is
/// no separate question about `active()` — and the three answers keep their meaning in [`Reply`].
pub(crate) fn handle(event: &Event, state: &mut MsgDialogState) -> Reply {
    match state.handle(event, Dialog) {
        Outcome::Continue => Reply::Ignored,
        Outcome::Unchanged => Reply::Consumed,
        Outcome::Changed => Reply::Repaint,
    }
}

/// The dialog's styles, with the button's focus cue replaced by reverse video.
///
/// **Every state that is not the pointer is a state**: the focused button and the pressed one are
/// both reversed, so pressing an `Ok` the keyboard is not on cannot look like a second kind of
/// focus. `hover` is cleared because a control under the pointer is not a state of the control
/// (D3).
fn dialog_style(theme: &SalsaTheme) -> MsgDialogStyle {
    let mut style: MsgDialogStyle = theme.style(WidgetStyle::MSG_DIALOG);

    if let Some(button) = style.button.as_mut() {
        button.focus = Some(Style::new().reversed());
        button.armed = Some(Style::new().reversed());
        button.hover = None;
    }

    style
}

/// The test tools the openers' tests share: a region, a theme and the picture of a frame.
#[cfg(test)]
pub(crate) mod testing {
    use rat_theme4::theme::SalsaTheme;
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;

    use crate::config::Config;

    /// The region the dialog is drawn into: the same size the frame gives it on a 40×10 screen.
    pub(crate) const REGION: Rect = Rect::new(0, 1, 40, 8);

    /// The theme the dialog is drawn in.
    pub(crate) fn theme() -> SalsaTheme {
        Config::default().theme
    }

    /// The picture of the dialog, as text.
    pub(crate) fn picture(buffer: &Buffer) -> String {
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
}

#[cfg(test)]
mod tests {
    use super::testing::{REGION, theme};
    use super::*;
    use rat_widget::button::{Button, ButtonState};
    use ratatui_core::style::Modifier;
    use ratatui_crossterm::crossterm::event::{
        Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind,
    };

    /// A dialog with a message, opened and painted once.
    fn drawn() -> (MsgDialogState, Buffer) {
        let mut state = MsgDialogState::new_active("Title", "A message");

        let mut buffer = Buffer::empty(REGION);
        draw(REGION, &mut buffer, &mut state, &theme());

        (state, buffer)
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

    /// The event the terminal reports for the pointer moving to `(column, row)`.
    fn moved(column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// **The dialog's three answers keep their meaning in the router's vocabulary** (D6): a
    /// dialog that is down ignores the event, one that keeps it without a change consumes it, and
    /// one that changes owes the frame.
    #[test]
    fn the_three_answers_become_the_routers_reply() {
        let mut down = MsgDialogState::default();
        assert_eq!(
            handle(&key(KeyCode::Esc), &mut down),
            Reply::Ignored,
            "a dialog that is down answered the event"
        );

        let (mut up, _) = drawn();
        assert_eq!(
            handle(&key(KeyCode::Right), &mut up),
            Reply::Consumed,
            "a key the dialog kept without a change owed a frame"
        );

        let (mut closing, _) = drawn();
        assert_eq!(
            handle(&key(KeyCode::Esc), &mut closing),
            Reply::Repaint,
            "the escape that closes the dialog owed no frame"
        );
    }

    /// **The message holds the focus when the dialog opens, and `Tab` moves to `Ok`** (rule 11):
    /// the focused button is reversed, and it was not before the key.
    #[test]
    fn tab_moves_from_the_message_to_the_ok_and_the_ok_is_reversed() {
        let (mut state, buffer) = drawn();
        let (column, line) = crate::frame::cell_of(&buffer, "Ok");

        assert!(
            !reversed(&buffer, column, line),
            "the Ok started focused, so the test proves nothing"
        );

        let _ = handle(&key(KeyCode::Tab), &mut state);

        let mut after = Buffer::empty(REGION);
        draw(REGION, &mut after, &mut state, &theme());

        assert!(
            reversed(&after, column, line),
            "Tab did not move the focus to the Ok"
        );
    }

    /// **A control under the pointer is not a state** (D3): the pointer over the `Ok` changes
    /// nothing, because the button has no hover style and the focus is still on the message.
    #[test]
    fn a_control_under_the_pointer_is_not_a_state() {
        let (mut state, buffer) = drawn();
        let (column, line) = crate::frame::cell_of(&buffer, "Ok");

        let _ = handle(&moved(column, line), &mut state);

        let mut after = Buffer::empty(REGION);
        draw(REGION, &mut after, &mut state, &theme());

        assert!(
            !reversed(&after, column, line),
            "the pointer over the Ok was a state of the control"
        );
    }

    /// **A pressed `Ok` that is not focused is reversed too** (D3), so the two cues never compete.
    ///
    /// **The framework's dialog owns the button's state**, and its click moves the keyboard to the
    /// button before arming it. What the application promises is the style it hands over, so the
    /// pressed state is shown on a button of that style with the focus held back.
    #[test]
    fn a_pressed_ok_that_is_not_focused_is_reversed() {
        let style = dialog_style(&theme())
            .button
            .expect("the theme gives the dialog a button");
        assert!(
            style.hover.is_none(),
            "the pointer over the Ok is still a style"
        );

        let area = Rect::new(0, 0, 10, 1);
        let mut buffer = Buffer::empty(area);
        let mut state = ButtonState {
            armed: true,
            ..ButtonState::default()
        };
        state.focus.set(false);

        Button::new("Ok")
            .styles(style)
            .render(area, &mut buffer, &mut state);

        let (column, line) = crate::frame::cell_of(&buffer, "Ok");
        assert!(
            reversed(&buffer, column, line),
            "a pressed Ok that is not focused is not reversed"
        );
    }

    /// **The dialog grows with its message**, so a message of several lines is shown whole where
    /// the region has room for it.
    #[test]
    fn a_message_of_several_lines_is_shown_whole() {
        let tall = Rect::new(0, 0, 40, 20);
        let mut state = MsgDialogState::new_active("Title", "line one\nline two\nline three");

        let mut buffer = Buffer::empty(tall);
        draw(tall, &mut buffer, &mut state, &theme());
        let frame = testing::picture(&buffer);

        for line in ["line one", "line two", "line three"] {
            assert!(
                frame.contains(line),
                "the dialog hid {line:?} although the region had room: {frame:?}"
            );
        }
    }
}
