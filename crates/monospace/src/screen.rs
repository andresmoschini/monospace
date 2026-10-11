//! The draw pass: what the screen is made of, in what order, and where each control ended up.
//!
//! [`docs/application-model.md`](../../../docs/application-model.md) owns the rules this file
//! carries out; §3 of it is the geometry, which is here.
//!
//! # Design notes
//!
//! **The order is the whole of rule 16.** The drop-down and the pop-up are drawn *after* the
//! region they cover, so a repaint of the region does not erase them — which is not a subtlety but
//! a difference between a working overlay and a flicker, and it is the thing the spike was built to
//! settle. The two applications the specification measures differ by one line of draw order and
//! nothing else.
//!
//! **Every region is painted on every frame, and so is every overlay, because `Terminal::draw`
//! diffs against the previous frame rather than clearing.** A region that simply stops being drawn
//! keeps whatever was last in it, and the result reads as a drawing bug rather than a layout one.
//! The drop-down and the pop-up paint [`Clear`] first for the same reason: they are over a region
//! that was painted this frame, and without it they would let the region's border show through
//! them.
//!
//! **The bar's height and the status bar's height are the solver's, not this file's arithmetic.**
//! `ratatui` is asked for three regions of `1`, the rest and `1`, and whatever falls out is what is
//! drawn. The consequence is that a terminal too short for the three still lays out — `Min(0)`
//! takes what is left and may take nothing — and every rectangle below is intersected with the
//! region it belongs to, so a terminal one row tall draws a bar and nothing else rather than
//! writing outside itself.
//!
//! **Each control's rectangle is stored where it was drawn.** That is D3, and the arithmetic is
//! here because it is a property of the screen and not of the click handler: a control's
//! rectangle is its label with one space either side, laid out left to right along the bar.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Clear, Paragraph},
};

use crate::chrome::{Chrome, ControlId, ControlState, bar, label_of, menu};

/// The application's name, on the status bar and in the pop-up that says what this is.
///
/// **A constant in this binary rather than the package name**, so that the status bar and the pop-up
/// cannot disagree about it and so that renaming the binary is not a second thing to look for.
pub(crate) const APPLICATION: &str = "monospace";

/// The bar's height: one row, which is what a row of controls with no border needs.
const BAR_HEIGHT: u16 = 1;

/// The status bar's height, for the same reason and in the same way.
const STATUS_HEIGHT: u16 = 1;

/// One blank column either side of a control's label.
///
/// **Padding and not a wider label**, so that the reverse video of a focused control covers a
/// block with a word in it rather than the word alone — which is what `menu_fore`'s `A_REVERSE`
/// looks like in `menu_attributes(3x)`, and what every tool that follows it draws.
const PADDING: u16 = 1;

/// How far left of its title a drop-down's border sits.
///
/// **One column, and the arithmetic belongs to Midnight Commander rather than to this file.** A
/// menu bar whose drop-down lines up with the first letter of the label it belongs to reads as a
/// panel; one that lines up with the left edge of the screen reads as a dialog.
const DROP_DOWN_LEFT_OF_TITLE: u16 = 1;

/// The pop-up's size, as a constant rather than as something derived from what it holds.
///
/// **Fixed so that a snapshot does not move when a word of the pop-up's text changes**, which is
/// the same reason `monospace-cli` names its window rather than measuring it.
const POPUP_WIDTH: u16 = 24;
const POPUP_HEIGHT: u16 = 7;

/// Which row inside the pop-up holds its name, and which holds its button.
const POPUP_NAME_ROW: u16 = 2;
const POPUP_BUTTON_ROW: u16 = 4;

/// The three regions the screen is in, in the order they are drawn.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Regions {
    /// The bar across the top.
    pub(crate) bar: Rect,
    /// The region a diagram is drawn into.
    pub(crate) region: Rect,
    /// The status bar along the bottom.
    pub(crate) status: Rect,
}

/// Splits `area` into the three regions.
pub(crate) fn regions(area: Rect) -> Regions {
    let [bar, region, status] = Layout::vertical([
        Constraint::Length(BAR_HEIGHT),
        Constraint::Min(0),
        Constraint::Length(STATUS_HEIGHT),
    ])
    .areas(area);
    Regions {
        bar,
        region,
        status,
    }
}

