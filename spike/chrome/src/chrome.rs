//! Q3/Q4/Q6/Q7 shared: a screen with a top menu bar (one label opens a drop-down),
//! an ASCII box-drawing canvas region, a status bar, and a modal dialog.
//!
//! Every component borrows `App` through a handle and reads it in `view()`.

use std::cell::RefCell;
use std::rc::Rc;

use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{
    Event, Key, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind, NoUserEvent,
};
use tuirealm::props::{AttrValue, Attribute, Props, QueryResult};
use tuirealm::ratatui::Frame;
use tuirealm::ratatui::layout::Rect;
use tuirealm::ratatui::style::{Color, Style};
use tuirealm::ratatui::text::Line;
use tuirealm::ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use tuirealm::state::State;

use crate::state::{Action, App};

pub type Handle = Rc<RefCell<App>>;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    MenuBar,
    FileMenu,
    Canvas,
    StatusBar,
    Dialog,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Msg {
    Action(Action),
    /// a menu label was activated (by key or by mouse) -> route to an Action
    MenuHit(String),
    /// the Close button of the dialog was pressed -> route to an Action
    CloseClicked,
}

// ---------------------------------------------------------------- menu bar

/// The top bar. Renders label strings; the labels are just text, and the
/// component records which one was hit, so the drop-down decision stays in App.
pub struct MenuBar {
    handle: Handle,
    props: Props,
}

impl MenuBar {
    pub fn new(handle: Handle) -> Self {
        Self {
            handle,
            props: Props::default(),
        }
    }
    /// The labels, and the x-ranges each occupies. This is the hand-written
    /// hit-test table — tuirealm does not compute it.
    pub fn labels() -> Vec<(&'static str, std::ops::Range<u16>)> {
        vec![("File", 0..4), ("Edit", 6..10), ("View", 12..16)]
    }
}

impl Component for MenuBar {
    fn view(&mut self, f: &mut Frame, area: Rect) {
        let app = self.handle.borrow();
        let spans: Vec<Line> = MenuBar::labels()
            .into_iter()
            .map(|(label, range)| {
                let active = app.menu_open && label == "File";
                let style = if active {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Gray)
                };
                // pad to the declared width so the drawn box matches the table
                let pad = (range.end - range.start) as usize;
                Line::from(format!("{:<pad$}", label)).style(style)
            })
            .collect();
        f.render_widget(Paragraph::new(spans), area);
    }
    fn query<'a>(&'a self, a: Attribute) -> Option<QueryResult<'a>> {
        self.props.get_for_query(a)
    }
    fn attr(&mut self, a: Attribute, v: AttrValue) {
        self.props.set(a, v);
    }
    fn state(&self) -> State {
        State::None
    }
    fn perform(&mut self, _: Cmd) -> CmdResult {
        CmdResult::Invalid(Cmd::None)
    }
}

impl AppComponent<Msg, NoUserEvent> for MenuBar {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        // keyboard
        if let Some(KeyEvent { code, modifiers }) = ev.as_keyboard()
            && *modifiers == KeyModifiers::NONE
        {
            return match code {
                Key::Char('f') => Some(Msg::MenuHit("File".into())),
                Key::Char('e') => Some(Msg::MenuHit("Edit".into())),
                _ => None,
            };
        }
        // mouse: manual coordinate hit-test against the table above
        if let Some(MouseEvent { kind, column, row, .. }) = ev.as_mouse()
            && matches!(kind, MouseEventKind::Down(MouseButton::Left))
            && *row == 0
            && area_contains(row, column)
        {
            for (label, range) in MenuBar::labels() {
                if range.contains(column) {
                    return Some(Msg::MenuHit(label.into()));
                }
            }
        }
        None
    }
}

fn area_contains(_row: &u16, _col: &u16) -> bool {
    true
}

// ---------------------------------------------------------------- drop-down

/// The drop-down list. Only rendered when `App::menu_open` — it reads App to
/// decide, it does not own "open" itself.
pub struct FileMenu {
    handle: Handle,
    props: Props,
}

impl FileMenu {
    pub fn new(handle: Handle) -> Self {
        Self {
            handle,
            props: Props::default(),
        }
    }
    pub fn items() -> Vec<&'static str> {
        vec!["Open", "Save", "Export", "Quit"]
    }
}

