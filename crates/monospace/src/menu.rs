//! The bar across the top, and the drop-down it opens.
//!
//! **The bar is drawn in two pieces with the body between them, so that the drop-down covers the
//! body instead of pushing it down.** The menu widget splits itself into a line and a pop-up, and
//! the two are drawn at different moments: the line with the rest of the frame, the pop-up after
//! the region it covers.
//!
//! **The bar is one of the two leaves that hold the keyboard.** Which one holds it is the
//! application's answer — kept in `AppState` — and it is given to the bar here: the widget draws
//! its selected label differently only while it has the focus, and it answers a key only while it
//! has it. **The bar also asks for it and gives it back**: `Alt` plus a navigation character is
//! given to the bar before it is asked, `F10` asks for the keyboard from either leaf, and `Esc`
//! with nothing left to close gives it back. The focus itself is the application's, so the bar
//! answers with an action and the application carries it out. The rules about the focus are
//! #189's.
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
use ratatui_core::style::Style;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::vocabulary::Action;

/// The bar, and what each of its entries means.
///
/// **The shortcut on the right of the entry is drawn by the widget and answered to by the
/// application**, which is what the table in `shortcuts` is for. The two are two places that can
/// disagree, and joining them is #180's.
///
/// **Every entry is offered all the time.** The bar carries no state about what is selected or
/// whether a step can be given back, so the Edit menu's entries are active at every position of the
/// history: taking one with nothing to do is an action that changes nothing and repaints all the
/// same, the same answer the key gives (D2 of #181).
///
/// **The label's navigation character is the letter after its `_`**, and it is the only table of
/// names: `Alt` reaches the same character here that a bare press answers when the bar holds the
/// keyboard (D4 of #189). Help is last, so adding it leaves File and Edit where they were.
static MENU: StaticMenu = StaticMenu {
    menu: &[
        ("_File", &["_Quit|Ctrl+Q"]),
        (
            "_Edit",
            &["_Undo|Ctrl+Z", "_Redo|Ctrl+Shift+Z", "_Delete|Del"],
        ),
        ("_Help", &["_About"]),
    ],
};

/// The label of the way out.
const FILE: usize = 0;

/// The entry of the way out.
const QUIT: usize = 0;

/// The label of the editing commands.
const EDIT: usize = 1;

/// The entry that gives back the step behind.
const UNDO: usize = 0;

/// The entry that gives back the step ahead.
const REDO: usize = 1;

/// The entry that removes the selected figure.
const DELETE: usize = 2;

/// The label of the dialog.
const HELP: usize = 2;

/// The entry that says what the application is.
const ABOUT: usize = 0;

