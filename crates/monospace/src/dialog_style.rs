//! The style every `DialogFrame` overlay is drawn in.
//!
//! **One style, every dialog.** The open dialog, the save dialog and the text editor are all the
//! framework's `DialogFrame`, and the cue that says which control holds the keyboard is the same in
//! all three (D3 of #189). Keeping it here is what stops the same six lines from being copied into
//! every overlay that arrives.
//!
//! **Every state that is not the pointer is a state**: the focused button and the pressed one are
//! both reversed, so pressing an `Ok` the keyboard is not on cannot look like a second kind of
//! focus. `hover` is cleared because a control under the pointer is not a state of the control.

use rat_theme4::WidgetStyle;
use rat_theme4::theme::SalsaTheme;
use rat_widget::dialog_frame::DialogFrameStyle;
use ratatui_core::style::Style;

/// The style of a `DialogFrame`, with the buttons' focus cue replaced by reverse video.
pub(crate) fn style(theme: &SalsaTheme) -> DialogFrameStyle {
    let mut style: DialogFrameStyle = theme.style(WidgetStyle::DIALOG_FRAME);

    if let Some(button) = style.button_style.as_mut() {
        button.focus = Some(Style::new().reversed());
        button.armed = Some(Style::new().reversed());
        button.hover = None;
    }

    style
}