/// Draws one whole frame, and records where every control landed.
///
/// **The geometry is emptied first**, so a control this frame does not draw — the drop-down's
/// entries while it is closed, the pop-up's button while it is not up — holds an empty rectangle
/// and answers no click. Without that, closing a pop-up would leave its button where it was.
pub(crate) fn draw(frame: &mut Frame, chrome: &mut Chrome) {
    chrome.forget_geometry();
    let regions = regions(frame.area());

    draw_bar(frame, regions.bar, chrome);
    draw_region(frame, regions.region);
    draw_status_bar(frame, regions.status);

    // After the region, in this order. The pop-up is last because it is the one that goes over the
    // other; the chrome never shows both at once, and this order is what it would take if it did.
    if chrome.drop_down_is_open() {
        draw_drop_down(frame, regions.region, chrome);
    }
    if chrome.pop_up_is_open() {
        draw_pop_up(frame, regions.region, chrome);
    }
}

/// Draws the bar across the top and stores where each of its controls landed.
fn draw_bar(frame: &mut Frame, area: Rect, chrome: &mut Chrome) {
    let controls: Vec<(ControlId, &'static str)> = bar().collect();
    let mut x = area.x;
    for (control, label) in controls {
        let control_area = control_area(label, x, area.y, area);
        frame.render_widget(
            Paragraph::new(padded(label)).style(style_of(chrome, control)),
            control_area,
        );
        chrome.geometry_of_mut().set(control, control_area);

        x = x.saturating_add(control_area.width);
    }
}

/// Draws the drop-down under the bar's menu, over the region.
fn draw_drop_down(frame: &mut Frame, region: Rect, chrome: &mut Chrome) {
    let entries: Vec<(usize, &'static str)> = menu().collect();
    let widest = entries
        .iter()
        .map(|(_, label)| columns_of(label))
        .max()
        .unwrap_or_default();
    let width = widest + 2 * PADDING + 2;
    let height = u16::try_from(entries.len()).unwrap_or(u16::MAX) + 2;

    // One column left of the title it belongs to, which is Midnight Commander's rule and not one
    // derived here — see [`DROP_DOWN_LEFT_OF_TITLE`].
    let left_of = chrome.geometry().area_of(ControlId::File);
    let area = Rect::new(
        left_of.x.saturating_sub(DROP_DOWN_LEFT_OF_TITLE),
        region.y,
        width,
        height,
    )
    .intersection(region);

    // `Clear` first, and it is the whole of why the drop-down is not transparent: the region's
    // border runs behind it, and without this the two frames would share a cell.
    frame.render_widget(Clear, area);
    frame.render_widget(Block::bordered(), area);

    for (index, label) in entries {
        let control = ControlId::Menu(index);
        let entry = control_area(
            label,
            area.x + 1,
            area.y + 1 + u16::try_from(index).unwrap_or(u16::MAX),
            area,
        );

        frame.render_widget(
            Paragraph::new(padded(label)).style(style_of(chrome, control)),
            entry,
        );
        chrome.geometry_of_mut().set(control, entry);
    }
}

/// Draws the region a diagram goes into, which this slice draws nothing into.
///
/// **Its border is never drawn in reverse video, and that is a decision rather than an omission.**
/// Rule 12 asks for the region holding the keyboard to be drawn with a different border from the
/// others, and the established tools do it — Midnight Commander in a heavier line, `tmux` with
/// `status-style reverse` — but a frame is already carrying one meaning and adding a second to it
/// costs more than it gives. **A frame that says where the keyboard is is redundant with the
/// control that says where the keyboard is**, and it is redundant in the worse direction: a border
/// is a hundred and fifty cells of reverse video saying something the seven cells of the focused
/// control say exactly. When the pop-up takes the keyboard away from the region, the button inside
/// it lights up and the region does not change at all, which is a smaller and more precise answer
/// than a whole panel inverting.
///
/// **What this gives up** is the cue above the item level, and it was wanted for a reason: a person
/// who cannot tell which control has the keyboard has nothing else to go on. What it buys is that
/// reverse video means one thing — this control is about to answer — everywhere on the screen. See
/// §4 and §5 of [the application model](../../../docs/application-model.md).
fn draw_region(frame: &mut Frame, area: Rect) {
    frame.render_widget(Block::bordered(), area);
}

/// Draws the status bar along the bottom.
///
/// **The application's name and nothing else, on purpose.** A hint of the keys would be a fourth
/// thing for the chrome to keep in step — one more list to go stale the day a binding changes —
/// and what this row is for is what the next slice puts there.
fn draw_status_bar(frame: &mut Frame, area: Rect) {
    frame.render_widget(Paragraph::new(format!(" {APPLICATION}")), area);
}

/// Draws the pop-up over the region, with the application's name and the one control that closes
/// it.
fn draw_pop_up(frame: &mut Frame, region: Rect, chrome: &mut Chrome) {
    let area = centred(POPUP_WIDTH, POPUP_HEIGHT, region);
    let inner = area.width.saturating_sub(2);

    frame.render_widget(Clear, area);
    frame.render_widget(Block::bordered().title(" About "), area);

    let name = control_area(
        APPLICATION,
        area.x + centred_at(inner, columns_of(APPLICATION) + 2 * PADDING),
        area.y + POPUP_NAME_ROW,
        area,
    );
    frame.render_widget(Paragraph::new(padded(APPLICATION)), name);

    let label = label_of(ControlId::Close);
    let button = control_area(
        label,
        area.x + centred_at(inner, columns_of(label) + 2 * PADDING),
        area.y + POPUP_BUTTON_ROW,
        area,
    );
    frame.render_widget(
        Paragraph::new(padded(label)).style(style_of(chrome, ControlId::Close)),
        button,
    );
    chrome.geometry_of_mut().set(ControlId::Close, button);
}

/// The rectangle a control of `label` occupies at `(x, y)`: the label with a blank column either
/// side, clipped to `within`.
///
/// **Every control's rectangle is built here**, which is what makes D3 cheap to hold to: the bar,
/// the drop-down and the pop-up's button all come out of this one function, so there is no second
/// place where a rectangle could be worked out differently from the one that was stored.
fn control_area(label: &str, x: u16, y: u16, within: Rect) -> Rect {
    Rect::new(x, y, columns_of(label) + 2 * PADDING, 1).intersection(within)
}

/// How many columns `text` takes.
///
/// **A byte count, because every label in this crate is ASCII.** A label that was not would need
/// the cell width of each glyph, which is what the specification puts out of scope: the chrome
/// draws what the terminal can show and computes no glyph width of its own.
fn columns_of(text: &str) -> u16 {
    u16::try_from(text.len()).unwrap_or(u16::MAX)
}

/// The style a control is drawn in, which is the whole of rules 12 and 13.
///
/// **At rest is the empty style**, so a control in no state is drawn with the terminal's own
/// foreground and background rather than with a color this application chose — which is the only
/// thing that looks right on a light terminal and on a dark one, and the chrome cannot tell which
/// it is on. **Focused and pressed are the same style**, so the two never compete: either one means
/// this is the control about to answer.
fn style_of(chrome: &Chrome, control: ControlId) -> Style {
    match chrome.state_of(control) {
        ControlState::AtRest => Style::new(),
        ControlState::Focused | ControlState::Pressed => {
            Style::new().add_modifier(Modifier::REVERSED)
        }
    }
}

/// `label` with [`PADDING`] blank columns either side, so that a focused control's reverse video
/// covers a block rather than a word.
fn padded(label: &str) -> String {
    let blank = " ".repeat(usize::from(PADDING));
    format!("{blank}{label}{blank}")
}

/// Where a `width` by `height` rectangle sits in `area`, centred and clipped to it.
fn centred(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + centred_at(area.width, width);
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height).intersection(area)
}

/// Where a `width` block starts in a `total` wide row, so that it sits in the middle of it.
fn centred_at(total: u16, width: u16) -> u16 {
    total.saturating_sub(width) / 2
}

#[cfg(test)]
mod tests {
    use ratatui::{
        Terminal,
        backend::TestBackend,
        buffer::Buffer,
        layout::{Position, Rect},
        style::{Color, Modifier},
    };

    use super::{APPLICATION, Regions, draw, regions};
    use crate::{
        app::App,
        chrome::{Action, Chrome, ControlId, ControlState, Move},
    };

    /// The screen every picture in this file is taken at.
    ///
    /// **Ten rows, because the spike's is and because the three regions, the drop-down and the
    /// pop-up all have to fit on one.** A screen taller than the application needs proves nothing a
    /// shorter one would not.
    const WIDTH: u16 = 40;
    const HEIGHT: u16 = 10;

    /// A terminal over `TestBackend`, which is what makes a picture of the screen possible with no
    /// terminal involved at all.
    fn terminal() -> Terminal<TestBackend> {
        Terminal::new(TestBackend::new(WIDTH, HEIGHT)).expect("the test backend is a backend")
    }

    /// One frame, drawn, with the chrome it was drawn for.
    fn draw_once(chrome: &mut Chrome) -> Terminal<TestBackend> {
        let mut terminal = terminal();
        terminal
            .draw(|frame| draw(frame, chrome))
            .expect("the frame is drawn");
        terminal
    }

    /// One frame drawn, as a buffer, for a test that only wants to look at the picture.
    fn picture(chrome: &mut Chrome) -> Buffer {
        draw_once(chrome).backend().buffer().clone()
    }

    /// The whole frame as text, with the trailing blanks each row carries dropped.
    ///
    /// **Rows are kept and columns are not**, so a row of nothing but the drop-down's own frame is
    /// a blank line rather than a missing one: the row a thing is on is half of what these tests
    /// are about.
    fn as_text(buffer: &Buffer) -> String {
        buffer
            .content()
            .chunks(usize::from(buffer.area.width))
            .map(|row| {
                row.iter()
                    .map(ratatui::buffer::Cell::symbol)
                    .collect::<String>()
                    .trim_end()
                    .to_owned()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The attributes on the cell at `position`, for the tests that ask what a cell was drawn with.
    fn modifier_at(buffer: &Buffer, position: Position) -> Modifier {
        buffer[position].modifier
    }

    /// A point inside `area`, which is where a test that means "on this control" points.
    fn a_point_in(area: Rect) -> Position {
        Position {
            x: area.x,
            y: area.y,
        }
    }

    #[test]
    fn the_bar_is_at_the_top_and_the_status_bar_is_at_the_bottom() {
        let Regions {
            bar,
            region,
            status,
        } = regions(Rect::new(0, 0, WIDTH, HEIGHT));

        assert_eq!(bar.y, 0, "the bar is not across the top of the screen");
        assert_eq!(
            status.y + status.height,
            HEIGHT,
            "the status bar does not reach the bottom of the screen"
        );
        assert!(
            region.y >= bar.bottom() && region.bottom() <= status.y,
            "the region does not sit between the two bars: {region:?}"
        );
    }

    #[test]
    fn the_region_between_them_takes_the_rest_of_the_screen() {
        for height in [4, 10, 24, 60] {
            let Regions {
                bar,
                region,
                status,
            } = regions(Rect::new(0, 0, WIDTH, height));
            assert_eq!(bar.height, 1, "at height {height} the bar is not one row");
            assert_eq!(
                status.height, 1,
                "at height {height} the status bar is not one row"
            );
            assert_eq!(
                bar.height + region.height + status.height,
                height,
                "at height {height} the three regions do not account for every row"
            );
            assert_eq!(
                region.width, WIDTH,
                "at height {height} the region does not span the screen"
            );
        }
    }

    #[test]
    fn the_bars_row_is_drawn_on_every_frame() {
        // A backend seeded with a screen that is nothing like this one, which is what a row nothing
        // drew looks like. `Terminal::new` does not clear it and the first frame diffs against an
        // empty buffer, so everything this frame draws lands over what was already there.
        let junk: Vec<String> = (0..HEIGHT)
            .map(|_| "X".repeat(usize::from(WIDTH)))
            .collect();
        let mut terminal = Terminal::new(TestBackend::with_lines(junk)).expect("a test backend");
        let mut chrome = Chrome::default();
        terminal
            .draw(|frame| draw(frame, &mut chrome))
            .expect("the frame is drawn");

        let row = as_text(terminal.backend().buffer())
            .lines()
            .next()
            .unwrap_or_default()
            .to_owned();
        assert!(
            row.contains("File"),
            "the bar's row was not drawn and kept what the screen already had in it: {row}"
        );
    }

    #[test]
    fn a_region_that_stops_being_drawn_does_not_keep_what_was_last_in_it() {
        let mut chrome = Chrome::default();
        chrome.apply(Action::ShowAbout);
        let mut terminal = draw_once(&mut chrome);
        let with_the_pop_up = as_text(terminal.backend().buffer());
        assert!(
            with_the_pop_up.contains("About"),
            "the pop-up was not drawn, so the rest of this test would be about nothing"
        );

        chrome.apply(Action::HideAbout);
        terminal
            .draw(|frame| draw(frame, &mut chrome))
            .expect("the frame is drawn");

        let closed = as_text(terminal.backend().buffer());
        let mut bare = Chrome::default();
        assert_eq!(
            closed,
            as_text(&picture(&mut bare)),
            "the frame kept what the pop-up last had in it instead of the region underneath"
        );
    }

    /// **"Nothing else" is checked by counting, not by looking for a word.** `About` is on the bar
    /// as a control and is also the pop-up's title, so a search for the word would fail for the
    /// right reason and pass for the wrong one. What rule 15 means is that the region carries the
    /// diagram's own drawing and no overlay, and a frame with an overlay in it has a second frame
    /// drawn over the first — which is a count of rows carrying a border, and nothing else does.
    #[test]
    fn the_region_holds_the_drawing_and_nothing_else() {
        let mut chrome = Chrome::default();
        let text = as_text(&picture(&mut chrome));

        assert!(
            text.contains('┌') && text.contains('└'),
            "the region is not drawn at all"
        );
        assert_eq!(
            text.lines().filter(|row| row.contains("About")).count(),
            1,
            "the pop-up's title is on the screen with nothing open: {text}"
        );
        assert!(
            !text.contains("Quit"),
            "the drop-down is on the screen with nothing open: {text}"
        );
        assert!(
            text.lines()
                .last()
                .is_some_and(|row| row.contains(APPLICATION)),
            "the status bar does not name the application: {text}"
        );
        // Between the region's own two borders there is nothing at all, which is what "holds the
        // drawing and nothing else" says when the drawing is none: no overlay, no message, and no
        // border of a second frame showing through. The rows between the region's borders are the
        // screen's less the bar, the status bar and the two borders themselves.
        let between = usize::from(HEIGHT) - 4;
        assert!(
            text.lines()
                .skip(2)
                .take(between)
                .all(|row| row.trim_matches(|c| c == '│' || c == ' ').is_empty()),
            "something is drawn inside the region: {text}"
        );
    }

    /// **Rule 12's frame half is deliberately not implemented, and this is the test that says so.**
    /// The region holding the keyboard is *not* drawn with a different border, because a frame
    /// carrying reverse video would be a second thing saying where the keyboard is, at a hundred
    /// times the size and with less precision than the focused control already has. What is held
    /// instead is the narrower claim the decision makes: reverse video is spent on controls and on
    /// nothing else, so wherever it appears on screen it means *this is the control about to
    /// answer*.
    #[test]
    fn a_frame_is_never_drawn_in_reverse_video() {
        let mut chrome = Chrome::default();
        for open in [
            Vec::new(),
            vec![Action::OpenDropDown],
            vec![Action::ShowAbout],
        ] {
            for action in open {
                chrome.apply(action);
            }
            let buffer = picture(&mut chrome);
            for (index, cell) in buffer.content().iter().enumerate() {
                if !cell.modifier.contains(Modifier::REVERSED) {
                    continue;
                }
                let column = u16::try_from(index).unwrap_or(u16::MAX) % WIDTH;
                let row = u16::try_from(index).unwrap_or(u16::MAX) / WIDTH;
                assert!(
                    row == 0 || column != 0 && column != WIDTH - 1,
                    "a cell of the region's frame is drawn in reverse video at column {column} \
                     of row {row}: {}",
                    as_text(&buffer)
                );
            }
        }
    }

    #[test]
    fn a_focused_control_is_drawn_in_reverse_video_and_a_pressed_unfocused_one_too() {
        let mut chrome = Chrome::default();
        chrome.apply(Action::ShowAbout);
        let with_focus = picture(&mut chrome);
        let button = a_point_in(chrome.geometry().area_of(ControlId::Close));

        assert!(
            modifier_at(&with_focus, button).contains(Modifier::REVERSED),
            "the control the keyboard is on is not drawn in reverse video"
        );

        // The same control with a pointer down on it while the keyboard is elsewhere.
        chrome.apply(Action::FocusAlongBar(Move::Down));
        chrome.apply(Action::Press(ControlId::Close));
        let pressed = picture(&mut chrome);
        assert_eq!(chrome.state_of(ControlId::Close), ControlState::Pressed);
        assert!(
            modifier_at(&pressed, button).contains(Modifier::REVERSED),
            "a pressed control that holds no focus is drawn differently from one at rest"
        );

        // And with nothing focused and nothing pressed, neither.
        let mut bare = Chrome::default();
        bare.apply(Action::ShowAbout);
        bare.apply(Action::FocusAlongBar(Move::Down));
        let at_rest = picture(&mut bare);
        assert!(
            !modifier_at(&at_rest, button).contains(Modifier::REVERSED),
            "nothing is focused and nothing is pressed, and the control is drawn in reverse video"
        );
    }

    #[test]
    fn a_focus_indicator_is_shown_for_as_long_as_the_focus_lasts() {
        let mut chrome = Chrome::default();
        chrome.apply(Action::ShowAbout);
        // The frame is drawn first, because the geometry it fills in is what says where the control
        // is: asking before the draw would ask about a rectangle of no columns.
        let mut terminal = draw_once(&mut chrome);
        let button = a_point_in(chrome.geometry().area_of(ControlId::Close));

        for _ in 0..3 {
            terminal
                .draw(|frame| draw(frame, &mut chrome))
                .expect("the frame is drawn");
            assert!(
                modifier_at(terminal.backend().buffer(), button).contains(Modifier::REVERSED),
                "the indicator for a focus that has not moved came and went between two frames"
            );
        }

        chrome.apply(Action::FocusAlongBar(Move::Down));
        terminal
            .draw(|frame| draw(frame, &mut chrome))
            .expect("the frame is drawn");
        assert!(
            !modifier_at(terminal.backend().buffer(), button).contains(Modifier::REVERSED),
            "the indicator is still shown after the focus left the control"
        );
    }

    #[test]
    fn a_control_in_no_state_is_drawn_with_the_terminals_own_colors() {
        let mut chrome = Chrome::default();
        let buffer = picture(&mut chrome);
        let about = a_point_in(chrome.geometry().area_of(ControlId::About));

        let cell = &buffer[about];
        assert_eq!(
            (cell.fg, cell.bg),
            (Color::Reset, Color::Reset),
            "a control in no state was given a color the terminal did not choose"
        );
        assert!(
            cell.modifier.is_empty(),
            "a control in no state was given an attribute: {:?}",
            cell.modifier
        );
    }

    /// **There is no color for `NO_COLOR` to switch off, and this is what proves it.** The chrome
    /// spends the terminal's own foreground and background everywhere and asks for one attribute,
    /// which is not a color and so is not what that variable is about. The frame is byte-identical
    /// with the variable set and without it, and that is the whole of the test: an implementation
    /// that had reached for a hue would fail here rather than pass it.
    #[test]
    fn no_color_switches_the_chromes_color_off() {
        let mut chrome = Chrome::default();
        chrome.apply(Action::ShowAbout);
        let without = as_text(&picture(&mut chrome));

        // SAFETY: one environment lookup is what the variable is, and the frame is built again on
        // the other side of it from a chrome that was never read in between. Nothing else in this
        // binary reads the environment.
        unsafe { std::env::set_var("NO_COLOR", "1") };
        let with = as_text(&picture(&mut chrome));
        unsafe { std::env::remove_var("NO_COLOR") };

        assert_eq!(
            with, without,
            "NO_COLOR changed the frame, so the chrome was spending a color"
        );
    }

    #[test]
    fn f9_opens_the_drop_down_and_escape_closes_it() {
        let mut app = App::default();
        assert!(!app.chrome().drop_down_is_open());

        app.apply(Action::OpenDropDown);
        assert!(app.chrome().drop_down_is_open());
        let open = as_text(&picture(app.chrome_mut()));
        assert!(open.contains("Quit"), "the drop-down was not drawn: {open}");

        app.apply(Action::CloseDropDown);
        assert!(!app.chrome().drop_down_is_open());
        let closed = as_text(&picture(app.chrome_mut()));
        assert!(
            !closed.contains("Quit"),
            "the drop-down is still drawn: {closed}"
        );
    }

    #[test]
    fn a_drop_down_leaves_the_region_byte_identical_when_it_closes() {
        let mut chrome = Chrome::default();
        let mut terminal = terminal();
        terminal
            .draw(|frame| draw(frame, &mut chrome))
            .expect("the frame is drawn");
        let before = as_text(terminal.backend().buffer());

        chrome.apply(Action::OpenDropDown);
        terminal
            .draw(|frame| draw(frame, &mut chrome))
            .expect("the frame is drawn");
        assert_ne!(
            as_text(terminal.backend().buffer()),
            before,
            "the drop-down was not drawn, so this test would pass for the wrong reason"
        );

        chrome.apply(Action::CloseDropDown);
        terminal
            .draw(|frame| draw(frame, &mut chrome))
            .expect("the frame is drawn");
        assert_eq!(
            as_text(terminal.backend().buffer()),
            before,
            "the region is not byte-identical to what it was before the drop-down opened"
        );
    }

    /// **The drop-down is drawn after the region it covers, and this is the line that says so.**
    /// Drawing it first would put the region's border and its `Clear` over the top of it, and the
    /// drop-down would survive only until the next repaint of the region.
    #[test]
    fn a_drop_down_is_not_erased_by_the_region_being_painted_after_it() {
        let mut chrome = Chrome::default();
        chrome.apply(Action::OpenDropDown);
        let buffer = picture(&mut chrome);
        let entry = chrome.geometry().area_of(ControlId::Menu(0));

        assert!(
            !entry.is_empty(),
            "the drop-down's entry holds no rectangle, so there is nothing to erase"
        );
        let row = as_text(&buffer);
        assert!(
            row.lines()
                .nth(usize::from(entry.y))
                .is_some_and(|line| line.contains("Quit")),
            "the drop-down's entry was erased by the region painted over it: {row}"
        );
    }

    /// **Every control the chrome can draw has a rectangle a click could find.** A control with no
    /// geometry is one a person can see and cannot press, which is the failure `#189` is about and
    /// the reason rule 11 exists.
    #[test]
    fn every_control_that_is_drawn_is_reachable_by_a_click() {
        let mut chrome = Chrome::default();
        draw_once(&mut chrome);
        for control in [ControlId::File, ControlId::About] {
            assert!(
                !chrome.geometry().area_of(control).is_empty(),
                "{control:?} is on the bar with no rectangle a click could find"
            );
        }

        chrome.apply(Action::OpenDropDown);
        draw_once(&mut chrome);
        assert!(
            !chrome.geometry().area_of(ControlId::Menu(0)).is_empty(),
            "the drop-down's entry is drawn with no rectangle a click could find"
        );

        chrome.apply(Action::ShowAbout);
        draw_once(&mut chrome);
        assert!(
            !chrome.geometry().area_of(ControlId::Close).is_empty(),
            "the pop-up's button is drawn with no rectangle a click could find"
        );
    }
}
