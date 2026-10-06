//! Q3 — Does a pop-up compose over an ASCII canvas region?
//!
//! Builds: menu bar / canvas / status bar, opens the dialog, snapshots before,
//! during and after. The "after" snapshot must equal the "before" snapshot.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use tuirealm::application::{Application, PollStrategy};
use tuirealm::listener::EventListenerCfg;
use tuirealm::props::Attribute;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout, Rect};
use tuirealm::terminal::{TerminalAdapter, TestTerminalAdapter};

use chrome_spike::chrome::{update, Canvas, Dialog, FileMenu, Handle, Id, MenuBar, Msg, StatusBar};

const W: u16 = 44;
const H: u16 = 14;

/// Build the app. The layout solver is ratatui's Layout, driven by hand.
fn build() -> (Application<Id, Msg, tuirealm::event::NoUserEvent>, Handle) {
    let handle: Handle = Rc::new(RefCell::new(Default::default()));

    let mut app: Application<Id, Msg, tuirealm::event::NoUserEvent> =
        Application::init(EventListenerCfg::default().tick_interval(Duration::from_millis(500)));
    app.mount(
        Id::MenuBar,
        Box::new(MenuBar::new(Rc::clone(&handle))),
        vec![],
    )
    .unwrap();
    app.mount(Id::Canvas, Box::new(Canvas::new(Rc::clone(&handle))), vec![])
        .unwrap();
    app.mount(
        Id::StatusBar,
        Box::new(StatusBar::new(Rc::clone(&handle))),
        vec![],
    )
    .unwrap();
    app.mount(Id::FileMenu, Box::new(FileMenu::new(Rc::clone(&handle))), vec![])
        .unwrap();
    app.mount(Id::Dialog, Box::new(Dialog::new(Rc::clone(&handle))), vec![])
        .unwrap();
    app.active(&Id::Canvas).unwrap();
    (app, handle)
}

/// THE DRAW PASS. This is the whole composition story: you compute Rects and
/// call app.view() in the order you want. tuirealm has no notion of "popup".
fn draw(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>, f: &mut tuirealm::ratatui::Frame) {
    let full = f.area();
    let [bar, canvas, status] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // menu bar
            Constraint::Min(1),    // canvas fills the middle
            Constraint::Length(1), // status bar
        ])
        .areas(full);

    // 1. base layer
    app.view(&Id::MenuBar, f, bar);
    app.view(&Id::Canvas, f, canvas);
    app.view(&Id::StatusBar, f, status);

    // 2. drop-down, positioned under the "File" label, clipped to the canvas band
    let dropdown = Rect::new(bar.x, canvas.y, 10, 5);
    app.view(&Id::FileMenu, f, dropdown.intersection(canvas));

    // 3. modal dialog, centred over everything
    let d = Rect::new(full.x + 6, full.y + 2, 30, 8);
    // tell the Dialog where its Close button is, since it can't remember
    let _ = app.attr(
        &Id::Dialog,
        Attribute::Text,
        tuirealm::props::AttrValue::String(format!(
            "{},{}",
            d.x + Dialog::close_button().x,
            d.y + Dialog::close_button().y
        )),
    );
    app.view(&Id::Dialog, f, d);
}

fn snap(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> String {
    // NOTE: no tuirealm helper renders a whole Application; we go through the
    // TestTerminalAdapter + buffer_to_string pair ourselves.
    let mut t = TestTerminalAdapter::new(tuirealm::ratatui::layout::Size::new(W, H)).unwrap();
    let frame = t.draw(|f| draw(app, f)).unwrap();
    tuirealm::testing::buffer_to_string(&frame.buffer)
}

fn pump(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>, handle: &Handle, n: usize) {
    for _ in 0..n {
        if let Ok(msgs) = app.tick(PollStrategy::Once(Duration::from_millis(5))) {
            for m in msgs {
                update(handle, m);
            }
        }
    }
}

fn main() {
    let (mut app, handle) = build();

    println!("=========== 1. BASE SCREEN (menu closed, dialog closed) ===========");
    let base = snap(&mut app);
    println!("{base}");

    println!("=========== 2. DROP-DOWN OPEN (menu_open = true) ===========");
    handle.borrow_mut().apply(chrome_spike::state::Action::ToggleMenu);
    let menu = snap(&mut app);
    println!("{menu}");

    println!("=========== 3. DIALOG OPEN OVER THE CANVAS (modal_open = true) ===========");
    handle.borrow_mut().apply(chrome_spike::state::Action::ToggleModal);
    let modal = snap(&mut app);
    println!("{modal}");

    println!("=========== 4. CLOSE DIALOG — is the canvas intact? ===========");
    handle.borrow_mut().apply(chrome_spike::state::Action::ToggleModal);
    let after = snap(&mut app);
    println!("{after}");

    println!("=========== VERDICT ===========");
    // compare like-for-like: `menu` and `after` both have the drop-down open;
    // the ONLY difference is the dialog having been drawn and then closed.
    println!("drop-down changed the screen            : {}", base != menu);
    println!("dialog changed the screen               : {}", menu != modal);
    println!("screen restored exactly after close     : {}", menu == after);
    println!(
        "canvas bytes restored after close       : {}",
        canvas_region(&menu) == canvas_region(&after)
    );
    if menu != after {
        println!("--- diff (menu -> after) ---");
        for (i, (a, b)) in menu.lines().zip(after.lines()).enumerate() {
            if a != b {
                println!("  row {i:>2} menu  |{a}|");
                println!("  row {i:>2} after |{b}|");
            }
        }
    }
}

fn canvas_region(s: &str) -> Vec<&str> {
    s.lines().skip(1).take(12).collect()
}
