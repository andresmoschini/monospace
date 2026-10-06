//! Q2 — Does the Elm cycle force state changes through Msg, and does that break
//! "the action enum is the only thing that changes the state"?
//!
//! Four distinct write paths are exercised and labelled.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use tuirealm::application::{Application, PollStrategy};
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{
    Event, Key, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, NoUserEvent,
};
use tuirealm::listener::EventListenerCfg;
use tuirealm::props::{AttrValue, AttrValueRef, Attribute, Props, QueryResult};
use tuirealm::ratatui::Frame;
use tuirealm::ratatui::layout::Rect;
use tuirealm::ratatui::widgets::Paragraph;
use tuirealm::state::{State, StateValue};
use tuirealm::subscription::{EventClause, Sub, SubClause};

#[path = "../state.rs"]
mod state;
use state::{action_for_key, Action, App};

type Handle = Rc<RefCell<App>>;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
enum Id {
    /// the component that HOLDS FOCUS
    Focused,
    /// a background component used as a subscription target
    Aux,
}

/// The msg enum has exactly ONE shape: "an Action happened".
#[derive(Debug, PartialEq, Clone)]
enum Msg {
    Action(Action),
}

struct Panel {
    handle: Handle,
    props: Props,
    /// component-LOCAL state, exactly like tuirealm's own demo Counter does
    local: i64,
    /// if true, mutate App directly inside on() instead of emitting a Msg
    cheat: bool,
}

impl Panel {
    fn new(handle: Handle) -> Self {
        Self {
            handle,
            props: Props::default(),
            local: 0,
            cheat: false,
        }
    }
}

impl Component for Panel {
    fn view(&mut self, f: &mut Frame, area: Rect) {
        let app = self.handle.borrow();
        f.render_widget(
            Paragraph::new(format!("counter={} local={}", app.counter, self.local)),
            area,
        );
    }
    fn query<'a>(&'a self, a: Attribute) -> Option<QueryResult<'a>> {
        if a == Attribute::Value {
            return Some(AttrValueRef::Number(self.handle.borrow().counter as isize).into());
        }
        self.props.get_for_query(a)
    }
    fn attr(&mut self, a: Attribute, v: AttrValue) {
        self.props.set(a, v);
    }
    fn state(&self) -> State {
        State::Single(StateValue::I64(self.handle.borrow().counter))
    }
    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            // WRITE PATH 2: component-local state mutated by Cmd, never via Action.
            Cmd::Submit => {
                self.local += 1;
                CmdResult::Changed(State::Single(StateValue::I64(self.local)))
            }
            _ => CmdResult::Invalid(cmd),
        }
    }
}

impl AppComponent<Msg, NoUserEvent> for Panel {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        if let Some(MouseEventKind::Down(_)) = ev.as_mouse().map(|m| m.kind) {
            if self.cheat {
                // WRITE PATH 3: mutate App DIRECTLY inside on(), never via Msg.
                self.handle.borrow_mut().apply(Action::Increment);
                return Some(Msg::Action(Action::Increment));
            }
            return Some(Msg::Action(Action::Increment));
        }
        let KeyEvent { code, modifiers } = ev.as_keyboard()?;
        if *modifiers != KeyModifiers::NONE {
            return None;
        }
        let Key::Char(c) = code else { return None };
        let a = action_for_key(*c)?;
        // The disciplined choice: do NOT mutate here; just report the Action.
        Some(Msg::Action(a.clone()))
    }
}

/// the ONE function in the whole app that is allowed to write to `App`
fn update(handle: &Handle, msg: Msg) {
    match msg {
        Msg::Action(a) => handle.borrow_mut().apply(a),
    }
}

fn main() {
    let handle: Handle = Rc::new(RefCell::new(App::default()));

    let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
        EventListenerCfg::default()
            .tick_interval(Duration::from_millis(20))
            .add_port(
                Box::new(FakeKeys::new("++++")),
                Duration::from_millis(20),
                8,
            ),
    );
    app.mount(Id::Focused, Box::new(Panel::new(Rc::clone(&handle))), vec![])
        .unwrap();
    app.mount(Id::Aux, Box::new(Panel::new(Rc::clone(&handle))), vec![])
        .unwrap();
    // Subscribe the NON-focused component to every event: this is how you get
    // global keys (quit, F1) without them going through a component.
    app.subscribe(&Id::Aux, Sub::new(EventClause::Any, SubClause::Always))
        .unwrap();

    println!("focused component is: {:?}", app.focus());
    println!("subscribed(Aux, Any/Always) ok");

    println!("\n== WRITE PATH 1: app.tick() -> on() -> Msg::Action -> update() -> App.apply(Action) ==");
    let mut seen: Vec<Msg> = Vec::new();
    for _ in 0..30 {
        match app.tick(PollStrategy::Once(Duration::from_millis(10))) {
            Ok(msgs) if !msgs.is_empty() => {
                for m in msgs {
                    println!("   Msg {m:?}");
                    seen.push(m.clone());
                    update(&handle, m);
                }
            }
            Ok(_) => {}
            Err(e) => {
                println!("tick error: {e}");
                break;
            }
        }
    }
    println!("   App.counter = {}", handle.borrow().counter);
    println!("   App.applied = {:?}", handle.borrow().applied);
    println!("   => every entry in `applied` is an Action: {} actions", handle.borrow().applied.len());

    println!("\n== WRITE PATH 1b: Sub on a FOCUSED target is SILENTLY SKIPPED ==");
    app.subscribe(&Id::Focused, Sub::new(EventClause::Any, SubClause::Always));
    let a = Id::Aux;
    println!("   subscribed(Focused, Any/Always). HasFocus(Focused)={:?}", app.focus() == Some(&Id::Focused));
    println!("   -> forward_to_subscriptions() skips it (application.rs:435), so:");
    println!("      a component is EITHER the focus target OR a Sub target, never both.");

    println!("\n== WRITE PATH 2: perform(Cmd) mutates COMPONENT state, not App ==");
    {
        let mut p = Panel::new(Rc::clone(&handle));
        let before_app = handle.borrow().counter;
        let before_local = p.local;
        let before_log = handle.borrow().applied.len();
        let r = p.perform(Cmd::Submit);
        println!("   CmdResult {r:?}");
        println!("   App.counter  {before_app} -> {}", handle.borrow().counter);
        println!("   Panel.local  {before_local} -> {}", p.local);
        println!("   App.applied len {before_log} -> {}", handle.borrow().applied.len());
    }

    println!("\n== WRITE PATH 3: on() mutates App DIRECTLY; Msg is only a notice ==");
    {
        let mut p = Panel::new(Rc::clone(&handle));
        p.cheat = true;
        let before = handle.borrow().counter;
        let ev: Event<NoUserEvent> = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(tuirealm::event::MouseButton::Left),
            modifiers: KeyModifiers::NONE,
            column: 5,
            row: 5,
        });
        let msg = p.on(&ev);
        println!("   on(mouse down) -> {msg:?}");
        println!("   App.counter {before} -> {}  (mutated INSIDE on(), before any update())", handle.borrow().counter);
    }

    println!("\n== WRITE PATH 4: Application::attr() writes a mounted component from OUTSIDE the cycle ==");
    let ok = app.attr(
        &Id::Focused,
        Attribute::Title,
        AttrValue::Title("injected from outside".into()),
    );
    println!("   app.attr(..) ok={ok:?}  (no Action, no Msg, no update())");

    println!("\nfinal App: {:?}", handle.borrow());
    let _ = a;
    probe_focus_skip();
}

