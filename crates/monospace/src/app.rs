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
    use crossterm::event::{
        Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };
    use ratatui::{Terminal, backend::TestBackend, layout::Rect};

    use super::App;
    use crate::offer;
    use crate::{
        chrome::{Action, ControlId, Focus},
        screen,
    };

    fn chord(modifiers: KeyModifiers, code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    /// The two halves of a left click at `(x, y)`.
    fn clicked_at(x: u16, y: u16) -> [Event; 2] {
        let at = |kind| {
            Event::Mouse(MouseEvent {
                kind,
                column: x,
                row: y,
                modifiers: KeyModifiers::NONE,
            })
        };
        [
            at(MouseEventKind::Down(MouseButton::Left)),
            at(MouseEventKind::Up(MouseButton::Left)),
        ]
    }

    /// An application with one control drawn at `area`, which is what a click needs to land on.
    fn app_with(control: ControlId, area: Rect) -> App {
        let mut app = App::default();
        let mut terminal = Terminal::new(TestBackend::new(40, 10)).expect("a test backend");
        terminal
            .draw(|frame| screen::draw(frame, app.chrome_mut()))
            .expect("the frame is drawn");
        app.chrome_mut().geometry_mut_for_test().set(control, area);
        app
    }

    #[test]
    fn a_key_reaches_the_state_only_as_an_action() {
        let mut app = App::default();

        // The event on its own is worth nothing: `action_of` is the only thing that reads one, and
        // what it answers with is the only thing the state ever sees.
        let event = chord(KeyModifiers::CONTROL, KeyCode::Char('q'));
        assert!(app.running());

        offer(&mut app, &event);
        assert!(
            !app.running(),
            "the key did not reach the state even as an action"
        );
    }

    /// **What this slice can prove about rule 8, and no more.** Nothing in it performs a command,
    /// so the strongest assertion available from a crate that deliberately names no diagram type is
    /// that the session's history never moves: a command performed on a session is always a step,
    /// and a step is always something to give back.
    #[test]
    fn the_diagram_is_changed_only_by_a_command_performed_on_the_session() {
        let mut app = App::default();
        for event in [
            chord(KeyModifiers::NONE, KeyCode::F(9)),
            chord(KeyModifiers::NONE, KeyCode::Enter),
            chord(KeyModifiers::NONE, KeyCode::Esc),
            chord(KeyModifiers::NONE, KeyCode::Down),
            chord(KeyModifiers::NONE, KeyCode::Tab),
        ] {
            offer(&mut app, &event);
        }
        app.apply(Action::ShowAbout);
        app.apply(Action::HideAbout);
        app.apply(Action::Press(ControlId::File));
        app.apply(Action::Release(Some(ControlId::File)));

        assert!(
            !app.session().can_undo() && !app.session().can_redo(),
            "something reached the session's history, and nothing in this slice performs a command"
        );
    }

    #[test]
    fn the_way_out_is_one_action_reached_by_a_key_a_click_and_the_focused_control() {
        // `Ctrl+Q` and `Ctrl+C` are keys of the application, wherever the keyboard is.
        let by_key = {
            let mut app = App::default();
            offer(&mut app, &chord(KeyModifiers::CONTROL, KeyCode::Char('q')));
            app.running()
        };
        let by_control_c = {
            let mut app = App::default();
            offer(&mut app, &chord(KeyModifiers::CONTROL, KeyCode::Char('c')));
            app.running()
        };
        // A click names a control by the rectangle the draw pass stored, so the rectangle is what
        // the draw pass would have written.
        let by_click = {
            let mut app = app_with(ControlId::File, Rect::new(2, 1, 6, 1));
            for event in clicked_at(3, 1) {
                offer(&mut app, &event);
            }
            offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::Enter));
            app.running()
        };
        // The way-out control is inside the drop-down, and its own key is `Enter` while it is
        // selected.
        let by_its_own_key = {
            let mut app = app_with(ControlId::Menu(0), Rect::new(1, 2, 6, 1));
            offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::F(9)));
            offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::Enter));
            app.running()
        };

        assert!(!by_key, "Ctrl+Q did not reach the way out");
        assert!(!by_control_c, "Ctrl+C did not reach the way out");
        assert!(
            !by_click,
            "a click on the control did not reach the way out"
        );
        assert!(
            !by_its_own_key,
            "the focused way-out control's own key did not reach the way out"
        );
    }

    #[test]
    fn a_bare_q_names_no_action() {
        let mut app = App::default();
        offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::Char('q')));
        assert!(app.running(), "a bare q ended the loop");
        assert_eq!(app.chrome().focus(), Focus::Region);
        assert!(!app.chrome().drop_down_is_open());
    }

    #[test]
    fn a_click_on_the_control_and_its_key_apply_the_same_action() {
        let mut by_click = app_with(ControlId::File, Rect::new(2, 1, 6, 1));
        for event in clicked_at(3, 1) {
            offer(&mut by_click, &event);
        }

        // The keyboard reaches the same control by the stack rather than by a key: `F9` opens the
        // drop-down and `Esc` closes it and leaves the keyboard on `File`. That is the path the
        // resting focus stack allows, and rule 9 says the two must apply the same action.
        let mut by_key = App::default();
        offer(&mut by_key, &chord(KeyModifiers::NONE, KeyCode::F(9)));
        offer(&mut by_key, &chord(KeyModifiers::NONE, KeyCode::Esc));
        assert_eq!(
            by_key.chrome().focus(),
            Focus::Bar(ControlId::File),
            "`Esc` did not leave the keyboard on the control that opened the drop-down"
        );
        offer(&mut by_key, &chord(KeyModifiers::NONE, KeyCode::Enter));

        assert_eq!(
            by_click.chrome().drop_down_is_open(),
            by_key.chrome().drop_down_is_open(),
            "the click and the key reached the same control by different paths"
        );
        assert_eq!(by_click.chrome().focus(), by_key.chrome().focus());
    }

    #[test]
    fn a_click_outside_every_rectangle_names_no_action() {
        let mut app = app_with(ControlId::File, Rect::new(2, 1, 6, 1));
        for event in clicked_at(30, 8) {
            offer(&mut app, &event);
        }
        assert!(app.running());
        assert_eq!(app.chrome().focus(), Focus::Region);
        assert!(!app.chrome().drop_down_is_open());
    }

    #[test]
    fn f9_opens_the_drop_down_and_escape_closes_it() {
        let mut app = App::default();
        assert!(!app.chrome().drop_down_is_open());

        offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::F(9)));
        assert!(app.chrome().drop_down_is_open());

        offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::Esc));
        assert!(
            !app.chrome().drop_down_is_open(),
            "Esc did not close the drop-down that F9 opened"
        );
    }

    /// **`F9` is the application's own key, and this is the order rule 14 states.** The pop-up is
    /// offered every event first, so the keys it takes are its own and nothing reaches past it to
    /// the diagram — and `F9` is not one of the pop-up's, so it falls through and opens the
    /// drop-down. A reading in which the pop-up went deaf to the application's own keys would be a
    /// different rule, and the test is written against the one the specification states.
    #[test]
    fn the_diagrams_keys_do_nothing_while_a_pop_up_is_open() {
        let mut app = App::default();
        app.apply(Action::ShowAbout);
        assert_eq!(app.chrome().focus(), Focus::PopUp);

        // `Esc` belonged to nobody before the pop-up opened and belongs to it now.
        assert!(app.chrome().pop_up_is_open());
        offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::Esc));
        assert!(
            !app.chrome().pop_up_is_open(),
            "the key that cancels the pop-up reached the application instead"
        );

        // And the pop-up takes the control the keyboard was on before it opened, so `Enter` acts on
        // the pop-up's button and not on the bar's.
        app.apply(Action::ShowAbout);
        assert_eq!(app.chrome().focus(), Focus::PopUp);
        offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::Enter));
        assert!(
            !app.chrome().drop_down_is_open(),
            "`Enter` reached the bar's control while the pop-up held the keyboard"
        );
    }

    #[test]
    fn a_key_that_names_no_action_changes_nothing_and_reports_nothing() {
        let mut app = App::default();
        let before = app.chrome().focus();

        offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::Char('z')));
        offer(&mut app, &chord(KeyModifiers::NONE, KeyCode::Home));
        offer(&mut app, &chord(KeyModifiers::SHIFT, KeyCode::Char('A')));

        assert!(app.running());
        assert_eq!(app.chrome().focus(), before);
        assert!(!app.chrome().drop_down_is_open());
        assert!(!app.chrome().pop_up_is_open());
    }

    #[test]
    fn a_click_that_names_no_action_changes_nothing_and_reports_nothing() {
        let mut app = app_with(ControlId::File, Rect::new(2, 1, 6, 1));

        for event in clicked_at(30, 8) {
            offer(&mut app, &event);
        }
        assert!(app.running());
        assert_eq!(app.chrome().focus(), Focus::Region);
    }

    /// **Every control the chrome can draw has a rectangle a click can find.** A control with no
    /// geometry would be one a person can see and cannot press, which is the failure `#189` is
    /// about and the reason rule 11 exists.
    #[test]
    fn every_control_the_draw_pass_draws_is_reachable_by_a_click() {
        let mut app = App::default();
        draw(&mut app);
        for control in [ControlId::File, ControlId::About] {
            assert!(
                !app.chrome().geometry().area_of(control).is_empty(),
                "{control:?} is on the bar with no rectangle a click could find"
            );
        }

        app.apply(Action::OpenDropDown);
        draw(&mut app);
        assert!(
            !app.chrome()
                .geometry()
                .area_of(ControlId::Menu(0))
                .is_empty(),
            "the drop-down's entry is drawn with no rectangle a click could find"
        );

        app.apply(Action::ShowAbout);
        draw(&mut app);
        assert!(
            !app.chrome().geometry().area_of(ControlId::Close).is_empty(),
            "the pop-up's button is drawn with no rectangle a click could find"
        );
    }

    /// One frame drawn, which is what fills the geometry in.
    fn draw(app: &mut App) {
        let mut terminal = Terminal::new(TestBackend::new(40, 10)).expect("a test backend");
        terminal
            .draw(|frame| screen::draw(frame, app.chrome_mut()))
            .expect("the frame is drawn");
    }
}
