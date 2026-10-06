//! The application as the user sees it: what it holds, what a key does to it, and what it draws.
//!
//! **The state is a struct and the action is an enum over it, and there is no third way into
//! either.** An event becomes an action, and the action is the only thing that changes the state —
//! which is what makes undo a case to add rather than a refactor to survive. The alternative was
//! writing the loop against the terminal and pulling the seam out when undo arrives, and that
//! extraction is the one change whose shape is decided by whatever undo turns out to need.

use std::io;

use crossterm::event::{Event, KeyEventKind};

use crate::screen::{GivenBack, Screen};

/// The side of the frame, on every row that is not a border.
const SIDE: char = '│';

/// What a border is drawn with, between its two corners.
const ACROSS: char = '─';

/// What the application performs.
///
/// **One action, and it is the way out.** An enum rather than a bare `bool` because the second
/// action is not a variant of "quit" — it is a different thing to perform, and a slice that has to
/// turn `apply` into something else is the cost of not having this.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    /// The way out: applying it ends the application.
    Quit,
}

/// The application: a menu, and whether it is still up.
///
/// **Nothing is held here that the screen does not draw**, and nothing is held here that an action
/// does not change. Both are one line today and neither is a rule: the second is what keeps the
/// state honest as fields arrive, and the first is what keeps the drawing honest about the state.
struct App {
    /// What the application offers, which is the whole of what it draws.
    menu: Menu,
    /// Whether the application is still up.
    running: bool,
}

/// What the application offers: a name and the leaves under it.
///
/// **Held as it is drawn rather than beside it**, because a menu whose leaves lived anywhere but the
/// state would be a menu whose leaves could not be undone.
struct Menu {
    /// The name above them.
    title: &'static str,
    /// What there is to choose from, in the order it is offered.
    leaves: Vec<Leaf>,
}

/// One thing the menu offers and what choosing it does.
struct Leaf {
    /// What it is called on the screen.
    label: &'static str,
    /// What choosing it does.
    action: Action,
}

impl App {
    /// The application as it opens: a menu whose only leaf is the way out, and nothing chosen.
    fn new() -> Self {
        Self {
            menu: Menu {
                title: "monospace",
                leaves: vec![Leaf {
                    label: "Quit",
                    action: Action::Quit,
                }],
            },
            running: true,
        }
    }

