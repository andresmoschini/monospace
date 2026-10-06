//! Q1-negative: can a mounted component hold a BORROW of the app state instead of
//! a shared handle? This file is EXPECTED TO FAIL TO COMPILE. The error is the evidence.
use tuirealm::application::Application;
use tuirealm::event::NoUserEvent;
use tuirealm::listener::EventListenerCfg;

#[path = "../state.rs"]
mod state;
use state::App;

use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::props::{AttrValue, QueryResult, Attribute};
use tuirealm::state::State;

struct Borrowing<'a> {
    app: &'a mut App,
}

impl Component for Borrowing<'_> {
    fn view(&mut self, _: &mut tuirealm::ratatui::Frame, _: tuirealm::ratatui::layout::Rect) {}
    fn query<'a>(&'a self, _: Attribute) -> Option<QueryResult<'a>> {
        None
    }
    fn attr(&mut self, _: Attribute, _: AttrValue) {}
    fn state(&self) -> State {
        State::None
    }
    fn perform(&mut self, _: Cmd) -> CmdResult {
        self.app.counter += 1;
        CmdResult::Changed(State::None)
    }
}

impl AppComponent<(), NoUserEvent> for Borrowing<'_> {
    fn on(&mut self, _: &tuirealm::event::Event<NoUserEvent>) -> Option<()> {
        None
    }
}

fn main() {
    let mut app_state = App::default();
    let comp = Borrowing { app: &mut app_state };
    let mut app: Application<(), (), NoUserEvent> =
        Application::init(EventListenerCfg::default());
    app.mount((), Box::new(comp), vec![]).unwrap();
}
