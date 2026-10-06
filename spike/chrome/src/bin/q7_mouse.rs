//! Q7 — Mouse hit-testing. Does a click reach the component, and how?
//!
//! Four mechanisms are compared:
//!   1. a Sub with EventClause::Mouse(MouseEventClause{ column: Range, row: Range })
//!   2. a hand-written coordinate test inside AppComponent::on
//!   3. the component being told its own Rect by the app (a prop round-trip)
//!   4. the same thing in plain ratatui, for comparison

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use tuirealm::application::{Application, PollStrategy};
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{
    Event, Key, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind, NoUserEvent,
};
use tuirealm::listener::EventListenerCfg;
use tuirealm::props::{AttrValue, Attribute, Props, QueryResult};
use tuirealm::ratatui::Frame;
use tuirealm::ratatui::layout::Rect;
use tuirealm::ratatui::widgets::Paragraph;
use tuirealm::state::State;
use tuirealm::subscription::{EventClause, MouseEventClause, Sub, SubClause};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
enum Id {
    /// target of mechanism 1 (Sub with a coordinate range)
    SubTarget,
    /// target of mechanisms 2 and 3 (manual hit test)
    Manual,
}

#[derive(Debug, PartialEq, Clone)]
enum Msg {
    SubGotClick { column: u16, row: u16 },
    ManualGotClick { column: u16, row: u16 },
    ManualMissed { column: u16, row: u16 },
    Keyed,
}

/// Mechanism 3: the component is TOLD its rect via a prop, because tuirealm
/// does not remember the Rect it was drawn into.
struct Widget {
    props: Props,
}

impl Component for Widget {
    fn view(&mut self, f: &mut Frame, area: Rect) {
        f.render_widget(Paragraph::new("widget"), area);
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

impl Widget {
    /// mechanism 3: read the rect back out of the prop
    fn told_area(&self) -> Option<Rect> {
        let s = self.props.get(Attribute::Title).and_then(|v| match v {
            AttrValue::Title(t) => Some(t.content.spans.iter().map(|sp| sp.content.as_ref()).collect::<String>()),
            _ => None,
        })?;
        let p: Vec<&str> = s.split(',').collect();
        Some(Rect::new(
            p.get(0)?.parse().ok()?,
            p.get(1)?.parse().ok()?,
            p.get(2)?.parse().ok()?,
            p.get(3)?.parse().ok()?,
        ))
    }
}

impl AppComponent<Msg, NoUserEvent> for Widget {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        if let Some(KeyEvent { code, .. }) = ev.as_keyboard()
            && *code == Key::Char('x')
        {
            return Some(Msg::Keyed);
        }
        let MouseEvent { kind, column, row, .. } = ev.as_mouse()?;
        if !matches!(kind, MouseEventKind::Down(MouseButton::Left)) {
            return None;
        }
        if self.told_area().is_some_and(|a| {
            a.x <= *column && *column < a.x + a.width && a.y <= *row && *row < a.y + a.height
        }) {
            Some(Msg::ManualGotClick {
                column: *column,
                row: *row,
            })
        } else {
            Some(Msg::ManualMissed {
                column: *column,
                row: *row,
            })
        }
    }
}

struct OneClick {
    c: u16,
    r: u16,
    done: bool,
}

impl tuirealm::listener::Poll<NoUserEvent> for OneClick {
    fn poll(&mut self) -> tuirealm::listener::PortResult<Option<Event<NoUserEvent>>> {
        if self.done {
            return Ok(None);
        }
        self.done = true;
        Ok(Some(click(self.c, self.r)))
    }
}

fn click(column: u16, row: u16) -> Event<NoUserEvent> {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        modifiers: KeyModifiers::NONE,
        column,
        row,
    })
}

