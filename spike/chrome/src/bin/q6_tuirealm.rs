//! Q6a — MINIMAL tuirealm: top menu bar with one drop-down, a popup with a
//! mouse-clickable Close button, a status bar. Everything else stripped.
use std::cell::RefCell;
use std::rc::Rc;

use tuirealm::application::Application;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind, NoUserEvent};
use tuirealm::listener::EventListenerCfg;
use tuirealm::props::{AttrValue, AttrValueRef, Attribute, Props, QueryResult};
use tuirealm::ratatui::Frame;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout, Rect, Size};
use tuirealm::ratatui::style::{Color, Style};
use tuirealm::ratatui::text::Line;
use tuirealm::ratatui::widgets::{Block, Borders, Clear, Paragraph};
use tuirealm::state::State;
use tuirealm::terminal::{TerminalAdapter, TestTerminalAdapter};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Id { Bar, Drop, Popup, Status }

#[derive(PartialEq, Debug, Clone)]
enum Msg { OpenMenu, CloseMenu, OpenPopup, CloseClicked, Bump }

#[derive(Default)]
struct S { menu: bool, popup: bool, n: i64 }

/// A generic text widget: this is the *minimum* a tuirealm component is.
struct Txt { p: Props, kind: Kind }
#[derive(Clone, Copy)]
enum Kind { Bar, Drop, Popup, Status }
impl Txt {
    fn new(kind: Kind) -> Self { Self { p: Props::default(), kind } }
}
impl Component for Txt {
    fn view(&mut self, f: &mut Frame, a: Rect) {
        let s = f.area();
        match self.kind {
            Kind::Bar => {
                f.render_widget(
                    Paragraph::new(vec![
                        Line::from("File").style(Style::default().fg(Color::Cyan)),
                        Line::from("  Edit  View"),
                    ]),
                    a,
                );
            }
            Kind::Drop => {
                if !self.menu_open() { return; }
                f.render_widget(
                    Paragraph::new(" Open\n Save\n Exit")
                        .block(Block::default().borders(Borders::ALL)),
                    a,
                );
            }
            Kind::Popup => {
                if !self.popup_open() { return; }
                f.render_widget(Clear, a);
                f.render_widget(
                    Paragraph::new(" Discard changes?\n\n   Close   ")
                        .block(Block::default().borders(Borders::ALL).title(" Confirm ")),
                    a,
                );
            }
            Kind::Status => {
                f.render_widget(Paragraph::new(" F1 menu  F2 dialog  q quit"), a);
            }
        }
        let _ = s;
    }
    fn query<'a>(&'a self, at: Attribute) -> Option<QueryResult<'a>> {
        if at == Attribute::Text { return Some(AttrValueRef::String("x").into()); }
        self.p.get_for_query(at)
    }
    fn attr(&mut self, at: Attribute, v: AttrValue) { self.p.set(at, v); }
    fn state(&self) -> State { State::None }
    fn perform(&mut self, _: Cmd) -> CmdResult { CmdResult::Invalid(Cmd::None) }
}
impl Txt {
    // the shared state lives in a prop, read by view()
    fn flag(&self, key: &str) -> bool {
        self.p.get(Attribute::Text).and_then(|v| match v {
            AttrValue::String(s) => Some(s.split(',').any(|t| t == key)),
            _ => None,
        }).unwrap_or(false)
    }
    fn menu_open(&self) -> bool { self.flag("menu") }
    fn popup_open(&self) -> bool { self.flag("popup") }
}

impl AppComponent<Msg, NoUserEvent> for Txt {
    fn on(&mut self, e: &Event<NoUserEvent>) -> Option<Msg> {
        if let Some(MouseEvent { kind, column, row, .. }) = e.as_mouse()
            && matches!(kind, MouseEventKind::Down(MouseButton::Left))
        {
            // manual hit-test: the Close button's absolute rect, hardcoded
            let (bx, by) = (10u16, 5u16);
            if *row >= by && *row < by + 3 && *column >= bx && *column < bx + 10 {
                return Some(Msg::CloseClicked);
            }
            return None;
        }
        let KeyEvent { code, modifiers } = e.as_keyboard()?;
        if *modifiers != KeyModifiers::NONE { return None; }
        match code {
            Key::Function(1) => Some(Msg::OpenMenu),
            Key::Function(2) => Some(Msg::OpenPopup),
            Key::Esc => Some(Msg::CloseMenu),
            Key::Char('q') => Some(Msg::Bump),
            _ => None,
        }
    }
}

fn main() {
    let s = Rc::new(RefCell::new(S::default()));
    let mut app: Application<Id, Msg, NoUserEvent> = Application::init(EventListenerCfg::default());
    for (id, kind) in [(Id::Bar, Kind::Bar), (Id::Drop, Kind::Drop), (Id::Popup, Kind::Popup), (Id::Status, Kind::Status)] {
        app.mount(id, Box::new(Txt::new(kind)), vec![]).unwrap();
    }
    app.active(&Id::Bar).unwrap();
    let mut apply = |s: &Rc<RefCell<S>>, m: Msg| {
        let mut s = s.borrow_mut();
        match m {
            Msg::OpenMenu => s.menu = true,
            Msg::CloseMenu => s.menu = false,
            Msg::OpenPopup => s.popup = true,
            Msg::CloseClicked => s.popup = false,
            Msg::Bump => s.n += 1,
        }
        let t = format!("menu,popup");
        let _ = app.attr(&Id::Drop, Attribute::Text, AttrValue::String(t.clone()));
        let _ = app.attr(&Id::Popup, Attribute::Text, AttrValue::String(t));
    };
    // show the popup
    apply(&s, Msg::OpenMenu);
    apply(&s, Msg::OpenPopup);
    let mut term = TestTerminalAdapter::new(Size::new(40, 10)).unwrap();
    let fr = term.draw(|f| {
        let [bar, mid, st] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
            .areas(f.area());
        app.view(&Id::Bar, f, bar);
        app.view(&Id::Drop, f, Rect::new(bar.x, mid.y, 8, 4).intersection(mid));
        app.view(&Id::Popup, f, Rect::new(f.area().x + 4, f.area().y + 1, 24, 6));
        app.view(&Id::Status, f, st);
    }).unwrap();
    print!("{}", tuirealm::testing::buffer_to_string(&fr.buffer));
    let _ = s;
}