    /// Applies `action` to the state and answers whether the application is still up.
    ///
    /// **The only thing that writes the state.** That is the whole of rule _a key reaches the state
    /// only as an action_: it is a fact about this function rather than a convention a reader has to
    /// hold in their head, and the answer is a second question rather than a field the loop reads,
    /// so the way out cannot be ended twice or skipped.
    fn apply(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => self.running = false,
        }
        self.running
    }

    /// The action an event names, or `None` where it names none.
    ///
    /// **The only route from an event to the state**, so an event that names no action reaches the
    /// state as nothing at all — which is why the loop below skips the drawing rather than drawing
    /// the same screen again: there would be nothing new to report.
    ///
    /// **A key names a leaf rather than the action itself.** The user is looking at a menu, so what
    /// they press is what they can read on it: `q` chooses the leaf called `Quit`, because that is
    /// what the name begins with — in either case, since a menu is written in capitals and a key is
    /// not. The alternative — a key naming the action and the leaf being a label beside it — is a
    /// menu whose leaves say nothing about what choosing them does, and a second leaf would arrive
    /// with a second convention beside it rather than with the one already there.
    ///
    /// **Only a key going down.** A key coming up and a key repeating name no leaf, and a platform
    /// that reports either would otherwise end the application on the way out of the key that ended
    /// it. A key that is not a character names none either, which is where the arrow keys are until
    /// the menu has something to move between.
    fn action_for(&self, event: &Event) -> Option<Action> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let pressed = key.code.as_char()?;
                self.menu
                    .leaves
                    .iter()
                    .find(|leaf| {
                        leaf.label
                            .chars()
                            .next()
                            .is_some_and(|first| first.eq_ignore_ascii_case(&pressed))
                    })
                    .map(|leaf| leaf.action)
            }
            _ => None,
        }
    }

    /// Draws the whole screen, the size the terminal reports.
    fn draw(&self, screen: &mut impl Screen) -> io::Result<()> {
        let (width, height) = screen.size()?;
        screen.draw(&self.drawn(width, height))
    }

    /// The whole screen as the application draws it, `width` by `height`.
    ///
    /// **Drawn rather than diffed**, which is the cost the specification's D6 names and the smallest
    /// thing that can be right: a scroll of an ASCII canvas changes nearly the whole screen, which is
    /// what a double buffer is for. **The frame is drawn here rather than asked of the core**, because
    /// this crate holds no domain logic and reaches no other crate in the workspace — the screen is
    /// text, and a menu is not a diagram.
    fn drawn(&self, width: u16, height: u16) -> String {
        let width = usize::from(width);
        let mut rows = vec![framed(('┌', '┐'), ACROSS, width), a_blank_row(width)];
        rows.push(centred(self.menu.title, width));
        rows.push(a_blank_row(width));
        for (index, leaf) in self.menu.leaves.iter().enumerate() {
            // Every other row, so a list of leaves reads as a list rather than as a block of text.
            if index % 2 == 1 {
                rows.push(a_blank_row(width));
            }
            rows.push(centred(leaf.label, width));
        }
        // **A blank row below the last leaf as well as above the first**, so the block is framed by
        // the same space on both sides and adding a leaf does not move the frame.
        rows.push(a_blank_row(width));
        rows.push(framed(('└', '┘'), ACROSS, width));

        // **The screen is the height the terminal reports, and no taller than what was drawn.** A
        // terminal narrower than the frame or shorter than the menu gets the rows that fit and no
        // panic, which is the whole of what a terminal can be asked for and the reason the width is
        // saturating rather than checked.
        let mut screen = String::new();
        for row in rows.iter().take(usize::from(height)) {
            screen.push_str(row);
            screen.push('\n');
        }
        screen
    }
}

/// A row of the frame: its two corners and as much of `across` as fits between them.
fn framed(corners: (char, char), across: char, width: usize) -> String {
    let (left, right) = corners;
    let across = across.to_string().repeat(width.saturating_sub(2));
    format!("{left}{across}{right}")
}

/// A row of the frame's sides and nothing between them.
fn a_blank_row(width: usize) -> String {
    framed((SIDE, SIDE), ' ', width)
}

/// A row with `label` centred between the sides, or as much of it as fits.
///
/// **An odd remainder goes right**, so `monospace` in a forty-two-wide interior is sixteen spaces and
/// then seventeen. One space, and a reader with a ruler can check which side of the label it is on.
fn centred(label: &str, width: usize) -> String {
    let inside = width.saturating_sub(2);
    let label: String = label.chars().take(inside).collect();
    let left = inside.saturating_sub(label.chars().count()) / 2;
    let right = inside
        .saturating_sub(left)
        .saturating_sub(label.chars().count());
    let (left, right) = (" ".repeat(left), " ".repeat(right));
    format!("{SIDE}{left}{label}{right}{SIDE}")
}

/// Runs the application on `screen` until it ends, and answers how it went.
///
/// **The screen is borrowed into a guard rather than held here**, which is what makes the terminal
/// come back on the way a panic takes as well as on the three ways out that return.
pub(crate) fn run(screen: &mut impl Screen) -> io::Result<()> {
    let mut app = App::new();
    let mut taken = GivenBack::of(screen);
    taken.take()?;
    app.draw(taken.screen())?;
    until_the_way_out(taken.screen(), &mut app)
}

