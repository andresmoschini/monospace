//! What the chrome is: the controls on it, the three states a control can be in, and where the
//! keyboard is.
//!
//! [`docs/application-model.md`](../../../docs/application-model.md) is the design this file
//! implements, and [`specs/174-a-full-screen-application-whose-menu-hol.md`](../../../specs/174-a-full-screen-application-whose-menu-hol.md)
//! is the specification it was built against. Nothing here knows about a diagram: the chrome holds
//! no `Diagram` and performs no `Command`, and a `Session` reaches this crate from the other side
//! of [`crate::app`].
//!
//! # Design notes
//!
//! **Focus is one place, and a control's state is derived from it rather than stored beside it.**
//! Two things that both answer "where is the keyboard" have to be kept in step, and the one that
//! falls out of step is the one that draws the wrong thing. So [`Chrome`] holds a [`Focus`] and
//! nothing else about focus, and [`Chrome::state_of`] reads it. The one field that is stored
//! separately is `pressed`, because a pointer going down on a control and a keyboard reaching it
//! are two different questions and only one of them is about focus.
//!
//! **A control under the mouse is not a state.** There are three — at rest, focused and pressed —
//! and moving the pointer over a control produces no action at all, so there is nothing on screen
//! that follows the pointer. The cost is stated in D6: the chrome cannot invite a mouse user by
//! changing under the cursor, and what reopens the question is the first control whose only
//! sensible gesture is pointing at it.
//!
//! **A control is a rectangle the draw pass stored, and the click handler reads it back.** The
//! arithmetic that decides where a rectangle goes is in [`crate::screen`]; what happens here is
//! only that the number was kept. That is D3, and the alternative — coordinate ranges written
//! beside the key handler — survives a test and fails a resize, because the range does not follow
//! the layout and nothing reports that it stopped.

use ratatui::layout::{Position, Rect};

/// One thing on screen that a key or a click can reach.
///
/// **An enum rather than a label**, because a control is named in three places that have to agree —
/// the draw pass, the hit test and the action it carries — and a string in three of them is three
/// chances to misspell it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ControlId {
    /// The bar's menu, which holds the way out.
    File,
    /// The bar's control that opens the pop-up.
    About,
    /// One entry of [`MENU`], identified by where it is in it.
    ///
    /// **An index and not a variant per item**, so that the command `Load` arrives with — which is
    /// the next thing the menu is for — is one line in [`MENU`] and nothing else here.
    Menu(usize),
    /// The pop-up's button, which closes it.
    Close,
}

/// The states a control is drawn in, and the whole of what the chrome can say about one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ControlState {
    /// Neither the keyboard nor the pointer has anything to do with it.
    AtRest,
    /// The keyboard is on it, and it is drawn in reverse video.
    Focused,
    /// A pointer is down on it, and it is drawn in reverse video too.
    ///
    /// **The same cue as [`Focused`](Self::Focused), deliberately.** D6 asks for one mechanism with
    /// two ways in rather than two mechanisms competing, and the two never need telling apart:
    /// either one means the control is the one about to answer.
    Pressed,
}

/// Which way a selection moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Move {
    /// Toward the first entry.
    Up,
    /// Toward the last entry.
    Down,
}

/// The label a control is drawn with.
///
/// **Nothing rather than a panic for a control that is on no list.** A `Menu` index past the end of
/// [`MENU`] is not reachable from the state — the selection is clamped and the geometry is written
/// from the same list — so a label of no columns is a way of not inventing a failure mode to
/// handle a case that cannot arrive.
///
/// **A function of the lists rather than a method**, because the chrome's own state has no say in
/// what a control is called: the draw pass, the hit test and the action a control carries all read
/// the one answer, and a method would let one of them reach past it.
pub(crate) fn label_of(control: ControlId) -> &'static str {
    match control {
        ControlId::File | ControlId::About => BAR
            .iter()
            .find(|(held, _)| *held == control)
            .map_or("", |(_, label)| *label),
        ControlId::Menu(index) => MENU.get(index).map_or("", |(label, _)| *label),
        ControlId::Close => CLOSE,
    }
}

/// The action a control carries, or `None` where the control is on no list.
///
/// **A function of the lists rather than a method**, for the reason [`label_of`] gives.
pub(crate) fn action_of(control: ControlId) -> Option<Action> {
    match control {
        ControlId::File => Some(Action::OpenDropDown),
        ControlId::About => Some(Action::ShowAbout),
        ControlId::Menu(index) => MENU.get(index).map(|(_, action)| *action),
        ControlId::Close => Some(Action::HideAbout),
    }
}

