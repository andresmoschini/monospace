//! Q4 — Can the WHOLE screen be snapshot-tested?
//!
//! Three paths are tried and the result of each printed:
//!   A. tuirealm::testing::render_to_string  (per-COMPONENT only)
//!   B. TestTerminalAdapter + our own draw pass  (whole Application)
//!   C. raw ratatui TestBackend                  (no tuirealm at all)

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use tuirealm::application::Application;
use tuirealm::listener::EventListenerCfg;
use tuirealm::props::Attribute;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout, Rect, Size};
use tuirealm::terminal::{TerminalAdapter, TestTerminalAdapter};

use chrome_spike::chrome::{
    update, Canvas, Dialog, FileMenu, Handle, Id, MenuBar, Msg, StatusBar,
};
use chrome_spike::state::Action;

const W: u16 = 44;
const H: u16 = 14;

fn build() -> (Application<Id, Msg, tuirealm::event::NoUserEvent>, Handle) {
    let handle: Handle = Rc::new(RefCell::new(Default::default()));
    let mut app: Application<Id, Msg, tuirealm::event::NoUserEvent> =
        Application::init(EventListenerCfg::default().tick_interval(Duration::from_millis(500)));
    app.mount(Id::MenuBar, Box::new(MenuBar::new(Rc::clone(&handle))), vec![]).unwrap();
    app.mount(Id::Canvas, Box::new(Canvas::new(Rc::clone(&handle))), vec![]).unwrap();
    app.mount(Id::StatusBar, Box::new(StatusBar::new(Rc::clone(&handle))), vec![]).unwrap();
    app.mount(Id::FileMenu, Box::new(FileMenu::new(Rc::clone(&handle))), vec![]).unwrap();
    app.mount(Id::Dialog, Box::new(Dialog::new(Rc::clone(&handle))), vec![]).unwrap();
    app.active(&Id::Canvas).unwrap();
    (app, handle)
}

fn draw(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>, f: &mut tuirealm::ratatui::Frame) {
    let full = f.area();
    let [bar, canvas, status] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .areas(full);
    app.view(&Id::MenuBar, f, bar);
    app.view(&Id::Canvas, f, canvas);
    app.view(&Id::StatusBar, f, status);
    app.view(&Id::FileMenu, f, Rect::new(bar.x, canvas.y, 10, 5).intersection(canvas));
    let d = Rect::new(full.x + 6, full.y + 2, 30, 8);
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

/// PATH B: the whole screen as a snapshot-able String.
fn snapshot(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> String {
    let mut t = TestTerminalAdapter::new(Size::new(W, H)).unwrap();
    let frame = t.draw(|f| draw(app, f)).unwrap();
    tuirealm::testing::buffer_to_string(&frame.buffer)
}

/// PATH C: the same picture via ratatui's own TestBackend, tuirealm bypassed.
fn snapshot_plain(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> String {
    let backend = tuirealm::ratatui::backend::TestBackend::new(W, H);
    let mut term = tuirealm::ratatui::Terminal::new(backend).unwrap();
    let done = term
        .draw(|f| draw(app, f))
        .unwrap();
    tuirealm::testing::buffer_to_string(done.buffer)
}

fn main() {
    let (mut app, handle) = build();

    println!("##### PATH A: tuirealm::testing::render_to_string — ONE COMPONENT ONLY #####");
    {
        // Signature: (component: &mut dyn Component, size). No Application, no layout.
        let mut canvas = Canvas::new(Rc::clone(&handle));
        let s = tuirealm::testing::render_to_string(&mut canvas, Size::new(40, 7));
        println!("{s}");
        println!("=> renders a SINGLE component into a bare buffer. It cannot see the");
        println!("   Application, the layout, or any other component. Cannot snapshot the app.");
    }

    println!("\n##### PATH B: TestTerminalAdapter + our draw pass — THE WHOLE SCREEN #####");
    let s1 = snapshot(&mut app);
    println!("{s1}");

    println!("##### PATH C: ratatui TestBackend directly, tuirealm bypassed #####");
    let s2 = snapshot_plain(&mut app);
    println!("{s2}");
    println!("Path B == Path C : {}", s1 == s2);

    println!("\n##### IS THE SNAPSHOT STATE-SENSITIVE (insta would catch a regression)? #####");
    handle.borrow_mut().apply(Action::Increment);
    handle.borrow_mut().apply(Action::Increment);
    let s3 = snapshot(&mut app);
    println!("after 2x Increment, differs from before: {}", s1 != s3);
    for (i, (a, b)) in s1.lines().zip(s3.lines()).enumerate() {
        if a != b {
            println!("  row {i:>2} before |{a}|");
            println!("  row {i:>2} after  |{b}|");
        }
    }

    println!("\n##### msg -> update -> render, the full loop, still headless #####");
    handle.borrow_mut().apply(Action::ToggleModal);
    let s4 = snapshot(&mut app);
    println!("modal snapshot differs: {}", s3 != s4);
    // feed a synthetic Esc through the real on() path
    let ev = tuirealm::event::Event::Keyboard(tuirealm::event::KeyEvent::new(
        tuirealm::event::Key::Esc,
        tuirealm::event::KeyModifiers::NONE,
    ));
    // NOTE: get_component() returns &dyn (immutable) so on() cannot be called on it;
    // get_component_mut() returns &mut dyn and can.
    let msg = app
        .get_component_mut(&Id::Dialog)
        .map(|c| c.on(&ev))
        .flatten();
    println!("Dialog::on(Esc) -> {msg:?}");
    if let Some(m) = msg {
        update(&handle, m);
    }
    println!("snapshot after Esc identical to pre-modal: {}", s3 == snapshot(&mut app));
}
