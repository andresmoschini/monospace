//! The menu: the bar, the drop-down a label opens, and what an entry asks for.
//!
//! The bar is drawn in two pieces with the body between them, so that the drop-down covers the
//! body instead of pushing it down.

use rat_salsa::Control;
use rat_widget::event::MenuOutcome;
use rat_widget::menu::{Menubar, MenubarLine, MenubarPopup, MenubarState, StaticMenu, menubar};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, StatefulWidget};

use crate::vocabulary::AppEvent;

static MENU: StaticMenu = StaticMenu {
    menu: &[("_File", &["_Quit|Ctrl+Q"]), ("_Edit", &["_Increment|+"])],
};

const FILE: usize = 0;

const QUIT: usize = 0;

const EDIT: usize = 1;

const INCREMENT: usize = 0;

/// Where a key or a click goes while the bar is the whole of the chrome.
///
/// The bar holds the keyboard outright, so it is always given the focus. The menubar takes the
/// drop-down while it is up and the bar otherwise. The bar answers a click only after it has been
/// drawn once, because the areas it tests against come from the last frame.
pub(crate) fn handle(raw: &Event, state: &mut MenubarState) -> Control<AppEvent> {
    let outcome = menubar::handle_popup_events(state, true, raw);

    match outcome {
        MenuOutcome::MenuActivated(label, entry) => {
            // The entry closes the drop-down, and a known one becomes an `AppEvent` for the loop
            // to feed back to `event` — the same place a key arrives.
            state.set_popup_active(false);
            // The bar carries text and nothing else: an entry means what its position says.
            match (label, entry) {
                (FILE, QUIT) => Control::Event(AppEvent::Quit),
                (EDIT, INCREMENT) => Control::Event(AppEvent::Increment),
                _ => Control::Changed,
            }
        }
        outcome => outcome.into(),
    }
}

pub(crate) fn draw_bar(area: Rect, buf: &mut Buffer, state: &mut MenubarState) {
    let (bar, _) = widgets();
    bar.render(area, buf, state);
}

/// Draws the open drop-down over the body, from the bar's area: it hangs from the bar and covers
/// what is under it.
pub(crate) fn draw_popup(area: Rect, buf: &mut Buffer, state: &mut MenubarState) {
    let (_, drop_down) = widgets();
    drop_down.render(area, buf, state);
}