impl Component for FileMenu {
    fn view(&mut self, f: &mut Frame, area: Rect) {
        if !self.handle.borrow().menu_open {
            return; // nothing drawn; the canvas underneath is untouched
        }
        let lines: Vec<Line> = FileMenu::items()
            .into_iter()
            .map(|i| Line::from(format!(" {i}")))
            .collect();
        f.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            ),
            area,
        );
    }
    fn query<'a>(&'a self, a: Attribute) -> Option<QueryResult<'a>> {
        self.props.get_for_query(a)
    }
    fn attr(&mut self, a: Attribute, v: AttrValue) {
        self.props.set(a, v);
    }
    fn state(&self) -> State {
        State::None
    }
    fn perform(&mut self, _: Cmd) -> CmdResult {
        CmdResult::Invalid(Cmd::None)
    }
}

impl AppComponent<Msg, NoUserEvent> for FileMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        if !self.handle.borrow().menu_open {
            return None;
        }
        if let Some(KeyEvent { code, .. }) = ev.as_keyboard()
            && *code == Key::Esc
        {
            return Some(Msg::Action(Action::ToggleMenu));
        }
        None
    }
}

// ---------------------------------------------------------------- canvas

/// The ASCII box-drawing canvas.
pub struct Canvas {
    handle: Handle,
    props: Props,
}

impl Canvas {
    pub fn new(handle: Handle) -> Self {
        Self {
            handle,
            props: Props::default(),
        }
    }
    /// The art. Fixed lines; the last one is padded so box edges line up.
    pub fn lines(counter: i64) -> Vec<String> {
        vec![
            "┌──────────────────────────────────────┐".to_string(),
            "│                                      │".to_string(),
            "│   monospace                          │".to_string(),
            "│                                      │".to_string(),
            format!("│   counter = {counter:<26}│"),
            "│                                      │".to_string(),
            "└──────────────────────────────────────┘".to_string(),
        ]
    }
}

impl Component for Canvas {
    fn view(&mut self, f: &mut Frame, area: Rect) {
        let app = self.handle.borrow();
        let lines: Vec<Line> = Canvas::lines(app.counter)
            .into_iter()
            .map(|l| Line::from(l))
            .collect();
        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), area);
    }
    fn query<'a>(&'a self, a: Attribute) -> Option<QueryResult<'a>> {
        self.props.get_for_query(a)
    }
    fn attr(&mut self, a: Attribute, v: AttrValue) {
        self.props.set(a, v);
    }
    fn state(&self) -> State {
        State::None
    }
    fn perform(&mut self, _: Cmd) -> CmdResult {
        CmdResult::Invalid(Cmd::None)
    }
}

impl AppComponent<Msg, NoUserEvent> for Canvas {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        // keys that apply while the canvas region has focus
        if let Some(KeyEvent { code, modifiers }) = ev.as_keyboard()
            && *modifiers == KeyModifiers::NONE
        {
            return match code {
                Key::Char('+') => Some(Msg::Action(Action::Increment)),
                Key::Char('-') => Some(Msg::Action(Action::Decrement)),
                Key::Char('d') => Some(Msg::Action(Action::ToggleModal)),
                Key::Char('m') => Some(Msg::Action(Action::ToggleMenu)),
                _ => None,
            };
        }
        None
    }
}

// ---------------------------------------------------------------- status bar

pub struct StatusBar {
    handle: Handle,
    props: Props,
}

impl StatusBar {
    pub fn new(handle: Handle) -> Self {
        Self {
            handle,
            props: Props::default(),
        }
    }
}

impl Component for StatusBar {
    fn view(&mut self, f: &mut Frame, area: Rect) {
        let app = self.handle.borrow();
        let hint = if app.modal_open {
            "[Esc] close dialog   [+/-] counter"
        } else {
            "[f] File  [m] menu  [d] dialog  [+/-] counter  [q] quit"
        };
        f.render_widget(
            Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
            area,
        );
    }
    fn query<'a>(&'a self, a: Attribute) -> Option<QueryResult<'a>> {
        self.props.get_for_query(a)
    }
    fn attr(&mut self, a: Attribute, v: AttrValue) {
        self.props.set(a, v);
    }
    fn state(&self) -> State {
        State::None
    }
    fn perform(&mut self, _: Cmd) -> CmdResult {
        CmdResult::Invalid(Cmd::None)
    }
}

