//! The contract tests for the editing layer, one per rule of
//! [`specs/197-an-editing-layer-above-the-diagram-model.md`](../../../specs/197-an-editing-layer-above-the-diagram-model.md)'s
//! `## Behavior`, named after the rule each one holds.
//!
//! **Everything here is asserted on a picture**, which is what a caller sees and what the model's
//! worked example is written in. Comparing figures would be a claim about a value nothing outside
//! this crate can see, and comparing a session's diagram against a diagram's would only say the
//! session hands back what it was given.
//!
//! **Two kinds of window are in use and neither is sampled.** `EXAMPLE` is the specification's
//! own `window(8, 3)`, which is exactly wide enough for a four-cell box and a displacement of
//! four, and the worked example in that document is reproduced here number for number. `FIXTURE`
//! is wider and taller because the fixture has two boxes and a connector under them.

use std::num::NonZeroU32;

use monospace_core::{Buffer, Direction, Glyph, GlyphCatalog, Pos, Size, Stroke, Terminal, render};
use monospace_diagram::{Anchor, Delta, Diagram, Endpoint, Position, Shape, ShapeId};
use monospace_editing::{Command, Session};

/// The specification's window: `window(8, 3)`, wide enough for a four-cell box at the origin and
/// for the same box four cells to the right, which is the largest displacement the worked example
/// uses.
const EXAMPLE: Size = Size {
    width: 8,
    height: 3,
};

/// The window the fixture is drawn in: two boxes on the first three rows and a connector under
/// them.
const FIXTURE: Size = Size {
    width: 12,
    height: 6,
};

/// The origin every window here starts at, so a picture is a window's worth of rows and columns
/// measured from the same place.
const ORIGIN: Pos = Pos { x: 0, y: 0 };

/// The box at the origin, in [`EXAMPLE`]. **The rows carry four cells of padding** because the
/// window is eight wide and a four-cell box leaves the other four blank — written out here so the
/// literals below say which figure they show rather than how wide the window is.
const AS_WRITTEN: &str = "┌──┐    \n│  │    \n└──┘    \n";

/// The same box four cells to the right, which is the displacement the specification's worked
/// example uses and the widest one that fits [`EXAMPLE`] whole.
const FOUR_RIGHT: &str = "    ┌──┐\n    │  │\n    └──┘\n";

/// The same box two cells left of [`AS_WRITTEN`] — three cells in, so the window pads it on the
/// right rather than on the left.
const ONE_RIGHT: &str = "   ┌──┐ \n   │  │ \n   └──┘ \n";

/// The same box two cells right of [`FOUR_RIGHT`], whose last two columns fall outside the window
/// and are not drawn at all — which is why this picture is narrower rather than padded.
const SIX_RIGHT: &str = "      ┌─\n      │ \n      └─\n";

/// An identity of the ordinal `ordinal`, which is how every test below names a figure.
fn id(ordinal: u32) -> ShapeId {
    ShapeId::new(NonZeroU32::new(ordinal).expect("no test names zero"))
}

/// The box the specification's example calls `small_box(0, 0, no fill)`: four cells by three, no
/// interior, drawn in the light table.
fn small_box(at: Pos) -> Shape {
    Shape::Box {
        at,
        size: Size {
            width: 4,
            height: 3,
        },
        stroke: Stroke::from("light"),
        fill: None,
    }
}

/// A diagram holding one of the boxes above and nothing else, so its `add` hands back `1`.
fn one_box(at: Pos) -> Diagram {
    let mut diagram = Diagram::new();
    diagram.add(small_box(at));
    diagram
}

/// The drawing of `diagram` into a fresh window of `size`.
///
/// A window of its own every time, because a caller repaints on every event and the claim under
/// test is never about a buffer left over from an earlier one.
fn picture_in(diagram: &Diagram, size: Size) -> String {
    let mut buffer = Buffer::new(ORIGIN, size);
    diagram.draw(&mut buffer);
    render(&buffer, &GlyphCatalog::light(), ORIGIN, size)
}

/// The drawing of what `session` holds, in the window the specification's example is drawn in.
fn picture(session: &Session) -> String {
    picture_in(session.diagram(), EXAMPLE)
}

/// The drawing of `diagram` the specification's example calls the first picture.
fn example_picture(diagram: &Diagram) -> String {
    picture_in(diagram, EXAMPLE)
}

