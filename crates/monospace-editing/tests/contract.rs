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

use std::num::NonZeroU16;

use monospace_core::{
    Buffer, Direction, Glyph, GlyphCatalog, Pos, Size, Stroke, Terminal, Text, Window, render,
};
use monospace_diagram::{Anchor, Delta, Diagram, Endpoint, Shape, ShapeId};
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
fn id(ordinal: u16) -> ShapeId {
    ShapeId::new(NonZeroU16::new(ordinal).expect("no test names zero"))
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
        text: None,
    }
}

/// The smallest a box shrinks to: a `2×2`, under which `BoxShape` draws nothing.
fn smallest_box(at: Pos) -> Shape {
    Shape::Box {
        at,
        size: Size {
            width: 2,
            height: 2,
        },
        stroke: Stroke::from("light"),
        fill: None,
        text: None,
    }
}

/// A diagram holding one of the boxes above and nothing else, so its `add` hands back `1`.
fn one_box(at: Pos) -> Diagram {
    Diagram::new().add(small_box(at)).0
}

/// The drawing of `diagram` into a fresh window of `size`.
///
/// A window of its own every time, because a caller repaints on every event and the claim under
/// test is never about a buffer left over from an earlier one.
fn picture_in(diagram: &Diagram, size: Size) -> String {
    let mut buffer = Buffer::new(Window {
        origin: ORIGIN,
        size,
    });
    diagram.draw(&mut buffer);
    render(&buffer, &GlyphCatalog::light(), buffer.window())
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
    let session = Session::new(the_fixture()).perform(command.clone());
    picture_in(session.diagram(), FIXTURE)
}

/// What the diagram's own operation leaves behind, over the same fixture.
///
/// **Taken as a function rather than written out four times**, so the two sides of the comparison
/// are the command and the operation and not a restatement of the command written in the diagram's
/// vocabulary, which would make the test agree with itself.
fn after_operating(on_diagram: impl FnOnce(&Diagram) -> Diagram) -> String {
    let diagram = the_fixture();
    picture_in(&on_diagram(&diagram), FIXTURE)
}

/// A diagram with something for every command to act on: two overlapping boxes, where the order
/// decides a shared cell, and a connector whose two ends are both plain points, where hanging one
/// from a side shows.
fn the_fixture() -> Diagram {
    let mut diagram = Diagram::new();
    diagram = diagram.add(small_box(ORIGIN)).0;
    diagram = diagram
        .add(Shape::Box {
            at: Pos { x: 2, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: Stroke::from("heavy"),
            fill: None,
            text: None,
        })
        .0;
    diagram = diagram
        .add(Shape::Connector {
            from: Endpoint::absolute(Pos { x: 0, y: 4 }, Direction::Right, Terminal::Arm),
            to: Endpoint::absolute(
                Pos { x: 8, y: 4 },
                Direction::Left,
                Terminal::Glyph {
                    glyph: Glyph::new("▶").expect("\"▶\" is one glyph"),
                },
            ),
            stroke: Stroke::from("light"),
        })
        .0;
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
        text: None,
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

    let theirs = the_callers_own.forward(id(1)).remove(id(1));

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
    let given_nothing = Session::new(one_box(ORIGIN));
    let given = given_nothing.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    });

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

/// Rule 10 — no method of a session changes the session it was given.
///
/// **Every public method of `Session`, taken as a value of its exact signature.** What this pins is
/// each one, so a method that took `&mut self` or that returned nothing where it returns a session
/// now fails here. **There is no `Command::perform` in the list because there is no such method:**
/// a command is handed to a session and the way in is one method rather than two, which is the
/// change this list exists to record.
///
/// What this does not pin is the absence of a method added later, which no test written against a
/// type can see; the `compile_fail` documentation example on [`Session`] carries that half, and
/// neither of the two is a substitute for reading the surface.
#[test]
fn no_session_method_changes_the_session_it_was_given() {
    let _: fn(Diagram) -> Session = Session::new;
    let _: for<'a> fn(&'a Session) -> &'a Diagram = Session::diagram;
    let _: fn(&Session) -> bool = Session::can_undo;
    let _: fn(&Session) -> bool = Session::can_redo;
    let _: fn(&Session) -> Session = Session::undo;
    let _: fn(&Session) -> Session = Session::redo;
    let _: fn(&Session, Command) -> Session = Session::perform;
}

