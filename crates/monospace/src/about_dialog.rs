use rat_widget::event::Outcome;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState, handle_dialog_events};
use ratatui::{
    buffer::Buffer,
    crossterm::event::Event,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, BorderType, Padding, StatefulWidget},
};

const DIALOG_WIDTH: Constraint = Constraint::Length(40);
const DIALOG_HEIGHT: Constraint = Constraint::Length(16);

const TITLE: &str = "About Monospace";

pub(crate) fn open(state: &mut MsgDialogState) {
    state.clear();
    state.title(TITLE);
    state.append(&text());
}

fn text() -> String {
    format!(
        "Monospace {}\n\nA diagram editor for the terminal.",
        env!("CARGO_PKG_VERSION")
    )
}

pub(crate) fn draw(body_area: Rect, buf: &mut Buffer, state: &mut MsgDialogState) {
    if !state.active() {
        return;
    }

    let area = Layout::horizontal([DIALOG_WIDTH])
        .flex(Flex::Center)
        .areas::<1>(body_area)[0];
    let area = Layout::vertical([DIALOG_HEIGHT])
        .flex(Flex::Center)
        .areas::<1>(area)[0];

    MsgDialog::new()
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(Padding::uniform(1)),
        )
        .render(area, buf, state);
}

/// What the dialog does with an event.
///
/// The widget swallows every event while it is open and closes on `esc` and on its `Ok` button, so
/// the application sees nothing until the dialog is gone.
pub(crate) fn handle(raw: &Event, state: &mut MsgDialogState) -> Outcome {
    handle_dialog_events(state, raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use ratatui::widgets::Widget;

    const WIDTH: u16 = 60;
    const HEIGHT: u16 = 20;

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn key_of_kind(code: KeyCode, kind: KeyEventKind) -> Event {
        Event::Key(KeyEvent::new_with_kind(code, KeyModifiers::NONE, kind))
    }

    fn frame(state: &mut MsgDialogState) -> String {
        let area = Rect::new(0, 0, WIDTH, HEIGHT);
        let mut buffer = Buffer::empty(area);
        // The drawing underneath, so that the test can tell whether it survived.
        Block::bordered()
            .title("monospace")
            .render(area, &mut buffer);
        draw(area, &mut buffer, state);
        (buffer.area.top()..buffer.area.bottom())
            .map(|y| {
                (buffer.area.left()..buffer.area.right())
                    .map(|x| buffer[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    #[test]
    fn a_closed_dialog_draws_nothing() {
        let mut state = MsgDialogState::default();

        let frame = frame(&mut state);

        assert!(frame.contains("monospace"), "{frame:?}");
    }

    #[test]
    fn an_open_dialog_carries_its_title_and_its_text() {
        let mut state = MsgDialogState::default();
        open(&mut state);

        let frame = frame(&mut state);

        assert!(frame.contains("About"), "{frame:?}");
        assert!(frame.contains("A diagram editor"), "{frame:?}");
    }

    #[test]
    fn opening_the_dialog_again_shows_the_text_again() {
        let mut state = MsgDialogState::default();
        open(&mut state);
        // The widget empties the message when the dialog closes.
        state.clear();

        open(&mut state);
        let frame = frame(&mut state);

        assert!(frame.contains("About"), "{frame:?}");
        assert!(frame.contains("A diagram editor"), "{frame:?}");
    }

    #[test]
    fn an_open_dialog_leaves_the_drawing_behind_it_standing() {
        let mut state = MsgDialogState::default();
        state.set_active(true);

        let frame = frame(&mut state);

        // The frame of the region is still there, around the dialog.
        assert!(frame.contains("monospace"), "{frame:?}");
        assert!(frame.starts_with('┌'), "{frame:?}");
        assert!(frame.contains("┐"), "{frame:?}");
    }

    #[test]
    fn the_dialog_is_a_box_in_the_middle_and_not_the_whole_frame() {
        let mut state = MsgDialogState::default();
        state.set_active(true);

        let frame = frame(&mut state);

        // The corners of the region are the corners of the drawing: the dialog did not take them.
        assert!(frame.starts_with('┌'), "{frame:?}");
        assert!(
            frame
                .lines()
                .nth(HEIGHT as usize - 1)
                .is_some_and(|row| row.starts_with('└')),
            "{frame:?}"
        );
    }

    #[test]
    fn escape_closes_the_dialog() {
        let mut state = MsgDialogState::default();
        open(&mut state);

        handle(&key(KeyCode::Esc), &mut state);

        assert!(!state.active(), "escape did not close the dialog");
    }

    #[test]
    fn the_ok_button_closes_the_dialog() {
        let mut state = MsgDialogState::default();
        open(&mut state);

        // The button fires on the release where the terminal reports key releases, and on the press
        // where it does not, so both are sent.
        handle(
            &key_of_kind(KeyCode::Enter, KeyEventKind::Press),
            &mut state,
        );
        handle(
            &key_of_kind(KeyCode::Enter, KeyEventKind::Release),
            &mut state,
        );

        assert!(!state.active(), "the Ok button did not close the dialog");
    }

    #[test]
    fn a_key_the_dialog_does_not_use_leaves_it_open() {
        let mut state = MsgDialogState::default();
        open(&mut state);

        handle(&key(KeyCode::Char('z')), &mut state);

        assert!(state.active(), "the dialog closed on a key it does not use");
    }
}