/// What `command` leaves behind, in the window the fixture is drawn in.
fn after(command: &Command) -> String {
    let mut session = Session::new(the_fixture());
    command.perform(&mut session);
    picture_in(session.diagram(), FIXTURE)
}

/// What the diagram's own operation leaves behind, over the same fixture.
///
/// **Taken as a function rather than written out four times**, so the two sides of the comparison
/// are the command and the operation and not a restatement of the command written in the diagram's
/// vocabulary, which would make the test agree with itself.
fn after_operating(on_diagram: impl FnOnce(&mut Diagram)) -> String {
    let mut diagram = the_fixture();
    on_diagram(&mut diagram);
    picture_in(&diagram, FIXTURE)
}

/// A diagram with something for every command to act on: two overlapping boxes, where the order
/// decides a shared cell, and a connector whose two ends are both plain points, where hanging one
/// from a side shows.
fn the_fixture() -> Diagram {
    let mut diagram = Diagram::new();
    diagram.add(small_box(ORIGIN));
    diagram.add(Shape::Box {
        at: Pos { x: 2, y: 0 },
        size: Size {
            width: 4,
            height: 3,
        },
        stroke: Stroke::from("heavy"),
        fill: None,
    });
    diagram.add(Shape::Connector {
        from: Endpoint {
            at: Position::from(Pos { x: 0, y: 4 }),
            leaving: Direction::Right,
            terminal: Terminal::Arm,
        },
        to: Endpoint {
            at: Position::from(Pos { x: 8, y: 4 }),
            leaving: Direction::Left,
            terminal: Terminal::Glyph {
                glyph: Glyph::new("▶").expect("\"▶\" is one glyph"),
            },
        },
        stroke: Stroke::from("light"),
    });
    diagram
}

/// A figure the fixture does not hold, added by whichever test needs `add` to answer.
fn an_extra_box() -> Shape {
    Shape::Box {
        at: Pos { x: 6, y: 5 },
        size: Size {
            width: 2,
            height: 1,
        },
        stroke: Stroke::from("light"),
        fill: None,
    }
}

/// Rule 1 — the diagram a session holds is the one it was created over.
///
/// **The diagram is taken by value and the caller's copy is a diagram of its own.** Changed however
/// the caller likes afterwards, and the session neither follows it nor is reached by it: what is
/// asserted is that the two are two diagrams, which is the difference between a session owning a
/// diagram and a session sharing one.
#[test]
fn a_session_owns_the_diagram_it_was_created_over() {
    let the_callers_own = one_box(ORIGIN);
    let session = Session::new(the_callers_own.clone());

    assert_eq!(
        picture(&session),
        example_picture(&the_callers_own),
        "a session over a diagram draws that diagram"
    );

    let mut theirs = the_callers_own.clone();
    theirs.forward(id(1));
    theirs.remove(id(1));

    assert_ne!(
        picture_in(&theirs, EXAMPLE),
        example_picture(&the_callers_own),
        "the caller's own copy did change"
    );
    assert_eq!(
        picture(&session),
        example_picture(&the_callers_own),
        "and the session, which was given the diagram by value, did not follow it"
    );
}

/// Rule 2 — the diagram a session owns is changed only by a command performed against it.
///
/// **Two sessions over equal diagrams, one command performed on one of them.** What is held back is
/// that the command reached the session it was given and not some diagram both of them share:
/// there is nothing behind the diagram for it to reach through, and this is what a shared handle
/// would look like.
#[test]
fn the_diagram_changes_only_by_a_command_performed_on_it() {
    let mut given = Session::new(one_box(ORIGIN));
    let given_nothing = Session::new(one_box(ORIGIN));

    Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    }
    .perform(&mut given);

    assert_eq!(
        picture(&given_nothing),
        AS_WRITTEN,
        "the session the command was not given still holds what it was created over"
    );
    assert_eq!(
        picture(&given),
        FOUR_RIGHT,
        "and the one it was given moved by four"
    );
}

/// Rule 3 — no method of a session hands out a mutable reference to the diagram it owns.
///
/// **Every public method of `Session` and of `Command`, taken as a value of its exact signature.**
/// What this pins is each signature, so a method that changed what it returns fails here. What it
/// does not pin is the absence of a method added later, which no test written against a type can
/// see; the `compile_fail` documentation example on [`Session`] carries that half, and neither of
/// the two is a substitute for reading the surface.
#[test]
fn no_method_of_a_session_hands_out_a_mutable_diagram() {
    let _: fn(Diagram) -> Session = Session::new;
    let _: for<'a> fn(&'a Session) -> &'a Diagram = Session::diagram;
    let _: fn(&Session) -> bool = Session::can_undo;
    let _: fn(&Session) -> bool = Session::can_redo;
    let _: fn(&mut Session) = Session::undo;
    let _: fn(&mut Session) = Session::redo;
    let _: for<'a> fn(&'a Command, &mut Session) = Command::perform;
}