/// Rule 4 — every command becomes one of the diagram's seven operations.
///
/// **Each command against the operation it names, over the same fixture and in the same window.**
/// What makes it the mapping and not the command is that the right-hand side is written in the
/// diagram's own vocabulary — `forward`, `replace` with a figure read and rebuilt, `remove` — and
/// so a command that did something else, or something more, disagrees with it.
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "the contract test covers each command in the set against its operation"
)]
fn each_command_becomes_one_of_the_diagrams_seven_operations() {
    let two_down = Delta { dx: 0, dy: 2 };

    assert_eq!(
        after(&Command::Move {
            id: id(1),
            by: two_down,
        }),
        after_operating(|diagram| {
            let moved = diagram.get(id(1)).map(|shape| shape.displaced_by(two_down));
            if let Some(moved) = moved {
                diagram.replace(id(1), moved)
            } else {
                diagram.clone()
            }
        }),
        "`Move` is `replace` with the figure displaced"
    );

    assert_eq!(
        after(&Command::Resize {
            id: id(1),
            by: two_down,
        }),
        after_operating(|diagram| {
            let resized = diagram.get(id(1)).map(|shape| shape.resized_by(two_down));
            if let Some(resized) = resized {
                diagram.replace(id(1), resized)
            } else {
                diagram.clone()
            }
        }),
        "`Resize` is `replace` with the figure resized"
    );

    assert_eq!(
        after(&Command::SetStroke {
            id: id(1),
            stroke: Stroke::from("double"),
        }),
        after_operating(|diagram| {
            let updated = diagram
                .get(id(1))
                .map(|shape| shape.with_stroke(Stroke::from("double")));
            if let Some(updated) = updated {
                diagram.replace(id(1), updated)
            } else {
                diagram.clone()
            }
        }),
        "`SetStroke` is `replace` with the figure's stroke changed"
    );

    assert_eq!(
        after(&Command::SetText {
            id: id(1),
            text: Some(Text::new("hi").unwrap()),
        }),
        after_operating(|diagram| {
            let updated = diagram
                .get(id(1))
                .map(|shape| shape.with_text(Some(Text::new("hi").unwrap())));
            if let Some(updated) = updated {
                diagram.replace(id(1), updated)
            } else {
                diagram.clone()
            }
        }),
        "`SetText` is `replace` with the figure's text changed"
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
                        from: Endpoint::reference(
                            monospace_diagram::Reference::new(
                                id(1),
                                Anchor::Bottom,
                                Delta { dx: 0, dy: 0 },
                                0,
                            ),
                            from.terminal,
                        ),
                        to,
                        stroke,
                    },
                )
            } else {
                diagram.clone()
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
        after(&Command::Backward { id: id(2) }),
        after_operating(|diagram| diagram.backward(id(2))),
        "`Backward` is `backward`"
    );

    assert_eq!(
        after(&Command::ToFront { id: id(1) }),
        after_operating(|diagram| diagram.to_front(id(1))),
        "`ToFront` is `to_front`"
    );

    assert_eq!(
        after(&Command::ToBack { id: id(2) }),
        after_operating(|diagram| diagram.to_back(id(2))),
        "`ToBack` is `to_back`"
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
        Command::Backward { id: id(2) },
        Command::ToFront { id: id(1) },
        Command::ToBack { id: id(2) },
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
        Command::Resize {
            id: id(7),
            by: Delta { dx: 2, dy: 1 },
        },
        Command::Remove { id: id(2) },
        Command::Remove { id: id(7) },
    ] {
        session = session.perform(command.clone());
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

    let (copy, issued) = diagram.clone().add(an_extra_box());
    assert_eq!(
        issued,
        id(4),
        "no command reached `add`, so the next ordinal is the one the fixture issued"
    );
    // And the copy is where the addition landed: a copy advances its own counter and the diagram
    // the session hands out is not the one that was added to.
    assert!(
        copy.get(issued).is_some(),
        "the copy holds the figure that was added to it"
    );
    assert!(
        diagram.get(issued).is_none(),
        "and the diagram the session hands out does not"
    );
}