/// The bar and the drop-down the menubar splits into, returned apart because they are drawn
/// separately.
fn widgets() -> (MenubarLine<'static>, MenubarPopup<'static>) {
    Menubar::new(&MENU)
        .popup_block(Block::bordered())
        .into_widgets()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{
        KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };

    const MENU_BAR_HEIGHT: u16 = 1;
    const WIDTH: u16 = 60;
    const HEIGHT: u16 = 14;

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn click(column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    fn draw(state: &mut MenubarState) -> Buffer {
        let bar_area = Rect::new(0, 0, WIDTH, MENU_BAR_HEIGHT);
        let mut buffer = Buffer::empty(Rect::new(0, 0, WIDTH, HEIGHT));
        draw_bar(bar_area, &mut buffer, state);
        draw_popup(bar_area, &mut buffer, state);
        buffer
    }

    /// The bar only knows where its labels are after a frame has drawn them.
    fn drawn() -> MenubarState {
        let mut state = MenubarState::default();
        draw(&mut state);
        state
    }

    fn frame_of(state: &mut MenubarState) -> String {
        let buffer = draw(state);
        (buffer.area.top()..buffer.area.bottom())
            .map(|y| {
                (buffer.area.left()..buffer.area.right())
                    .map(|x| buffer[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn press(code: KeyCode, state: &mut MenubarState) {
        let outcome = handle(&key(code), state);
        assert_eq!(outcome, Control::Changed, "{code:?} changed nothing");
    }

    /// Opens a label's drop-down by its letter, leaving the first entry selected.
    fn open(label: char, state: &mut MenubarState) {
        press(KeyCode::Char(label), state);
        draw(state);
    }

    /// Where a piece of the bar was drawn, found in the frame so that a growing label does not
    /// move the click away from it.
    fn cell_of(state: &mut MenubarState, text: &str) -> (u16, u16) {
        frame_of(state)
            .lines()
            .enumerate()
            .find_map(|(y, line)| line.find(text).map(|x| (x, y)))
            .map_or_else(
                || panic!("{text} is not on the frame"),
                |(x, y)| {
                    (
                        u16::try_from(x).expect("a column fits a frame"),
                        u16::try_from(y).expect("a row fits a frame"),
                    )
                },
            )
    }

    fn column_of_label(state: &mut MenubarState, label: &str) -> u16 {
        let frame = frame_of(state);
        let bar = frame.lines().next().unwrap_or_default();
        let column = bar.find(label.trim_start_matches('_')).unwrap_or_else(|| {
            panic!("{label} is not on the bar: {bar:?}");
        });

        u16::try_from(column).expect("a column fits a frame")
    }

    #[test]
    fn the_bar_holds_the_labels_under_test() {
        let labels: Vec<&str> = MENU.menu.iter().map(|(label, _)| *label).collect();

        assert_eq!(
            labels,
            ["_File", "_Edit"],
            "the bar is not the one under test"
        );
    }

    #[test]
    fn every_entry_asks_for_something() {
        for (label, entries) in MENU.menu {
            for (entry, text) in entries.iter().enumerate() {
                let mut state = drawn();
                let letter = label
                    .chars()
                    .find(|c| c.is_alphanumeric())
                    .expect("a label has a letter")
                    .to_ascii_lowercase();
                open(letter, &mut state);
                for _ in 0..entry {
                    press(KeyCode::Down, &mut state);
                    draw(&mut state);
                }

                let outcome = handle(&key(KeyCode::Enter), &mut state);

                assert!(
                    !matches!(outcome, Control::Changed),
                    "{text:?} is on the bar and nothing knows what it means"
                );
            }
        }
    }

    #[test]
    fn the_key_of_a_label_opens_its_drop_down() {
        let mut state = drawn();

        let outcome = handle(&key(KeyCode::Char('f')), &mut state);

        assert!(matches!(outcome, Control::Changed));
        assert!(state.popup_active(), "the drop-down did not open");
        let frame = frame_of(&mut state);
        assert!(frame.contains("Quit"), "the drop-down is empty: {frame:?}");
    }

    #[test]
    fn taking_quit_out_of_the_drop_down_becomes_the_application_event() {
        let mut state = drawn();
        open('f', &mut state);

        let outcome = handle(&key(KeyCode::Enter), &mut state);

        assert!(
            matches!(outcome, Control::Event(AppEvent::Quit)),
            "the entry did not become an application event: {outcome:?}"
        );
        assert!(!state.popup_active(), "the drop-down stayed open behind it");
    }

    #[test]
    fn taking_the_increment_entry_becomes_its_application_event() {
        let mut state = drawn();
        open('e', &mut state);

        let outcome = handle(&key(KeyCode::Enter), &mut state);

        assert!(
            matches!(outcome, Control::Event(AppEvent::Increment)),
            "the entry did not become its application event: {outcome:?}"
        );
    }

    #[test]
    fn a_click_on_a_label_opens_its_drop_down() {
        let mut state = drawn();
        let column = column_of_label(&mut state, "_File");

        let outcome = handle(&click(column, 0), &mut state);

        assert!(
            matches!(outcome, Control::Changed),
            "the click was not consumed"
        );
        assert!(state.popup_active(), "the click did not open the drop-down");
    }

    #[test]
    fn a_click_on_the_bar_that_is_not_on_a_label_is_nothing() {
        let mut state = drawn();

        let outcome = handle(&click(WIDTH - 1, 0), &mut state);

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
        assert!(
            !state.popup_active(),
            "a click on the bar opened a drop-down"
        );
    }

    #[test]
    fn a_click_on_an_entry_takes_it_out_of_the_drop_down() {
        let mut state = drawn();
        open('f', &mut state);
        let (column, row) = cell_of(&mut state, "Quit");

        let outcome = handle(&click(column, row), &mut state);

        assert!(
            matches!(outcome, Control::Event(AppEvent::Quit)),
            "the click did not become the application event: {outcome:?}"
        );
    }

    #[test]
    fn a_key_the_bar_does_not_use_is_the_applications_own() {
        let mut state = drawn();

        let outcome = handle(&key(KeyCode::Char('+')), &mut state);

        assert!(matches!(outcome, Control::Continue), "{outcome:?}");
    }

    #[test]
    fn escape_closes_the_drop_down() {
        let mut state = drawn();
        press(KeyCode::Char('f'), &mut state);

        press(KeyCode::Esc, &mut state);

        assert!(!state.popup_active(), "escape did not close the drop-down");
    }

    #[test]
    fn a_repeat_is_a_press_as_far_as_the_bar_is_concerned() {
        let mut state = drawn();
        let repeat = Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char('f'),
            KeyModifiers::NONE,
            KeyEventKind::Repeat,
        ));

        let outcome = handle(&repeat, &mut state);

        assert!(matches!(outcome, Control::Changed));
        assert!(state.popup_active());
    }
}