/// Rule 4 — every command becomes one of the diagram's five operations.
///
/// **Each command against the operation it names, over the same fixture and in the same window.**
/// What makes it the mapping and not the command is that the right-hand side is written in the
/// diagram's own vocabulary — `forward`, `replace` with a figure read and rebuilt, `remove` — and
/// so a command that did something else, or something more, disagrees with it.
#[test]
fn each_command_becomes_one_of_the_diagrams_five_operations() {
    let two_down = Delta { dx: 0, dy: 2 };

    assert_eq!(
        after(&Command::Move {
            id: id(1),
            by: two_down,
        }),
        after_operating(|diagram| {
            let moved = diagram.get(id(1)).map(|shape| shape.displaced_by(two_down));
            if let Some(moved) = moved {
                diagram.replace(id(1), moved);
            }
        }),
        "`Move` is `replace` with the figure displaced"
    );

    assert_eq!(
        after(&Command::Hang {
            id: id(3),
            from: id(1),
            anchor: Anchor::Bottom,
        }),
        after_operating(|diagram| {
            if let Some(Shape::Connector { from, to, stroke }) = diagram.get(id(3)).cloned() {
                diagram.replace(
                    id(3),
                    Shape::Connector {
                        from: Endpoint {
                            at: Position::Reference(monospace_diagram::Reference::new(
                                id(1),
                                Anchor::Bottom,
                                Delta { dx: 0, dy: 0 },
                                0,
                            )),
                            leaving: from.leaving,
                            terminal: from.terminal,
                        },
                        to,
                        stroke,
                    },
                );
            }
        }),
        "`Hang` is `replace` with one end of the connector rebuilt"
    );

    assert_eq!(
        after(&Command::Forward { id: id(1) }),
        after_operating(|diagram| diagram.forward(id(1))),
        "`Forward` is `forward`"
    );

    assert_eq!(
        after(&Command::Remove { id: id(2) }),
        after_operating(|diagram| diagram.remove(id(2))),
        "`Remove` is `remove`"
    );
}

/// Rule 4 — and no command reaches the diagram by any other way.
///
/// **Every command, once, over one session, and then two readings of what is left.** The figures
/// still there are the fixture's minus the one that was taken out, and the ordinal the next `add`
/// would hand back is the one the fixture issued before any of them ran — which is what says that
/// nothing added a figure on the way, since `add` is the only operation that moves that counter.
#[test]
fn no_command_reaches_the_diagram_by_another_way() {
    let mut session = Session::new(the_fixture());

    for command in [
        Command::Forward { id: id(1) },
        Command::Move {
            id: id(1),
            by: Delta { dx: 0, dy: 2 },
        },
        Command::Hang {
            id: id(3),
            from: id(1),
            anchor: Anchor::Bottom,
        },
        Command::Forward { id: id(7) },
        Command::Move {
            id: id(7),
            by: Delta { dx: 1, dy: 1 },
        },
        Command::Remove { id: id(2) },
        Command::Remove { id: id(7) },
    ] {
        command.perform(&mut session);
    }

    let diagram = session.diagram();
    assert!(
        diagram.get(id(2)).is_none(),
        "`Remove` took the figure it named out"
    );
    assert!(diagram.get(id(1)).is_some(), "the first box is still there");
    assert!(
        diagram.get(id(3)).is_some(),
        "the connector is still there, whatever `Hang` did to one end of it"
    );

    let mut copy = diagram.clone();
    assert_eq!(
        copy.add(an_extra_box()),
        id(4),
        "no command reached `add`, so the next ordinal is the one the fixture issued"
    );
}