/// Rule 5 — a command that changes nothing is not a step.
///
/// **The specification's fifth line**, a command naming a figure the diagram does not hold. It
/// changes nothing, records no step, and leaves what was ahead of the position untouched — which is
/// the rule as a caller meets it.
#[test]
fn a_command_naming_nothing_records_no_step() {
    let mut session = Session::new(one_box(ORIGIN));
    let as_written = picture(&session);

    session = session.perform(Command::Move {
        id: id(7),
        by: Delta { dx: 1, dy: 0 },
    });

    assert_eq!(
        picture(&session),
        as_written,
        "a command naming a figure the diagram does not hold changes nothing"
    );
    assert!(!session.can_undo(), "and records no step");
    assert!(
        !session.can_redo(),
        "and leaves nothing ahead of a position that did not move"
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
        session = session.perform(Command::Move {
            id: id(1),
            by: Delta { dx: 2, dy: 0 },
        });

        forwards.push(picture(&session));
    }

    let mut backwards = Vec::new();
    while session.can_undo() {
        session = session.undo();
        backwards.push(picture(&session));
    }

    let expected: Vec<String> = forwards[..3].iter().rev().cloned().collect();
    assert_eq!(
        backwards, expected,
        "three commands left three steps, and giving them back one at a time walks the pictures \
         back in the order they were made"
    );
}

/// Rule 7 — undo restores the diagram one step back, whole.
///
/// **The specification's second and third blocks.** The picture an undo produces is the one the
/// session held before any command, byte for byte — not a reconstruction of it, and not the box
/// put back where it was, which is what an inverted command would have had to be. **One command is
/// what makes this the end of the history rather than the middle of it**, so what an undo reaches
/// here is the diagram the session was created over, which is the one state no step holds.
#[test]
fn undo_restores_the_diagram_one_step_back() {
    let mut session = Session::new(one_box(ORIGIN));

    session = session.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    });

    assert_eq!(
        picture(&session),
        FOUR_RIGHT,
        "the figure moved four to the right"
    );
    assert!(session.can_undo(), "which left a step behind it");

    session = session.undo();

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
/// ahead holds the state the command left, and the diagram is that state again.
#[test]
fn redo_restores_the_diagram_the_step_ahead_holds() {
    let mut session = Session::new(one_box(ORIGIN));

    session = session.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    });

    session = session.undo();
    session = session.redo();

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

    session = session.undo();

    assert_eq!(picture(&session), as_written, "nothing was given back");
    assert!(!session.can_undo(), "and nothing is behind the position");
}

/// Rule 9 — redoing with no step ahead changes nothing.
#[test]
fn redo_with_nothing_ahead_changes_nothing() {
    let mut session = Session::new(one_box(ORIGIN));

    session = session.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    });

    let as_left = picture(&session);

    session = session.redo();

    assert_eq!(picture(&session), as_left, "nothing was carried out again");
    assert!(!session.can_redo(), "and nothing is ahead of the position");
}

/// Rule 11 of `specs/212` — a step holds the diagram the command left.
///
/// **The step is the state a command produced, and giving the change back is what shows it.** A
/// session that recorded nothing but the state it was already holding would satisfy every
/// assertion here on its own, so the ones that matter are the ones where a state is reached *from
/// both ends*: the state the session holds, the state one give back reaches, and the state two give
/// back reaches. **All three directions are asserted**, because a history that kept only one of the
/// two states satisfies either assertion alone.
///
/// **A second command on top of the first**, so the state the first left is what the second found
/// and the walk back reaches three states rather than the same one twice. The deltas are four and
/// two columns, which is why `AS_WRITTEN`, `FOUR_RIGHT` and `SIX_RIGHT` are the three pictures
/// rather than three literals written out again.
#[test]
fn a_step_holds_the_diagram_the_command_left() {
    let session = Session::new(one_box(ORIGIN));
    let after_one = session.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    });
    let after_two = after_one.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 2, dy: 0 },
    });

    assert_eq!(
        picture(&session),
        AS_WRITTEN,
        "the session the commands were not given is the one it was created over"
    );
    assert_eq!(
        picture(&after_two),
        SIX_RIGHT,
        "and both commands were carried out"
    );
    assert_eq!(
        picture(&after_two.undo()),
        FOUR_RIGHT,
        "one give back is the state the first command left, which is the step behind the position"
    );
    assert_eq!(
        picture(&after_two.redo()),
        SIX_RIGHT,
        "and the step ahead is the state the second left"
    );
    assert_eq!(
        picture(&after_two.undo().undo()),
        AS_WRITTEN,
        "giving both back reaches the diagram as it was rather than the one in between twice"
    );
}

