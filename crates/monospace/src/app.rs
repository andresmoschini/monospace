//! What the application is: a session, the chrome around it, and the one place an action lands.
//!
//! [`docs/application-model.md`](../../../docs/application-model.md) owns the rules; §2 of it is
//! what this file is.
//!
//! # Design notes
//!
//! **Two pieces of state and nothing else.** A [`Session`] holding a diagram, and a [`Chrome`]
//! holding what is on screen. There is no third thing that knows about both: an action that means
//! something to a diagram becomes a `Command` performed on the session, and anything else lands on
//! the chrome. **This slice performs no command at all** — editing a diagram and undo are out of
//! it, and `monospace-editing` is here to say the application sits on a layer rather than to be
//! used yet — so the diagram behind the session is empty and nothing in this file names one.
//!
//! **The session is opened over a `Diagram` that is never written down.** `Session::new` takes
//! one, and naming the type would mean depending on `monospace-diagram`, which rule 17 forbids:
//! this crate reaches `monospace-editing` and two terminal crates and nothing else.
//! `Default::default()` is inferred rather than spelled out, and the diagram it produces is empty
//! because an empty diagram is what the default is. That is the whole of how a crate that holds a
//! diagram and is forbidden to know what a diagram is opens one.
//!
//! **There is no `perform` here, and its absence is the state of this slice rather than a
//! shortcut.** Building a `Command` needs a `ShapeId` and a `Delta`, both of which live in
//! `monospace-diagram`, so a method that took one could not be written without reaching past the
//! crate the specification names. The first slice that performs a command will also be the first
//! that has to say so.

use monospace_editing::Session;

use crate::chrome::{Action, Chrome};

/// The application: what is being edited, and what is drawn around it.
pub(crate) struct App {
    /// The diagram being edited and the steps behind and ahead of it.
    ///
    /// **The only reach into a diagram this crate has**, and it is the editing layer's rather than
    /// the core's: nothing here can change the diagram except by performing a command on it.
    ///
    /// **It is built over a `Default::default()` and not over a `Diagram::new()` written out**, and
    /// the lint that asks for the name is permitted because the name is the whole point: naming it
    /// means depending on `monospace-diagram`, which rule 17 forbids, and the inference is what
    /// lets this crate reach `monospace-editing` and nothing else.
    ///
    /// **Nothing reads it yet, and that is the state of this slice rather than an oversight.**
    /// Rule 8 says the state is a `Session` and the chrome's own, and editing a diagram is out of
    /// scope, so the field is here and the only thing that can reach it is a `Command` — of which
    /// this slice performs none. The next slice to draw into the region reads it, and that is what
    /// the permission below is waiting for. The same shape is named for rule 17: a claim about what
    /// this crate depends on that no step in the gate can reach, written down because `README.md`
    /// promises it and reviewed by reading the manifest.
    #[allow(
        dead_code,
        reason = "rule 8 makes the session part of the state and no slice performs \
        a command on it yet"
    )]
    session: Session,
    /// What is on screen around it.
    chrome: Chrome,
}

impl Default for App {
    fn default() -> Self {
        #[allow(
            clippy::default_trait_access,
            reason = "naming the type would mean depending on `monospace-diagram`, which rule 17 \
                forbids this crate to reach"
        )]
        let empty = Default::default();
        Self {
            // The diagram is empty in this slice and is never named; see the design notes.
            session: Session::new(empty),
            chrome: Chrome::default(),
        }
    }
}

impl App {
    /// Whether the loop should keep going.
    pub(crate) fn running(&self) -> bool {
        self.chrome.running()
    }

    /// The chrome, for the tests to look at.
    pub(crate) fn chrome(&self) -> &Chrome {
        &self.chrome
    }

    /// The chrome, for the draw pass to fill the geometry in.
    pub(crate) fn chrome_mut(&mut self) -> &mut Chrome {
        &mut self.chrome
    }

    /// The session, for the tests that ask whether anything reached it.
    #[cfg(test)]
    pub(crate) fn session(&self) -> &Session {
        &self.session
    }