/// What an event means to the chrome.
///
/// **The bar is asked about every event and says what it was worth**: `None` if it took no
/// interest, `ScreenChanged` if it changed what the screen shows, and the action the event means —
/// the entry that was taken, or the keyboard leaving the bar or arriving there. A shortcut is not
/// the bar's to answer: the application's own are asked around it, in `callbacks` — before it the
/// ones that reach everything, after it the ones nobody has claimed — so the bar says nothing about
/// one.
///
/// **`None` is `Continue` and `Unchanged`, and the application makes no difference between them**:
/// the widget handled the event and there is nothing for the screen to paint, so the shortcuts
/// nobody has claimed are still asked. **The widget only answers a key while the bar holds the
/// keyboard**: `focused` is what the application's focus says, and a bar that does not hold it
/// answers no key and closes the drop-down the keyboard left behind.
///
/// **`Alt` and a navigation character reach a bar that already holds the keyboard**: the router
/// gives it to the bar before asking (D4), so the translated press is a character the widget
/// already answers.
///
/// **`F10` asks for the keyboard and `Esc` gives it back** (D4, amended; rule 5): `F10` from either
/// leaf, `Esc` while the bar holds the keyboard and no drop-down is open. **The bar does not move
/// the focus itself**, because the focus is the application's: it answers `Action::FocusBar` or
/// `Action::FocusRegion`, and the action is carried out where every other one is.
pub(crate) fn handle(raw: &Event, state: &mut MenubarState, focused: bool) -> Option<Action> {
    // **`F10` asks for the bar** (D4, amended): the key every terminal reports, where a bare `Alt`
    // is not one. It asks for the keyboard and opens nothing, so the bar looks focused with no
    // drop-down in the way.
    if !focused && asks_for_the_bar(raw) {
        return Some(Action::FocusBar);
    }

    // **`Esc` on a bar with nothing left to close gives the keyboard back** (rule 5): the widget
    // answers the key while a drop-down is open, and this is the press after it.
    if focused && steps_back(raw, state) {
        return Some(Action::FocusRegion);
    }

    let translated;
    let raw = match alt_navigation(raw) {
        Some(character) => {
            translated = Event::Key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE));
            &translated
        }
        None => raw,
    };

    match menubar::handle_popup_events(state, focused, raw) {
        // An entry was taken: it means an action, and `callbacks` applies it. The loop is not
        // asked again, so an entry and the key that means the same thing end in one place.
        MenuOutcome::MenuActivated(label, entry) => {
            state.set_popup_active(false);
            Some(meaning(label, entry))
        }

        // **The widget's vocabulary is finer than the application's.** Selecting, activating and
        // hiding are all "the screen changed"; the other two leave the application nothing to do,
        // and the shortcuts nobody has claimed are asked after it.
        MenuOutcome::Changed
        | MenuOutcome::Hide
        | MenuOutcome::Selected(_)
        | MenuOutcome::Activated(_)
        | MenuOutcome::MenuSelected(_, _) => Some(Action::ScreenChanged),

        MenuOutcome::Continue | MenuOutcome::Unchanged => None,
    }
}

/// The navigation character an `Alt` press names, or `None` where the event is not one.
///
/// **The answer comes from the labels** (D4): the character the menu answers to is the letter
/// after its `_`, and looking it up here keeps the bar the only table of names. **A press of any
/// other key, or with any other modifier, is not one**: a bare `Alt` is not a key, and a
/// combination that names no label changes nothing.
pub(crate) fn alt_navigation(raw: &Event) -> Option<char> {
    let Event::Key(key) = raw else {
        return None;
    };

    if key.kind != KeyEventKind::Press || key.modifiers != KeyModifiers::ALT {
        return None;
    }

    let KeyCode::Char(character) = key.code else {
        return None;
    };
    let character = character.to_ascii_lowercase();

    MENU.menu
        .iter()
        .any(|(label, _)| navigation(label) == Some(character))
        .then_some(character)
}

/// Whether the event is the `F10` press that asks for the bar.
///
/// **`F10` is the key every terminal reports** (D4, amended): a bare `Alt` is not one — the
/// framework asks for the terminal's extended keys only off Windows, and a terminal without them
/// sends nothing for `Alt` alone — so the bar is reachable from the region with a key that is
/// always there. **It asks only for the keyboard**: no drop-down opens, and the bar draws its first
/// label in reverse video because that is what holding the keyboard looks like. [`handle`] asks it
/// only while the bar does not hold the keyboard, and answers that the keyboard goes there.
fn asks_for_the_bar(event: &Event) -> bool {
    matches!(
        event,
        Event::Key(KeyEvent {
            code: KeyCode::F(10),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            ..
        })
    )
}

/// Whether the event tells the bar to give the keyboard back.
///
/// **A bar with a drop-down open has something to close**, so `Esc` is the widget's there; with none
/// open the key has nothing left to do here, and this is the bar's own reading of it: the bar steps
/// back and the keyboard goes to the region (rule 5). [`handle`] asks it only while the bar holds
/// the keyboard, and answers that the keyboard goes back.
fn steps_back(event: &Event, state: &MenubarState) -> bool {
    !state.popup_active() && is_escape(event)
}

