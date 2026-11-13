//! The bar across the top, and the drop-down it opens.
//!
//! **The bar is drawn in two pieces with the body between them, so that the drop-down covers the
//! body instead of pushing it down.** The menu widget splits itself into a line and a pop-up, and
//! the two are drawn at different moments: the line with the rest of the frame, the pop-up after
//! the region it covers.
//!
//! **The bar is one of the two leaves that hold the keyboard.** Which one holds it is the
//! application's answer — kept in `AppState` — and the bar is handed it: the widget draws its
//! selected label differently only while it has the focus, and it answers a key only while it has
//! it. **The bar asks for the keyboard and gives it back**: `F10` asks for it from either leaf,
//! `Alt` plus a navigation character brings it with the press, and `Esc` with nothing left to close
//! gives it back. **A leaf is not a meaning** (D1 of #189): these three keys move the keyboard
//! where they are read, so the bar moves it and no action names a leaf. The rules about the focus
//! are #189's.
//!
//! **The framework's keys are the framework's.** The navigation character opens a drop-down, the
//! arrows move it, `Enter` takes an entry and `Esc` closes it, and all four are the menu
//! widget's own handling. What the bar adds is what an entry means.

use rat_theme4::WidgetStyle;
use rat_theme4::theme::SalsaTheme;
use rat_widget::event::MenuOutcome;
use rat_widget::menu::menubar;
use rat_widget::menu::{
    MenuBuilder, MenuStructure, MenuStyle, Menubar, MenubarLine, MenubarPopup, MenubarState,
};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Modifier, Style};
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};

use crate::chrome::Chrome;
use crate::vocabulary::{Action, Reply};

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
/// The top-level labels on the bar, in display order.
///
/// **The label's navigation character is the letter after its `_`**, and it is the only table of
/// names: `Alt` reaches the same character here that a bare press answers when the bar holds the
/// keyboard (D4 of #189). Help is last, so adding it leaves File and Edit where they were.
const TOP_LABELS: [&str; 3] = ["_File", "_Edit", "_Help"];

/// Per-frame flags saying which conditional menu entries are active.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct MenuFlags {
    /// Whether `Edit > Undo` can walk a step back.
    pub(crate) can_undo: bool,

    /// Whether `Edit > Redo` can walk a step forward.
    pub(crate) can_redo: bool,
}

/// The per-frame menu structure built from [`MenuFlags`].
#[derive(Debug, Clone, Copy)]
struct Menu {
    flags: MenuFlags,
}

impl Menu {
    /// Builds the menu structure for this frame's `flags`.
    fn new(flags: MenuFlags) -> Self {
        Self { flags }
    }

    /// Disabled flags for every entry in submenu `n`, matching [`MenuStructure::submenu`].
    fn submenu_disabled(self, n: usize) -> Vec<bool> {
        let mut builder = MenuBuilder::new();
        self.submenu(n, &mut builder);
        builder
            .items()
            .into_iter()
            .map(|item| item.disabled)
            .collect()
    }
}

impl<'a> MenuStructure<'a> for Menu {
    fn menus(&'a self, menu: &mut MenuBuilder<'a>) {
        for label in TOP_LABELS {
            menu.item_parsed(label);
        }
    }

    fn submenu(&'a self, n: usize, submenu: &mut MenuBuilder<'a>) {
        match n {
            FILE => {
                submenu
                    .item_parsed("_New|Ctrl+N")
                    .item_parsed("_Open|Ctrl+O")
                    .item_parsed("_Save|Ctrl+S")
                    .item_parsed("_Export as Text")
                    .item_parsed("Open _Demo")
                    .item_parsed("_Quit|Ctrl+Q");
            }
            EDIT => {
                submenu
                    .item_parsed("_Undo|Ctrl+Z")
                    .disabled(!self.flags.can_undo)
                    .item_parsed("_Redo|Ctrl+Shift+Z")
                    .disabled(!self.flags.can_redo);
            }
            HELP => {
                submenu.item_parsed("_About");
            }
            _ => {}
        }
    }
}

/// The bar, and what each of its entries means.
///
/// **The shortcut on the right of the entry is drawn by the widget and answered to by the
/// application**, which is what the table in `shortcuts` is for. The two are two places that can
/// disagree, and joining them is #180's.
///
/// **Edit entries reflect [`MenuFlags`] on every frame while keeping the same labels and
/// positions**: `Edit > Undo` is disabled when `!can_undo` and `Edit > Redo` is disabled when
/// `!can_redo`. All `File` and `Help` entries remain enabled.
///
/// The label of the way out.
const FILE: usize = 0;

/// The entry that starts a fresh blank diagram.
const NEW: usize = 0;

/// The entry that asks for a description by name.
const OPEN: usize = 1;

/// The entry that saves a description by name.
const SAVE: usize = 2;

/// The entry that exports the diagram as plain text by name.
const EXPORT_TEXT: usize = 3;

/// The entry that loads the demonstration description.
const OPEN_DEMO: usize = 4;

/// The entry of the way out.
const QUIT: usize = 5;

/// The label of the editing commands.
const EDIT: usize = 1;

/// The entry that gives back the step behind.
const UNDO: usize = 0;

/// The entry that gives back the step ahead.
const REDO: usize = 1;

/// The label of the dialog.
const HELP: usize = 2;

/// The entry that says what the application is.
const ABOUT: usize = 0;