/// The bar's controls in the order they are laid out, with the label each is drawn with.
///
/// **The order lives in [`BAR`] and the wording in [`label_of`], and the draw pass reads both from
/// here rather than either on its own** — which is what keeps a control's label from being written
/// down a second time somewhere the hit test cannot see.
pub(crate) fn bar() -> impl Iterator<Item = (ControlId, &'static str)> {
    BAR.iter()
        .map(|(control, _)| (*control, label_of(*control)))
}

/// The drop-down's entries in the order they are listed, with the label each is drawn with.
pub(crate) fn menu() -> impl Iterator<Item = (usize, &'static str)> {
    (0..MENU.len()).map(|index| (index, label_of(ControlId::Menu(index))))
}

/// Where the keyboard is.
///
/// **A place and not a mode.** A mode answers "which of these handlers runs", and a focus answers
/// the same question without a second thing to keep in step; a mode earns its place only when keys
/// mean different *verbs* rather than going to different places, and this application has no such
/// pair. See D4 in the specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    /// On a control in the bar.
    Bar(ControlId),
    /// In the drop-down, which is drawn over the region and holds the keyboard while it is open.
    DropDown,
    /// In the pop-up, which is drawn over the region and holds the keyboard while it is open.
    PopUp,
}

/// Everything an event can ask the application to do.
///
/// **An action is the only thing that changes the state**, so this enum and the `match` in
/// [`Chrome::apply`] are together the whole of what the application can do. A key does not reach
/// the state and a click does not either: both become one of these first.
/// **It is written in [`crate::event`] and re-exported here**, because it is what an event becomes
/// and this is where an action lands, and the two halves are one thing said from either side.
pub(crate) use crate::event::Action;

/// The bar's controls, in the order they are laid out across the top of the screen.
///
/// **Named here rather than written into the draw pass**, so that the layout, the hit test and the
/// action a control carries are all reading one list.
pub(crate) const BAR: &[(ControlId, &str)] =
    &[(ControlId::File, "File"), (ControlId::About, "About")];

/// What the drop-down holds, and what each entry does.
///
/// **One entry today, and the list is a list rather than a single `Option` for that reason.** The
/// next commands this application gains go in here as well as onto a key, which is what D7 asks
/// for, and the cost of that promise being kept is one line per command.
pub(crate) const MENU: &[(&str, Action)] = &[("Quit", Action::Quit)];

/// The pop-up's button, and the only thing in it that is a control.
pub(crate) const CLOSE: &str = "Close";

/// Where each control was drawn on the last frame.
///
/// **Stored by the draw pass and read back by the click handler**, which is the whole of D3, and it
/// is why a control answers a click after the terminal is resized: the frame is laid out again at
/// the new size and the number stored is the new one.
///
/// **A control the frame did not draw holds an empty rectangle**, so the hit test finds it
/// nowhere without anything having to say which controls were on screen. The geometry is emptied
/// at the top of every frame rather than overwritten field by field, so a control that stops being
/// drawn cannot keep answering from where it used to be.
#[derive(Debug, Clone, Default)]
pub(crate) struct Geometry {
    /// The bar's controls, indexed the way [`BAR`] is.
    bar: [Rect; BAR.len()],
    /// The drop-down's entries, indexed the way [`MENU`] is.
    menu: [Rect; MENU.len()],
    /// The pop-up's button.
    close: Rect,
}

impl Geometry {
    /// The rectangle `control` was drawn into.
    pub(crate) fn area_of(&self, control: ControlId) -> Rect {
        match control {
            ControlId::File => self.bar[0],
            ControlId::About => self.bar[1],
            ControlId::Menu(index) => self.menu[index],
            ControlId::Close => self.close,
        }
    }

    /// Records where `control` was drawn.
    pub(crate) fn set(&mut self, control: ControlId, area: Rect) {
        match control {
            ControlId::File => self.bar[0] = area,
            ControlId::About => self.bar[1] = area,
            ControlId::Menu(index) => self.menu[index] = area,
            ControlId::Close => self.close = area,
        }
    }

    /// Every control, in the order [`BAR`] is followed by the drop-down's and the pop-up's.
    pub(crate) fn controls() -> impl Iterator<Item = ControlId> {
        BAR.iter()
            .map(|(control, _)| *control)
            .chain((0..MENU.len()).map(ControlId::Menu))
            .chain(std::iter::once(ControlId::Close))
    }
}

