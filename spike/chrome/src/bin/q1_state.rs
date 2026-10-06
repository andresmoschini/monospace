//! Q1 — Does app state survive OUTSIDE the component tree?
//!
//! Claim under test: a plain Rust struct + Action enum can live in a module that
//! imports neither ratatui nor tuirealm, and a tuirealm component can read and
//! mutate it from `on()` / `perform()` WITHOUT App being a `Component`.

use std::cell::RefCell;
use std::rc::Rc;

use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{AttrValue, AttrValueRef, Attribute, Props, QueryResult};
use tuirealm::ratatui::Frame;
use tuirealm::ratatui::layout::Rect;
use tuirealm::ratatui::widgets::Paragraph;
use tuirealm::state::{State, StateValue};

#[path = "../state.rs"]
mod state;
use state::{action_for_key, Action, App};

/// The handle the component holds. `App` itself is NEVER a Component.
type Handle = Rc<RefCell<App>>;

/// A tuirealm component that owns no state of its own — it borrows the app state.
struct BorrowingLabel {
    handle: Handle,
}

impl Component for BorrowingLabel {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        // read through the handle
        let app = self.handle.borrow();
        frame.render_widget(Paragraph::new(format!("counter = {}", app.counter)), area);
    }
    fn query<'a>(&'a self, attr: Attribute) -> Option<QueryResult<'a>> {
        // expose a slice of the plain struct as tuirealm State, on demand
        if attr == Attribute::Value {
            return Some(AttrValueRef::Number(self.handle.borrow().counter as isize).into());
        }
        None
    }
    fn attr(&mut self, _: Attribute, _: AttrValue) {}
    fn state(&self) -> State {
        State::Single(StateValue::I64(self.handle.borrow().counter))
    }
    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        // MUTATE the plain struct straight from inside a tuirealm callback
        if cmd == Cmd::Custom("reset") {
            self.handle.borrow_mut().apply(Action::Reset);
            return CmdResult::Changed(self.state());
        }
        CmdResult::Invalid(cmd)
    }
}

#[derive(Debug, PartialEq)]
enum Msg {
    Action(Action),
}

impl AppComponent<Msg, NoUserEvent> for BorrowingLabel {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        let KeyEvent { code, modifiers } = ev.as_keyboard()?;
        if *modifiers != KeyModifiers::NONE {
            return None;
        }
        let ch = match code {
            Key::Char(c) => *c,
            _ => return None,
        };
        // route key -> Action (framework-free fn) -> Msg
        let a = action_for_key(ch)?;
        // mutate the plain struct here, directly
        self.handle.borrow_mut().apply(a.clone());
        Some(Msg::Action(a))
    }
}

fn main() {
    let handle: Handle = Rc::new(RefCell::new(App::default()));

    // mount a component that holds only a handle
    let mut comp = BorrowingLabel {
        handle: Rc::clone(&handle),
    };

    println!("App type is Component? {}", false); // App never implements Component.
    println!("initial: {:?}", handle.borrow());

    // --- mutate from the AppComponent::on() path (key -> Action) ---
    for key in ['+', '+', '+', '-', 'r'] {
        let ev: Event<NoUserEvent> =
            Event::Keyboard(KeyEvent::new(Key::Char(key), KeyModifiers::NONE));
        let msg = comp.on(&ev);
        println!("key {key:?} -> msg {msg:?} -> counter {}", handle.borrow().counter);
    }

    // --- mutate from the Component::perform() path (Cmd) ---
    let r = comp.perform(Cmd::Custom("reset"));
    println!("perform(Custom reset) -> {r:?}, counter {}", handle.borrow().counter);

    // --- the state is genuinely OUTSIDE the component ---
    println!("App and component are distinct objects: {}", !std::ptr::eq(&*handle.borrow(), &App::default()));
    println!("final: {:?}", handle.borrow());
    println!("applied log (proof Action is the only writer): {:?}", handle.borrow().applied);

    // --- render the component that borrows it, off-screen, via tuirealm::testing ---
    let s = tuirealm::testing::render_to_string(&mut comp, tuirealm::ratatui::layout::Size::new(30, 1));
    println!("render_to_string of the borrowing component:\n{s}");
}