/// Rule 13 of `specs/212` — the history is a line and a position cannot disagree with it.
///
/// **The whole walk both ways, asking at every place whether there is something behind and whether
/// there is something ahead.** The two answers are the position read two ways, so a session that
/// disagreed with itself about where it stood would be caught by one of them; and the two walks
/// are asserted to be reverses of each other, which is what a line is and what a pair of stacks
/// with a shared position could also be — so the pictures are what says it.
///
/// **A place in the middle has both answers true**, and that is asserted rather than left out: it
/// is the only place where `can_undo` and `can_redo` can disagree with each other if one of them
/// is derived from the wrong end.
#[test]
fn the_history_is_a_line_a_position_cannot_disagree_with() {
    let start = Session::new(one_box(ORIGIN));
    let after_one = start.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    });
    let after_two = after_one.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 2, dy: 0 },
    });

    assert!(
        after_two.can_undo() && !after_two.can_redo(),
        "at the newest position there is something behind and nothing ahead"
    );

    let mut backwards = vec![picture(&after_two)];
    let mut behind = after_two.clone();
    while behind.can_undo() {
        behind = behind.undo();
        backwards.push(picture(&behind));
    }
    assert_eq!(
        backwards,
        vec![SIX_RIGHT, FOUR_RIGHT, AS_WRITTEN],
        "walking back passes through every state in the order it was made, the other way round"
    );

    assert!(
        !behind.can_undo(),
        "and at the oldest there is nothing behind"
    );
    assert!(
        behind.can_redo(),
        "while everything that was walked over is ahead, which is the same position read twice"
    );
    assert_eq!(
        picture(&behind.undo()),
        AS_WRITTEN,
        "and asking again at the oldest end answers with the session as it stands"
    );

    let mut forwards = vec![picture(&behind)];
    let mut ahead = behind;
    while ahead.can_redo() {
        ahead = ahead.redo();
        forwards.push(picture(&ahead));
    }
    assert_eq!(
        forwards,
        vec![AS_WRITTEN, FOUR_RIGHT, SIX_RIGHT],
        "walking forward retraces the same pictures in the other order"
    );
    assert!(
        !ahead.can_redo(),
        "and at the newest end there is nothing ahead"
    );
    assert_eq!(
        picture(&ahead.redo()),
        SIX_RIGHT,
        "and asking again at the newest end answers with the session as it stands"
    );
}

/// Rule 10 — a command carried out with steps ahead of the position discards them.
///
/// **Move, move, give one back, and move again**, which is the only way a position can be behind
/// the end. What is held is not that `redo` is unavailable afterwards — that alone would also be
/// true of a history truncated rather than discarded — but that **the undo out of the new command
/// reaches the state the first move left and not the state the second one left**, which is what the
/// discard dropped. The two differ by one cell, and the walk back over the discarded step would
/// land on the wrong one.
#[test]
fn a_command_with_steps_ahead_discards_them() {
    let mut session = Session::new(one_box(ORIGIN));

    session = session.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    });

    session = session.perform(Command::Move {
        id: id(1),
        by: Delta { dx: -1, dy: 0 },
    });

    session = session.undo();
    assert_eq!(
        picture(&session),
        FOUR_RIGHT,
        "one give back is the state the first move left"
    );

    session = session.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 2, dy: 0 },
    });

    assert_eq!(
        picture(&session),
        SIX_RIGHT,
        "the new command was carried out from where the session stood"
    );
    assert!(
        !session.can_redo(),
        "the step that was ahead of the position was discarded"
    );

    session = session.undo();
    assert_eq!(
        picture(&session),
        FOUR_RIGHT,
        "one back is the state the first move left"
    );
    assert_ne!(
        picture(&session),
        ONE_RIGHT,
        "and not the state the discarded step held, which stood one cell nearer"
    );

    session = session.undo();
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

    session = session.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    });

    assert!(
        (session.can_undo(), session.can_redo()) == (true, false),
        "after one command there is a step behind and none ahead"
    );
    assert!(
        session.can_undo(),
        "asked again, and the picture is untouched by being asked"
    );

    session = session.undo();

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

    let mut as_created = Buffer::new(Window {
        origin: ORIGIN,
        size: EXAMPLE,
    });
    diagram.draw(&mut as_created);

    let session = Session::new(diagram);
    let mut borrowed = Buffer::new(Window {
        origin: ORIGIN,
        size: EXAMPLE,
    });
    session.diagram().draw(&mut borrowed);

    assert_eq!(
        render(&borrowed, &GlyphCatalog::light(), borrowed.window()),
        render(&as_created, &GlyphCatalog::light(), as_created.window()),
        "a drawing of what a session holds is a drawing of the diagram it was created over"
    );
    assert_eq!(
        render(&borrowed, &GlyphCatalog::light(), borrowed.window()),
        AS_WRITTEN,
        "and it is the picture that diagram draws"
    );

    // And a copy is a diagram like any other: free of the session and unconnected to it, which is
    // what §7 is about. Nothing here is handed out but a borrow, so the copy is the caller's own.
    let theirs = session.diagram().clone();
    let mut session = session;
    session = session.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 4, dy: 0 },
    });

    assert_eq!(
        picture_in(&theirs, EXAMPLE),
        AS_WRITTEN,
        "the copy did not move with the session's diagram"
    );
    assert_eq!(picture(&session), FOUR_RIGHT, "which did");
}