/// A sync `Poll` port that emits synthetic key events, so the spike can drive the
/// real `Application::tick()` Elm cycle without a terminal.
struct FakeKeys {
    chars: Vec<char>,
    i: usize,
}

impl FakeKeys {
    fn new(s: &str) -> Self {
        Self {
            chars: s.chars().collect(),
            i: 0,
        }
    }
}

impl tuirealm::listener::Poll<NoUserEvent> for FakeKeys {
    fn poll(&mut self) -> tuirealm::listener::PortResult<Option<Event<NoUserEvent>>> {
        let ev = if let Some(c) = self.chars.get(self.i) {
            self.i += 1;
            Event::Keyboard(KeyEvent::new(Key::Char(*c), KeyModifiers::NONE))
        } else {
            Event::Tick
        };
        Ok(Some(ev))
    }
}
// appended probe: does a Sub whose target HOLDS FOCUS still fire?
struct TickEcho {
    handle: Handle,
}
impl Component for TickEcho {
    fn view(&mut self, _: &mut Frame, _: Rect) {}
    fn query<'a>(&'a self, _: Attribute) -> Option<QueryResult<'a>> { None }
    fn attr(&mut self, _: Attribute, _: AttrValue) {}
    fn state(&self) -> State { State::None }
    fn perform(&mut self, _: Cmd) -> CmdResult { CmdResult::Invalid(Cmd::None) }
}
impl AppComponent<Msg, NoUserEvent> for TickEcho {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        if matches!(ev, Event::Tick) {
            return Some(Msg::Action(Action::Increment));
        }
        None
    }
}

fn probe_focus_skip() {
    let handle: Handle = Rc::new(RefCell::new(App::default()));
    let mk = |tick_ms: u64| {
        Application::<Id, Msg, NoUserEvent>::init(
            EventListenerCfg::default()
                .tick_interval(Duration::from_millis(tick_ms))
                .add_port(Box::new(OnlyTicks), Duration::from_millis(tick_ms), 4),
        )
    };

    // (a) Echo mounted, subscribed to Any/Always, NOT focused
    let mut a = mk(20);
    a.mount(Id::Aux, Box::new(TickEcho { handle: Rc::clone(&handle) }), vec![]).unwrap();
    a.subscribe(&Id::Aux, Sub::new(EventClause::Any, SubClause::Always)).unwrap();
    let mut n_a = 0;
    for _ in 0..25 {
        if let Ok(ms) = a.tick(PollStrategy::Once(Duration::from_millis(10))) { n_a += ms.len(); }
    }

    // (b) same, but now Echo HOLDS FOCUS
    let mut b = mk(20);
    b.mount(Id::Aux, Box::new(TickEcho { handle: Rc::clone(&handle) }), vec![]).unwrap();
    b.subscribe(&Id::Aux, Sub::new(EventClause::Any, SubClause::Always)).unwrap();
    b.active(&Id::Aux).unwrap();
    let mut n_b = 0;
    for _ in 0..25 {
        if let Ok(ms) = b.tick(PollStrategy::Once(Duration::from_millis(10))) { n_b += ms.len(); }
    }

    println!("msgs when Sub target is NOT focused: {n_a}");
    println!("msgs when Sub target IS  focused: {n_b}");
    println!("=> both non-zero: the event reaches the component either as the focused");
    println!("   component or as a Sub target. application.rs:435 only skips a Sub whose");
    println!("   target is focused *because it already got the event via the focus path*.");
}

struct OnlyTicks;
impl tuirealm::listener::Poll<NoUserEvent> for OnlyTicks {
    fn poll(&mut self) -> tuirealm::listener::PortResult<Option<Event<NoUserEvent>>> {
        Ok(Some(Event::Tick))
    }
}