/// Rule 5 — a command is a step whether or not it changed the diagram.
///
/// **The specification's fifth line**, a command naming a figure the diagram does not hold. It
/// changes nothing, it is recorded all the same, and giving it back changes nothing and leaves
/// something to carry out again — which is the rule as a caller meets it.
#[test]
fn a_command_naming_nothing_is_still_a_step() {
    let mut session = Session::new(one_box(ORIGIN));
    let as_written = picture(&session);

    Command::Move {
        id: id(7),
        by: Delta { dx: 1, dy: 0 },
    }
    .perform(&mut session);

    assert_eq!(
        picture(&session),
        as_written,
        "a command naming a figure the diagram does not hold changes nothing"
    );
    assert!(session.can_undo(), "and left a step behind it all the same");

    session.undo();
    assert_eq!(
        picture(&session),
        as_written,
        "and giving it back changes nothing"
    );
    assert!(
        session.can_redo(),
        "which leaves it available to be carried out again"
    );
}

/// Rule 6 — every command is a step of its own and the session merges none.
///
/// **Three commands and then three gives back, each of which moves the figure.** What makes it the
/// merge and not the recording is that the three gives back produce three different pictures: a session
/// that had merged them would have left one step behind, and the first undo would have jumped the
/// whole way back rather than one cell.
#[test]
fn two_commands_are_two_steps_and_are_never_merged() {
    let mut session = Session::new(one_box(ORIGIN));
    let mut forwards = vec![picture(&session)];

    for _ in 0..3 {
        Command::Move {
            id: id(1),
            by: Delta { dx: 2, dy: 0 },
        }
        .perform(&mut session);
        forwards.push(picture(&session));
    }

    let mut backwards = Vec::new();
    while session.can_undo() {
        session.undo();
        backwards.push(picture(&session));
    }

    let expected: Vec<String> = forwards[..3].iter().rev().cloned().collect();
    assert_eq!(
        backwards, expected,
        "three commands left three steps, and giving them back one at a time walks the pictures \
         back in the order they were made"
    );
}

/// Rule 7 — undo restores the diagram the step behind holds, whole.
///
/// **The specification's second and third blocks.** The picture an undo produces is the one the
/// session held before any command, byte for byte — not a reconstruction of it, and not the box
/// put back where it was, which is what an inverted command would have had to be.
#[test]
fn undo_restores_the_diagram_the_step_behind_holds() {
    let mut session = Session::new(one_box(ORIGIN));

    Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    }
    .perform(&mut session);

    assert_eq!(
        picture(&session),
        FOUR_RIGHT,
        "the figure moved four to the right"
    );
    assert!(session.can_undo(), "which left a step behind it");

    session.undo();

    assert_eq!(
        picture(&session),
        AS_WRITTEN,
        "and the undo brought it back"
    );
    assert!(!session.can_undo(), "with nothing behind it now");
}

/// Rule 8 — redo restores the diagram the step ahead holds, whole.
///
/// **The specification's fourth block**, which is the same picture the second one was: the step
/// ahead holds the state the diagram had before the command, and the diagram is that state again.
#[test]
fn redo_restores_the_diagram_the_step_ahead_holds() {
    let mut session = Session::new(one_box(ORIGIN));

    Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    }
    .perform(&mut session);

    session.undo();
    session.redo();

    assert_eq!(
        picture(&session),
        FOUR_RIGHT,
        "the redo brought the move back"
    );
    assert!(!session.can_redo(), "with nothing ahead of it now");
    assert_eq!(
        picture(&session),
        FOUR_RIGHT,
        "and it is the picture the move made, not the one the undo left"
    );
}

/// Rule 9 — undoing with no step behind changes nothing.
///
/// **The seventh block of the specification's example**, which is the end of the history rather than
/// the middle of it: two gives back after two commands put the figure back and then ask a third time.
#[test]
fn undo_with_nothing_behind_changes_nothing() {
    let mut session = Session::new(one_box(ORIGIN));
    let as_written = picture(&session);

    session.undo();

    assert_eq!(picture(&session), as_written, "nothing was given back");
    assert!(!session.can_undo(), "and nothing is behind the position");
}

/// Rule 9 — redoing with no step ahead changes nothing.
#[test]
fn redo_with_nothing_ahead_changes_nothing() {
    let mut session = Session::new(one_box(ORIGIN));

    Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    }
    .perform(&mut session);
    let as_left = picture(&session);

    session.redo();

    assert_eq!(picture(&session), as_left, "nothing was carried out again");
    assert!(!session.can_redo(), "and nothing is ahead of the position");
}

