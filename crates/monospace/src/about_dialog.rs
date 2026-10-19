use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, BorderType, Padding, StatefulWidget},
};

const DIALOG_WIDTH: Constraint = Constraint::Length(40);
const DIALOG_HEIGHT: Constraint = Constraint::Length(16);

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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    const WIDTH: u16 = 60;
    const HEIGHT: u16 = 20;

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
}