#[test]
fn a_resize_command_changes_the_size() {
    let session = Session::new(the_fixture()).perform(Command::Resize {
        id: id(1),
        by: Delta { dx: 2, dy: 1 },
    });

    let Shape::Box { size, .. } = session.diagram().get(id(1)).expect("box is still there") else {
        panic!("expected box");
    };

    assert_eq!(
        *size,
        Size {
            width: 6,
            height: 4
        }
    );
}

#[test]
fn a_resize_naming_nothing_leaves_the_diagram_untouched_and_records_no_step() {
    let session = Session::new(the_fixture()).perform(Command::Resize {
        id: id(999),
        by: Delta { dx: 2, dy: 1 },
    });
    assert_eq!(
        picture_in(session.diagram(), FIXTURE),
        picture_in(&the_fixture(), FIXTURE)
    );
    assert!(!session.can_undo(), "and no step was recorded");
}

#[test]
fn a_resize_at_the_smallest_size_records_no_step() {
    let diagram = Diagram::new().add(smallest_box(ORIGIN)).0;
    let session = Session::new(diagram);
    let before = picture_in(session.diagram(), EXAMPLE);

    let shrunk = session.perform(Command::Resize {
        id: id(1),
        by: Delta { dx: -1, dy: -1 },
    });
    assert_eq!(
        picture_in(shrunk.diagram(), EXAMPLE),
        before,
        "shrinking a 2×2 box changes nothing"
    );
    assert!(!shrunk.can_undo(), "and records no step");

    let grown = shrunk.perform(Command::Resize {
        id: id(1),
        by: Delta { dx: 1, dy: 0 },
    });
    assert!(
        grown.can_undo(),
        "but growing it again records its step as any other command"
    );
}

#[test]
fn hanging_an_end_where_it_already_hangs_records_no_step() {
    let session = Session::new(the_fixture());
    let initial_pic = picture_in(session.diagram(), FIXTURE);

    let hung = session.perform(Command::Hang {
        id: id(3),
        from: id(1),
        anchor: Anchor::Bottom,
    });
    assert!(hung.can_undo(), "hanging an end records its step");

    let hung_again = hung.perform(Command::Hang {
        id: id(3),
        from: id(1),
        anchor: Anchor::Bottom,
    });
    assert_eq!(
        picture_in(hung_again.diagram(), FIXTURE),
        picture_in(hung.diagram(), FIXTURE),
        "hanging it where it already hangs changes nothing"
    );

    let undone = hung_again.undo();
    assert_eq!(
        picture_in(undone.diagram(), FIXTURE),
        initial_pic,
        "so a single undo returns to the initial state"
    );
}