/// The chrome's own state: what is open, where the keyboard is, and where everything was drawn.
#[derive(Debug, Clone)]
pub(crate) struct Chrome {
    /// Where each control was drawn on the last frame.
    geometry: Geometry,
    /// Where the keyboard is.
    focus: Focus,
    /// The control a pointer is down on, if one is.
    pressed: Option<ControlId>,
    /// Whether the drop-down is open.
    drop_down: bool,
    /// Which entry of [`MENU`] the drop-down has selected.
    selected: usize,
    /// Whether the pop-up is open.
    pop_up: bool,
    /// Whether the way out has been taken, which is the only thing that ends the loop.
    quit: bool,
}

impl Default for Chrome {
    /// **The keyboard starts on the bar's first control.** Rule 12 is about the region holding the
    /// keyboard, and an application whose only visible thing at rest is a bar is one whose region
    /// holds nothing yet — so the first thing a person can do is press `Enter`, and the second is
    /// `Right`.
    fn default() -> Self {
        Self {
            geometry: Geometry::default(),
            focus: Focus::Bar(ControlId::File),
            pressed: None,
            drop_down: false,
            selected: 0,
            pop_up: false,
            quit: false,
        }
    }
}

impl Chrome {
    /// Where each control was drawn on the last frame, for the draw pass to fill in and the click
    /// handler to read back.
    pub(crate) fn geometry(&self) -> &Geometry {
        &self.geometry
    }

    /// Forgets where everything was drawn, so that a control this frame does not draw answers no
    /// click at all.
    pub(crate) fn forget_geometry(&mut self) {
        self.geometry = Geometry::default();
    }

    /// The geometry, so that a test can stand in for the draw pass.
    ///
    /// **Under `cfg(test)` and not public**, because the production reach is the one direction:
    /// [`crate::screen`] fills the geometry in through [`Self::geometry_of_mut`], and nothing
    /// outside this crate's tests has a reason to place a control by hand.
    #[cfg(test)]
    pub(crate) fn geometry_mut_for_test(&mut self) -> &mut Geometry {
        &mut self.geometry
    }

    /// Where each control was drawn, for the draw pass to fill in.
    pub(crate) fn geometry_of_mut(&mut self) -> &mut Geometry {
        &mut self.geometry
    }

    /// Whether the loop should keep going.
    pub(crate) fn running(&self) -> bool {
        !self.quit
    }

    /// Whether the drop-down is open.
    pub(crate) fn drop_down_is_open(&self) -> bool {
        self.drop_down
    }

    /// Whether the pop-up is open.
    pub(crate) fn pop_up_is_open(&self) -> bool {
        self.pop_up
    }

    /// Where the keyboard is.
    pub(crate) fn focus(&self) -> Focus {
        self.focus
    }

    /// Whether the region rather than the bar holds the keyboard.
    ///
    /// **This is what rule 12 reads**: the region holding the keyboard is drawn with a different
    /// border from the others, and while the drop-down or the pop-up is open the region is what
    /// holds it, because both are drawn inside the region.
    pub(crate) fn region_holds_the_keyboard(&self) -> bool {
        !matches!(self.focus, Focus::Bar(_))
    }

    /// The state `control` is drawn in.
    pub(crate) fn state_of(&self, control: ControlId) -> ControlState {
        if self.pressed == Some(control) {
            ControlState::Pressed
        } else if self.focused() == control {
            ControlState::Focused
        } else {
            ControlState::AtRest
        }
    }

    /// The control the keyboard is on.
    ///
    /// **Never nothing**, which is what lets `Activate` be one action rather than one per place:
    /// every place the keyboard can be in has a control under it, and the tables that name what
    /// each control does are the only place a control's meaning is written down.
    pub(crate) fn focused(&self) -> ControlId {
        match self.focus {
            Focus::Bar(control) => control,
            Focus::DropDown => ControlId::Menu(self.selected),
            Focus::PopUp => ControlId::Close,
        }
    }

    /// The control drawn at `position`, or `None` where the click fell on none of them.
    ///
    /// **Tested against the stored rectangles and nothing else**, so it costs the same after a
    /// resize as before it.
    pub(crate) fn control_at(&self, position: Position) -> Option<ControlId> {
        Geometry::controls().find(|control| self.geometry.area_of(*control).contains(position))
    }