/// What an event means to the chrome.
///
/// **The bar is asked about every event and says what it was worth**: `Ignored` if it took no
/// interest, `Repaint` if it changed what the screen shows, `Consumed` if it took the event without
/// changing anything, and `Act` for the entry that was taken. A shortcut is not the bar's to
/// answer: the application's own are asked around it, in `callbacks` — before it the ones that
/// reach everything, after it the ones nobody has claimed — so the bar says nothing about one.
///
/// **The widget's `Continue` is `Ignored` and its `Unchanged` is `Consumed`**, and the difference
/// is the point: an event the drop-down trapped on its border is the drop-down's, and the region
/// behind it must not be asked. **The widget's outcome is not the whole answer**: `rat-menu` closes
/// the pop-up when the bar loses the keyboard and reports `Continue` all the same, so the bar
/// compares the drop-down and the keyboard before and after it asks. A change made while the event
/// was taken upgrades `Consumed` to `Repaint`; an event the bar ignored goes on to the next
/// handler, and the keyboard that left is what owes that frame. **The widget only answers a key
/// while the bar holds the keyboard**: the leaf it is handed says whether it does, and a bar that
/// does not hold it answers no key and closes the drop-down the keyboard left behind.
///
/// **The three keys that move the keyboard are read here, and the bar moves it** (rule 5 of #189).
/// `F10` asks for the bar from either leaf (D4, amended). `Alt` plus a navigation character brings
/// the keyboard with the press: the character is looked up in the labels, so the menu stays the only
/// table of names, and the keyboard is the bar's before the widget is asked (D4). `Esc` while the
/// bar holds it and no drop-down is open gives it back to the region. **A leaf is not a meaning**:
/// the keyboard moves where the event is addressed, and for these three keys that is here — which is
/// why no action names a leaf, and why this is the one handler that moves the keyboard.
///
/// **A bar that moved the keyboard answers**, with the frame owed because a label reversed is a
/// label that changed. The caller's chain stops there, so nothing after the bar is asked with the
/// keyboard somewhere other than where it was when the event arrived.
pub(crate) fn handle(raw: &Event, chrome: &mut Chrome) -> Reply {
    let bar = chrome.is_bar();

    // **`F10` asks for the bar** (D4, amended): the key every terminal reports, where a bare `Alt`
    // is not one. It asks for the keyboard and opens nothing, so the bar looks focused with no
    // drop-down in the way.
    if !bar && asks_for_the_bar(raw) {
        chrome.move_to_bar();
        return Reply::Repaint;
    }

    // **`Esc` on a bar with nothing left to close gives the keyboard back** (rule 5): the widget
    // answers the key while a drop-down is open, and this is the press after it.
    if bar && steps_back(raw, chrome.menu()) {
        chrome.move_to_region();
        return Reply::Repaint;
    }

    let translated;
    let alt_char = alt_navigation(raw);
    let (raw, arriving) = match alt_char {
        Some(character) => {
            translated = Event::Key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE));
            let arrived = !chrome.is_bar();
            chrome.move_to_bar();
            (&translated, arrived)
        }
        None => (raw, false),
    };

    if alt_char.is_none()
        && let Some(reply) = guard_disabled_popup_event(raw, chrome)
    {
        return reply;
    }

    let before_open = chrome.popup_active();
    let before_focus = chrome.leaf();
    let focused = chrome.is_bar();

    let answer = match menubar::handle_popup_events(chrome.menu_mut(), focused, raw) {
        // An entry was taken: it means an answer, and `callbacks` carries it out — which is also
        // where the keyboard goes back to the region, because a meaning carried out is left there.
        // The loop is not asked again, so an entry and the key that means the same thing end in one
        // place. An entry nobody knows still closed the drop-down, so it owes the frame.
        MenuOutcome::MenuActivated(label, entry) => {
            chrome.close_popup();
            meaning(label, entry)
        }

        // **The widget's vocabulary is finer than the application's.** Selecting, activating and
        // hiding are all "the screen changed"; `Unchanged` took the event and changed nothing, so
        // the chain stops there; `Continue` leaves it to the next handler.
        MenuOutcome::Changed
        | MenuOutcome::Hide
        | MenuOutcome::Selected(_)
        | MenuOutcome::Activated(_)
        | MenuOutcome::MenuSelected(_, _) => Reply::Repaint,

        MenuOutcome::Unchanged => Reply::Consumed,

        MenuOutcome::Continue => Reply::Ignored,
    };

    // **The widget can change the state and still say nothing.** `rat-menu` closes the drop-down
    // when the bar loses the keyboard and answers `Continue`, so a `Consumed` that changed
    // something is a frame owed rather than a silent event.
    //
    // **An ignored event stays ignored even where the state moved.** The drop-down closes because
    // the keyboard had already left the bar, and the frame that closing owes is [`crate::callbacks`]'
    // address answer: claiming the event here would stop the chain and the region would never see
    // the press that took the keyboard away.
    let state_changed = chrome.popup_active() != before_open || chrome.leaf() != before_focus;
    let answer = if state_changed && answer != Reply::Ignored {
        answer.and_repaint()
    } else {
        answer
    };

    // **The keyboard arrived with the translation** (D4): the label reversed and the drop-down
    // opened are changes the widget has answered, and the frame is owed even where it said nothing
    // — the keyboard that arrived is the screen changing. It cannot be an entry: an entry is only
    // taken from an open drop-down, and an open drop-down means the bar already held the keyboard.
    if arriving {
        answer.and_repaint()
    } else {
        answer
    }
}

/// Guards an open drop-down against `rat-menu 3.1.1` paths that ignore `disabled` (D5 of #231).
///
/// Prevents `Enter`, left mouse clicks, `Up`, `Down`, `Home`, `End`, and bare navigation keys from
/// selecting or activating a disabled entry, activating another entry, panicking when
/// `selected == None`, closing the drop-down, or leaking a disabled navigation character behind it.
fn guard_disabled_popup_event(raw: &Event, chrome: &mut Chrome) -> Option<Reply> {
    if !(chrome.is_bar() && chrome.popup_active()) {
        return None;
    }

    let popup = &mut chrome.menu_mut().popup;

    if let Event::Mouse(mouse) = raw {
        if mouse.kind == MouseEventKind::Down(MouseButton::Left)
            && let Some(idx) = popup.item_at((mouse.column, mouse.row))
            && popup.disabled.get(idx).copied().unwrap_or(false)
        {
            return Some(Reply::Consumed);
        }
        return None;
    }

    let Event::Key(key) = raw else {
        return None;
    };
    if key.kind != KeyEventKind::Press {
        return None;
    }

    if key.modifiers == KeyModifiers::NONE {
        match key.code {
            KeyCode::Enter => {
                let disabled_or_none = popup
                    .selected
                    .is_none_or(|idx| popup.disabled.get(idx).copied().unwrap_or(true));
                return disabled_or_none.then_some(Reply::Consumed);
            }
            KeyCode::Up | KeyCode::Down if !popup.disabled.is_empty() => {
                let enabled_count = popup.disabled.iter().filter(|&&d| !d).count();
                if enabled_count == 0
                    || (enabled_count == 1
                        && popup.selected == popup.disabled.iter().position(|&d| !d))
                {
                    return Some(Reply::Consumed);
                }
            }
            KeyCode::Home | KeyCode::End if !popup.disabled.is_empty() => {
                let target = if key.code == KeyCode::Home {
                    popup.disabled.iter().position(|&d| !d)
                } else {
                    popup
                        .disabled
                        .iter()
                        .enumerate()
                        .rev()
                        .find_map(|(idx, &d)| (!d).then_some(idx))
                };
                return Some(match target {
                    Some(idx) if popup.selected != Some(idx) => {
                        popup.selected = Some(idx);
                        Reply::Repaint
                    }
                    _ => Reply::Consumed,
                });
            }
            _ => {}
        }
    }

    if let KeyCode::Char(character) = key.code
        && !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    {
        let lower = character.to_ascii_lowercase();
        // cspell:ignore navchar -- external field name on rat_menu::PopupMenuState
        let matches_disabled = popup.navchar.iter().enumerate().any(|(idx, &nav)| {
            nav == Some(lower) && popup.disabled.get(idx).copied().unwrap_or(false)
        });
        if matches_disabled {
            return Some(Reply::Consumed);
        }
    }

    None
}

