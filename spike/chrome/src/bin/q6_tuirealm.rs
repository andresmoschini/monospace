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
use tuirealm::application::PollStrategy;
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalAdapter, TestTerminalAdapter};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Id { Bar, Drop, Popup, Status }

#[derive(PartialEq, Debug, Clone)]
enum Msg { OpenMenu, CloseMenu, OpenPopup, CloseClicked, Bump, Quit }

#[derive(Default)]
struct S { menu: bool, popup: bool, n: i64, quit: bool }

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
                // **Painting the area whether or not the menu is open is the fix for the screen
                // that would not clear.** ratatui's `Terminal::draw` diffs the frame against the
                // previous one, so a component that returns early leaves whatever it drew last time
                // still on screen. This is D6's old objection to a widget framework, and it is
                // true: a widget is not a canvas and it does not clear what it is not using.
                if !self.menu_open() {
                    f.render_widget(Clear, a);
                    return;
                }
                f.render_widget(
                    Paragraph::new(" Open\n Save\n Exit")
                        .block(Block::default().borders(Borders::ALL)),
                    a,
                );
            }
            Kind::Popup => {
                if !self.popup_open() {
                    f.render_widget(Clear, a);
                    return;
                }
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
    /// The shared state travels as a prop, read in `view`. **A string of two booleans is what a
    /// real component would replace with typed props**, and it is here because a minimal tuirealm
    /// component has nowhere else to put it.
    fn flags(&self) -> (bool, bool) {
        self.p.get(Attribute::Text).and_then(|v| match v {
            AttrValue::String(s) => {
                let mut it = s.split(',');
                let menu = it.next() == Some("true");
                let popup = it.next() == Some("true");
                Some((menu, popup))
            }
            _ => None,
        }).unwrap_or((false, false))
    }
    fn menu_open(&self) -> bool { self.flags().0 }
    fn popup_open(&self) -> bool { self.flags().1 }
}

impl AppComponent<Msg, NoUserEvent> for Txt {
    fn on(&mut self, e: &Event<NoUserEvent>) -> Option<Msg> {
        if let Some(MouseEvent { kind, column, row, .. }) = e.as_mouse()
            && matches!(kind, MouseEventKind::Down(MouseButton::Left))
        {
            // Manual hit-test against the Close button. **These constants are the cost**: the
            // component was drawn into whatever `draw` computed from the real screen size and was
            // never told what that was. Resize the terminal and the button moves while these
            // numbers do not, and nothing reports that they stopped matching. `q7-mouse` goes
            // through all four ways of getting this right.
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
            Key::Char('Q') => Some(Msg::Quit),
            _ => None,
        }
    }
}

/// The three regions, and the pop-up's own rectangle. One `draw` used by both the live terminal and
/// the headless print, so the two cannot drift.
fn draw(f: &mut Frame, app: &mut Application<Id, Msg, NoUserEvent>) {
    let [bar, mid, st] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
        .areas(f.area());
    app.view(&Id::Bar, f, bar);
    app.view(&Id::Drop, f, Rect::new(bar.x, mid.y, 8, 4).intersection(mid));
    app.view(&Id::Popup, f, popup_rect(f.area()));
    app.view(&Id::Status, f, st);
}

/// The pop-up's rectangle, computed from the whole screen. **This is the thing a tuirealm
/// component is never told**: it is computed out here and the hit-test in `on()` re-derives the same
/// numbers from a constant, because nothing hands the component where it was drawn.
fn popup_rect(whole: Rect) -> Rect {
    Rect::new(whole.x + 4, whole.y + 1, 24, 6)
}

/// Applies a message to the state and pushes the flags the components read back out of their props.
///
/// **A function rather than the closure this started as**, because a closure capturing `&mut app`
/// cannot coexist with the loop that also borrows it — and that is the first of several small frictions
/// the Elm shape costs a program that wants an ordinary `&mut`.
fn apply(s: &Rc<RefCell<S>>, app: &mut Application<Id, Msg, NoUserEvent>, m: Msg) {
    let mut s = s.borrow_mut();
    match m {
        Msg::OpenMenu => s.menu = true,
        Msg::CloseMenu => s.menu = false,
        Msg::OpenPopup => s.popup = true,
        Msg::CloseClicked => s.popup = false,
        Msg::Bump => s.n += 1,
        Msg::Quit => s.quit = true,
    }
    let t = format!("{},{}", s.menu, s.popup);
    let _ = app.attr(&Id::Drop, Attribute::Text, AttrValue::String(t.clone()));
    let _ = app.attr(&Id::Popup, Attribute::Text, AttrValue::String(t));
}

fn main() {
    let s = Rc::new(RefCell::new(S::default()));
    let mut app: Application<Id, Msg, NoUserEvent> = Application::init(EventListenerCfg::default());
    for (id, kind) in [(Id::Bar, Kind::Bar), (Id::Drop, Kind::Drop), (Id::Popup, Kind::Popup), (Id::Status, Kind::Status)] {
        app.mount(id, Box::new(Txt::new(kind)), vec![]).unwrap();
    }
    app.active(&Id::Bar).unwrap();
    // Open both, so the first frame is the interesting one rather than an empty screen.
    apply(&s, &mut app, Msg::OpenMenu);
    apply(&s, &mut app, Msg::OpenPopup);

    if std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        // The live application, the same one `q6-plain-full` runs: F1 menu, F2 dialog, Esc closes
        // the menu, q bumps, Q quits, and Close answers the mouse. The mouse is captured, so the
        // terminal's own selection is gone for as long as this runs.
        let mut term = CrosstermTerminalAdapter::new().unwrap();
        term.enable_mouse_capture().unwrap();
        while !s.borrow().quit {
            term.draw(|f| draw(f, &mut app)).unwrap();
            match app.tick(PollStrategy::BlockCollectUpTo(1)) {
                Ok(msgs) => {
                    for m in msgs {
                        apply(&s, &mut app, m);
                    }
                }
                Err(_) => break,
            }
        }
        term.disable_mouse_capture().unwrap();
        return;
    }

    let mut term = TestTerminalAdapter::new(Size::new(40, 10)).unwrap();
    let fr = term.draw(|f| draw(f, &mut app)).unwrap();
    print!("{}", tuirealm::testing::buffer_to_string(&fr.buffer));
}
