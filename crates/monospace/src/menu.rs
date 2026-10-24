//! The bar across the top, and the drop-down it opens.
//!
//! **The bar is drawn in two pieces with the body between them, so that the drop-down covers the
//! body instead of pushing it down.** The menu widget splits itself into a line and a pop-up, and
//! the two are drawn at different moments: the line with the rest of the frame, the pop-up after
//! the region it covers.
//!
//! **The bar holds the keyboard.** The chrome of this slice is the bar and nothing else, so it is
//! always given the focus; the rules about the focus are #189's, and they are about a chrome
//! with more than one thing in it.
//!
//! **The framework's keys are the framework's.** The navigation character opens a drop-down, the
//! arrows move it, `Enter` takes an entry and `Esc` closes it, and all four are the menu
//! widget's own handling. What the bar adds is what an entry means.

use rat_theme4::WidgetStyle;
use rat_theme4::theme::SalsaTheme;
use rat_widget::event::MenuOutcome;
use rat_widget::menu::menubar;
use rat_widget::menu::{MenuStyle, Menubar, MenubarLine, MenubarPopup, MenubarState, StaticMenu};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;

use crate::vocabulary::{Action, Answer};

/// The bar, and what each of its entries means.
///
/// **The shortcut on the right of the entry is drawn by the widget and answered to by the
/// application**, which is what the table in `shortcuts` is for. The two are two places that can
/// disagree, and joining them is #180's.
static MENU: StaticMenu = StaticMenu {
    menu: &[("_File", &["_Quit|Ctrl+Q"])],
};

/// The label of the only menu of the bar.
const FILE: usize = 0;

/// The entry of the only menu of the bar.
const QUIT: usize = 0;

/// What an event means to the chrome.
///
/// **The bar is asked about every event and answers what it was worth**: nothing if it took no
/// interest, a repaint if it changed what the screen shows, and an action if an entry was taken.
/// A shortcut is not the bar's to answer: the application's own are asked around it, in
/// `callbacks` — before it the ones that reach everything, after it the ones nobody has claimed —
/// so the bar says nothing about one.
pub(crate) fn handle(raw: &Event, state: &mut MenubarState) -> Answer {
    let outcome = menubar::handle_popup_events(state, true, raw);

    match outcome {
        // An entry was taken: it means an action, and `callbacks` applies it. The loop is not
        // asked again, so an entry and the key that means the same thing end in one place.
        MenuOutcome::MenuActivated(label, entry) => {
            state.set_popup_active(false);
            meaning(label, entry)
        }

        outcome => outcome.into(),
    }
}

/// What an entry of the bar means to the application.
///
/// **The bar carries text and nothing else**: an entry means what its position says.
fn meaning(label: usize, entry: usize) -> Answer {
    match (label, entry) {
        (FILE, QUIT) => Answer::Action(Action::Quit),

        // An entry nobody knows still closed the drop-down, so the frame has to be painted again.
        _ => Answer::Repaint,
    }
}

/// What the bar's own outcomes are worth to the application.
///
/// **The widget's vocabulary is finer than the application's.** Selecting, activating and hiding
/// are all "the screen changed"; the one that is more than that is the entry that was taken, and
/// `handle` has taken it before this conversion matters.
impl From<MenuOutcome> for Answer {
    fn from(outcome: MenuOutcome) -> Self {
        match outcome {
            MenuOutcome::Continue | MenuOutcome::Unchanged => Self::Nothing,

            MenuOutcome::Changed
            | MenuOutcome::Hide
            | MenuOutcome::Selected(_)
            | MenuOutcome::Activated(_)
            | MenuOutcome::MenuSelected(_, _)
            | MenuOutcome::MenuActivated(_, _) => Self::Repaint,
        }
    }
}

/// Paints the bar across the top.
pub(crate) fn draw_bar(area: Rect, buf: &mut Buffer, state: &mut MenubarState, theme: &SalsaTheme) {
    let (bar, _) = widgets(theme);
    bar.render(area, buf, state);
}

/// Paints the drop-down the bar opens, over what is under it.
///
/// The area is not the bar's: the pop-up hangs from the bar and covers the region, and the
/// widget works out where that is from the frame it was drawn in.
pub(crate) fn draw_drop_down(
    area: Rect,
    buf: &mut Buffer,
    state: &mut MenubarState,
    theme: &SalsaTheme,
) {
    let (_, drop_down) = widgets(theme);
    drop_down.render(area, buf, state);
}

