//! The screen, pinned as pictures.
//!
//! **Every picture of the screen in this crate is one of these**, which is the thing `crossterm`
//! alone could not do and half of what proves the slice: `ratatui`'s `TestBackend` renders the
//! whole buffer with no terminal involved, so the three regions, the three states of a control and
//! the pop-up over the region can all be looked at rather than argued about.
//!
//! **Each case is a caption and a frame, and a frame is its symbols and its reversed runs.** The
//! symbols alone would not show the half of rule 12 that matters — reverse video is an attribute
//! rather than a character, and a picture of characters cannot move when it appears — so every
//! frame is followed by the rows that carry it and the columns they cover. A snapshot that cannot
//! see the difference this slice exists to make would pin the drawing and not the decision.
//!
//! **One file per case and every case is in the same file**, because these are contract snapshots
//! rather than the characterization `monospace-cli` sweeps over: a case is one decision, and a
//! diff is read as one question — did the screen change, and in which state.

use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};
use std::fmt::Write as _;

use crate::{
    chrome::{Action, Chrome, ControlId, Move},
    screen,
};

/// The screen these are taken at: ten rows, because the three regions, the drop-down and the
/// pop-up all have to fit on one.
const WIDTH: u16 = 40;
const HEIGHT: u16 = 10;

/// One case, as its caption above its frame and nothing below.
fn case(caption: &str, chrome: &mut Chrome) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{caption}");
    let buffer = draw(chrome);
    for line in lines(&buffer) {
        let _ = writeln!(out, "{line}");
    }
    for (row, columns) in reversed(&buffer) {
        let _ = writeln!(
            out,
            "  reverse  row {row}, columns {}",
            columns
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    out
}

/// One frame, drawn, with the chrome it was drawn for.
fn draw(chrome: &mut Chrome) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(WIDTH, HEIGHT)).expect("a test backend");
    terminal
        .draw(|frame| screen::draw(frame, chrome))
        .expect("the frame is drawn");
    terminal.backend().buffer().clone()
}

/// The frame as text, with the trailing blanks each row carries dropped.
fn lines(buffer: &Buffer) -> Vec<String> {
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
        .collect()
}

/// Every row carrying reverse video, with the columns it covers.
///
/// **Runs rather than every cell**, because a control drawn in reverse video is a run of seven or
/// eight columns and listing them one by one would make a diff of a moved control unreadable.
fn reversed(buffer: &Buffer) -> Vec<(u16, Vec<u16>)> {
    let mut found: Vec<(u16, Vec<u16>)> = Vec::new();
    let width = buffer.area.width;
    for (index, cell) in buffer.content().iter().enumerate() {
        if !cell.modifier.contains(ratatui::style::Modifier::REVERSED) {
            continue;
        }
        // A cell's place is its index in the buffer: the buffer is a row-major grid of one area.
        let column = u16::try_from(index).unwrap_or(u16::MAX) % width;
        let row = u16::try_from(index).unwrap_or(u16::MAX) / width;
        match found.last_mut() {
            Some((held, columns)) if *held == row => columns.push(column),
            _ => found.push((row, vec![column])),
        }
    }
    found
}

/// Pins one case, with the path the contract snapshots live at.
fn pin(name: &str, body: &str) {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/snapshots/contract"
    ));
    settings.set_description(concat!(
        "A picture of the screen, one case per block. A change here is a change to what a person ",
        "looking at the terminal sees, and it is reviewed as such.",
    ));
    settings.bind(|| insta::assert_snapshot!(name, body));
}

/// **The three regions and nothing else**, which is rules 5 and 15 in one picture: the bar across
/// the top, the region between the two bars, and the status bar along the bottom.
#[test]
fn the_screen_at_rest() {
    pin(
        "the_screen_at_rest",
        &case("the screen at rest", &mut Chrome::default()),
    );
}

/// **Rule 7, in the three frames that show it.** The same control — `About`, the second one in the
/// bar — at rest, focused by the keyboard, and pressed with the keyboard on the other control.
/// **Only the third case has the keyboard somewhere else**, which is what makes the three states two
/// cues rather than one: the first two differ in where the focus is, the third in where the
/// pointer is, and the two cues draw the same way so they never compete.
#[test]
fn the_control_has_three_states_and_shows_which_one_it_is_in() {
    // At rest the keyboard is in the region, which has no control under it: **nothing on screen is
    // in any state**, and this is the case that shows it.
    let mut at_rest = Chrome::default();

    // The keyboard reaches the bar through the focus stack rather than through a key — `F9` and
    // then `Esc` leave it on `File`, and one `Tab` further along is `About`.
    let mut focused = Chrome::default();
    focused.apply(Action::OpenDropDown);
    focused.apply(Action::CloseDropDown);
    focused.apply(Action::FocusAlongBar(Move::Down));

    let mut pressed = Chrome::default();
    pressed.apply(Action::OpenDropDown);
    pressed.apply(Action::CloseDropDown);
    pressed.apply(Action::Press(ControlId::About));

    let cases = [
        case("at rest, with the keyboard in the region", &mut at_rest),
        case("focused by the keyboard", &mut focused),
        case(
            "pressed, with the keyboard on the other control",
            &mut pressed,
        ),
    ];
    pin("the_control_has_three_states", &cases.join("\n"));
}

/// **Rule 16, and the one thing prose cannot establish:** the bar, the region, a drop-down over the
/// region and the status bar, all on ten rows.
#[test]
fn the_drop_down_open_over_the_region() {
    let mut chrome = Chrome::default();
    chrome.apply(Action::OpenDropDown);
    pin(
        "the_drop_down_open_over_the_region",
        &case("the drop-down open over the region", &mut chrome),
    );
}

/// **Rules 12 and 14 together**, which is the whole of what a pop-up over a region has to get
/// right: the region's border changes because the region holds the keyboard, and the pop-up's
/// button is in reverse video because that is where the keyboard is.
#[test]
fn the_pop_up_over_the_region_with_the_keyboard_in_it() {
    let mut chrome = Chrome::default();
    chrome.apply(Action::ShowAbout);
    pin(
        "the_pop_up_over_the_region",
        &case(
            "the pop-up over the region, with the keyboard in it",
            &mut chrome,
        ),
    );
}

/// **The same screen at two widths, with the rectangle each control was drawn into.** The bar is
/// laid out from the left, so nothing in it moves; **the pop-up is centred, so its button does** —
/// and the numbers under each frame are the ones the click handler reads. That is D3 in a picture a
/// person can check: the click is tested against what the layout put on the screen, and the layout
/// is asked again on every frame.
#[test]
fn a_control_is_where_the_layout_puts_it_at_any_width() {
    let mut out = String::new();
    for width in [40, 72] {
        let mut terminal = Terminal::new(TestBackend::new(width, HEIGHT)).expect("a test backend");
        let mut chrome = Chrome::default();
        chrome.apply(Action::ShowAbout);
        terminal
            .draw(|frame| screen::draw(frame, &mut chrome))
            .expect("the frame is drawn");

        let _ = writeln!(out, "{width} columns");
        for line in lines(terminal.backend().buffer()) {
            let _ = writeln!(out, "{line}");
        }
        for control in [ControlId::File, ControlId::About, ControlId::Close] {
            let area = chrome.geometry().area_of(control);
            let _ = writeln!(
                out,
                "  {control:?}  columns {} to {}, row {}",
                area.x,
                area.x + area.width,
                area.y
            );
        }
    }
    pin("a_control_at_any_width", &out);
}