#[test]
fn a_forward_of_the_front_most_figure_records_no_step() {
    let session = Session::new(the_fixture());
    let initial_pic = picture_in(session.diagram(), FIXTURE);

    let no_op = session.perform(Command::Forward { id: id(3) });
    assert_eq!(
        picture_in(no_op.diagram(), FIXTURE),
        initial_pic,
        "forwarding the front-most figure changes nothing"
    );
    assert!(!no_op.can_undo(), "and records no step");

    let moved = session.perform(Command::Forward { id: id(1) });
    assert!(
        moved.can_undo(),
        "forwarding an obscured figure records a step"
    );
}

#[test]
fn a_remove_naming_nothing_records_no_step() {
    let session = Session::new(one_box(ORIGIN));
    let initial_pic = picture(&session);

    let after_remove = session.perform(Command::Remove { id: id(9) });
    assert_eq!(picture(&after_remove), initial_pic);
    assert!(!after_remove.can_undo());
}

#[test]
fn a_move_by_nothing_records_no_step() {
    let session = Session::new(one_box(ORIGIN));
    let initial_pic = picture(&session);

    let after_move = session.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 0, dy: 0 },
    });
    assert_eq!(picture(&after_move), initial_pic);
    assert!(!after_move.can_undo());
}

#[test]
fn a_command_that_changes_nothing_leaves_what_was_ahead() {
    let session = Session::new(one_box(ORIGIN));
    let moved = session.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 2, dy: 0 },
    });
    let moved_pic = picture(&moved);

    let undone = moved.undo();
    assert!(undone.can_redo());

    let no_op = undone.perform(Command::Move {
        id: id(9),
        by: Delta { dx: 1, dy: 0 },
    });
    assert!(
        no_op.can_redo(),
        "a command that changed nothing preserves what was ahead"
    );

    let redone = no_op.redo();
    assert_eq!(
        picture(&redone),
        moved_pic,
        "and redo restores the state that was ahead"
    );
}

#[test]
fn two_resizes_are_two_steps() {
    let session = Session::new(one_box(ORIGIN));
    let after_one = session.perform(Command::Resize {
        id: id(1),
        by: Delta { dx: 1, dy: 0 },
    });
    let after_two = after_one.perform(Command::Resize {
        id: id(1),
        by: Delta { dx: 0, dy: 1 },
    });

    let Shape::Box { size: size_two, .. } = after_two.diagram().get(id(1)).expect("box") else {
        panic!("expected box");
    };
    assert_eq!(
        *size_two,
        Size {
            width: 5,
            height: 4
        }
    );

    let undone = after_two.undo();
    let Shape::Box { size: size_one, .. } = undone.diagram().get(id(1)).expect("box") else {
        panic!("expected box");
    };
    assert_eq!(
        *size_one,
        Size {
            width: 5,
            height: 3
        }
    );

    let undone_twice = undone.undo();
    let Shape::Box {
        size: size_orig, ..
    } = undone_twice.diagram().get(id(1)).expect("box")
    else {
        panic!("expected box");
    };
    assert_eq!(
        *size_orig,
        Size {
            width: 4,
            height: 3
        }
    );
}

#[test]
fn setting_a_figures_stroke_records_one_step() {
    let session = Session::new(one_box(ORIGIN));
    let updated = session.perform(Command::SetStroke {
        id: id(1),
        stroke: Stroke::from("double"),
    });

    assert!(
        updated.can_undo(),
        "setting a different stroke records a step"
    );
    let Shape::Box { stroke, .. } = updated.diagram().get(id(1)).expect("box") else {
        panic!("expected box");
    };
    assert_eq!(*stroke, Stroke::from("double"));

    let undone = updated.undo();
    let Shape::Box {
        stroke: orig_stroke,
        ..
    } = undone.diagram().get(id(1)).expect("box")
    else {
        panic!("expected box");
    };
    assert_eq!(*orig_stroke, Stroke::from("light"));
}

#[test]
fn setting_the_stroke_a_figure_already_has_records_no_step() {
    let session = Session::new(one_box(ORIGIN));
    let moved = session.perform(Command::Move {
        id: id(1),
        by: Delta { dx: 1, dy: 0 },
    });
    let moved_pic = picture(&moved);

    let undone = moved.undo();
    assert!(undone.can_redo());

    let no_op = undone.perform(Command::SetStroke {
        id: id(1),
        stroke: Stroke::from("light"),
    });
    assert!(
        no_op.can_redo(),
        "setting the stroke the figure already has preserves what was ahead"
    );

    let redone = no_op.redo();
    assert_eq!(
        picture(&redone),
        moved_pic,
        "and redo restores the state that was ahead"
    );
}