fn main() {
    println!("=== MECHANISM 1: Sub with EventClause::Mouse(column: Range, row: Range) ===");
    println!("The clause carries ABSOLUTE screen ranges. tuirealm does the containment test.");
    {
        let mut app: Application<Id, Msg, NoUserEvent> =
            Application::init(EventListenerCfg::default());
        app.mount(Id::SubTarget, Box::new(Widget { props: Props::default() }), vec![])
            .unwrap();
        // pretend the button occupies rows 4..7, cols 10..20
        let clause = MouseEventClause {
            kind: MouseEventKind::Down(MouseButton::Left),
            modifiers: KeyModifiers::NONE,
            column: 10..20,
            row: 4..7,
        };
        app.subscribe(
            &Id::SubTarget,
            Sub::new(EventClause::Mouse(clause), SubClause::Always),
        )
        .unwrap();

        // EventClause::forward() is private, so drive the clause through the
        // real Application::tick() instead, with a Port that emits the click.
        fn run(emit: (u16, u16)) -> Vec<Msg> {
            let mut a: Application<Id, Msg, NoUserEvent> = Application::init(
                EventListenerCfg::default()
                    .tick_interval(Duration::from_millis(20))
                    .add_port(
                        Box::new(OneClick { c: emit.0, r: emit.1, done: false }),
                        Duration::from_millis(20),
                        1,
                    ),
            );
            a.mount(Id::SubTarget, Box::new(Widget { props: Props::default() }), vec![]).unwrap();
            // NO active() call on purpose: with no focus, the Sub clause is the
            // ONLY path the click can take, so the result is attributable.
            a.subscribe(
                &Id::SubTarget,
                Sub::new(
                    EventClause::Mouse(MouseEventClause {
                        kind: MouseEventKind::Down(MouseButton::Left),
                        modifiers: KeyModifiers::NONE,
                        column: 10..20,
                        row: 4..7,
                    }),
                    SubClause::Always,
                ),
            )
            .unwrap();
            let mut got = Vec::new();
            for _ in 0..30 {
                if let Ok(ms) = a.tick(PollStrategy::Once(Duration::from_millis(5))) {
                    got.extend(ms);
                }
            }
            got
        }
        for (c, r) in [(15u16, 5u16), (25, 5), (15, 9), (9, 4)] {
            let got = run((c, r));
            println!(
                "  click ({c:>2},{r:>2}) inside 10..20 x 4..7 -> msgs: {got:?}  forwarded: {}",
                !got.is_empty()
            );
        }
        println!("  NOTE: the ranges are ABSOLUTE screen coords, hardcoded. They do not");
        println!("        track the Rect the component was actually drawn into.");
        println!("  NOTE: SubClause::HasAttrValue / IsMounted can gate on component state,");
        println!("        but nothing makes the range follow the widget's geometry.");
    }

    println!("\n=== MECHANISM 2 + 3: manual hit test inside on(), rect handed in as a prop ===");
    let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
        EventListenerCfg::default().tick_interval(Duration::from_millis(500)),
    );
    app.mount(Id::Manual, Box::new(Widget { props: Props::default() }), vec![])
        .unwrap();
    // the app computes the Rect and pushes it in
    let area = Rect::new(10, 4, 10, 3);
    let title = tuirealm::props::Title::from(format!(
        "{},{},{},{}",
        area.x, area.y, area.width, area.height
    ));
    app.attr(&Id::Manual, Attribute::Title, AttrValue::Title(title))
        .unwrap();
    println!("  app pushed Rect {area:?} into the component via attr()");
    println!("  (tuirealm has no other way to tell a component where it was drawn)");

    for (c, r) in [(12u16, 5u16), (30, 5), (12, 20), (9, 5)] {
        let msg = app.get_component_mut(&Id::Manual).unwrap().on(&click(c, r));
        println!("  click ({c:>2},{r:>2}) -> {msg:?}");
    }
    println!("  => the click DOES reach the component, but only because the app pushed");
    println!("     the Rect down first. There is no built-in hit test.");

    println!("\n=== MECHANISM 4: plain ratatui, for comparison ===");
    {
        let area = Rect::new(10, 4, 10, 3);
        let btn = area;
        let hit = |c: u16, r: u16| {
            btn.x <= c && c < btn.x + btn.width && btn.y <= r && r < btn.y + btn.height
        };
        for (c, r) in [(12u16, 5u16), (30, 5)] {
            println!("  click ({c:>2},{r:>2}) -> hit: {}", hit(c, r));
        }
        println!("  => identical arithmetic. ratatui gives you the area at draw time in a");
        println!("     closure, so you must store it; tuirealm additionally makes you");
        println!("     round-trip it through props/attr().");
    }

    println!("\n=== DOES THE COMPONENT SEE ITS OWN AREA ANYWHERE? ===");
    {
        let w = Widget { props: Props::default() };
        println!("  Component trait methods: view/attr/query/state/perform + on().");
        println!("  Is there a `area()` / `rect()` accessor? {}", false);
        println!("  With no attr() push, told_area() = {:?}", w.told_area());
    }
}