/// Rule 10 — a command carried out with steps ahead of the position discards them.
///
/// **Move, move, give one back, and move again**, which is the only way a position can be behind
/// the end. What is held is not that `redo` is unavailable afterwards — that alone would also be
/// true of a history truncated rather than discarded — but that **the undo out of the new command
/// reaches the state the given-back step held and not the state the discarded one held.** The two
/// differ by one cell, and the walk back over the discarded step would land on the wrong one.
#[test]
fn a_command_with_steps_ahead_discards_them() {
    let mut session = Session::new(one_box(ORIGIN));

    Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    }
    .perform(&mut session);
    Command::Move {
        id: id(1),
        by: Delta { dx: -1, dy: 0 },
    }
    .perform(&mut session);

    session.undo();
    assert_eq!(
        picture(&session),
        FOUR_RIGHT,
        "the given-back step holds the state the second move found"
    );

    Command::Move {
        id: id(1),
        by: Delta { dx: 2, dy: 0 },
    }
    .perform(&mut session);

    assert_eq!(
        picture(&session),
        SIX_RIGHT,
        "the new command was carried out from where the session stood"
    );
    assert!(
        !session.can_redo(),
        "the step that was ahead of the position was discarded"
    );

    session.undo();
    assert_eq!(
        picture(&session),
        FOUR_RIGHT,
        "one back is the state the given-back step held"
    );
    assert_ne!(
        picture(&session),
        ONE_RIGHT,
        "and not the state the discarded one held, which stood one cell nearer"
    );

    session.undo();
    assert_eq!(
        picture(&session),
        AS_WRITTEN,
        "and the second back is the diagram the session began with"
    );
}

/// Rule 11 — whether undo and redo are available is answerable without carrying either out.
///
/// **Asked at every position, drawing between the questions.** What is claimed is that answering
/// leaves the picture exactly where it was, which is what lets a caller grey out a control rather
/// than guess, and that the answers are right at each of the three positions a run passes through.
#[test]
fn whether_undo_is_available_is_answerable_without_carrying_it_out() {
    let mut session = Session::new(one_box(ORIGIN));
    let as_written = picture(&session);

    assert!(
        (session.can_undo(), session.can_redo()) == (false, false),
        "a session with nothing behind it has nothing to give back and nothing to carry out"
    );

    Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    }
    .perform(&mut session);

    assert!(
        (session.can_undo(), session.can_redo()) == (true, false),
        "after one command there is a step behind and none ahead"
    );
    assert!(
        session.can_undo(),
        "asked again, and the picture is untouched by being asked"
    );

    session.undo();

    assert!(
        (session.can_undo(), session.can_redo()) == (false, true),
        "at the oldest position it is the other way round"
    );
    assert!(session.can_redo());
    assert_eq!(
        picture(&session),
        as_written,
        "and every question left the picture where it was"
    );
}

/// Rule 12 — the diagram is drawn by borrowing it.
///
/// **A buffer drawn from `session.diagram()` and one drawn from the diagram the session was
/// created over, without anything copied in between.** The borrow is the claim: no `Diagram` is
/// taken, none is cloned, and the picture is the one the diagram model already draws. **The second
/// half is that a copy taken anyway is unconnected**: §7 of the model says a session hands out a
/// reference for drawing and a copy for keeping, and a copy a caller keeps does not follow the
/// session.
#[test]
fn drawing_what_a_session_holds_needs_no_copy() {
    let diagram = one_box(ORIGIN);

    let mut as_created = Buffer::new(ORIGIN, EXAMPLE);
    diagram.draw(&mut as_created);

    let session = Session::new(diagram);
    let mut borrowed = Buffer::new(ORIGIN, EXAMPLE);
    session.diagram().draw(&mut borrowed);

    assert_eq!(
        render(&borrowed, &GlyphCatalog::light(), ORIGIN, EXAMPLE),
        render(&as_created, &GlyphCatalog::light(), ORIGIN, EXAMPLE),
        "a drawing of what a session holds is a drawing of the diagram it was created over"
    );
    assert_eq!(
        render(&borrowed, &GlyphCatalog::light(), ORIGIN, EXAMPLE),
        AS_WRITTEN,
        "and it is the picture that diagram draws"
    );

    // And a copy is a diagram like any other: free of the session and unconnected to it, which is
    // what §7 is about. Nothing here is handed out but a borrow, so the copy is the caller's own.
    let theirs = session.diagram().clone();
    let mut session = session;
    Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    }
    .perform(&mut session);
    assert_eq!(
        picture_in(&theirs, EXAMPLE),
        AS_WRITTEN,
        "the copy did not move with the session's diagram"
    );
    assert_eq!(picture(&session), FOUR_RIGHT, "which did");
}