    /// Carries `action` out, and answers with the action it turned into where it turned into one.
    ///
    /// **A release is the only action that is rewritten, and it is rewritten once.** A click is the
    /// way out only after the pointer comes off the control it went down on, and the action it
    /// turns into is the one that control carries. Nothing carries a release, so nothing recurses.
    pub(crate) fn apply(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Quit => {
                self.quit = true;
                None
            }
            Action::OpenDropDown => {
                self.drop_down = true;
                self.selected = 0;
                self.pop_up = false;
                self.focus = Focus::DropDown;
                None
            }
            Action::CloseDropDown => {
                self.drop_down = false;
                self.focus = Focus::Bar(ControlId::File);
                None
            }
            Action::MoveInDropDown(direction) => {
                self.selected = moved(self.selected, direction, MENU.len());
                None
            }
            Action::FocusAlongBar(direction) => {
                let index = BAR
                    .iter()
                    .position(|(control, _)| *control == self.focused())
                    .unwrap_or_default();
                let next = moved(index, direction, BAR.len());
                self.focus = Focus::Bar(BAR[next].0);
                None
            }
            Action::Activate => {
                let focused = self.focused();
                self.pressed = None;
                action_of(focused)
            }
            Action::ShowAbout => {
                self.pop_up = true;
                self.drop_down = false;
                self.focus = Focus::PopUp;
                None
            }
            Action::HideAbout => {
                self.pop_up = false;
                self.focus = Focus::Bar(ControlId::About);
                None
            }
            Action::Press(control) => {
                self.pressed = Some(control);
                None
            }
            Action::Release(released) => {
                let pressed = self.pressed.take();
                if released.is_some_and(|control| Some(control) == pressed) {
                    released.and_then(action_of)
                } else {
                    None
                }
            }
        }
    }
}

/// Where `selected` lands when it moves by `direction` in a list of `len`, stopping at either end.
fn moved(selected: usize, direction: Move, len: usize) -> usize {
    match direction {
        Move::Up => selected.saturating_sub(1),
        Move::Down => (selected + 1).min(len.saturating_sub(1)),
    }
}

#[cfg(test)]
mod tests {
    use ratatui::layout::{Position, Rect};

    use super::{
        Action, BAR, Chrome, ControlId, ControlState, Focus, MENU, Move, action_of, label_of, moved,
    };

    /// A chrome whose geometry says where every control is, for the tests about clicking.
    fn chrome_with_controls_drawn_at(rectangle: Rect) -> Chrome {
        let mut chrome = Chrome::default();
        for control in super::Geometry::controls() {
            chrome.geometry_mut_for_test().set(control, rectangle);
        }
        chrome
    }

    #[test]
    fn moved_stops_at_either_end_rather_than_wrapping() {
        assert_eq!(moved(0, Move::Up, 3), 0);
        assert_eq!(moved(2, Move::Down, 3), 2);
        assert_eq!(moved(0, Move::Down, 3), 1);
        assert_eq!(moved(2, Move::Up, 3), 1);
    }

    #[test]
    fn a_control_the_frame_did_not_draw_answers_no_click() {
        let chrome = Chrome::default();
        assert!(
            chrome.control_at(Position { x: 1, y: 0 }).is_none(),
            "a control whose geometry was never filled in was hit at a rectangle that contains \
             nothing"
        );
    }

    #[test]
    fn a_click_inside_a_controls_rectangle_names_its_action() {
        let chrome = chrome_with_controls_drawn_at(Rect::new(4, 2, 6, 1));

        let hit = chrome
            .control_at(Position { x: 7, y: 2 })
            .expect("the click landed inside the control");
        assert_eq!(hit, ControlId::File);
        assert_eq!(
            action_of(hit),
            Some(Action::OpenDropDown),
            "the control that was hit does not carry the action it is drawn for"
        );
    }

    #[test]
    fn a_click_outside_every_rectangle_names_no_action() {
        let chrome = chrome_with_controls_drawn_at(Rect::new(4, 2, 6, 1));

        assert_eq!(chrome.control_at(Position { x: 10, y: 2 }), None);
        assert_eq!(chrome.control_at(Position { x: 7, y: 9 }), None);
    }