#[test]
fn setting_the_stroke_of_a_figure_the_diagram_does_not_hold_changes_nothing() {
    let session = Session::new(one_box(ORIGIN));
    let initial_pic = picture(&session);

    let no_op = session.perform(Command::SetStroke {
        id: id(999),
        stroke: Stroke::from("heavy"),
    });
    assert_eq!(picture(&no_op), initial_pic);
    assert!(!no_op.can_undo(), "and no step was recorded");
}

#[test]
fn a_set_text_command_changes_only_a_box() {
    let text = Text::new("hi").expect("\"hi\" holds no control character");

    assert_eq!(
        after(&Command::SetText {
            id: id(1),
            text: Some(text.clone()),
        }),
        after_operating(|diagram| {
            let updated = diagram
                .get(id(1))
                .map(|shape| shape.with_text(Some(text.clone())));
            if let Some(updated) = updated {
                diagram.replace(id(1), updated)
            } else {
                diagram.clone()
            }
        }),
        "`SetText` is `replace` with the figure's text changed"
    );
}

#[test]
fn setting_the_same_text_records_no_step() {
    let text = Text::new("hi").expect("\"hi\" holds no control character");
    let with_text = Session::new(one_box(ORIGIN)).perform(Command::SetText {
        id: id(1),
        text: Some(text.clone()),
    });

    let twice = with_text.perform(Command::SetText {
        id: id(1),
        text: Some(text),
    });
    let back = twice.undo();

    let Shape::Box { text: restored, .. } = back.diagram().get(id(1)).expect("box") else {
        panic!("expected box");
    };
    assert_eq!(
        *restored, None,
        "the second command recorded a step, so one undo stayed on the same text"
    );
}

#[test]
fn clearing_a_text_is_one_step() {
    let text = Text::new("hi").expect("\"hi\" holds no control character");
    let with_text = Session::new(one_box(ORIGIN)).perform(Command::SetText {
        id: id(1),
        text: Some(text.clone()),
    });
    let cleared = with_text.perform(Command::SetText {
        id: id(1),
        text: None,
    });

    assert!(cleared.can_undo(), "clearing the text recorded no step");
    let Shape::Box { text: now, .. } = cleared.diagram().get(id(1)).expect("box") else {
        panic!("expected box");
    };
    assert_eq!(*now, None);

    let undone = cleared.undo();
    let Shape::Box { text: restored, .. } = undone.diagram().get(id(1)).expect("box") else {
        panic!("expected box");
    };
    assert_eq!(*restored, Some(text), "one undo did not give the text back");
}

/// A movement at the end it is already at changes nothing and records no step.
#[test]
fn the_moves_that_change_nothing_record_no_step() {
    let session = Session::new(the_fixture());
    let initial_pic = picture_in(session.diagram(), FIXTURE);

    for command in [
        Command::Backward { id: id(1) },
        Command::ToFront { id: id(3) },
        Command::ToBack { id: id(1) },
    ] {
        let no_op = session.perform(command.clone());
        assert_eq!(
            picture_in(no_op.diagram(), FIXTURE),
            initial_pic,
            "{command:?} changed the picture"
        );
        assert!(!no_op.can_undo(), "{command:?} recorded a step");
    }
}

/// `Home` and `End` are one command each, so one undo reverses them whole.
#[test]
fn each_absolute_move_is_one_step_that_undo_reverses() {
    for command in [
        Command::ToFront { id: id(1) },
        Command::ToBack { id: id(2) },
    ] {
        let session = Session::new(the_fixture());
        let before = picture_in(session.diagram(), FIXTURE);
        let moved = session.perform(command.clone());

        assert!(moved.can_undo(), "{command:?} recorded no step");
        assert!(!moved.can_redo(), "{command:?} recorded more than one step");
        assert_eq!(
            picture_in(moved.undo().diagram(), FIXTURE),
            before,
            "one undo did not reverse {command:?}"
        );
    }
}