impl AppComponent<Msg, NoUserEvent> for StatusBar {
    fn on(&mut self, _: &Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

// ---------------------------------------------------------------- dialog

/// The modal pop-up with a Close button.
pub struct Dialog {
    handle: Handle,
    props: Props,
}

impl Dialog {
    pub fn new(handle: Handle) -> Self {
        Self {
            handle,
            props: Props::default(),
        }
    }
    /// The Close button's rect RELATIVE to the dialog's own area.
    pub fn close_button() -> Rect {
        Rect::new(4, 3, 10, 1)
    }
}

impl Component for Dialog {
    fn view(&mut self, f: &mut Frame, area: Rect) {
        if !self.handle.borrow().modal_open {
            return;
        }
        // Clear first: this is what makes it overlay rather than blend.
        f.render_widget(Clear, area);
        let btn = Dialog::close_button();
        // dialog chrome, with the button's row left to the button widget
        f.render_widget(
            Paragraph::new("Discard changes?")
                .block(Block::default().borders(Borders::ALL).title(" Confirm "))
                .style(Style::default().fg(Color::Yellow)),
            area,
        );
        // the Close button, drawn as its own bordered widget inside the dialog
        f.render_widget(
            Paragraph::new(" Close ").style(Style::default().fg(Color::Black).bg(Color::Yellow)),
            Rect::new(area.x + btn.x, area.y + btn.y, btn.width, btn.height),
        );
    }
    fn query<'a>(&'a self, a: Attribute) -> Option<QueryResult<'a>> {
        self.props.get_for_query(a)
    }
    fn attr(&mut self, a: Attribute, v: AttrValue) {
        self.props.set(a, v);
    }
    fn state(&self) -> State {
        State::None
    }
    fn perform(&mut self, _: Cmd) -> CmdResult {
        CmdResult::Invalid(Cmd::None)
    }
}

impl AppComponent<Msg, NoUserEvent> for Dialog {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        if !self.handle.borrow().modal_open {
            return None;
        }
        // keyboard close
        if let Some(KeyEvent { code, .. }) = ev.as_keyboard()
            && *code == Key::Esc
        {
            return Some(Msg::Action(Action::ToggleModal));
        }
        // mouse close: does the click land inside the Close button?
        if let Some(MouseEvent { kind, column, row, .. }) = ev.as_mouse()
            && matches!(kind, MouseEventKind::Down(MouseButton::Left))
        {
            // `self.area` is NOT available: tuirealm does not store the Rect the
            // component was last drawn into, so the app must hand it back.
            let hit = self.hits(*column, *row);
            println!(
                "   [dialog] click at ({column},{row}) {} Close (btn area {:?})",
                if hit { "HIT" } else { "MISSED" },
                self.button_area()
            );
            if hit {
                return Some(Msg::CloseClicked);
            }
        }
        None
    }
}

impl Dialog {
    /// The click test. tuirealm does NOT remember the Rect it drew into, so the
    /// app must stash it; we pack it into a prop as "x,y".
    fn button_area(&self) -> Option<Rect> {
        let s = self.props.get(Attribute::Text).and_then(|v| match v {
            AttrValue::String(s) => Some(s.clone()),
            _ => None,
        })?;
        let (x, y) = s.split_once(',')?;
        Some(Rect::new(
            x.parse().ok()?,
            y.parse().ok()?,
            Dialog::close_button().width,
            Dialog::close_button().height + 2,
        ))
    }

    fn hits(&self, column: u16, row: u16) -> bool {
        self.button_area().is_some_and(|a| {
            a.x <= column && column < a.x + a.width && a.y <= row && row < a.y + a.height
        })
    }
}

/// update: the ONE writer of App.
pub fn update(handle: &Handle, msg: Msg) {
    match msg {
        Msg::Action(a) => handle.borrow_mut().apply(a),
        Msg::MenuHit(name) => {
            let mut app = handle.borrow_mut();
            app.apply(if name == "File" {
                Action::ToggleMenu
            } else {
                Action::ToggleMenu
            });
        }
        Msg::CloseClicked => handle.borrow_mut().apply(Action::ToggleModal),
    }
}