/// Every event, until the application ends: read one, turn it into the action it names, and apply it.
///
/// **An event that names no action reaches the state as nothing at all**, so the screen is not drawn
/// again. Drawing the same screen would be reporting something, and there is nothing to report.
fn until_the_way_out(screen: &mut impl Screen, app: &mut App) -> io::Result<()> {
    while let Some(event) = screen.next_event()? {
        let Some(action) = app.action_for(&event) else {
            continue;
        };
        if !app.apply(action) {
            break;
        }
        app.draw(screen)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::event::{
        Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };

    use super::{Action, App, run, until_the_way_out};
    use crate::screen::{Recorded, THE_PICTURE};

    /// A key event, which is what every test here arrives at the application as.
    fn press(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    /// The way out as a key, which is the key the specification's `## Examples` arrives with.
    fn the_way_out() -> Event {
        press(KeyCode::Char('q'))
    }

    /// A click, which is the event _a click arrives and changes nothing_ is about: it arrives, and no
    /// action is a click yet.
    fn a_click() -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// The screen the specification's `## Examples` draws, written out rather than asked of the
    /// drawing, because a test that says _the screen holds the menu and nothing else_ and then asks
    /// the drawing what it holds has said nothing. Forty-four columns by seven rows.
    ///
    /// **One space from the picture the specification draws, on the title row**, which is written
    /// forty-five columns wide with seventeen spaces on each side of `monospace` where a
    /// forty-two-wide interior holds sixteen and seventeen. The picture is labelled hypothetical and
    /// was never observed, and a title cannot be centred in forty-three.
    const THE_MENU: &str = "\
┌──────────────────────────────────────────┐
│                                          │
│                monospace                 │
│                                          │
│                   Quit                   │
│                                          │
└──────────────────────────────────────────┘
";

    /// The menu offers one leaf and it is the way out: what it calls the leaf, and what choosing it
    /// does.
    ///
    /// **Both halves are pinned because the screen joins them.** A menu offering two leaves, or one
    /// leaf labelled `Quit` that chose something else, draws exactly the same picture until it is
    /// chosen.
    #[test]
    fn the_menu_offers_one_leaf_and_it_is_the_way_out() {
        let app = App::new();

        let offered: Vec<(&str, Action)> = app
            .menu
            .leaves
            .iter()
            .map(|leaf| (leaf.label, leaf.action))
            .collect();

        assert_eq!(offered, vec![("Quit", Action::Quit)]);
    }

    /// A key reaches the state only as an action: the key the application quits on arrives as the way
    /// out, and the state is what that action did to it.
    ///
    /// **The claim is about the route and not about the outcome**, so it is made from both ends of
    /// the route rather than from the outcome alone — a program that ended on `q` for any other reason
    /// would satisfy the second assertion. The other half is
    /// `a_key_that_names_no_action_changes_nothing_and_reports_nothing`: an event that names no
    /// action reaches the state as nothing at all.
    #[test]
    fn a_key_reaches_the_state_only_as_an_action() {
        let key = the_way_out();
        let mut app = App::new();

        assert_eq!(
            app.action_for(&key),
            Some(Action::Quit),
            "the key arrives as the way out and as nothing else"
        );

        let mut screen = Recorded::scripted([key]);
        until_the_way_out(&mut screen, &mut app).expect("a script that runs out is not a failure");

        assert!(!app.running, "and the state is what that action did to it");
    }

    /// Applying the way out ends the application, and it is the only action there is.
    ///
    /// **The second half is not tested, and that is worth saying**: an enum holding one variant is a
    /// fact about the type rather than about a run, and the run below cannot tell a one-variant enum
    /// from a two-variant one whose second variant nothing performs.
    #[test]
    fn applying_the_way_out_ends_the_application() {
        let mut app = App::new();

        let still_up = app.apply(Action::Quit);

        assert!(
            !still_up,
            "applying the way out answers that the application has ended"
        );
        assert!(!app.running, "and the state is what that answer is made of");
    }

    /// A key that names no action changes nothing and reports nothing.
    ///
    /// **"Nothing" is the state as it was, which today is one flag** — the menu is compiled in and
    /// nothing writes it — and **"reports nothing" is nothing drawn**: the screen is not written
    /// again, because writing the same screen is a report. The keys are two, so a program that
    /// ignored the first and honoured the second could not pass this.
    #[test]
    fn a_key_that_names_no_action_changes_nothing_and_reports_nothing() {
        let mut app = App::new();
        let mut screen = Recorded::scripted([press(KeyCode::Char('a')), press(KeyCode::Char('z'))]);

        until_the_way_out(&mut screen, &mut app).expect("a script that runs out is not a failure");

        assert!(
            app.running,
            "neither key reached the state, so the application is as it was"
        );
        assert!(
            screen.drawn.is_empty(),
            "and neither of them reported anything: {:?}",
            screen.drawn
        );
    }

    /// A click arrives and changes nothing, because no action is a click yet.
    ///
    /// **The first assertion is the one that makes the second mean anything.** A screen that
    /// delivered no clicks at all would also draw nothing and change nothing, so what arrived is
    /// pinned before what it did.
    #[test]
    fn a_click_arrives_and_changes_nothing_because_no_action_is_a_click() {
        let mut app = App::new();
        let mut screen = Recorded::scripted([a_click(), a_click()]);

        until_the_way_out(&mut screen, &mut app).expect("a script that runs out is not a failure");

        assert_eq!(
            screen.arrived,
            vec![a_click(), a_click()],
            "both clicks arrived"
        );
        assert!(
            app.running,
            "and neither reached the state, because no action is a click yet"
        );
        assert!(
            screen.drawn.is_empty(),
            "and neither of them reported anything: {:?}",
            screen.drawn
        );
    }

    /// The screen a run produces is the menu and nothing else: one whole screen, the picture the
    /// specification drew, and no second one.
    ///
    /// **The run rather than the drawing**, because the claim is about what the application does
    /// rather than about what its drawing function returns when asked.
    #[test]
    fn the_screen_holds_the_menu_and_nothing_else() {
        let mut screen = Recorded::scripted([the_way_out()]);

        run(&mut screen).expect("a run over a script of events does not fail");

        assert_eq!(
            screen.drawn,
            [THE_MENU.to_owned()],
            "the screen is the menu, drawn once, with nothing after it"
        );
    }

    /// A screen narrower than the frame, or shorter than the menu, is drawn rather than refused.
    ///
    /// **Not a rule of the model and not in the specification's acceptance list.** It is here because
    /// the size is asked at draw time rather than taken from a resize event — the cost the
    /// specification's D6 names — and a terminal that can be resized to nothing must not be able to
    /// end the application by arithmetic.
    #[test]
    fn a_screen_too_small_for_the_frame_is_drawn_rather_than_refused() {
        let app = App::new();

        assert_eq!(app.drawn(0, 0), "", "no columns and no rows is no screen");
        assert_eq!(
            app.drawn(3, 1),
            "┌─┐\n",
            "one row of a three-column frame is the top of it"
        );
        assert_eq!(
            app.drawn(3, 2),
            "┌─┐\n│ │\n",
            "and two rows are the top and the first blank row"
        );
    }

    /// The menu is drawn at the size the specification's picture is written at.
    ///
    /// **Two constants that have to agree, and this is what makes them agree.** [`THE_MENU`] is
    /// forty-four columns by seven rows written out by hand and [`THE_PICTURE`] is that size as two
    /// numbers; nothing else in the crate says how wide a hand-written picture is meant to be, and a
    /// picture wider than the screen it is compared with would be a comparison of two things.
    #[test]
    fn the_menu_is_drawn_at_the_size_the_specifications_picture_is_written_at() {
        let app = App::new();

        assert_eq!(app.drawn(THE_PICTURE.0, THE_PICTURE.1), THE_MENU);
        assert_eq!(
            THE_MENU.lines().count(),
            usize::from(THE_PICTURE.1),
            "and the picture is as tall as the size it is drawn at"
        );
    }
}