/// Whether the event is an `Esc` press, with no modifier on it and no kind but a press.
fn is_escape(event: &Event) -> bool {
    matches!(
        event,
        Event::Key(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            ..
        })
    )
}

/// The navigation character a label carries: the letter after its `_`.
fn navigation(label: &str) -> Option<char> {
    label
        .strip_prefix('_')
        .and_then(|rest| rest.chars().next())
        .map(|character| character.to_ascii_lowercase())
}

/// What an entry of the bar means to the application.
///
/// **The bar carries text and nothing else**: an entry means what its position says.
fn meaning(label: usize, entry: usize) -> Action {
    match (label, entry) {
        (FILE, QUIT) => Action::Quit,
        (EDIT, UNDO) => Action::Undo,
        (EDIT, REDO) => Action::Redo,
        (EDIT, DELETE) => Action::Delete,
        (HELP, ABOUT) => Action::About,

        // An entry nobody knows still closed the drop-down, so the frame has to be painted again.
        _ => Action::ScreenChanged,
    }
}

/// Paints the bar across the top.
///
/// **The focus is given before the frame is drawn**, because the widget paints its selected label
/// differently only while it holds the keyboard: a frame that did not say which leaf holds it
/// would draw the label the way the last event left it.
pub(crate) fn draw_bar(
    area: Rect,
    buf: &mut Buffer,
    state: &mut MenubarState,
    theme: &SalsaTheme,
    focused: bool,
) {
    let (bar, _) = widgets(theme);
    state.bar.focus.set(focused);
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
///
/// **The focus is reverse video over the theme's own colors** (D3 of #189): the two focus styles
/// are replaced with the modifier alone, so the base style painted under the label keeps its
/// colors and the widget adds `Modifier::REVERSED` to the cells it owns. The theme's `focus`
/// background is not used, which is what makes the bar and the dialog's `Ok` say the same thing.
fn widgets(theme: &SalsaTheme) -> (MenubarLine<'static>, MenubarPopup<'static>) {
    let mut style: MenuStyle = theme.style(WidgetStyle::MENU);
    style.focus = Some(Style::new().reversed());
    style.popup_focus = Some(Style::new().reversed());
    Menubar::new(&MENU).styles(style).into_widgets()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use ratatui_crossterm::crossterm::event::{
        KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
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
        draw_with(state, true)
    }

    /// The bar and its drop-down, with the keyboard on the bar or not.
    fn draw_with(state: &mut MenubarState, focused: bool) -> Buffer {
        let area = Rect::new(0, 0, WIDTH, HEIGHT);
        let mut buffer = Buffer::empty(area);
        draw_bar(
            Rect::new(0, 0, WIDTH, 1),
            &mut buffer,
            state,
            &theme(),
            focused,
        );
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
            .map(|line| row_of(buffer, line))
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// One row of the frame, as text.
    fn row_of(buffer: &Buffer, line: u16) -> String {
        (buffer.area.left()..buffer.area.right())
            .map(|column| buffer[(column, line)].symbol().to_string())
            .collect()
    }

    /// Where a piece of text is on the frame, found in the frame itself so that a growing
    /// label does not move the click away from it.
    fn cell_of(buffer: &Buffer, text: &str) -> (u16, u16) {
        crate::frame::cell_of(buffer, text)
    }

    /// Whether the cell at a frame position is painted in reverse video.
    fn reversed(buffer: &Buffer, column: u16, line: u16) -> bool {
        buffer[(column, line)]
            .style()
            .add_modifier
            .contains(ratatui_core::style::Modifier::REVERSED)
    }

    /// Hands the event to the bar with the keyboard on it, where the test is not about the focus.
    fn on_the_bar(event: &Event, state: &mut MenubarState) -> Option<Action> {
        handle(event, state, true)
    }

    /// Opens the drop-down of the label, the way a person does.
    ///
    /// **The frame is drawn after the key as well as before it**: the widget knows what it
    /// holds only once it has been drawn, and a frame that is not drawn leaves the drop-down
    /// without an entry to take.
    fn open_with(state: &mut MenubarState, navigation: char) {
        let _ = on_the_bar(&key(KeyCode::Char(navigation)), state);
        draw(state);
    }

    /// Opens the File drop-down.
    fn open(state: &mut MenubarState) {
        open_with(state, 'f');
    }

    /// Takes the entry whose text is `text`, by clicking it on the frame, the way a person does.
    ///
    /// **The entry is found by its name**: a menu that gains entries above one moves where it is
    /// but not what it says, and a test that pressed `Enter` would take whatever happens to be
    /// first. **The pointer moves onto it before the button goes down**, because the widget
    /// activates the entry it has selected and a press alone selects nothing.
    fn take(state: &mut MenubarState, text: &str) -> Option<Action> {
        let buffer = draw(state);
        let (column, row) = cell_of(&buffer, text);
        let _ = on_the_bar(&moved(column, row), state);

        on_the_bar(&click(column, row), state)
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

    /// The event the terminal reports for the pointer moving to `(column, row)`.
    fn moved(column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// **The control the keyboard is on is drawn in reverse video over the theme's colors**
    /// (D3 of #189): the focused label gains the modifier and keeps the foreground and background
    /// the base style gave it, so the theme's `focus` background is not what says where the
    /// keyboard is.
    #[test]
    fn a_control_the_keyboard_is_on_is_drawn_in_reverse_video() {
        let mut without = MenubarState::default();
        let unfocused = draw_with(&mut without, false);
        let (column, row) = cell_of(&unfocused, "File");
        let plain = unfocused[(column, row)].style();

        let mut with = drawn();
        let focused = draw(&mut with);
        let lit = focused[(column, row)].style();

        assert!(
            !reversed(&unfocused, column, row),
            "a label is reversed while the bar does not hold the keyboard"
        );
        assert!(
            reversed(&focused, column, row),
            "the label the keyboard is on is not reversed"
        );
        assert_eq!(lit.fg, plain.fg, "the focus changed the label's foreground");
        assert_eq!(
            lit.bg, plain.bg,
            "the theme's focus background is used, not reverse video"
        );

        // The rest of the bar keeps the plain style: the cue is one label, not the whole row.
        let (column, row) = cell_of(&focused, "Edit");
        assert!(
            !reversed(&focused, column, row),
            "another label is reversed with the one the keyboard is on"
        );
    }

    #[test]
    fn the_navigation_character_opens_the_drop_down_and_escape_closes_it() {
        let mut state = drawn();

        let outcome = on_the_bar(&key(KeyCode::Char('f')), &mut state);

        assert!(
            matches!(outcome, Some(Action::ScreenChanged)),
            "the navigation character did not report a screen change: {outcome:?}"
        );
        assert!(state.popup_active(), "the drop-down did not open");

        let outcome = on_the_bar(&key(KeyCode::Esc), &mut state);

        assert!(
            matches!(outcome, Some(Action::ScreenChanged)),
            "escape did not report a screen change: {outcome:?}"
        );
        assert!(!state.popup_active(), "escape did not close the drop-down");
    }

    /// **The key that leaves the bar is a press and nothing else** (rule 5): a release, a repeat,
    /// and a press with a modifier on it are not the bar stepping back, and neither is a press
    /// while a drop-down is open.
    #[test]
    fn an_escape_press_is_the_bars_way_out() {
        let mut state = drawn();

        assert!(
            steps_back(&key(KeyCode::Esc), &state),
            "the escape press was not read"
        );

        for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
            let event = Event::Key(KeyEvent::new_with_kind(
                KeyCode::Esc,
                KeyModifiers::NONE,
                kind,
            ));

            assert!(!steps_back(&event, &state), "{kind:?} was read as it");
        }

        assert!(
            !steps_back(&key_with(KeyCode::Esc, KeyModifiers::CONTROL), &state),
            "a modified escape was read as it"
        );
        assert!(
            !steps_back(&key(KeyCode::Enter), &state),
            "another key was read as it"
        );

        open(&mut state);
        assert!(
            !steps_back(&key(KeyCode::Esc), &state),
            "the escape that closes the drop-down was read as the bar stepping back"
        );
    }

    /// **The key that asks for the bar is `F10` and nothing else** (D4, amended): not with a
    /// modifier on it, not released or repeated, and not a character a label already answers to.
    #[test]
    fn a_key_that_asks_for_the_bar_is_a_press_and_nothing_else() {
        assert!(
            asks_for_the_bar(&key(KeyCode::F(10))),
            "F10 did not ask for the bar"
        );
        assert!(
            !asks_for_the_bar(&key_with(KeyCode::F(10), KeyModifiers::SHIFT)),
            "a shifted F10 asked for the bar"
        );
        assert!(
            !asks_for_the_bar(&key(KeyCode::F(9))),
            "another function key asked for the bar"
        );
        assert!(
            !asks_for_the_bar(&key(KeyCode::Char('f'))),
            "a navigation character asked for the bar on its own"
        );

        for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
            let event = Event::Key(KeyEvent::new_with_kind(
                KeyCode::F(10),
                KeyModifiers::NONE,
                kind,
            ));

            assert!(!asks_for_the_bar(&event), "{kind:?} asked for the bar");
        }
    }

    #[test]
    fn taking_the_quit_entry_becomes_the_action_of_the_way_out() {
        let mut state = drawn();
        open(&mut state);

        let outcome = on_the_bar(&key(KeyCode::Enter), &mut state);

        assert!(
            matches!(outcome, Some(Action::Quit)),
            "the entry did not become the action of the way out: {outcome:?}"
        );
        assert!(!state.popup_active(), "the drop-down stayed open behind it");
    }

    /// **The Edit menu's entry is the key's action, and the bar holds no selection to condition
    /// it on.** The entry is always offered, and with nothing selected the application carries the
    /// action out as a deletion that removes nothing rather than the menu refusing to open it.
    #[test]
    fn taking_the_delete_entry_becomes_the_action_of_the_key() {
        let mut state = drawn();
        open_with(&mut state, 'e');

        let outcome = take(&mut state, "Delete");

        assert!(
            matches!(outcome, Some(Action::Delete)),
            "the entry did not become the action of the delete key: {outcome:?}"
        );
        assert!(!state.popup_active(), "the drop-down stayed open behind it");
    }

    /// **The Edit menu's entries are the keys' actions.** The undo is the first, and taking it says
    /// `Action::Undo` rather than a second spelling of the walk.
    #[test]
    fn taking_the_undo_entry_becomes_the_action_of_the_key() {
        let mut state = drawn();
        open_with(&mut state, 'e');

        let outcome = take(&mut state, "Undo");

        assert!(
            matches!(outcome, Some(Action::Undo)),
            "the entry did not become the action of the undo key: {outcome:?}"
        );
        assert!(!state.popup_active(), "the drop-down stayed open behind it");
    }

    /// **Help is the third label and About its only entry, with no shortcut drawn** (rule 8):
    /// About opens a dialog rather than a command, so there is no key to show beside it.
    #[test]
    fn the_help_menu_holds_about_and_draws_no_shortcut() {
        let mut state = drawn();
        open_with(&mut state, 'h');

        let buffer = draw(&mut state);
        let frame = picture(&buffer);

        assert!(frame.contains("Help"), "the bar has no Help: {frame:?}");
        assert!(
            frame.contains("About"),
            "the Help menu offers no About: {frame:?}"
        );

        let (_, row) = cell_of(&buffer, "About");
        let line = row_of(&buffer, row);

        assert!(
            !line.contains("Ctrl") && !line.contains("Del"),
            "the About entry draws a shortcut: {line:?}"
        );
    }

    /// **Taking About mints `Action::About`** (rule 8): the entry is a meaning, and the dialog is
    /// opened where the action is carried out rather than by the bar.
    #[test]
    fn taking_the_about_entry_mints_the_about_action() {
        let mut state = drawn();
        open_with(&mut state, 'h');

        let outcome = take(&mut state, "About");

        assert!(
            matches!(outcome, Some(Action::About)),
            "the entry did not become the action of the About dialog: {outcome:?}"
        );
        assert!(!state.popup_active(), "the drop-down stayed open behind it");
    }

    /// **The redo entry is the redo key's action**, the same way.
    #[test]
    fn taking_the_redo_entry_becomes_the_action_of_the_key() {
        let mut state = drawn();
        open_with(&mut state, 'e');

        let outcome = take(&mut state, "Redo");

        assert!(
            matches!(outcome, Some(Action::Redo)),
            "the entry did not become the action of the redo key: {outcome:?}"
        );
        assert!(!state.popup_active(), "the drop-down stayed open behind it");
    }

    /// **The Edit menu holds the three entries in the order the keys are learned**: Undo, Redo and
    /// Delete, each with the shortcut the table answers to (rule 9).
    #[test]
    fn the_edit_menu_shows_undo_and_redo_with_their_shortcuts() {
        let mut state = drawn();
        open_with(&mut state, 'e');

        let frame = picture(&draw(&mut state));

        assert!(
            frame.contains("Undo"),
            "the Edit menu offers no undo: {frame:?}"
        );
        assert!(
            frame.contains("Redo"),
            "the Edit menu offers no redo: {frame:?}"
        );
        assert!(
            frame.contains("Ctrl+Z"),
            "the undo entry shows no shortcut: {frame:?}"
        );
        assert!(
            frame.contains("Ctrl+Shift+Z"),
            "the redo entry shows no shortcut: {frame:?}"
        );

        let undo = frame.find("Undo").expect("the undo entry is on the frame");
        let redo = frame.find("Redo").expect("the redo entry is on the frame");
        let delete = frame
            .find("Delete")
            .expect("the delete entry is on the frame");

        assert!(
            undo < redo && redo < delete,
            "the entries are not Undo, Redo, Delete: {frame:?}"
        );
    }

    #[test]
    fn the_edit_menu_is_on_the_bar_and_shows_the_delete_entry() {
        let mut state = drawn();
        open_with(&mut state, 'e');

        let frame = picture(&draw(&mut state));

        assert!(
            frame.contains("Edit"),
            "the bar has no Edit menu: {frame:?}"
        );
        assert!(
            frame.contains("Delete"),
            "the Edit menu offers no delete: {frame:?}"
        );
        assert!(
            frame.contains("Del"),
            "the entry shows no shortcut: {frame:?}"
        );
    }

    #[test]
    fn the_shortcuts_are_not_the_bars() {
        let mut state = drawn();

        let outcome = handle(
            &key_with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut state,
            true,
        );

        assert!(
            outcome.is_none(),
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

        let outcome = on_the_bar(&click(column, row), &mut state);

        assert!(
            matches!(outcome, Some(Action::Quit)),
            "the click did not become the action of the way out: {outcome:?}"
        );
    }

    #[test]
    fn a_click_on_the_label_opens_its_drop_down() {
        let mut state = drawn();
        let buffer = draw(&mut state);
        let (column, row) = cell_of(&buffer, "File");

        let outcome = on_the_bar(&click(column, row), &mut state);

        assert!(
            matches!(outcome, Some(Action::ScreenChanged)),
            "the click did not report a screen change: {outcome:?}"
        );
        assert!(state.popup_active(), "the click did not open the drop-down");
    }

    #[test]
    fn a_key_the_bar_does_not_use_is_nothing() {
        let mut state = drawn();

        let outcome = on_the_bar(&key(KeyCode::Char('z')), &mut state);

        assert!(outcome.is_none(), "{outcome:?}");
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