/// The bar and the drop-down, as the two widgets the menu splits itself into.
fn widgets(theme: &SalsaTheme) -> (MenubarLine<'static>, MenubarPopup<'static>) {
    let style: MenuStyle = theme.style(WidgetStyle::MENU);
    Menubar::new(&MENU).styles(style).into_widgets()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use ratatui_crossterm::crossterm::event::{
        KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };

    /// The width of the screen these tests draw.
    const WIDTH: u16 = 40;

    /// The height of the screen these tests draw.
    const HEIGHT: u16 = 10;

    /// The theme the chrome is drawn in.
    fn theme() -> SalsaTheme {
        Config::default().theme
    }

    /// The bar and its drop-down, drawn into a buffer of the whole screen.
    fn draw(state: &mut MenubarState) -> Buffer {
        let area = Rect::new(0, 0, WIDTH, HEIGHT);
        let mut buffer = Buffer::empty(area);
        draw_bar(Rect::new(0, 0, WIDTH, 1), &mut buffer, state, &theme());
        draw_drop_down(area, &mut buffer, state, &theme());
        buffer
    }

    /// The bar, drawn once so that the widget knows where its labels are.
    fn drawn() -> MenubarState {
        let mut state = MenubarState::default();
        draw(&mut state);
        state
    }

    /// The picture of the frame, as text.
    fn picture(buffer: &Buffer) -> String {
        (buffer.area.top()..buffer.area.bottom())
            .map(|line| {
                (buffer.area.left()..buffer.area.right())
                    .map(|column| buffer[(column, line)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Where a piece of text is on the frame, found in the frame itself so that a growing
    /// label does not move the click away from it.
    fn cell_of(buffer: &Buffer, text: &str) -> (u16, u16) {
        crate::frame::cell_of(buffer, text)
    }

    /// Opens the drop-down of the label, the way a person does.
    ///
    /// **The frame is drawn after the key as well as before it**: the widget knows what it
    /// holds only once it has been drawn, and a frame that is not drawn leaves the drop-down
    /// without an entry to take.
    fn open(state: &mut MenubarState) {
        let _ = handle(&key(KeyCode::Char('f')), state);
        draw(state);
    }

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn key_with(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    fn click(column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    #[test]
    fn the_navigation_character_opens_the_drop_down_and_escape_closes_it() {
        let mut state = drawn();

        let outcome = handle(&key(KeyCode::Char('f')), &mut state);

        assert!(
            matches!(outcome, Answer::Repaint),
            "the navigation character changed nothing: {outcome:?}"
        );
        assert!(state.popup_active(), "the drop-down did not open");

        let outcome = handle(&key(KeyCode::Esc), &mut state);

        assert!(
            matches!(outcome, Answer::Repaint),
            "escape changed nothing: {outcome:?}"
        );
        assert!(!state.popup_active(), "escape did not close the drop-down");
    }

    #[test]
    fn taking_the_quit_entry_becomes_the_action_of_the_way_out() {
        let mut state = drawn();
        open(&mut state);

        let outcome = handle(&key(KeyCode::Enter), &mut state);

        assert!(
            matches!(outcome, Answer::Action(Action::Quit)),
            "the entry did not become the action of the way out: {outcome:?}"
        );
        assert!(!state.popup_active(), "the drop-down stayed open behind it");
    }

    #[test]
    fn the_shortcuts_are_not_the_bars() {
        let mut state = drawn();

        let outcome = handle(
            &key_with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert!(
            matches!(outcome, Answer::Nothing),
            "the bar answered a shortcut: {outcome:?}"
        );
        assert!(!state.popup_active(), "a shortcut opened a drop-down");
    }

    #[test]
    fn a_click_on_the_entry_becomes_the_same_action() {
        let mut state = drawn();
        open(&mut state);
        let buffer = draw(&mut state);
        let (column, row) = cell_of(&buffer, "Quit");

        let outcome = handle(&click(column, row), &mut state);

        assert!(
            matches!(outcome, Answer::Action(Action::Quit)),
            "the click did not become the action of the way out: {outcome:?}"
        );
    }

    #[test]
    fn a_click_on_the_label_opens_its_drop_down() {
        let mut state = drawn();
        let buffer = draw(&mut state);
        let (column, row) = cell_of(&buffer, "File");

        let outcome = handle(&click(column, row), &mut state);

        assert!(
            matches!(outcome, Answer::Repaint),
            "the click was not consumed: {outcome:?}"
        );
        assert!(state.popup_active(), "the click did not open the drop-down");
    }

    #[test]
    fn a_key_the_bar_does_not_use_is_nothing() {
        let mut state = drawn();

        let outcome = handle(&key(KeyCode::Char('z')), &mut state);

        assert!(matches!(outcome, Answer::Nothing), "{outcome:?}");
        assert!(!state.popup_active(), "a key opened a drop-down");
    }

    #[test]
    fn the_drop_down_shows_the_shortcut_the_entry_answers_to() {
        let mut state = drawn();
        open(&mut state);

        let buffer = draw(&mut state);
        let frame = picture(&buffer);

        assert!(
            frame.contains("Ctrl+Q"),
            "the drop-down shows no shortcut: {frame:?}"
        );
    }
}
