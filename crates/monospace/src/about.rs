//! The About dialog: the first pop-up, and the first control that is not a menu entry.
//!
//! **The framework's message dialog, and nothing of ours.** `rat-widget`'s `MsgDialog` is the
//! one-button pop-up, and this module says what it says and which colors it is drawn in. The title
//! and the body are the application's, and the `Ok` button is the framework's (D5 of #189).
//!
//! **It is drawn last of all and asked first of all.** The dialog is the overlay: the frame paints
//! it after the drop-down so that it covers everything, and `callbacks` asks it before the chrome
//! while it is up, so a key or a press behind it reaches nothing (D6).
//!
//! **The focus is reverse video over the theme's colors.** The theme gives the dialog its colors
//! and the button its three states; the application replaces the button's `focus` and `armed`
//! styles with the modifier alone and clears `hover`, so the `Ok` says "focused" the way a menu
//! label does and a pointer over it is not a state of the control (D3).

use rat_theme4::WidgetStyle;
use rat_theme4::theme::SalsaTheme;
use rat_widget::event::{Dialog, HandleEvent, Outcome};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState, MsgDialogStyle};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Rect};
use ratatui_core::style::Style;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;

use crate::status;

/// The largest the pop-up grows: wide enough for a message, short enough to leave the chrome
/// around it visible.
const MAX_WIDTH: u16 = 40;

/// The largest the pop-up grows, in rows.
const MAX_HEIGHT: u16 = 9;

/// Opens the dialog over the application's own name and version.
///
/// **The message is built from `status`**, where the name and the version already live: the status
/// line and the dialog cannot disagree about what the application is. Opening it a second time
/// replaces the message rather than appending to it.
pub(crate) fn open(state: &mut MsgDialogState) {
    state.clear();
    state.title(format!("About {}", status::NAME));
    state.append(&format!("{} {}", status::NAME, status::VERSION));
}

/// Paints the dialog, centered in `area`.
///
/// **The dialog is given a rectangle and the caller keeps it small** (D5): `MsgDialog` lays its
/// content out inside the area it is handed but does not build a pop-up of its own, so the size
/// and place are the application's. **The rectangle is the layout's**: `centered` clamps a `Max`
/// constraint to the area it is given, so a region smaller than the maximum shrinks the pop-up
/// rather than the frame around it, and the one cell an odd margin leaves over goes to the top and
/// the left — where the arithmetic this replaced gave it to the bottom and the right.
pub(crate) fn draw(area: Rect, buf: &mut Buffer, state: &mut MsgDialogState, theme: &SalsaTheme) {
    let pop_up = area.centered(Constraint::Max(MAX_WIDTH), Constraint::Max(MAX_HEIGHT));

    MsgDialog::new()
        .styles(dialog_style(theme))
        .render(pop_up, buf, state);
}

/// What an event means to the dialog.
///
/// **The dialog's own focus runs while it is up** (D6): the framework moves between the message and
/// the `Ok` with `Tab`, arms and presses the button, and closes on `Esc`. The answer is the
/// framework's `Outcome`, so the router can paint when the dialog changed and leave everything
/// alone when it only consumed the event.
pub(crate) fn handle(event: &Event, state: &mut MsgDialogState) -> Outcome {
    state.handle(event, Dialog)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use rat_widget::button::{Button, ButtonState};
    use ratatui_core::style::Modifier;
    use ratatui_crossterm::crossterm::event::{
        Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind,
    };

    /// The region the dialog is drawn into: the same size the frame gives it on a 40×10 screen.
    const REGION: Rect = Rect::new(0, 1, 40, 8);

    /// The theme the dialog is drawn in.
    fn theme() -> SalsaTheme {
        Config::default().theme
    }

    /// The dialog, opened and painted once.
    fn drawn() -> (MsgDialogState, Buffer) {
        let mut state = MsgDialogState::default();
        open(&mut state);

        let mut buffer = Buffer::empty(REGION);
        draw(REGION, &mut buffer, &mut state, &theme());

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

    /// The event the terminal reports for the pointer moving to `(column, row)`.
    fn moved(column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// **The title and the body come from the application's own name and version** (D5), so the
    /// dialog and the status line cannot say two different things.
    #[test]
    fn the_about_dialog_carries_the_name_and_the_version() {
        let (_, buffer) = drawn();
        let frame = picture(&buffer);

        assert!(
            frame.contains(&format!("About {}", status::NAME)),
            "the dialog has no title: {frame:?}"
        );
        assert!(
            frame.contains(&format!("{} {}", status::NAME, status::VERSION)),
            "the dialog does not carry the name and the version: {frame:?}"
        );
        assert!(frame.contains("Ok"), "the dialog has no way out: {frame:?}");
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
}