/// The navigation character an `Alt` press names, or `None` where the event is not one.
///
/// **The answer comes from the labels** (D4): the character the menu answers to is the letter
/// after its `_`, and looking it up here keeps the bar the only table of names. **A press of any
/// other key, or with any other modifier, is not one**: a bare `Alt` is not a key, and a
/// combination that names no label changes nothing.
fn alt_navigation(raw: &Event) -> Option<char> {
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

    TOP_LABELS
        .iter()
        .any(|label| navigation(label) == Some(character))
        .then_some(character)
}

/// Whether the event is the `F10` press that asks for the bar.
///
/// **`F10` is the key every terminal reports** (D4, amended): a bare `Alt` is not one — the
/// framework asks for the terminal's extended keys only off Windows, and a terminal without them
/// sends nothing for `Alt` alone — so the bar is reachable from the region with a key that is
/// always there. **It asks only for the keyboard**: no drop-down opens, and the bar draws its first
/// label in reverse video because that is what holding the keyboard looks like. [`handle`] asks it
/// only while the bar does not hold the keyboard, and moves the keyboard there.
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
/// the keyboard, and moves the keyboard back.
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
/// **The bar carries text and nothing else**: an entry means what its position says, and an entry
/// nobody knows means nothing — it still closed the drop-down, so it owes the frame, which is
/// [`Reply::Repaint`] said here rather than by the caller.
fn meaning(label: usize, entry: usize) -> Reply {
    match (label, entry) {
        (FILE, NEW) => Reply::Act(Action::New),
        (FILE, OPEN) => Reply::Act(Action::Open),
        (FILE, SAVE) => Reply::Act(Action::Save),
        (FILE, EXPORT_TEXT) => Reply::Act(Action::ExportText),
        (FILE, OPEN_DEMO) => Reply::Act(Action::OpenDemo),
        (FILE, QUIT) => Reply::quit(),
        (EDIT, UNDO) => Reply::Act(Action::Undo),
        (EDIT, REDO) => Reply::Act(Action::Redo),
        (HELP, ABOUT) => Reply::Act(Action::About),

        _ => Reply::Repaint,
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
    flags: MenuFlags,
) {
    let menu = Menu::new(flags);
    let (bar, _) = widgets(&menu, theme);
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
    flags: MenuFlags,
) {
    let menu = Menu::new(flags);
    let active_disabled = state
        .bar
        .selected()
        .filter(|_| state.popup_active())
        .map(|label| menu.submenu_disabled(label));

    // Normalize before rendering so `rat-menu`'s `Some(0)` fallback does not select or reverse a
    // disabled first entry, and using `menu.submenu_disabled` avoids stale flags after switching
    // top-level labels (D4 of #231).
    if let Some(disabled) = &active_disabled {
        normalize_selection(state, disabled);
    }

    let (_, drop_down) = widgets(&menu, theme);
    drop_down.render(area, buf, state);

    // Normalize again after rendering because `render_popup_menu` overwrites `None` with
    // `Some(0)` whenever all entries in the submenu are disabled.
    if let Some(disabled) = &active_disabled {
        normalize_selection(state, disabled);
        normalize_disabled_rows(buf, state, theme);
    }
}

/// Moves `state.popup.selected` to the first enabled entry (or `None` when none is enabled) when
/// it is unset, out of bounds, or pointing at a disabled entry.
fn normalize_selection(state: &mut MenubarState, disabled: &[bool]) {
    let invalid = match state.popup.selected {
        None => true,
        Some(idx) => disabled.get(idx).copied().unwrap_or(true),
    };
    if invalid {
        state.popup.selected = disabled.iter().position(|&is_disabled| !is_disabled);
    }
}

/// Resets every cell of each disabled drop-down row to `MENU_BASE + Modifier::DIM` (D3 of #231).
///
/// `rat-menu` applies `popup_disabled` only to plain label spans, leaving the navigation character
/// underlined and the right-hand shortcut in `KEY_BINDING`, and paints item `0` in `REVERSED`
/// whenever selection falls on it. Resetting each disabled item row clears those per-span styles
/// while keeping the row's glyphs intact.
fn normalize_disabled_rows(buf: &mut Buffer, state: &MenubarState, theme: &SalsaTheme) {
    let base_style = menu_base_style(theme);
    let disabled_style = Style::reset().patch(base_style).add_modifier(Modifier::DIM);

    for (idx, &item_area) in state.popup.item_areas.iter().enumerate() {
        if state.popup.disabled.get(idx).copied().unwrap_or(false) {
            buf.set_style(item_area, disabled_style);
        }
    }
}

/// Base style of the menu bar and pop-up (`MENU_BASE`).
fn menu_base_style(theme: &SalsaTheme) -> Style {
    let style: MenuStyle = theme.style(WidgetStyle::MENU);
    style.popup_style.unwrap_or(style.style)
}

/// The bar and the drop-down, as the two widgets the menu splits itself into.
///
/// **The focus is reverse video over the theme's own colors** (D3 of #189): the two focus styles
/// are replaced with the modifier alone, so the base style painted under the label keeps its
/// colors and the widget adds `Modifier::REVERSED` to the cells it owns. The theme's `focus`
/// background is not used, which is what makes the bar and the dialog's `Ok` say the same thing.
///
/// **Disabled pop-up items take `Modifier::DIM` instead of `Style::DISABLED`** (D3 of #231):
/// `rat-theme4`'s `Style::DISABLED` paints a high-contrast light-gray block, so `popup_disabled` is
/// replaced with `Style::new().dim()`.
fn widgets<'a>(menu: &'a Menu, theme: &SalsaTheme) -> (MenubarLine<'a>, MenubarPopup<'a>) {
    let mut style: MenuStyle = theme.style(WidgetStyle::MENU);
    style.focus = Some(Style::new().reversed());
    style.popup_focus = Some(Style::new().reversed());
    style.popup_disabled = Some(Style::new().dim());
    Menubar::new(menu).styles(style).into_widgets()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chrome::{Chrome, Focus};
    use crate::config::Config;
    use ratatui_crossterm::crossterm::event::{
        KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
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
    fn draw(chrome: &mut Chrome) -> Buffer {
        draw_with(chrome, true)
    }

    /// The bar and its drop-down, drawn with explicit `flags`.
    fn draw_flags(chrome: &mut Chrome, flags: MenuFlags) -> Buffer {
        draw_with_flags(chrome, true, flags)
    }

    /// The bar and its drop-down, with the keyboard on the bar or not.
    fn draw_with(chrome: &mut Chrome, focused: bool) -> Buffer {
        draw_with_flags(chrome, focused, MenuFlags::default())
    }

    /// The bar and its drop-down, with the keyboard on the bar or not and explicit `flags`.
    fn draw_with_flags(chrome: &mut Chrome, focused: bool, flags: MenuFlags) -> Buffer {
        let area = Rect::new(0, 0, WIDTH, HEIGHT);
        let mut buffer = Buffer::empty(area);
        draw_bar(
            Rect::new(0, 0, WIDTH, 1),
            &mut buffer,
            chrome.menu_mut(),
            &theme(),
            focused,
            flags,
        );
        draw_drop_down(area, &mut buffer, chrome.menu_mut(), &theme(), flags);
        buffer
    }

    /// The bar, drawn once so that the widget knows where its labels are.
    fn drawn() -> Chrome {
        let mut chrome = Chrome::new();
        draw(&mut chrome);
        chrome
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
    fn on_the_bar(event: &Event, chrome: &mut Chrome) -> Reply {
        chrome.move_to_bar();
        handle(event, chrome)
    }

    /// Opens the drop-down of the label, the way a person does.
    ///
    /// **The frame is drawn after the key as well as before it**: the widget knows what it
    /// holds only once it has been drawn, and a frame that is not drawn leaves the drop-down
    /// without an entry to take.
    fn open_with(chrome: &mut Chrome, navigation: char) {
        open_with_flags(chrome, navigation, MenuFlags::default());
    }

    /// Opens the drop-down of the label with explicit `flags`.
    fn open_with_flags(chrome: &mut Chrome, navigation: char, flags: MenuFlags) {
        let _ = on_the_bar(&key(KeyCode::Char(navigation)), chrome);
        draw_flags(chrome, flags);
    }

    /// Opens the File drop-down.
    fn open(chrome: &mut Chrome) {
        open_with(chrome, 'f');
    }

    /// Takes the entry whose text is `text`, by clicking it on the frame, the way a person does.
    ///
    /// **The entry is found by its name**: a menu that gains entries above one moves where it is
    /// but not what it says, and a test that pressed `Enter` would take whatever happens to be
    /// first. **The pointer moves onto it before the button goes down**, because the widget
    /// activates the entry it has selected and a press alone selects nothing.
    fn take(chrome: &mut Chrome, text: &str) -> Reply {
        take_with_flags(chrome, text, MenuFlags::default())
    }

    /// Takes the entry whose text is `text` after drawing with explicit `flags`.
    fn take_with_flags(chrome: &mut Chrome, text: &str, flags: MenuFlags) -> Reply {
        let buffer = draw_flags(chrome, flags);
        let (column, row) = cell_of(&buffer, text);
        let _ = on_the_bar(&moved(column, row), chrome);

        on_the_bar(&click(column, row), chrome)
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
        let mut without = Chrome::new();
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
        let mut chrome = drawn();

        let outcome = on_the_bar(&key(KeyCode::Char('f')), &mut chrome);

        assert!(
            matches!(outcome, Reply::Repaint),
            "the navigation character did not report a screen change: {outcome:?}"
        );
        assert!(chrome.popup_active(), "the drop-down did not open");

        let outcome = on_the_bar(&key(KeyCode::Esc), &mut chrome);

        assert!(
            matches!(outcome, Reply::Repaint),
            "escape did not report a screen change: {outcome:?}"
        );
        assert!(!chrome.popup_active(), "escape did not close the drop-down");
    }

    /// **The key that leaves the bar is a press and nothing else** (rule 5): a release, a repeat,
    /// and a press with a modifier on it are not the bar stepping back, and neither is a press
    /// while a drop-down is open.
    #[test]
    fn an_escape_press_is_the_bars_way_out() {
        let mut chrome = drawn();

        assert!(
            steps_back(&key(KeyCode::Esc), chrome.menu()),
            "the escape press was not read"
        );

        for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
            let event = Event::Key(KeyEvent::new_with_kind(
                KeyCode::Esc,
                KeyModifiers::NONE,
                kind,
            ));

            assert!(
                !steps_back(&event, chrome.menu()),
                "{kind:?} was read as it"
            );
        }

        assert!(
            !steps_back(
                &key_with(KeyCode::Esc, KeyModifiers::CONTROL),
                chrome.menu()
            ),
            "a modified escape was read as it"
        );
        assert!(
            !steps_back(&key(KeyCode::Enter), chrome.menu()),
            "another key was read as it"
        );

        open(&mut chrome);
        assert!(
            !steps_back(&key(KeyCode::Esc), chrome.menu()),
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

    /// **`F10` moves the keyboard where the key is read** (rule 5 and D4, amended): the bar is what
    /// knows the key, so the bar is where the leaf changes, and the label that stops being reversed
    /// is why the frame is owed. It opens nothing on the way.
    #[test]
    fn f10_moves_the_keyboard_to_the_bar_and_owes_the_frame() {
        let mut chrome = drawn();

        let outcome = handle(&key(KeyCode::F(10)), &mut chrome);

        assert_eq!(
            chrome.leaf(),
            Focus::Bar,
            "F10 left the keyboard where it was"
        );
        assert!(
            matches!(outcome, Reply::Repaint),
            "F10 answered {outcome:?}"
        );
        assert!(!chrome.popup_active(), "F10 opened a drop-down");
    }

    /// **The escape that closes nothing gives the keyboard back where it is read** (rule 5): the
    /// bar has nothing left to close, so this is the press that leaves, and the frame is owed
    /// because the label stops being reversed.
    #[test]
    fn the_escape_that_closes_nothing_gives_the_keyboard_back() {
        let mut chrome = drawn();
        chrome.move_to_bar();

        let outcome = handle(&key(KeyCode::Esc), &mut chrome);

        assert_eq!(
            chrome.leaf(),
            Focus::Region,
            "the escape left the keyboard on the bar"
        );
        assert!(
            matches!(outcome, Reply::Repaint),
            "the escape answered {outcome:?}"
        );
    }

    /// **A press the drop-down traps is consumed and not ignored** (rule 3 of #189): `rat-menu`
    /// answers `Unchanged` for a cell that is not an entry — the border or the padding — which is
    /// the bar's event with nothing changed. Mapping that to `Ignored` is what once let the press
    /// reach the diagram behind it.
    #[test]
    fn a_press_the_drop_down_traps_is_consumed_and_not_ignored() {
        let mut chrome = drawn();
        open_with(&mut chrome, 'e');

        let popup = chrome.menu().popup.area;
        let mut trapped = None;

        for row in popup.top()..popup.bottom() {
            for column in popup.left()..popup.right() {
                if chrome.menu().popup.item_at((column, row)).is_none() {
                    trapped = Some((column, row));
                    break;
                }
            }

            if trapped.is_some() {
                break;
            }
        }

        let (column, row) = trapped.expect("the drop-down has no cell that is not an entry");

        let outcome = on_the_bar(&click(column, row), &mut chrome);

        assert_eq!(
            outcome,
            Reply::Consumed,
            "a press the drop-down trapped answered {outcome:?}"
        );
        assert!(chrome.popup_active(), "the press closed the drop-down");
    }

    /// **The File menu's first entry is New, which mints [`Action::New`].**
    #[test]
    fn taking_the_new_entry_becomes_the_action_of_new() {
        let mut chrome = drawn();
        open(&mut chrome);

        let outcome = take(&mut chrome, "New");

        assert!(
            matches!(outcome, Reply::Act(Action::New)),
            "the entry did not become the action of new: {outcome:?}"
        );
        assert!(
            !chrome.popup_active(),
            "the drop-down stayed open behind it"
        );
    }

    /// **The File menu's second entry is Open, which mints [`Action::Open`].**
    #[test]
    fn taking_the_open_entry_becomes_the_action_of_the_open_dialog() {
        let mut chrome = drawn();
        open(&mut chrome);

        let outcome = take(&mut chrome, "Open");

        assert!(
            matches!(outcome, Reply::Act(Action::Open)),
            "the entry did not become the action of the open dialog: {outcome:?}"
        );
        assert!(
            !chrome.popup_active(),
            "the drop-down stayed open behind it"
        );
    }

    /// **The File menu's third entry is Save, which mints [`Action::Save`].**
    #[test]
    fn taking_the_save_entry_becomes_the_action_of_the_save_dialog() {
        let mut chrome = drawn();
        open(&mut chrome);

        let outcome = take(&mut chrome, "Save");

        assert!(
            matches!(outcome, Reply::Act(Action::Save)),
            "the entry did not become the action of the save dialog: {outcome:?}"
        );
        assert!(
            !chrome.popup_active(),
            "the drop-down stayed open behind it"
        );
    }

    /// **The File menu's fourth entry is Open Demo, which mints [`Action::OpenDemo`].**
    #[test]
    fn taking_the_open_demo_entry_becomes_the_action_of_open_demo() {
        let mut chrome = drawn();
        open(&mut chrome);

        let outcome = take(&mut chrome, "Open Demo");

        assert!(
            matches!(outcome, Reply::Act(Action::OpenDemo)),
            "the entry did not become the action of open demo: {outcome:?}"
        );
        assert!(
            !chrome.popup_active(),
            "the drop-down stayed open behind it"
        );
    }

    #[test]
    fn taking_the_quit_entry_becomes_the_answer_of_the_way_out() {
        let mut chrome = drawn();
        open(&mut chrome);

        let outcome = take(&mut chrome, "Quit");

        assert_eq!(
            outcome,
            Reply::quit(),
            "the entry did not become the answer of the way out"
        );
        assert!(
            !chrome.popup_active(),
            "the drop-down stayed open behind it"
        );
    }

    /// **The Edit menu's entries are the keys' actions.** The undo is the first, and taking it when
    /// enabled says `Action::Undo` rather than a second spelling of the walk.
    #[test]
    fn taking_the_undo_entry_becomes_the_action_of_the_key() {
        let flags = MenuFlags {
            can_undo: true,
            can_redo: false,
        };
        let mut chrome = drawn();
        open_with_flags(&mut chrome, 'e', flags);

        let outcome = take_with_flags(&mut chrome, "Undo", flags);

        assert!(
            matches!(outcome, Reply::Act(Action::Undo)),
            "the entry did not become the action of the undo key: {outcome:?}"
        );
        assert!(
            !chrome.popup_active(),
            "the drop-down stayed open behind it"
        );
    }

    /// **Help is the third label and About its only entry, with no shortcut drawn** (rule 8):
    /// About opens a dialog rather than a command, so there is no key to show beside it.
    #[test]
    fn the_help_menu_holds_about_and_draws_no_shortcut() {
        let mut chrome = drawn();
        open_with(&mut chrome, 'h');

        let buffer = draw(&mut chrome);
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
        let mut chrome = drawn();
        open_with(&mut chrome, 'h');

        let outcome = take(&mut chrome, "About");

        assert!(
            matches!(outcome, Reply::Act(Action::About)),
            "the entry did not become the action of the About dialog: {outcome:?}"
        );
        assert!(
            !chrome.popup_active(),
            "the drop-down stayed open behind it"
        );
    }

    /// **The redo entry is the redo key's action**, the same way.
    #[test]
    fn taking_the_redo_entry_becomes_the_action_of_the_key() {
        let flags = MenuFlags {
            can_undo: false,
            can_redo: true,
        };
        let mut chrome = drawn();
        open_with_flags(&mut chrome, 'e', flags);

        let outcome = take_with_flags(&mut chrome, "Redo", flags);

        assert!(
            matches!(outcome, Reply::Act(Action::Redo)),
            "the entry did not become the action of the redo key: {outcome:?}"
        );
        assert!(
            !chrome.popup_active(),
            "the drop-down stayed open behind it"
        );
    }

    /// **The Edit menu holds Undo and Redo with their shortcuts**: Undo and Redo, each with the
    /// shortcut the table answers to (rule 9).
    #[test]
    fn the_edit_menu_shows_undo_and_redo_with_their_shortcuts() {
        let mut chrome = drawn();
        open_with(&mut chrome, 'e');

        let frame = picture(&draw(&mut chrome));

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

        assert!(undo < redo, "the entries are not Undo, Redo: {frame:?}");
    }

    #[test]
    fn the_edit_menu_shows_undo_and_redo_and_nothing_else() {
        let mut chrome = drawn();
        open_with(&mut chrome, 'e');

        let frame = picture(&draw(&mut chrome));

        assert!(
            frame.contains("Edit"),
            "the bar has no Edit menu: {frame:?}"
        );
        assert!(
            !frame.contains("Delete"),
            "the Edit menu still offers delete: {frame:?}"
        );
        assert!(
            !frame.contains("Del"),
            "the Edit menu still shows Del: {frame:?}"
        );
    }

    #[test]
    fn the_shortcuts_are_not_the_bars() {
        let mut chrome = drawn();
        chrome.move_to_bar();

        let outcome = handle(
            &key_with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut chrome,
        );

        assert_eq!(
            outcome,
            Reply::Ignored,
            "the bar answered a shortcut: {outcome:?}"
        );
        assert_eq!(chrome.leaf(), Focus::Bar, "the bar gave the keyboard away");
        assert!(!chrome.popup_active(), "a shortcut opened a drop-down");
    }

    #[test]
    fn a_click_on_the_entry_becomes_the_same_answer() {
        let mut chrome = drawn();
        open(&mut chrome);

        let outcome = take(&mut chrome, "Quit");

        assert_eq!(
            outcome,
            Reply::quit(),
            "the click did not become the answer of the way out"
        );
    }

    #[test]
    fn a_click_on_the_label_opens_its_drop_down() {
        let mut chrome = drawn();
        let buffer = draw(&mut chrome);
        let (column, row) = cell_of(&buffer, "File");

        let outcome = on_the_bar(&click(column, row), &mut chrome);

        assert!(
            matches!(outcome, Reply::Repaint),
            "the click did not report a screen change: {outcome:?}"
        );
        assert!(
            chrome.popup_active(),
            "the click did not open the drop-down"
        );
    }

    #[test]
    fn a_key_the_bar_does_not_use_is_nothing() {
        let mut chrome = drawn();

        let outcome = on_the_bar(&key(KeyCode::Char('z')), &mut chrome);

        assert_eq!(outcome, Reply::Ignored, "{outcome:?}");
        assert!(!chrome.popup_active(), "a key opened a drop-down");
    }

    #[test]
    fn the_drop_down_shows_the_shortcut_the_entry_answers_to() {
        let mut chrome = drawn();
        open(&mut chrome);

        let buffer = draw(&mut chrome);
        let frame = picture(&buffer);

        assert!(
            frame.contains("Ctrl+Q"),
            "the drop-down shows no shortcut: {frame:?}"
        );
    }

    #[test]
    fn taking_the_export_as_text_entry_becomes_the_action_of_the_export_dialog() {
        let mut chrome = drawn();
        open(&mut chrome);

        let outcome = take(&mut chrome, "Export as Text");

        assert!(
            matches!(outcome, Reply::Act(Action::ExportText)),
            "the entry did not become the action of the export dialog: {outcome:?}"
        );
        assert!(
            !chrome.popup_active(),
            "the drop-down stayed open behind it"
        );
    }

    #[test]
    fn edit_entries_follow_menu_flags_while_file_and_help_stay_enabled() {
        for (can_undo, can_redo) in [(false, false), (true, false), (false, true), (true, true)] {
            let menu = Menu::new(MenuFlags { can_undo, can_redo });

            let mut edit = MenuBuilder::new();
            menu.submenu(EDIT, &mut edit);
            let edit_items = edit.items();
            assert_eq!(edit_items.len(), 2);
            assert_eq!(edit_items[UNDO].disabled, !can_undo);
            assert_eq!(edit_items[REDO].disabled, !can_redo);

            for submenu_index in [FILE, HELP] {
                let mut builder = MenuBuilder::new();
                menu.submenu(submenu_index, &mut builder);
                assert!(
                    builder.items().iter().all(|item| !item.disabled),
                    "submenu {submenu_index} had a disabled entry for ({can_undo}, {can_redo})"
                );
            }
        }
    }

    #[test]
    fn disabled_rows_paint_with_menu_base_and_dim_across_label_nav_and_shortcut() {
        let theme = theme();
        let menu_style: MenuStyle = theme.style(WidgetStyle::MENU);
        let base_style = menu_style.popup_style.unwrap_or(menu_style.style);
        let key_binding_bg = menu_style.popup_right.and_then(|s| s.bg);
        let theme_disabled_bg = menu_style.popup_disabled.and_then(|s| s.bg);

        let mut chrome = drawn();
        let flags = MenuFlags {
            can_undo: false,
            can_redo: false,
        };
        open_with_flags(&mut chrome, 'e', flags);
        let buffer = draw_flags(&mut chrome, flags);

        for item_idx in [UNDO, REDO] {
            let row_area = chrome.menu().popup.item_areas[item_idx];
            assert!(row_area.width > 0 && row_area.height == 1);
            for col in row_area.left()..row_area.right() {
                let style = buffer[(col, row_area.y)].style();
                assert_eq!(style.fg, base_style.fg, "col {col} fg mismatch");
                assert_eq!(style.bg, base_style.bg, "col {col} bg mismatch");
                assert!(
                    style.add_modifier.contains(Modifier::DIM),
                    "col {col} missing DIM: {style:?}"
                );
                assert!(
                    !style.add_modifier.contains(Modifier::REVERSED),
                    "col {col} has REVERSED: {style:?}"
                );
                assert!(
                    !style.add_modifier.contains(Modifier::UNDERLINED),
                    "col {col} has UNDERLINED: {style:?}"
                );
                if let Some(kb_bg) = key_binding_bg {
                    assert_ne!(style.bg, Some(kb_bg), "col {col} kept KEY_BINDING bg");
                }
                if let Some(dis_bg) = theme_disabled_bg {
                    assert_ne!(style.bg, Some(dis_bg), "col {col} used Style::DISABLED bg");
                }
            }
        }
    }

    #[test]
    fn enabled_entries_keep_active_styling_and_underlined_nav_char() {
        let theme = theme();
        let menu_style: MenuStyle = theme.style(WidgetStyle::MENU);
        let key_binding_bg = menu_style.popup_right.and_then(|s| s.bg);

        let mut chrome = drawn();
        let flags = MenuFlags {
            can_undo: true,
            can_redo: true,
        };
        open_with_flags(&mut chrome, 'e', flags);
        let buffer = draw_flags(&mut chrome, flags);

        // Undo is selected by default; Redo is unselected and enabled.
        let redo_area = chrome.menu().popup.item_areas[REDO];
        for col in redo_area.left()..redo_area.right() {
            let style = buffer[(col, redo_area.y)].style();
            assert!(
                !style.add_modifier.contains(Modifier::DIM),
                "enabled Redo col {col} is DIM"
            );
            assert!(
                !style.add_modifier.contains(Modifier::REVERSED),
                "unselected Redo col {col} is REVERSED"
            );
        }

        let r_col = (redo_area.left()..redo_area.right())
            .find(|&col| buffer[(col, redo_area.y)].symbol() == "R")
            .expect("R cell in Redo");
        assert!(
            buffer[(r_col, redo_area.y)]
                .style()
                .add_modifier
                .contains(Modifier::UNDERLINED),
            "enabled Redo navigation character is not underlined"
        );

        let shortcut_col = (redo_area.left()..redo_area.right())
            .find(|&col| buffer[(col, redo_area.y)].symbol() == "C")
            .expect("Ctrl+Shift+Z cell in Redo");
        assert_eq!(
            buffer[(shortcut_col, redo_area.y)].style().bg,
            key_binding_bg,
            "enabled Redo shortcut lost KEY_BINDING background"
        );
    }

    #[test]
    fn initial_selection_lands_on_first_enabled_entry_or_none() {
        // Neither Undo nor Redo enabled -> None, no cell reversed in either row.
        let mut both_off = drawn();
        let none_flags = MenuFlags {
            can_undo: false,
            can_redo: false,
        };
        open_with_flags(&mut both_off, 'e', none_flags);
        let buf = draw_flags(&mut both_off, none_flags);
        assert_eq!(both_off.menu().popup.selected(), None);
        for idx in [UNDO, REDO] {
            let area = both_off.menu().popup.item_areas[idx];
            assert!(!reversed(&buf, area.left(), area.y));
        }

        // Only Undo enabled -> Some(0), Undo reversed.
        let mut undo_only = drawn();
        let undo_flags = MenuFlags {
            can_undo: true,
            can_redo: false,
        };
        open_with_flags(&mut undo_only, 'e', undo_flags);
        let buf = draw_flags(&mut undo_only, undo_flags);
        assert_eq!(undo_only.menu().popup.selected(), Some(UNDO));
        let undo_area = undo_only.menu().popup.item_areas[UNDO];
        let redo_area = undo_only.menu().popup.item_areas[REDO];
        assert!(reversed(&buf, undo_area.left(), undo_area.y));
        assert!(!reversed(&buf, redo_area.left(), redo_area.y));

        // Only Redo enabled -> Some(1), Redo reversed and Undo not reversed.
        let mut redo_only = drawn();
        let redo_flags = MenuFlags {
            can_undo: false,
            can_redo: true,
        };
        open_with_flags(&mut redo_only, 'e', redo_flags);
        let buf = draw_flags(&mut redo_only, redo_flags);
        assert_eq!(redo_only.menu().popup.selected(), Some(REDO));
        let undo_area = redo_only.menu().popup.item_areas[UNDO];
        let redo_area = redo_only.menu().popup.item_areas[REDO];
        assert!(!reversed(&buf, undo_area.left(), undo_area.y));
        assert!(reversed(&buf, redo_area.left(), redo_area.y));
    }

    #[test]
    fn switching_from_file_to_edit_normalizes_selection_on_frame_one() {
        let mut chrome = drawn();
        let flags = MenuFlags {
            can_undo: false,
            can_redo: true,
        };
        open_with_flags(&mut chrome, 'f', flags);
        let _ = on_the_bar(&key(KeyCode::Down), &mut chrome);
        let _ = on_the_bar(&key(KeyCode::Down), &mut chrome);
        assert_eq!(chrome.menu().popup.selected(), Some(SAVE));

        // Switch from File to Edit and paint a single frame.
        let _ = on_the_bar(&key(KeyCode::Right), &mut chrome);
        let buf = draw_flags(&mut chrome, flags);

        assert_eq!(chrome.menu().bar.selected(), Some(EDIT));
        assert_eq!(chrome.menu().popup.selected(), Some(REDO));
        let undo_area = chrome.menu().popup.item_areas[UNDO];
        let redo_area = chrome.menu().popup.item_areas[REDO];
        assert!(!reversed(&buf, undo_area.left(), undo_area.y));
        assert!(reversed(&buf, redo_area.left(), redo_area.y));
    }

    #[test]
    fn arrows_home_and_end_respect_disabled_entries_and_keep_menu_open() {
        // Both disabled -> Up, Down, Home, End keep selection None and return Consumed.
        let mut none_enabled = drawn();
        let none_flags = MenuFlags {
            can_undo: false,
            can_redo: false,
        };
        open_with_flags(&mut none_enabled, 'e', none_flags);
        for code in [KeyCode::Up, KeyCode::Down, KeyCode::Home, KeyCode::End] {
            let reply = on_the_bar(&key(code), &mut none_enabled);
            assert_eq!(reply, Reply::Consumed, "{code:?} on all-disabled");
            assert!(none_enabled.popup_active(), "{code:?} closed menu");
            assert_eq!(
                none_enabled.menu().popup.selected(),
                None,
                "{code:?} selected disabled entry"
            );
        }

        // Only Redo enabled -> Up, Down, Home, End stay on Some(REDO) with Consumed.
        let mut redo_only = drawn();
        let redo_flags = MenuFlags {
            can_undo: false,
            can_redo: true,
        };
        open_with_flags(&mut redo_only, 'e', redo_flags);
        assert_eq!(redo_only.menu().popup.selected(), Some(REDO));
        for code in [KeyCode::Up, KeyCode::Down, KeyCode::Home, KeyCode::End] {
            let reply = on_the_bar(&key(code), &mut redo_only);
            assert_eq!(reply, Reply::Consumed, "{code:?} on sole Redo");
            assert!(redo_only.popup_active(), "{code:?} closed menu");
            assert_eq!(
                redo_only.menu().popup.selected(),
                Some(REDO),
                "{code:?} moved off sole enabled Redo"
            );
        }

        // Only Undo enabled -> Up, Down, Home, End stay on Some(UNDO) with Consumed.
        let mut undo_only = drawn();
        let undo_flags = MenuFlags {
            can_undo: true,
            can_redo: false,
        };
        open_with_flags(&mut undo_only, 'e', undo_flags);
        assert_eq!(undo_only.menu().popup.selected(), Some(UNDO));
        for code in [KeyCode::Up, KeyCode::Down, KeyCode::Home, KeyCode::End] {
            let reply = on_the_bar(&key(code), &mut undo_only);
            assert_eq!(reply, Reply::Consumed, "{code:?} on sole Undo");
            assert!(undo_only.popup_active(), "{code:?} closed menu");
            assert_eq!(
                undo_only.menu().popup.selected(),
                Some(UNDO),
                "{code:?} moved off sole enabled Undo"
            );
        }
    }

    #[test]
    fn enter_and_nav_keys_do_not_activate_disabled_entries() {
        // Both disabled: Enter, 'u', and 'r' all return Consumed and keep popup open.
        let mut none_enabled = drawn();
        let none_flags = MenuFlags {
            can_undo: false,
            can_redo: false,
        };
        open_with_flags(&mut none_enabled, 'e', none_flags);
        for code in [KeyCode::Enter, KeyCode::Char('u'), KeyCode::Char('r')] {
            let reply = on_the_bar(&key(code), &mut none_enabled);
            assert_eq!(reply, Reply::Consumed, "{code:?} on all-disabled");
            assert!(none_enabled.popup_active(), "{code:?} closed menu");
            assert_eq!(none_enabled.menu().popup.selected(), None);
        }

        // Only Redo enabled: 'u' is consumed without closing or changing selection; 'r' activates.
        let mut redo_only = drawn();
        let redo_flags = MenuFlags {
            can_undo: false,
            can_redo: true,
        };
        open_with_flags(&mut redo_only, 'e', redo_flags);
        assert_eq!(
            on_the_bar(&key(KeyCode::Char('u')), &mut redo_only),
            Reply::Consumed
        );
        assert!(redo_only.popup_active());
        assert_eq!(redo_only.menu().popup.selected(), Some(REDO));

        assert_eq!(
            on_the_bar(&key(KeyCode::Char('r')), &mut redo_only),
            Reply::Act(Action::Redo)
        );
        assert!(!redo_only.popup_active());

        // Only Undo enabled: 'r' is consumed; 'u' activates Undo.
        let mut undo_only = drawn();
        let undo_flags = MenuFlags {
            can_undo: true,
            can_redo: false,
        };
        open_with_flags(&mut undo_only, 'e', undo_flags);
        assert_eq!(
            on_the_bar(&key(KeyCode::Char('r')), &mut undo_only),
            Reply::Consumed
        );
        assert!(undo_only.popup_active());
        assert_eq!(undo_only.menu().popup.selected(), Some(UNDO));

        assert_eq!(
            on_the_bar(&key(KeyCode::Char('u')), &mut undo_only),
            Reply::Act(Action::Undo)
        );
        assert!(!undo_only.popup_active());
    }

    #[test]
    fn mouse_hover_and_click_on_disabled_entries_stay_consumed_and_open() {
        // Both disabled (selected == None): hover and click on Undo and Redo stay Consumed and do not panic.
        let mut none_enabled = drawn();
        let none_flags = MenuFlags {
            can_undo: false,
            can_redo: false,
        };
        open_with_flags(&mut none_enabled, 'e', none_flags);
        for idx in [UNDO, REDO] {
            let area = none_enabled.menu().popup.item_areas[idx];
            assert_eq!(
                on_the_bar(&moved(area.left(), area.y), &mut none_enabled),
                Reply::Consumed
            );
            assert_eq!(none_enabled.menu().popup.selected(), None);
            assert_eq!(
                on_the_bar(&click(area.left(), area.y), &mut none_enabled),
                Reply::Consumed
            );
            assert!(none_enabled.popup_active());
            assert_eq!(none_enabled.menu().popup.selected(), None);
        }

        // Only Redo enabled (selected == Some(REDO)): hover and click on disabled Undo do not activate Redo.
        let mut redo_only = drawn();
        let redo_flags = MenuFlags {
            can_undo: false,
            can_redo: true,
        };
        open_with_flags(&mut redo_only, 'e', redo_flags);
        let undo_area = redo_only.menu().popup.item_areas[UNDO];
        assert_eq!(
            on_the_bar(&moved(undo_area.left(), undo_area.y), &mut redo_only),
            Reply::Consumed
        );
        assert_eq!(redo_only.menu().popup.selected(), Some(REDO));
        assert_eq!(
            on_the_bar(&click(undo_area.left(), undo_area.y), &mut redo_only),
            Reply::Consumed
        );
        assert!(redo_only.popup_active());
        assert_eq!(redo_only.menu().popup.selected(), Some(REDO));
    }
}
