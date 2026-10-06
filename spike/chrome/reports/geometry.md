# What a component is not told

Measured 2026-10-06 on `tuirealm 4.1.0`, by `cargo run --bin q7-mouse` and by reading
`src/core/view.rs` and `src/terminal/adapter/crossterm.rs`.

## The finding

`Component` has six methods: `view`, `attr`, `query`, `state`, `perform`, `on`.

There is no `area()`. There is no `rect()`. Nothing on the trait returns where a component was
drawn.

    DOES THE COMPONENT SEE ITS OWN AREA ANYWHERE?
      Component trait methods: view/attr/query/state/perform + on().
      Is there a `area()` / `rect()` accessor? false
      With no attr() push, told_area() = None

`View::view` is a delegate (`view.rs:157`):

    pub fn view(&mut self, id: &ComponentId, f: &mut Frame, area: Rect) {
        if let Some(c) = self.components.get_mut(id) {
            c.view(f, area);
        }

The `Rect` is passed in and dropped. `View` stores no layout.

## The four ways a click reaches a control

All four were exercised. Only the last two work, and neither is free.

### 1. `EventClause` with a coordinate range — works, drifts silently

    EventClause::Mouse(MouseEventClause {
        kind: MouseEventKind::Down(MouseButton::Left),
        modifiers: KeyModifiers::NONE,
        column: 10..20,   // absolute screen columns
        row: 4..7,        // absolute screen rows
    })

    click (15, 5) inside 10..20 x 4..7 -> msgs: [ManualGotClick]  forwarded: true
    click (25, 5) inside 10..20 x 4..7 -> msgs: []               forwarded: false
    click (15, 9) inside 10..20 x 4..7 -> msgs: []               forwarded: false
    click ( 9, 4) inside 10..20 x 4..7 -> msgs: []               forwarded: false

The containment test is the framework's. The range is a pair of absolute integers that has to agree
with a layout computed somewhere else, and nothing checks that it still does:

    NOTE: the ranges are ABSOLUTE screen coords, hardcoded. They do not
          track the Rect the component was actually drawn into.

Resizing the terminal moves the button and leaves the range where it was.

### 2 and 3. Manual test in `on()`, with the `Rect` pushed in as a prop — works, and the push is the cost

The component is told its own geometry by the application writing it into the component's props,
because there is no other channel:

    app pushed Rect Rect { x: 10, y: 4, width: 10, height: 3 } into the component via attr()
    click (12, 5) -> Some(ManualGotClick { column: 12, row: 5 })
    click (30, 5) -> Some(ManualMissed { column: 30, row: 5 })
    click (12,20) -> Some(ManualMissed { column: 12, row: 20 })
    click ( 9, 5) -> Some(ManualMissed { column:  9, row: 5 })

This is what `chrome.rs` does, and it is correct. The friction is visible in `q6_tuirealm.rs`, where
the pop-up's rectangle is computed by `draw` and the hit-test in `on()` re-derives the numbers from
a constant beside the keyboard handler:

    // Manual hit-test against the Close button. These constants are the cost: the
    // component was drawn into whatever `draw` computed from the real screen size and was
    // never told what that was.
    let (bx, by) = (10u16, 5u16);

And the state the widgets read travels as a string parsed back out of a prop, because a minimal
component has nowhere else to put it:

    fn flags(&self) -> (bool, bool) {
        self.p.get(Attribute::Text).and_then(|v| match v {
            AttrValue::String(s) => {
                let mut it = s.split(',');
                let menu = it.next() == Some("true");
                let popup = it.next() == Some("true");

A real component uses typed props and the split goes away. It is in the minimal version because the
minimal version is what measures the floor.

### 4. Plain ratatui — the same arithmetic, one step fewer

In the draw closure, and kept by the program:

    let d = Rect::new(f.area().x + 4, f.area().y + 1, 24, 6);
    // ... store `d` in your own struct, and test against it in your event loop

    click (12, 5) -> hit: true
    click (30, 5) -> hit: false

Identical arithmetic. ratatui hands the area to the closure and lets the program keep it. tuirealm
hands the same area to `view` and throws it away, so the program has to push it back.

## What this costs in practice

Two lines of extra round-trip per control, and a layout change that has to be remembered in a second
place. For an application whose clickable regions _are_ the rendered layout — menu labels, a canvas,
buttons positioned by the solver — that second place is where a bug waits, because nothing fails
when it drifts.

**You can watch it happen.** Run `cargo run --bin q6-tuirealm`, press `F2`, and resize the terminal.
The dialog moves; the click target does not, and the button stops answering without a word from the
program. Run `q6-plain-full` and do the same: the button follows.

Mouse capture itself is not a problem. `CrosstermTerminalAdapter` implements `enable_mouse_capture`
with `EnableMouseCapture` plus `set_mode(Modes::MOUSE)` (`crossterm.rs:181`), and every mechanism
above was exercised through a real `Application::tick` with a synthetic click port.

## One thing to know about subscriptions

`application.rs:430`:

    for sub in &self.subs {
        // ! Active component must be different from sub !
        if self.view.has_focus(sub.target()) { continue; }

A subscription whose target currently holds focus is skipped, because the component already got the
event through the focus path. Measured, so that it is known to be a deliberate skip rather than a
lost event:

    msgs when Sub target is NOT focused: 22
    msgs when Sub target IS  focused: 23

A subscription is not a second route for the focused component. Anything that must work while a
component has focus has to work through `on`.