    /// **The one the spike demonstrated by failing.** The rectangle is stored by the draw pass and
    /// laid out again at the new size, so the click that used to land on the control lands on it
    /// again — where a range written beside the key handler would still be pointing at the old
    /// position and would report nothing when it stopped answering.
    ///
    /// **Only the one control is moved**, because the point is where one control is and not where
    /// the others are: every control drawn at the same place would answer at the old place too, and
    /// the test would pass for a hit test that answers everywhere.
    #[test]
    fn a_control_hit_tested_against_its_rectangle_answers_after_a_resize() {
        let mut chrome = Chrome::default();
        chrome
            .geometry_mut_for_test()
            .set(ControlId::File, Rect::new(2, 0, 6, 1));
        assert!(chrome.control_at(Position { x: 3, y: 0 }).is_some());

        // The frame is laid out again at the new size, and the number stored is the new one.
        chrome
            .geometry_mut_for_test()
            .set(ControlId::File, Rect::new(20, 0, 6, 1));
        assert_eq!(
            chrome.control_at(Position { x: 3, y: 0 }),
            None,
            "the control still answers at the position it had before the resize"
        );
        assert!(chrome.control_at(Position { x: 22, y: 0 }).is_some());
    }

    #[test]
    fn the_bars_row_holds_the_keyboard_until_something_inside_the_region_does() {
        let mut chrome = Chrome::default();
        assert!(!chrome.region_holds_the_keyboard());
        assert_eq!(chrome.state_of(ControlId::File), ControlState::Focused);
        assert_eq!(chrome.state_of(ControlId::About), ControlState::AtRest);

        chrome.apply(Action::OpenDropDown);
        assert!(chrome.region_holds_the_keyboard());
        assert_eq!(chrome.state_of(ControlId::Menu(0)), ControlState::Focused);
        assert_eq!(chrome.state_of(ControlId::File), ControlState::AtRest);
    }

    #[test]
    fn opening_the_drop_down_closes_the_pop_up_and_the_other_way_round() {
        let mut chrome = Chrome::default();

        chrome.apply(Action::ShowAbout);
        assert!(chrome.pop_up_is_open());
        assert!(!chrome.drop_down_is_open());

        chrome.apply(Action::OpenDropDown);
        assert!(chrome.drop_down_is_open());
        assert!(!chrome.pop_up_is_open());
        assert_eq!(chrome.focus(), Focus::DropDown);
    }

    #[test]
    fn the_way_out_is_reached_by_the_click_and_by_the_controls_own_key_the_same_way() {
        let mut by_click = Chrome::default();
        by_click.apply(Action::OpenDropDown);
        by_click.apply(Action::Press(ControlId::Menu(0)));
        let by_click = by_click.apply(Action::Release(Some(ControlId::Menu(0))));

        let mut by_key = Chrome::default();
        by_key.apply(Action::OpenDropDown);
        let by_key = by_key.apply(Action::Activate);

        assert_eq!(by_click, Some(Action::Quit));
        assert_eq!(
            by_key, by_click,
            "the click and the key reached the way out by different paths"
        );
    }

    #[test]
    fn a_release_off_the_control_that_was_pressed_does_nothing() {
        let mut chrome = Chrome::default();
        chrome.apply(Action::OpenDropDown);
        chrome.apply(Action::Press(ControlId::Menu(0)));

        assert_eq!(
            chrome.apply(Action::Release(None)),
            None,
            "the pointer came up over nothing and the control was activated anyway"
        );
    }

    #[test]
    fn a_pointer_that_goes_down_on_one_control_and_comes_up_on_another_activates_neither() {
        let mut chrome = Chrome::default();
        chrome.apply(Action::Press(ControlId::File));
        assert_eq!(
            chrome.apply(Action::Release(Some(ControlId::About))),
            None,
            "a click that started on one control and ended on another activated the second"
        );
    }

    #[test]
    fn a_pressed_control_is_released_when_the_pointer_comes_up_anywhere() {
        let mut chrome = Chrome::default();
        chrome.apply(Action::Press(ControlId::File));
        assert_eq!(chrome.state_of(ControlId::File), ControlState::Pressed);

        chrome.apply(Action::Release(None));
        assert_eq!(
            chrome.state_of(ControlId::File),
            ControlState::Focused,
            "the control is still drawn as pressed after the pointer came up"
        );
    }

    #[test]
    fn every_control_in_the_lists_carries_an_action_and_a_label() {
        for control in super::Geometry::controls() {
            assert!(
                !label_of(control).is_empty(),
                "{control:?} is drawn on no list"
            );
            assert!(
                action_of(control).is_some(),
                "{control:?} is on a list and does nothing"
            );
        }
        assert_eq!(
            BAR.len(),
            2,
            "the bar's controls moved and a test below names them by position"
        );
        assert_eq!(
            MENU.len(),
            1,
            "the drop-down's entries moved and a test below names the one by position"
        );
    }
}