    /// Carries `action` out, and anything it turns into.
    ///
    /// **This is the whole of rule 8**: an event becomes an action, and the action is the only
    /// thing that reaches the state. The one rewrite is a release turning into the action its
    /// control carries — a click is the way out only once the pointer comes off the control it
    /// went down on — and nothing carries a release, so this cannot go on for ever.
    pub(crate) fn apply(&mut self, action: Action) {
        if let Some(carried) = self.chrome.apply(action) {
            self.apply(carried);
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};

    use super::App;
    use crate::{
        chrome::{Action, ControlId, Move},
        screen,
    };

    /// An application with one frame drawn, which is what gives the controls a rectangle.
    fn drawn() -> App {
        let mut app = App::default();
        let mut terminal = Terminal::new(TestBackend::new(40, 10)).expect("a test backend");
        terminal
            .draw(|frame| screen::draw(frame, app.chrome_mut()))
            .expect("the frame is drawn");
        app
    }

    /// **The claim, and the reason it is asked here and not one module over.**
    ///
    /// The state only ever hears about actions. An event is turned into one by `action_of` — which
    /// is `event.rs`'s claim, with its own tests — and carried out by `apply`, and nothing in
    /// between. So the way out is reached by applying `Action::Quit`, and a control that carries
    /// that action reaches it by whatever became that action: `crate::offer` in `main.rs` is where
    /// the two ends are joined, and its tests are the ones that name four different events arriving
    /// at the same state.
    #[test]
    fn a_key_reaches_the_state_only_as_an_action() {
        let mut app = App::default();
        assert!(app.running(), "a fresh application is not running");

        app.apply(Action::Quit);
        assert!(
            !app.running(),
            "the way out applied and the state did not change: the action is the only thing that \
             reaches it, and nothing else is"
        );
    }

    /// **What this slice can prove about rule 8, and no more.** Nothing in it performs a command,
    /// so the strongest assertion available from a crate that deliberately names no diagram type is
    /// that the session's history never moves: a command performed on a session is always a step,
    /// and a step is always something to give back.
    ///
    /// **Every action the chrome can produce is carried out**, so the claim is over the whole set
    /// rather than over a convenient few.
    #[test]
    fn the_diagram_is_changed_only_by_a_command_performed_on_the_session() {
        let mut app = App::default();
        for action in [
            Action::Quit,
            Action::OpenDropDown,
            Action::CloseDropDown,
            Action::MoveInDropDown(Move::Up),
            Action::MoveInDropDown(Move::Down),
            Action::FocusAlongBar(Move::Up),
            Action::FocusAlongBar(Move::Down),
            Action::FocusTheRegion,
            Action::Activate,
            Action::ShowAbout,
            Action::HideAbout,
            Action::Press(ControlId::File),
            Action::Release(Some(ControlId::File)),
            Action::Release(None),
        ] {
            app.apply(action);
        }

        assert!(
            !app.session().can_undo() && !app.session().can_redo(),
            "something reached the session's history, and nothing in this slice performs a command"
        );
    }

    /// **Every control the chrome can draw has a rectangle a click could find.** A control with no
    /// geometry is one a person can see and cannot press, which is the failure `#189` is about and
    /// the reason rule 11 exists. **Each overlay is opened on its own**, because the chrome never
    /// shows two at once and a frame with both open would leave the one that is not there holding
    /// no rectangle for the wrong reason.
    #[test]
    fn every_control_the_draw_pass_draws_is_reachable_by_a_click() {
        let mut app = drawn();
        for control in [ControlId::File, ControlId::About] {
            assert!(
                !app.chrome().geometry().area_of(control).is_empty(),
                "{control:?} is on the bar with no rectangle a click could find"
            );
        }

        app.apply(Action::OpenDropDown);
        let mut terminal = Terminal::new(TestBackend::new(40, 10)).expect("a test backend");
        terminal
            .draw(|frame| screen::draw(frame, app.chrome_mut()))
            .expect("the frame is drawn");
        assert!(
            !app.chrome()
                .geometry()
                .area_of(ControlId::Menu(0))
                .is_empty(),
            "the drop-down's entry is drawn with no rectangle a click could find"
        );

        app.apply(Action::ShowAbout);
        terminal
            .draw(|frame| screen::draw(frame, app.chrome_mut()))
            .expect("the frame is drawn");
        assert!(
            !app.chrome().geometry().area_of(ControlId::Close).is_empty(),
            "the pop-up's button is drawn with no rectangle a click could find"
        );
    }
}
