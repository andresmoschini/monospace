//! Minimal non-interactive command-line front end for Monospace.
//!
//! It holds no domain logic of its own: it reads a description with `monospace-description`, which
//! turns the format's JSON into a `monospace_diagram::Diagram`, and draws it.
//!
//! # Design notes
//!
//! **A file is rendered once; only the bare run demonstrates.** Given a path this prints one
//! picture and nothing else — no caption, no shape moved forward, and nothing walked back. Given
//! no argument it prints the shipped demonstration: **eight** captioned pictures — as written,
//! with the back-most shape moved one place toward the front, with that same shape displaced,
//! with that same shape taken out, with the arrow rehung from the box it already pointed at and
//! that box displaced, with the arrow itself displaced as well, with the box the arrow hangs from
//! taken out, and with whichever shape the first picture records as deciding the position
//! `(20, 2)` taken out instead — and then **one picture per step, walked back**, until the session
//! holds the picture it began with.
//!
//! That split exists because the changes the bare run shows say something only about the
//! shipped demonstration, whose first two entries are two partially overlapping opaque boxes.
//! Applied to a file someone hands the binary they are a demonstration's assumptions imposed on
//! their description. It is also what lets `cargo xtask render` embed this output in a document: a
//! picture that goes into a Markdown fence cannot arrive wrapped in prose.
//!
//! **The whole demonstration runs on a `Session`, and this is the one caller in the repository
//! that fits without anything invented.** Every change below is a `Command` performed against it
//! rather than an operation on a diagram it holds, which is what turns the editing layer's
//! ownership rule from a sentence into something exercised. The seven pictures before the walk
//! back are the diagram model's own and this crate contributes only the lines under them.
//!
//! **The walk back is one picture per step, and a step exists only where the record named a
//! shape.** So the shipped description walks eight and an empty one walks none, and what a reader
//! counts is what the session holds rather than what this function chose to draw.
//!
//! **The demonstration reads its shapes out of the drawing rather than naming them, and there is
//! nothing written in beside the lookup to catch a wrong answer.** Every shape it acts on comes from
//! `Buffer::owner` at one of [`THE_BACK_MOST_AT`], [`THE_HUNG_FROM_AT`], [`THE_ARROW_AT`] and
//! [`THE_CROSSING_AT`], and where the record names none the step skips itself. An earlier version took
//! a written name and used it wherever the record came back empty, and **that makes a wrong offset
//! indistinguishable from a right one**: measured with `THE_BACK_MOST_AT` moved onto a cell no shape
//! wrote, the demonstration still printed eight pictures that read like a demonstration and **no test
//! failed**, because the written name went on answering every question the offset was supposed to. It
//! is gone, and the four offsets are now constants
//! `the_four_offsets_answer_the_shapes_the_demonstration_acts_on` quotes the answers of — the
//! coordinate is read from the code beside it and the identity at it is written out, which is the one
//! direction of that split that is not circular.
//!
//! **A step skipped is a step that still prints its picture**, which is what keeps a description the
//! demonstration can say nothing about working at all: an empty one, and a one-box one whose
//! four-by-three window does not reach three of the four offsets.
//!
//! **The window is named on the command line, and `--size` and `--origin` are how.** Which part of
//! a diagram to draw is the caller's question — see [`docs/diagram-model.md`](../../../docs/diagram-model.md),
//! which holds that the diagram sizes nothing and measures nothing — so two flags answer it and
//! neither is required. **A path given neither is drawn into this binary's own window**, the
//! demonstration's, which is a constant here rather than a rule any file states: every marker
//! states the size it is drawn at, so the default is a convenience for a person running the binary
//! and nothing a generated picture depends on.

use std::process::ExitCode;

use monospace_core::{Buffer, GlyphCatalog, Offset, Pos, Size};
use monospace_description::parse;
use monospace_diagram::{Anchor, Delta, Diagram};
use monospace_editing::{Command, Session};

/// The shipped demonstration description, embedded at compile time so the no-argument run works
/// from any working directory and from a binary copied outside a checkout.
const DEMO: &str = include_str!("../assets/demo.json");

/// The four offsets [`demonstrate`] asks the first drawing about.
///
/// **Named here rather than written into `demonstrate` so that a test can ask the same question about
/// the same numbers.** The offset is the thing such a test is checking, so it belongs beside the code
/// that uses it and not restated in a test where it would drift; the *answers* are the part a test
/// quotes and can fail on — see `the_four_offsets_answer_the_shapes_the_demonstration_acts_on`.
///
/// **The first three name a cell only that one shape wrote**, which is what makes the answer that
/// shape rather than a fact about the order: a cell two of them wrote resolves to whichever is in
/// front, so asking there would be asking about a crossing instead. Measured against the shipped
/// description — `(0, 0)` is `1`'s own top-left corner, `(9, 4)` is `3`'s bottom-left corner, and
/// `(14, 3)` is the middle of the arrow's own horizontal run.
///
/// **The fourth is deliberately the opposite case**: a cell two shapes *did* write, where the
/// character on screen cannot say which. `6`'s bottom border runs through it and `5`'s left border
/// runs down it, so it renders `┼`.
const THE_BACK_MOST_AT: Offset = Offset { x: 0, y: 0 };
const THE_HUNG_FROM_AT: Offset = Offset { x: 9, y: 4 };
const THE_ARROW_AT: Offset = Offset { x: 14, y: 3 };
const THE_CROSSING_AT: Offset = Offset { x: 20, y: 2 };

#[cfg(test)]
mod sweep;

/// The usage line, printed for anything [`parse_args`] refuses.
const USAGE: &str = "usage: monospace-cli [--size <width>x<height>] [--origin <x>,<y>] [path]";

/// The window a picture is drawn into, as the pair the two flags name.
///
/// **A name for the pair rather than the pair in a signature**, because the pair is what `render`
/// takes and what the flags build, and `parse_args` has to say both. It is private to this binary
/// and it is not the format crate's `Window` — that one is the type leaving the description, and
/// this is the caller's own.
type Window = (Pos, Size);

/// The window the demonstration draws into, and the one a path is drawn into when the command line
/// names none.
///
/// **A constant in this binary rather than a rule any file states**, which is why it is a default
/// and not a requirement. `cargo xtask render` always passes a size, because every marker states
/// one, so the only person this window serves is one running the binary by hand — and a default
/// only a human reaches for can be wrong without consequence. **It is not the terminal's own size
/// either**: a picture whose width depends on the machine that rendered it is a picture
/// `cargo xtask render --check` fails on for every reader but the one who wrote it.
const DEMO_WINDOW: Window = (
    Pos { x: 0, y: 0 },
    Size {
        width: 50,
        height: 13,
    },
);

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let (path, asked) = match parse_args(&args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    let text = match &path {
        None => DEMO.to_owned(),
        Some(path) => match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                eprintln!("{path}: {error}");
                return ExitCode::FAILURE;
            }
        },
    };

    let diagram = match parse(&text) {
        Ok((diagram, _)) => diagram,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    // **The flags name the window a path is drawn into, and a demonstration has its own.** The two
    // callers are not the same caller. `cargo xtask render` always passes a size, because every
    // marker states one, so the default is never what a generated picture uses; and a bare run
    // serves a person watching it, who gets the demonstration's own window whatever they typed.
    let (origin, size) = match asked {
        Some(asked) if path.is_some() => asked,
        _ => DEMO_WINDOW,
    };

    print!(
        "{}",
        if path.is_none() {
            demonstrate(diagram, origin, size)
        } else {
            render_once(&diagram, origin, size)
        }
    );
    ExitCode::SUCCESS
}

/// Reads the arguments into the path to draw and the window to draw it into, in either order.
///
/// **Two flags and a path, parsed by hand.** `xtask` takes no dependencies because it guards the
/// dependency policy, and this binary is what `cargo xtask render` runs once per generated picture
/// — twenty-two of them on a full pass. A crate to read `20x7` is a crate to build and read before
/// every one of those.
///
/// **Each flag names one half of the window and neither half is required.** `--size` alone is a
/// window at `(0, 0)`, `--origin` alone is a window of the default size somewhere else, and both
/// together are what `cargo xtask render` passes for every marker it rewrites. A flag given twice is
/// refused rather than read as the last word: a caller that says `--size 4x3 --size 6x4` has said
/// two windows and deserves to be asked which.
///
/// **A value that is not the shape the flag takes is refused and quoted.** `--size 20-7` names
/// nothing this binary can draw into, and answering it with half the window would be a picture
/// rather than an error.
fn parse_args(args: &[String]) -> Result<(Option<String>, Option<Window>), String> {
    let mut path: Option<String> = None;
    let mut origin: Option<Pos> = None;
    let mut size: Option<Size> = None;
    let mut rest = args.iter();

    while let Some(argument) = rest.next() {
        match argument.as_str() {
            "--size" => {
                let Some(value) = rest.next() else {
                    return Err(format!("`--size` was given no size\n{USAGE}"));
                };
                if size.is_some() {
                    return Err(format!("`--size` was given twice\n{USAGE}"));
                }
                size = Some(parse_size(value)?);
            }
            "--origin" => {
                let Some(value) = rest.next() else {
                    return Err(format!("`--origin` was given no origin\n{USAGE}"));
                };
                if origin.is_some() {
                    return Err(format!("`--origin` was given twice\n{USAGE}"));
                }
                origin = Some(parse_origin(value)?);
            }
            _ if argument.starts_with('-') => {
                return Err(format!("`{argument}` is not an option\n{USAGE}"));
            }
            _ if path.is_some() => {
                return Err(format!("`{argument}` is a second path\n{USAGE}"));
            }
            _ => path = Some(argument.clone()),
        }
    }

    let window = size.map(|size| (origin.unwrap_or(Pos { x: 0, y: 0 }), size));
    Ok((path, window))
}

/// The `20x7` of `--size`, as the pair of numbers a window is drawn into.
///
/// **Both halves are numbers or neither is.** `20-7`, `20` and `x7` are refused by name rather than
/// read as a width beside a height nobody gave, because half a window is a picture that looks like
/// an answer.
fn parse_size(text: &str) -> Result<Size, String> {
    let said = || format!("`--size` takes `<width>x<height>` and not `{text}`\n{USAGE}");
    let (width, height) = text.split_once('x').ok_or_else(said)?;

    Ok(Size {
        width: width.parse().map_err(|_| said())?,
        height: height.parse().map_err(|_| said())?,
    })
}

/// The `-3,-2` of `--origin`, as the corner the window is drawn from.
///
/// **The two halves may be negative**, because a window is not required to start at the top left of
/// the diagram: `specs/086` draws one at `(-3, -2)` so that a box at `(0, 0)` is two rows and three
/// columns inside it.
fn parse_origin(text: &str) -> Result<Pos, String> {
    let said = || format!("`--origin` takes `<x>,<y>` and not `{text}`\n{USAGE}");
    let (x, y) = text.split_once(',').ok_or_else(said)?;

    Ok(Pos {
        x: x.parse().map_err(|_| said())?,
        y: y.parse().map_err(|_| said())?,
    })
}

/// Renders `diagram` into `size` at `origin`, as one picture and nothing else.
fn render_once(diagram: &Diagram, origin: Pos, size: Size) -> String {
    picture(diagram, &glyph_catalog(), origin, size)
}

/// Draws `diagram` into a fresh window of `size` and hands back the buffer beside its rendering.
/// Drawing changes nothing about the diagram, which is what lets the demonstration draw one diagram
/// many times and change it between two of them.
///
/// **The buffer is handed back rather than dropped** because the demonstration asks the first
/// drawing a question: which shape decided the position it later takes out. Rendering alone cannot
/// answer that, and drawing the diagram a second time to ask would be a second drawing rather than
/// the first one.
fn drawn(diagram: &Diagram, catalog: &GlyphCatalog, origin: Pos, size: Size) -> (Buffer, String) {
    let mut buffer = Buffer::new(origin, size);
    diagram.draw(&mut buffer);
    let text = monospace_core::render(&buffer, catalog, origin, size);
    (buffer, text)
}

/// The rendering of `diagram` drawn into a fresh window, for the steps that only need the picture.
///
/// A `&Diagram` rather than a `&Session`, because the session hands its diagram out borrowed and
/// nothing here needs the session itself: this is the whole of what the demonstration asks of the
/// editing layer, and it is the question a caller repainting on every event actually has.
fn picture(diagram: &Diagram, catalog: &GlyphCatalog, origin: Pos, size: Size) -> String {
    drawn(diagram, catalog, origin, size).1
}

/// Renders `diagram` as written, then again with its first entry moved one place toward the
/// front, then again with that same entry displaced, then again with it taken out, then once with the
/// arrow rehung from the box it already pointed at and that box displaced, then once more with the
/// arrow itself displaced, then once more again with the box the arrow hangs from taken out, and
/// once more again with whichever shape the first picture records as deciding `(20, 2)` taken out.
/// Each picture is under a caption, and then the whole run is walked back one picture per step.
///
/// This is the shipped demonstration's output, and every one of its pictures carries a caption.
/// The changes it shows are meaningful only for that description, which is why a file the binary is
/// handed goes through [`render_once`] instead.
///
/// **Every change below is a `Command` performed against one `Session`**, and none of them is an
/// operation on a diagram this function holds — the diagram arrives here, goes into the session and
/// is only ever read back through [`Session::diagram`]. The eight pictures before the walk back are
/// the diagram model's own; what this function contributes is the line under each one and the
/// commands above it.
///
/// The first four pictures are about one figure, the entry the description lists first. The fifth
/// and sixth are appended after them rather than interleaved, and are about two others; the seventh
/// is appended after those and is about one of them again, which is what makes it the sixth with the
/// box gone rather than a picture of its own arrangement. The eighth is the only step that **reads
/// the drawing rather than applying a change to the diagram**: it asks the first picture which shape
/// decided one position and takes that shape out, so a reader sees the record decide something
/// rather than being told that it does.
///
/// **The three identities the first seven steps need are read out of the first drawing rather than
/// written by hand.** A description names its shapes by the identity it wrote, so the demonstration
/// could always say which entry it meant — but "which entry" and "which shape owns that cell" are
/// different questions, and only the second one is what the rest of this function acts on. Asking
/// replaces a written assumption about this description with the record, which is what an interactive
/// front end would have to do and what the first seven steps have no way to check. A figure added
/// before any of the three would move the answer, and the pictures below it are what catch that.
fn demonstrate(diagram: Diagram, origin: Pos, size: Size) -> String {
    let catalog = glyph_catalog();
    let mut session = Session::new(diagram);

    // The first drawing, kept rather than thrown away: the three identities are read out of it, and
    // so is the shape the eighth picture takes out. **None of them is written out here**, and that is
    // what makes the offsets above checkable — a name in this function would answer every offset,
    // including a wrong one, and there would be nothing left to fail.
    //
    // Each answer is a four-byte ordinal rather than a borrow of one, so nothing has to outlive the
    // borrow `owner` hands back and the `.cloned()` this used to need is gone: `the_back_most` is
    // read four times across the steps below, after the buffer has been drawn into again.
    //
    // The diagram is read through the session from here on, and it is a borrow that outlives nothing:
    // every step below is a command, and a command is handed the session rather than the diagram.
    let (as_written, first_picture) = drawn(session.diagram(), &catalog, origin, size);
    let the_back_most = as_written.owner(THE_BACK_MOST_AT);
    let the_hung_from = as_written.owner(THE_HUNG_FROM_AT);
    let the_arrow = as_written.owner(THE_ARROW_AT);

    // The caption names the offsets and not the answers, because the answers are whatever the record
    // holds and a line of text cannot know.
    let first_caption = format!(
        "As written, asking which shape decided ({}, {}), ({}, {}) and ({}, {}):\n",
        THE_BACK_MOST_AT.x,
        THE_BACK_MOST_AT.y,
        THE_HUNG_FROM_AT.x,
        THE_HUNG_FROM_AT.y,
        THE_ARROW_AT.x,
        THE_ARROW_AT.y
    );

    let mut out = first_caption;
    out.push_str(&first_picture);

    // How far the third picture's figure moves. A fixed value this function carries rather than a
    // field in the description format or an argument on the binary, so a file's picture is exactly
    // the one it was. An assumption about this demonstration, like the reorder below, and not
    // a rule about descriptions: nothing in the format carries it, and a caller who wanted one
    // would have to argue for it there. The value lands the figure on cells the shipped description
    // already draws, which is what lets the third picture tell a displacement from the reorder the
    // second one shows.
    let by = Delta { dx: 0, dy: 3 };

    // How far the fifth picture's box moves, beside the delta above. The destination was measured
    // against the shipped description's own rows rather than chosen by eye: the box lands at
    // `{13, 2}` occupying `x 13..16, y 2..4`, and the only cells anything else writes in that
    // rectangle are the arrow's own arm at `(13, 3)` through `(15, 3)` — the arm the fifth picture
    // replaces. So the box lands on cells the picture's own change has cleared.
    let four_right = Delta { dx: 4, dy: 0 };

    // How far the sixth picture's arrow moves, beside the two deltas above and **reusing neither**.
    // Two down and not three, for the same reason the third picture's figure moves three rather
    // than four: this function holds its deltas deliberately rather than taking them from the format
    // or the binary, and three rows would land the arrow on cells the shipped description already
    // draws. Two rows lands it clear of both boxes, which is what the picture is for.
    let two_down = Delta { dx: 0, dy: 2 };

    // The reorder, the displacement and the removal all follow the shape the record named, and all
    // three are skipped when it named none — which is what a description with nothing at `(0, 0)`
    // does, and what an empty description does at all three offsets. The picture is printed either
    // way, so a description the demonstration can say nothing about still prints eight of them.
    //
    // **The guards are the demonstration's own and not a command's silence.** A command naming
    // nothing is a step like any other, so `Command::Remove { id }` on an identity the session's
    // diagram does not hold would still be recorded and still be walked back over; what these
    // guards skip is a step the record never named at all, which is a different case and is a
    // decision about this description rather than about editing. It is also why the walk back
    // below is shorter for a description with less in it.
    if let Some(the_back_most) = the_back_most {
        Command::Forward { id: the_back_most }.perform(&mut session);
    }
    out.push_str("\nWith the back-most shape moved one place forward:\n");
    out.push_str(&picture(session.diagram(), &catalog, origin, size));

    if let Some(the_back_most) = the_back_most {
        Command::Move {
            id: the_back_most,
            by,
        }
        .perform(&mut session);
    }
    out.push_str("\nWith that same shape displaced:\n");
    out.push_str(&picture(session.diagram(), &catalog, origin, size));

    if let Some(the_back_most) = the_back_most {
        Command::Remove { id: the_back_most }.perform(&mut session);
    }
    out.push_str("\nWith that same shape taken out:\n");
    out.push_str(&picture(session.diagram(), &catalog, origin, size));

    // The arrow's `from` is rehung from the box's right side — the point the description already
    // spells for it, so the fifth picture opens on the same picture the fourth does and the only
    // thing that changes about the arrow is where it starts. It is **read back** rather than
    // restated, and `Command::Hang` is what reads it back: the command rebuilds the one end that
    // changes and keeps `to`, the direction, the terminal and the stroke, which are then the
    // description's by construction. A caller writing the connector out by hand instead would be
    // agreeing with the description about four values, where a disagreement would change the
    // picture rather than fail.
    //
    // **The direction and the terminal are the ones the end already had and are not derived from
    // the anchor**: §6 of the diagram model says both are the caller's, and deriving the direction
    // from the side is [#89](https://github.com/andresmoschini/monospace/issues/89)'s. A `Hang`
    // carries no field for either, so the command has nowhere to put a different one.
    //
    // A description whose tenth entry is not a connector, one with no tenth entry at all, and one
    // whose record names nothing at `(14, 3)`, all have nothing to rehang. Two of those the guard
    // below catches by asking whether the record named anything at all; the third is a command
    // naming an identity the diagram does not hold, which changes nothing and is still a step.
    if let (Some(the_arrow), Some(the_hung_from)) = (the_arrow, the_hung_from) {
        Command::Hang {
            id: the_arrow,
            from: the_hung_from,
            anchor: Anchor::Right,
        }
        .perform(&mut session);
    }

    // And now the figure it hangs from moves, which is what the reference is for: the arrow lands
    // on the box's new side and re-routes to the end that did not move, because that endpoint is
    // still a point and a displacement reaches points.
    if let Some(the_hung_from) = the_hung_from {
        Command::Move {
            id: the_hung_from,
            by: four_right,
        }
        .perform(&mut session);
    }
    out.push_str("\nWith the arrow now hanging from that box, and the box displaced:\n");
    out.push_str(&picture(session.diagram(), &catalog, origin, size));

    // And now the arrow itself moves, which is the sixth picture and the reason the fifth exists:
    // at this point **both** of the arrow's endpoints are references — its `from` rehung above and
    // the shipped `to`, which names `5`'s bottom with an offset of one in each axis — so a
    // displacement that did not reach a reference's offsets would draw a picture byte for byte
    // identical to the fifth, and that is the defect this step exists to show is gone. The sixth is
    // therefore the fifth with the arrow two rows lower and **both boxes standing exactly where
    // they stood**, which is the claim a reader checks with their eyes.
    if let Some(the_arrow) = the_arrow {
        Command::Move {
            id: the_arrow,
            by: two_down,
        }
        .perform(&mut session);
    }
    out.push_str("\nWith the arrow displaced as well:\n");
    out.push_str(&picture(session.diagram(), &catalog, origin, size));

    // And the seventh takes the figure the arrow hangs from **away**, which is the sixth picture with
    // the box gone and **the arrow exactly where it stood**. `Remove` freezes the end that named the
    // box at the point it was resolving to, so the box leaves the picture and the connector does
    // not: the seventh is the sixth with twelve cells blanked and nothing else touched.
    //
    // This is the evidence the rule is for. Every picture above shows what the demonstration does to
    // itself, and the sixth removes a figure that holds no reference — so nothing the shipped binary
    // printed before this step showed the rule at all. The seventh is where a reader sees it.
    //
    // The `if let` is around **which** shape, and there has to be one, because the record may name
    // none. `3` is the box the demonstration hangs the arrow from — the same identity the fifth
    // picture displaced — so this step takes out exactly what the arrow hangs from.
    if let Some(the_hung_from) = the_hung_from {
        Command::Remove { id: the_hung_from }.perform(&mut session);
    }
    out.push_str("\nWith the box the arrow hangs from taken out:\n");
    out.push_str(&picture(session.diagram(), &catalog, origin, size));

    // And the eighth asks the picture which shape decided one position, and takes that shape out.
    //
    // **This is the only step here that reads a drawing**, and it is the evidence for the record: the
    // seven above all ask at an offset chosen in advance and act on whatever came back, so nothing
    // the binary printed before this showed what the record is for. Here nothing is chosen — `(20, 2)`
    // is asked for and the record answers, and the eighth is the seventh with whatever it named gone.
    //
    // **The offset is measured, not chosen by eye** — see [`THE_CROSSING_AT`]. `5` is listed after
    // `6` and so is in front, and the record names `6` — the shape behind — because `6` reached the
    // cell first. Taking `6` out leaves `5`'s arm standing alone, and `┼` becomes `│`, which is the
    // one reader can check with their eyes. Asking anywhere else would give a cell one shape wrote,
    // where the picture already says which shape it was.
    //
    // **Asked of the first drawing and not of the seventh**, which is the point: the record belongs to
    // the drawing that produced it, so a buffer the demonstration has since redrawn would still be
    // holding it and a drawing taken now would answer about something else. `6` is not one of the
    // three shapes the earlier steps touch, so what is removed here is the record's answer and not a
    // side effect of them.
    if let Some(named) = as_written.owner(THE_CROSSING_AT) {
        Command::Remove { id: named }.perform(&mut session);
    }
    let crossing_caption = format!(
        "\nWith the shape the picture names at ({}, {}) taken out:\n",
        THE_CROSSING_AT.x, THE_CROSSING_AT.y
    );
    out.push_str(&crossing_caption);
    out.push_str(&picture(session.diagram(), &catalog, origin, size));

    // And then the whole run is walked back, one picture per step, until there is nothing behind
    // the position left to give.
    //
    // **The loop asks `can_undo` rather than counting, so the number of pictures is what the session
    // holds and not what this function chose to draw.** Eight commands reach it for the shipped
    // description — including the two the fifth picture shares, which is what makes the count eight
    // where the forward count is seven — and none for an empty one, which then prints the same eight
    // pictures it always did and walks nowhere.
    //
    // **The last caption says what is true rather than what is counted**: after the final `undo`
    // there is nothing behind the position, so what the picture shows is the diagram the session
    // was created over, which is the first picture of this run. It is the same equality in the
    // output and in the test that checks it, and the walk back is where a reader sees undo work
    // rather than being told that it does.
    while session.can_undo() {
        session.undo();
        let caption = if session.can_undo() {
            "\nGiven back one step:\n"
        } else {
            "\nGiven back every step, which is the picture this run began with:\n"
        };
        out.push_str(caption);
        out.push_str(&picture(session.diagram(), &catalog, origin, size));
    }

    out
}

/// The glyph catalog the CLI renders with: the union of every glyph set the core does not ship
/// and the core's own light table.
fn glyph_catalog() -> GlyphCatalog {
    GlyphCatalog::union([
        GlyphCatalog::light(),
        monospace_glyph_sets::ascii(),
        monospace_glyph_sets::double(),
        monospace_glyph_sets::heavy(),
        monospace_glyph_sets::light_round(),
        monospace_glyph_sets::light_double(),
        monospace_glyph_sets::light_heavy(),
        monospace_glyph_sets::light_round_double(),
        monospace_glyph_sets::light_round_heavy(),
    ])
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;
    use std::ops::Range;

    use monospace_core::{
        BoxShape, Buffer, Glyph, GlyphCatalog, Layer, Offset, Pos, Shape, Size, StampMode, Stroke,
        render,
    };
    use monospace_diagram::ShapeId;

    use super::{demonstrate, drawn, glyph_catalog, parse_args, render_once};
    use crate::parse;

    /// The window the shipped demonstration is drawn at, as the pair the flags name.
    fn in_the_demo_window() -> (Pos, Size) {
        (
            Pos { x: 0, y: 0 },
            Size {
                width: 50,
                height: 13,
            },
        )
    }

    /// The picture `json` renders to, parsed first, drawn into the window its own file names.
    ///
    /// Two helpers rather than one that takes the pair, because every caller here has text and
    /// wants a `String` and none of them is about the window on its own.
    fn render_json(json: &str) -> String {
        let (diagram, window) = parse(json).expect("well-formed description");
        render_once(&diagram, window.origin, window.size)
    }

    /// The demonstration over `json`, parsed first, drawn into the window its own file names.
    fn demonstrate_json(json: &str) -> String {
        let (diagram, window) = parse(json).expect("well-formed description");
        demonstrate(diagram, window.origin, window.size)
    }

    /// An identity of the ordinal `ordinal`, which is how every test below names one.
    fn identity(ordinal: u32) -> ShapeId {
        ShapeId::new(NonZeroU32::new(ordinal).expect("no test names zero"))
    }

    fn one_box_json() -> &'static str {
        r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 2,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░" }
            ]
        }"#
    }

    /// A one-box description renders the same text as a `BoxShape` drawn directly with the same
    /// parameters.
    #[test]
    fn a_one_box_description_renders_the_same_as_a_box_shape_drawn_directly() {
        let (diagram, _) = parse(one_box_json()).expect("well-formed description");

        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 4,
            height: 3,
        };
        let mut buffer = Buffer::new(origin, size);
        BoxShape {
            at: origin,
            size,
            stroke: Stroke::from("light"),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));
        let expected = render(&buffer, &GlyphCatalog::light(), origin, size);

        assert_eq!(render_once(&diagram, origin, size), expected);
    }

    /// A file's picture is the demonstration's first picture with nothing around it: no caption
    /// line, no blank line, and no second picture.
    ///
    /// This is what lets `cargo xtask render` paste the output straight into a Markdown fence.
    #[test]
    fn rendering_once_is_the_demonstrations_first_picture_and_nothing_else() {
        let (first, ..) = demonstrated_pictures(one_box_json());

        let (diagram, window) = parse(one_box_json()).expect("well-formed description");
        assert_eq!(render_once(&diagram, window.origin, window.size), first);
    }

    /// Every captioned block of a demonstration's output, as a picture without its caption.
    ///
    /// **The whole run rather than its first eight**, because the walk back is eight of the blocks
    /// now and a helper that stopped at eight would have made it invisible to every test here. Each
    /// carries exactly the trailing newline `render` gives it: the last block already holds one,
    /// since nothing follows it, so it is stripped and put back rather than doubled.
    fn demonstrated_blocks(json: &str) -> Vec<String> {
        let output = demonstrate_json(json);
        output
            .trim_end_matches('\n')
            .split("\n\n")
            .map(|block| {
                let (_caption, picture) = block
                    .split_once('\n')
                    .expect("a caption line precedes each picture");
                format!("{}\n", picture.strip_suffix('\n').unwrap_or(picture))
            })
            .collect()
    }

    /// The demonstration's first **eight** pictures — the ones taken as it changes the diagram —
    /// found by the blank line between them and returned without their captions, so nothing here
    /// pins a caption's wording.
    ///
    /// **The first eight of however many there are**, which is what keeps every claim this file
    /// made about them a claim about the same pictures it made before the walk back arrived. The
    /// eight are then comparable with each other and with a picture drawn on its own.
    ///
    /// **The closure needed no change when the eighth arrived, and that is worth knowing rather than
    /// assuming**: it splits on the blank line, strips the trailing newline and puts one back, and
    /// the last block gets the normalization the others always got. It is also why the sixth
    /// compared **equal to the fifth** before the rule landed rather than one character apart —
    /// measured, and the reason no test pins the raw text.
    #[allow(clippy::type_complexity)]
    fn demonstrated_pictures(
        json: &str,
    ) -> (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    ) {
        let blocks = demonstrated_blocks(json);
        let mut taken = 0;
        let mut next_picture = || {
            let block = blocks
                .get(taken)
                .expect("eight captioned pictures, each after a blank line")
                .clone();
            taken += 1;
            block
        };

        (
            next_picture(),
            next_picture(),
            next_picture(),
            next_picture(),
            next_picture(),
            next_picture(),
            next_picture(),
            next_picture(),
        )
    }

    /// Two partially overlapping opaque boxes demonstrate as two pictures — the first the two
    /// boxes in the order written, the second the two in the opposite order.
    ///
    /// This used to run the binary against a file. A file no longer demonstrates, so the claim is
    /// made here, against the function that does.
    #[test]
    fn two_overlapping_boxes_demonstrate_in_opposite_orders() {
        let (first, second, ..) = demonstrated_pictures(
            r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 4 } },
            "next_id": 3,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "░" },
                { "kind": "box", "id": 2, "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light", "fill": "▓" }
            ]
        }"#,
        );

        let size = Size {
            width: 4,
            height: 3,
        };
        let a = BoxShape {
            at: Pos { x: 0, y: 0 },
            size,
            stroke: Stroke::from("light"),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        };
        let b = BoxShape {
            at: Pos { x: 2, y: 1 },
            size,
            stroke: Stroke::from("light"),
            fill: Some(Glyph::new("▓").expect("\"▓\" is one glyph")),
        };

        assert_eq!(first, stamped_back_to_front(&a, &b));
        assert_eq!(second, stamped_back_to_front(&b, &a));
    }

    /// Stamps `back` then `front` with `StampMode::Above` into a six-by-four window — the picture
    /// a description listing them in that order produces, per _The two orders are equivalent_.
    fn stamped_back_to_front(back: &BoxShape, front: &BoxShape) -> String {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let mut buffer = Buffer::new(origin, size);
        back.draw(&mut Layer::new(&mut buffer, StampMode::Above));
        front.draw(&mut Layer::new(&mut buffer, StampMode::Above));
        render(&buffer, &GlyphCatalog::light(), origin, size)
    }

    /// The tenth entry's far endpoint stops being a point and becomes a reference to `5`'s bottom
    /// with an offset of one in each axis — spelled today as one cell along the side in the `offset`
    /// and one cell out of it in the `out` beside it — and **the first five pictures come out byte
    /// for byte what they were**.
    ///
    /// The evidence is in a file and the picture not moving is what proves the arithmetic. The
    /// other side of each comparison is not pinned as text but built: the same demonstration with
    /// that one entry's `to` put back to the point it spells, so a difference can only be the
    /// entry. `{21, 3} + (1, 1)` is `{22, 4}`, which is what the entry said outright, and the
    /// demonstration removes `1`, displaces `1` and displaces `3`, so the fifth shape is none of
    /// them and the reference resolves the same in every picture.
    ///
    /// **The sixth is compared between the two runs and not against a "before",** because it has no
    /// before: it is the demonstration's own step, added by this slice, and both runs produce it. The
    /// two are equal, and the reason is the rule rather than a coincidence: the run that spells `to`
    /// outright has that endpoint grown from `{22, 4}` as an absolute point, while the run that
    /// names a reference to `5`'s bottom with offset `(1, 1)` has the **offset** grown to `(1, 3)`
    /// and the reference resolving to `{22, 6}`. Same cell, two routes to it, which is the whole
    /// claim.
    ///
    /// A path still prints one picture and nothing else, which is what `cargo xtask render` embeds.
    #[test]
    fn the_tenth_entry_naming_a_reference_leaves_the_first_five_pictures_exactly_as_they_were() {
        // Built through `serde_json` rather than by replacing text in the file, because the file is
        // formatted and a needle written against one formatting of it is a test that stops matching
        // the day prettier disagrees.
        //
        // **The entry is found by the identity it carries, not by the shape of a field beside it.**
        // It used to be found by looking for a connector whose `from` was a `point` at x 12, which
        // made the form of the *near* end the only thing this test knew about an entry it is about
        // the *far* end of. Measured: writing the near end as a reference left this test green,
        // because the loop then matched nothing, `with_the_point` came back as the same diagram and
        // every comparison below compared the file with itself. A name is a thing the entry
        // carries; the form of one of its endpoints is not.
        let mut value: serde_json::Value =
            serde_json::from_str(super::DEMO).expect("the embedded description is well-formed");
        let tenth = value["shapes"]
            .as_array_mut()
            .expect("the description lists its shapes")
            .iter_mut()
            .find(|shape| {
                shape["kind"] == serde_json::json!("connector")
                    && shape["id"] == serde_json::json!(10)
            })
            .expect("the tenth entry is the one connector this description holds");

        // **Asserted on the value rather than on the text written below it**, because a `Value` read
        // and written back differs from the file it came from whether or not anything was replaced.
        // That reformatting alone is what satisfied the `assert_ne!` this replaces, so the claim it
        // carried — that the shipped file spells this endpoint as a reference — is now made where it
        // can fail.
        assert_eq!(
            tenth["to"]["at"]["kind"],
            serde_json::json!("reference"),
            "the tenth entry spells its far endpoint as a reference, which is the other side of every \
             comparison below"
        );
        tenth["to"]["at"] = serde_json::json!({ "kind": "point", "x": 22, "y": 4 });
        let with_the_point = value.to_string();

        let (first, second, third, fourth, fifth, sixth, seventh, eighth) =
            demonstrated_pictures(super::DEMO);
        let (first2, second2, third2, fourth2, fifth2, sixth2, seventh2, eighth2) =
            demonstrated_pictures(&with_the_point);
        assert_eq!(
            (&first, &second, &third, &fourth, &fifth),
            (&first2, &second2, &third2, &fourth2, &fifth2),
            "naming the far endpoint as a reference changed one of the first five pictures"
        );
        assert_eq!(
            sixth, sixth2,
            "the sixth picture differs between a spelled endpoint and a named one: both are this \
             slice's own step, and both reach the same cell two rows lower"
        );
        // **The seventh joins the sixth for the same stated reason, and the reason is the freeze.**
        // `remove(3)` freezes the arrow's `from` at `{16, 5}` in both runs whatever route it took
        // to get there: the run that spells `to` outright grows that absolute point from `{22, 4}`,
        // and the run that names a reference to `5`'s bottom grows the **offset** to `(1, 3)` and
        // freezes the other end there. Same cell, two routes to it — the claim the sixth's existing
        // comment already made, now with the seventh beside it. Neither has a "before": both are the
        // demonstration's own step and both runs produce them.
        assert_eq!(
            seventh, seventh2,
            "the seventh picture differs between a spelled endpoint and a named one"
        );
        // **And the eighth joins them for a different stated reason: it asks the picture.** The record
        // is drawn from the diagram as it stands, so a diagram whose tenth entry resolves its far end
        // by a different route records the same owners at `(20, 2)` — `6` in both runs — and removes
        // the same shape. The arrow's route to that cell does not reach it, so the spelling of the
        // tenth entry is not what decides which shape the eighth removes.
        assert_eq!(
            eighth, eighth2,
            "the eighth picture differs between a spelled endpoint and a named one"
        );

        // The first picture is the shipped file's own, byte for byte, and a path prints that and
        // nothing else.
        assert_eq!(first, render_json(super::DEMO));
        assert_eq!(first2, render_json(&with_the_point));

        // The fifth picture still shows the box the arrow hangs from displaced four cells right,
        // because that is the demonstration's own change to the picture and not the file's.
        assert_ne!(
            fourth, fifth,
            "the fifth picture must be the fourth with a figure moved"
        );
    }

    /// A bare run prints **sixteen** captioned pictures: eight as it changes the diagram and one per
    /// step as it walks back.
    ///
    /// The count comes from the blank lines the output holds, and no caption's wording is pinned —
    /// what is claimed is that there are sixteen of them and that the first is the one a file's run
    /// prints on its own. Which picture the sixth holds is `the_sixth_picture_moves_only_the_arrow`'s
    /// claim, which the seventh holds is
    /// `the_seventh_picture_takes_the_box_away_and_leaves_the_arrow`'s, and which the eighth holds is
    /// `the_shape_the_demonstration_finds_is_the_one_the_crossing_cell_names`'s. The eight after them
    /// are `the_demonstration_runs_every_step_through_a_session`'s.
    #[test]
    fn a_bare_run_prints_eight_captioned_pictures_the_first_being_the_description_as_written() {
        let output = demonstrate_json(super::DEMO);

        assert_eq!(
            output.split("\n\n").count(),
            16,
            "eight pictures as the diagram changes and eight walked back: {output:?}"
        );

        let (first, ..) = demonstrated_pictures(super::DEMO);
        assert_eq!(first, render_json(super::DEMO));
    }

    /// Rule 14 — the demonstration wraps its diagram in a session once and performs every step
    /// through it.
    ///
    /// **Eight pictures after the eighth, one for each step, which is the whole of what "through a
    /// session" is observable as.** The eight commands are not the seven pictures that follow the
    /// first: the fifth picture is printed after both the rehang and the displacement of the box it
    /// was rehung from, and the session records each of them, so the count of steps and the count of
    /// pictures part company here for the first time.
    ///
    /// **A description with less in it walks a shorter way, and that is the claim that says the
    /// count comes from the session rather than from a loop.** An empty description records no step
    /// at all, so it prints its eight pictures and walks nowhere; a description whose window reaches
    /// one of the offsets records three. Neither is pinned by a literal count below — this test is
    /// about the shipped description — but the empty one is what makes the rule readable.
    #[test]
    fn the_demonstration_runs_every_step_through_a_session() {
        let blocks = demonstrated_blocks(super::DEMO);

        assert_eq!(
            blocks.len(),
            16,
            "eight pictures as the diagram changes and one per step walked back"
        );

        let empty = demonstrated_blocks(
            r#"{
                "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
                "next_id": 1,
                "shapes": []
            }"#,
        );
        assert_eq!(
            empty.len(),
            8,
            "a description no offset reaches records no step, so there is nothing to walk back over \
             and the eight pictures are all it prints"
        );
    }

    /// Rule 15 — the demonstration walks back one picture per step after its last, and its final
    /// picture is the one it began with.
    ///
    /// **The final picture is the first one, byte for byte.** That is the whole of the walk: eight
    /// commands in and the description as written out again, with no figure rebuilt and no reference
    /// re-resolved on the way — which is what a diagram alone could not have done, because
    /// `Diagram::remove` freezes what hung from the figure it takes and nothing re-hangs it.
    ///
    /// **One of the eight is a picture the run never printed going forward.** The rehang hangs the
    /// arrow's `from` end from the box it already hangs from, so it changes nothing a picture can
    /// show — and it is a step all the same, which is why two of the eight walked-back pictures are
    /// the same picture and why the walk is eight pictures long where the forward run is seven.
    #[test]
    fn the_demonstration_walks_back_to_the_picture_it_began_with() {
        let blocks = demonstrated_blocks(super::DEMO);

        assert_eq!(
            *blocks.last().expect("the run printed pictures"),
            blocks[0],
            "the last picture walked back is the one the run began with"
        );

        // The pictures as the diagram changed, and the ones walked back, as the states they hold. The
        // forward run prints the states 0, 1, 2, 3, 5, 6, 7 and 8; the walk prints 7, 6, 5, 4, 3, 2,
        // 1 and 0 — eight pictures for eight commands, where the forward run has seven pictures for
        // the same eight because the fifth is printed after two of them. **The block number is
        // written out rather than counted**, because the two lists are of different lengths and a
        // counter would silently walk the second one along.
        let printed_going_forward = [0_usize, 1, 2, 3, 5, 6, 7, 8];
        for (walked, state) in [
            (8_usize, 7_usize),
            (9, 6),
            (10, 5),
            (12, 3),
            (13, 2),
            (14, 1),
            (15, 0),
        ] {
            let printed_at = printed_going_forward
                .iter()
                .position(|held| *held == state)
                .expect("every state named here was printed going forward");
            assert_eq!(
                blocks[walked], blocks[printed_at],
                "the picture walked back to state {state} is the one printed for it"
            );
        }

        // Block 11 holds state 4, which has no forward picture of its own, and the reason is the rule
        // rather than an accident: the rehang hangs the arrow's `from` end from the box it already
        // hangs from, so it changes nothing a picture can show — and it is a step all the same. Two
        // of the eight walked-back pictures are therefore the same picture, which is what "one
        // command is one step" looks like in the demonstration's own output.
        assert_eq!(
            blocks[11], blocks[12],
            "the rehang changed no cell and is a step all the same, so two walked-back pictures are \
             the same picture"
        );
        assert_eq!(
            blocks[11], blocks[3],
            "and it is the picture the run printed after the shape before it was taken out"
        );
    }

    /// `render_once` over a path prints one picture and nothing else, which is what `cargo xtask
    /// render` embeds in a document.
    ///
    /// Pinned as a **count and a shape**, not as a literal: one window's worth of rows, and the
    /// first picture's own first row. The caption is absent, because a file's run is a picture with
    /// nothing around it and the demonstration's is a picture under a caption.
    #[test]
    fn a_path_draws_one_picture_and_walks_nowhere() {
        let (first, ..) = demonstrated_pictures(super::DEMO);
        let once = render_json(super::DEMO);

        assert_eq!(
            once, first,
            "a path prints the first picture, byte for byte"
        );
        assert!(
            !once.contains("\n\n"),
            "a path prints no caption and no second picture: {once:?}"
        );
        assert_eq!(
            once.lines().count(),
            13,
            "the shipped window is thirteen rows tall and nothing is added: {once:?}"
        );
    }

    /// The glyph a picture shows at `(column, row)`, or a space where it draws nothing.
    ///
    /// What `render` gives for a cell holding nothing, so a claim about a cell being blank or about
    /// two pictures agreeing on one is made in the vocabulary a reader reads the picture in.
    fn the_glyph_at(picture: &str, at: (usize, usize)) -> char {
        let (x, y) = at;
        picture
            .lines()
            .nth(y)
            .and_then(|row| row.chars().nth(x))
            .unwrap_or(' ')
    }

    /// The columns and rows the figure the third picture displaces holds, in the second and the
    /// third picture of the shipped demonstration.
    ///
    /// Quoted rather than read from the code that produces them, because a contract test that asks
    /// the demonstration the same questions it answers itself checks nothing. The numbers are the
    /// shipped description's first entry, a four-by-three box at the origin, and the demonstration's
    /// own fixed delta of three rows down.
    fn the_first_figures_footprint() -> (Range<usize>, Range<usize>) {
        (0..4, 0..6)
    }

    /// The positions where two pictures of one window differ, as `(column, row)`.
    fn differing(first: &str, second: &str) -> Vec<(usize, usize)> {
        first
            .lines()
            .zip(second.lines())
            .enumerate()
            .flat_map(|(y, (one, other))| {
                one.chars()
                    .zip(other.chars())
                    .enumerate()
                    .filter(|(_, (one, other))| one != other)
                    .map(move |(x, _)| (x, y))
            })
            .collect()
    }

    /// The third picture differs from the second only in the cells the displaced figure holds, and
    /// the fourth differs from the third only in the cells that figure occupies.
    ///
    /// The fourth is pinned against the shipped description with its first entry left out and drawn
    /// on its own, which is the whole claim in one comparison: a removal leaves a gap rather than a
    /// hole punched in what was around it, and the cell each gap holds is whatever the figure
    /// behind it decides, or empty. Comparing the two pictures with each other would only say they
    /// differ.
    #[test]
    fn the_third_picture_moves_one_figure_and_the_fourth_takes_that_figure_out() {
        let (_first, second, third, fourth, ..) = demonstrated_pictures(super::DEMO);

        let (columns, rows) = the_first_figures_footprint();
        let moved = differing(&second, &third);
        assert!(!moved.is_empty(), "the displacement changed nothing at all");
        assert!(
            moved
                .iter()
                .all(|(x, y)| columns.contains(x) && rows.contains(y)),
            "the displacement reached outside the figure's own cells: {moved:?}"
        );

        assert_eq!(fourth, render_json(&demo_without_its_first_entry()));
    }

    /// The columns and rows the fifth picture's box holds before and after, and the column the
    /// connector's far endpoint holds throughout.
    ///
    /// Quoted rather than read from the code that produces them, because a contract test that asks
    /// the demonstration the same questions it answers itself checks nothing. The numbers are the
    /// shipped description's third entry — a four-by-three box at `{9, 2}` — the demonstration's own
    /// fixed delta of four columns right, and the tenth entry's `to` at `{22, 4}`, which the fifth
    /// picture does not touch.
    fn the_hung_figures_footprint() -> (
        Range<usize>,
        Range<usize>,
        Range<usize>,
        Range<usize>,
        usize,
    ) {
        ((9..13), (2..5), (13..17), (2..5), 22)
    }

    /// The fifth picture differs from the fourth in exactly the box's old cells, the box's new cells
    /// and the connector's route — the same shape of claim the third picture's own test makes,
    /// reached with the same `differing` helper.
    ///
    /// The route is bounded rather than listed: it is the cells from the box's **new** side across
    /// to the far endpoint, on the rows it runs along, so the range is read off the box's own
    /// footprint rather than written down. The second assertion is the one that says the arrow
    /// re-routed rather than carried along: nothing at or past the far endpoint's column moved,
    /// which is the endpoint the displacement does not reach.
    #[test]
    fn the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it() {
        let (_first, _second, _third, fourth, fifth, ..) = demonstrated_pictures(super::DEMO);

        let (old_columns, old_rows, new_columns, new_rows, the_far_end_column) =
            the_hung_figures_footprint();

        let reached = |at: (usize, usize)| {
            let (x, y) = at;
            let in_a_footprint = (old_columns.contains(&x) && old_rows.contains(&y))
                || (new_columns.contains(&x) && new_rows.contains(&y));
            let on_the_route = (new_rows.start..=new_rows.end + 1).contains(&y)
                && (new_columns.end..=22).contains(&x);
            in_a_footprint || on_the_route
        };

        let changed = differing(&fourth, &fifth);
        assert!(
            !changed.is_empty(),
            "the fifth picture changed nothing at all"
        );
        assert!(
            changed.iter().copied().all(reached),
            "the fifth picture changed a cell neither box holds nor the arrow runs through: {changed:?}"
        );
        assert!(
            changed.iter().all(|(x, _)| *x <= the_far_end_column),
            "the change reached past the endpoint that did not move: {changed:?}"
        );

        // And the box is on its new side rather than still on the old one: a cell inside the new
        // footprint is written now and held nothing before.
        let occupied_before = |(x, y): (usize, usize)| {
            fourth
                .lines()
                .nth(y)
                .and_then(|row| row.chars().nth(x))
                .is_some_and(|glyph| glyph != ' ')
        };
        assert!(
            (new_columns.clone())
                .any(|x| (new_rows.clone()).any(|y| reached((x, y)) && !occupied_before((x, y)))),
            "the box did not land where the demonstration says it lands"
        );
    }

    /// The shipped demonstration with its first entry left out, as the text `render_once` reads.
    ///
    /// Rewritten through `serde_json` rather than by hand, so the description the test draws is the
    /// one the binary embeds and only the first entry differs.
    ///
    /// **It renumbers nothing, and that is the cost this slice removes rather than a fix to this
    /// helper.** While an identity was issued in array order rather than carried, removing the first
    /// listing shifted every identity after it: the tenth entry's `to` named `5`, the fifth entry of
    /// *this* text, and re-reading the text issued the identities from scratch in array order, so
    /// `5` became the box at `{18, 0}` rather than the one at `{20, 1}`. The fourth picture would
    /// then have been a different picture from the one the test compares it to, and the difference
    /// begins at row 3. That loop rewriting `5` to `4` is what kept the two sides the same diagram.
    ///
    /// Every entry now carries the identity it was written with, so the twenty-five that survive
    /// keep the identities they had and `remove(0)` shifts nothing at all. The same `assert_eq!` in
    /// `the_third_picture_moves_one_figure_and_the_fourth_takes_that_figure_out` is the claim now
    /// rather than a fixture a loop had to hold in place.
    fn demo_without_its_first_entry() -> String {
        let mut value: serde_json::Value =
            serde_json::from_str(super::DEMO).expect("the embedded description is well-formed");
        let shapes = value["shapes"]
            .as_array_mut()
            .expect("the description lists its shapes");
        shapes.remove(0);
        value.to_string()
    }

    /// An empty description demonstrates as **eight** identical pictures and fails nothing.
    ///
    /// There is no back-most shape to move, no figure to displace, no shape to take out, no tenth
    /// entry to rehang and no third entry to displace, so every call in every picture is a no-op on
    /// an identity this diagram does not hold, and there is no branch here to get wrong.
    ///
    /// **The seventh is the case worth having**, and it is worth having because it is a `remove`
    /// rather than an `if let`. An empty description holds no `3`, so the seventh step calls
    /// `remove` on an identity this diagram does not hold: `find` answers `None` and `remove`
    /// returns before its loop, which is a different guard from the `if let` the sixth step carries
    /// and is the only reason a removal can be called unconditionally at all. The seventh picture is
    /// therefore the sixth, and a seventh `assert_eq!` is what makes the count seven mean something
    /// rather than being a count of pictures the helper happened to return.
    ///
    /// **The eighth is the case that keeps every offset honest.** An empty description records nothing
    /// at any offset, so all four `owner` lookups answer `None` and every step skips itself — which
    /// is the whole of `None`'s job, and the reason an empty description demonstrates rather than
    /// panicking on it. **It is also the case a fallback would have hidden**: a description this thin
    /// is exactly the one where a written-out name would have gone on answering every question, and
    /// the shipped description is the one whose four offsets
    /// `the_four_offsets_answer_the_shapes_the_demonstration_acts_on` pins.
    #[test]
    fn an_empty_description_demonstrates_as_eight_identical_pictures() {
        let pictures = demonstrated_pictures(
            r#"{
            "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
            "next_id": 2,
            "shapes": []
        }"#,
        );

        assert_eq!(pictures.0, pictures.1);
        assert_eq!(pictures.0, pictures.2);
        assert_eq!(pictures.0, pictures.3);
        assert_eq!(pictures.0, pictures.4);
        assert_eq!(pictures.0, pictures.5);
        assert_eq!(pictures.0, pictures.6);
        assert_eq!(pictures.0, pictures.7);
    }

    /// A description holding exactly one shape demonstrates **eight** pictures and fails nothing.
    ///
    /// The reorder changes nothing, because that shape is both front-most and back-most. The
    /// displacement does not: the demonstration's fixed delta carries the only figure out of this
    /// three-row window, which the figure filled, so the third picture is that window with nothing
    /// in it. The fourth is the same, because the figure is taken out.
    ///
    /// **These eight are therefore not identical**, and no value of the delta would make them so: a
    /// figure that fills its own window is moved partly or wholly out of it by any delta other than
    /// none. What the rule asks of this case is that the run succeeds, which it does — and the fifth
    /// through eighth add four more no-ops on identities a one-shape description does not hold, since
    /// it has no `3` and no `10`. See the specification's Clarifications for the 2026-09-28
    /// session, which corrected this scenario on the evidence of this test.
    #[test]
    fn one_shape_demonstrates_as_two_copies_of_itself_and_then_an_empty_window() {
        let pictures = demonstrated_pictures(one_box_json());

        assert_eq!(pictures.0, pictures.1);
        assert_eq!(pictures.2, pictures.3);
        assert_ne!(pictures.0, pictures.2);
        assert_eq!(pictures.3, pictures.4);
        // The sixth equals the fifth for the same reason the fourth equals the third: the
        // description holds no `10`, so the sixth step's `get` returns `None` and the step changes
        // nothing. **A one-shape description is the case that would break if the `if let` around
        // that step were dropped**, which is why it is asserted rather than assumed.
        assert_eq!(pictures.4, pictures.5);
        // And the seventh equals the sixth for the reason of its own: the description holds no
        // `3` either, so `remove` finds nothing and returns before its loop. The test's name is
        // unchanged and still true — this is two copies of itself and then an empty window,
        // whatever comes after — and the count beside it is what changed.
        assert_eq!(pictures.5, pictures.6);
        // **And the eighth equals the seventh for a third reason of its own**, and this one is the
        // one the record introduces: the window is four by three, so `(20, 2)` is outside it, the
        // record holds no owner there, and the eighth step asks a question about a cell this
        // description never had. This is the shape a caller would hit by asking about a position its
        // window does not hold, and the answer is a picture rather than a panic.
        assert_eq!(pictures.6, pictures.7);
    }

    /// The two boxes' footprints at the fifth picture, and the one cell of the first that the arrow
    /// legitimately writes.
    ///
    /// **Quoted rather than read from the code that produces them**, because a contract test that
    /// asks the demonstration the same questions it answers itself checks nothing. `3` is the
    /// shipped description's third entry, a four-by-three box at `{9, 2}` that the fifth picture
    /// displaces four columns right into `x 13..16, y 2..4`; `5` is its fifth entry, a four-by-three
    /// box at `{20, 1}` occupying `x 20..23, y 1..3`, which neither the fifth nor the sixth touches.
    ///
    /// **The attachment cell is named because a footprint is not the same as a figure's own cells.**
    /// The arrow's `from` hangs from `3`'s **right side**, whose centre is `{16, 3}` — and `{16, 3}`
    /// is inside `3`'s own rectangle, because the rectangle is the box and the box's border is its
    /// rightmost column. The fifth picture has the arrow's arm welded to that border cell and the
    /// sixth has it detached, so that one cell inside a footprint **must** change for the arrow to
    /// have moved at all. A test that forbade every cell inside either footprint would therefore
    /// fail on the very behavior it is meant to certify, and quoting the footprint alone would hide
    /// that. The far end is not in this position: `to` names `5`'s bottom with offset `(1, 1)`, and
    /// `{21, 3} + (1, 1)` is `{22, 4}` — **one row below** `5`, outside its rectangle — which is
    /// why the second box's footprint is untouched entire.
    fn the_two_boxes_and_the_attachment() -> [(Range<usize>, Range<usize>); 2] {
        [(13..17, 2..5), (20..24, 1..4)]
    }

    /// The one cell inside a box's footprint the arrow writes: `3`'s right side centre, `{16, 3}`.
    const THE_ATTACHMENT: (usize, usize) = (16, 3);

    /// The rectangle the arrow's own cells fall inside at the sixth picture, and the rectangle the
    /// box it hangs from stands in at the sixth and is gone by the seventh.
    ///
    /// **Quoted rather than read from the code that produces them**, because a contract test that
    /// asks the demonstration the same questions it answers itself checks nothing. `3` is the
    /// shipped description's third entry, a four-by-three box at `{9, 2}` that the fifth picture
    /// displaces four columns right into `x 13..16, y 2..4`. The arrow's `from` stands at
    /// `{16, 5}` — `3`'s right side centre at `{16, 3}` plus the offset `(0, 2)` the sixth picture
    /// grew — and its `to` names `5`'s bottom with offset `(1, 3)`, which is `{22, 6}`.
    ///
    /// **The arrow's rectangle is a bound and not its footprint, and that is the point.** Between
    /// those two cells the route writes **ten** of the twenty-one the rectangle holds, and it writes
    /// them in three rows: four across the top, two in the middle and four along the bottom. It is
    /// neither contiguous nor a rectangle, so nothing that reads it off a picture gets it right —
    /// which is how the specification's own `22 − 10 = 12` was wrong and how measurement corrected
    /// it. The count is therefore **asserted against the pictures** and the rectangle is only what
    /// bounds where to look.
    fn the_arrow_and_its_removed_box() -> [(Range<usize>, Range<usize>); 2] {
        [(16..23, 5..8), (13..17, 2..5)]
    }

    /// The seventh picture differs from the sixth **only** in the cells `3` held, which is the
    /// only statement in the slice that says the arrow stood still.
    ///
    /// Modelled on `the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it` and reusing its
    /// `differing` helper, with the claim **in both directions**: what the removal may reach, and
    /// what it may not touch. The first alone is not enough — a removal that moved the arrow
    /// elsewhere and blanked two rows of box would satisfy "twelve cells differ", and the second is
    /// what rules that out.
    #[test]
    fn the_seventh_picture_takes_the_box_away_and_leaves_the_arrow() {
        let (_first, _second, _third, _fourth, _fifth, sixth, seventh, _eighth) =
            demonstrated_pictures(super::DEMO);

        let [arrow, the_box] = the_arrow_and_its_removed_box();
        let (arrow_columns, arrow_rows) = (arrow.0, arrow.1);

        // What the removal may reach: exactly the twelve cells `3` stood in, and all twelve blank
        // rather than carrying a different glyph — a box's border cells are the only thing that
        // could still be written there.
        let changed = differing(&sixth, &seventh);
        assert_eq!(
            changed.len(),
            12,
            "the seventh differs from the sixth in something other than the twelve cells the box \
             held: {changed:?}"
        );
        assert!(
            changed
                .iter()
                .all(|(x, y)| the_box.0.contains(x) && the_box.1.contains(y)),
            "the removal reached outside the box's own rectangle: {changed:?}"
        );
        assert!(
            changed.iter().all(|at| the_glyph_at(&seventh, *at) == ' '),
            "a cell the removed box held is still drawn in the seventh: {changed:?}"
        );

        // What it may not touch: every cell the arrow held in the sixth, found rather than quoted —
        // the written cells inside the bound `the_arrow_and_its_removed_box` names, which is a
        // rectangle holding ten written cells out of twenty-one.
        let the_arrow_held: Vec<(usize, usize)> = arrow_rows
            .flat_map(|y| arrow_columns.clone().map(move |x| (x, y)))
            .filter(|at| the_glyph_at(&sixth, *at) != ' ')
            .collect();
        assert_eq!(
            the_arrow_held.len(),
            10,
            "the arrow's footprint is not the ten cells this test measured: {the_arrow_held:?}"
        );
        for at in the_arrow_held {
            assert_eq!(
                the_glyph_at(&sixth, at),
                the_glyph_at(&seventh, at),
                "the arrow did not stand where it stood, at {at:?}"
            );
        }
    }

    /// The sixth picture differs from the fifth **only** in the cells the arrow holds before and
    /// after, which is the only statement that says both boxes stood still.
    ///
    /// Modelled on `the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it` and reusing its
    /// `differing` helper, with the claim **mirrored**: that test bounds what the fifth may reach,
    /// and this one bounds what the sixth may reach in the same shape.
    ///
    /// The bound is **exact rather than one-sided**, and that is what makes it say "both boxes stood
    /// still" rather than "no box moved very far": the cells that changed inside the two footprints
    /// are exactly the one attachment cell and nothing else. A displacement that rewrote the shape a
    /// reference names — moving `3` or `5` to follow the arrow — would change cells inside a
    /// footprint that are not the attachment, and the assertion below is what rules it out.
    #[test]
    fn the_sixth_picture_moves_only_the_arrow() {
        let (_first, _second, _third, _fourth, fifth, sixth, _seventh, _eighth) =
            demonstrated_pictures(super::DEMO);

        let changed = differing(&fifth, &sixth);
        assert!(
            !changed.is_empty(),
            "the sixth picture changed nothing at all"
        );

        let inside: Vec<(usize, usize)> = changed
            .iter()
            .copied()
            .filter(|(x, y)| {
                the_two_boxes_and_the_attachment()
                    .iter()
                    .any(|(columns, rows)| columns.contains(x) && rows.contains(y))
            })
            .collect();
        assert_eq!(
            inside,
            vec![THE_ATTACHMENT],
            "the sixth picture changed a cell inside a box's own footprint other than the one the \
             arrow attaches to, so a box moved: {changed:?}"
        );

        // And the arrow did move, which the bound above cannot say on its own: the far end left the
        // cell it welded itself to. Named rather than derived, and it is a cell **outside** both
        // footprints — `{22, 4}` is `5`'s bottom centre plus the shipped offset, one row below the
        // box — so a displacement that moved nothing and a displacement that moved a box could not
        // both satisfy this.
        assert!(
            changed.contains(&(22, 4)),
            "the arrow's far end is still welded where it was: {changed:?}"
        );
        assert!(
            changed.contains(&(22, 6)),
            "the arrow's far end did not travel the two rows the demonstration's delta names: \
             {changed:?}"
        );
    }

    /// The three cells of the first picture the demonstration's own examples rest on, and the
    /// identity each resolves to.
    ///
    /// **Quoted positions rather than read from the code that produces them**, because a test that
    /// asks the demonstration the same questions it answers itself checks nothing. These are the
    /// shipped description's first and fourth entries — two filled boxes at `(0, 0)` and `(7, 1)`,
    /// the second listed later and so in front — and the three cells below are where their fills and
    /// a corner meet.
    const THE_CROSSING_CELLS: [(usize, usize, char, u32); 3] =
        [(9, 2, '░', 4), (10, 3, '┘', 4), (11, 3, '░', 3)];

    /// The shape a position resolves to is the front-most of the two that wrote it, and the character
    /// there cannot say which that is.
    ///
    /// **The first and third cells of [`THE_CROSSING_CELLS`] print the same character and belong to
    /// different shapes**: `(9, 2)` is inside `4`'s fill alone and `(11, 3)` is inside `3`'s fill
    /// alone, because `3` spans `x 9..12` and `4` spans `x 7..10`, so their interiors do not
    /// overlap. Both renders as `░`, and only the record tells them apart — which is the whole
    /// reason the record is kept beside the cell rather than derived from it.
    ///
    /// This is the assertion the demonstration's first caption rests on, and it is asked of the
    /// **drawn buffer** rather than of the demonstration's own lookup: the point is what the record
    /// says at a position, not what `demonstrate` decided to write in a caption.
    #[test]
    fn the_shape_a_position_resolves_to_is_the_front_most_of_the_two_that_wrote_it() {
        let (diagram, _) = parse(super::DEMO).expect("the shipped description reads");
        let catalog = glyph_catalog();
        let (origin, size) = in_the_demo_window();
        let (buffer, picture) = drawn(&diagram, &catalog, origin, size);

        for (x, y, glyph, expected) in THE_CROSSING_CELLS {
            let at = Pos {
                x: i32::try_from(x).expect("a fifty-column window"),
                y: i32::try_from(y).expect("a thirteen-row window"),
            };
            assert_eq!(the_glyph_at(&picture, (x, y)), glyph, "the cell at {at:?}");
            assert_eq!(
                buffer.owner(Offset {
                    x: u32::try_from(x).expect("a fifty-column window"),
                    y: u32::try_from(y).expect("a thirteen-row window"),
                }),
                Some(identity(expected)),
                "{at:?} is written by two shapes and resolves to the one in front"
            );
        }

        // And the first two cells resolve to `4` while the third does not, which is what makes the
        // first two a crossing and the third the shape behind: `4` is listed after `3` and is
        // therefore in front of it.
        assert_eq!(buffer.owner(Offset { x: 9, y: 2 }), Some(identity(4)));
        assert_eq!(
            buffer.owner(Offset { x: 11, y: 3 }),
            Some(identity(3)),
            "a cell only 3 wrote is 3's, whichever way the order runs"
        );
    }

    /// The shape the demonstration finds is the one the crossing cell names, and taking it out is
    /// what the eighth picture shows.
    ///
    /// **Measured on the demonstration's own output rather than on a drawing built beside it**, so
    /// the claim is about what a reader sees: at `(20, 2)` the seventh picture renders `┼` and the
    /// eighth renders `│`. Two shapes wrote that cell — `6`'s bottom border runs through it and
    /// `5`'s left border runs down it — and the record names `6`, the one **behind**, because `6`
    /// is listed later and so is in front. Taking `6` out leaves `5`'s arm standing alone.
    ///
    /// **The eighth is not the seventh with something blanked**: it differs at exactly the cells the
    /// record named, and every one of them is either now blank or now shows what was behind it.
    #[test]
    fn the_shape_the_demonstration_finds_is_the_one_the_crossing_cell_names() {
        let (_first, .., seventh, eighth) = demonstrated_pictures(super::DEMO);

        assert_eq!(
            the_glyph_at(&seventh, (20, 2)),
            '┼',
            "the crossing cell renders as a full junction, which is what two shapes wrote there"
        );
        assert_eq!(
            the_glyph_at(&eighth, (20, 2)),
            '│',
            "and with the shape the record named taken out, only the other shape's arm stands"
        );
        assert_ne!(seventh, eighth, "so the eighth picture is not the seventh");
    }

    /// Each of the demonstration's four offsets answers the shape it means on the shipped
    /// description.
    ///
    /// **This is the only thing that makes the offsets checkable**, and it exists because
    /// the demonstration has no written-out fallback to hide behind: a coordinate moved onto a cell
    /// another shape wrote, or onto one nothing wrote, would find nothing there and the demonstration
    /// would quietly act on a different shape or on none — eight pictures that still read like a
    /// demonstration. So the
    /// **offsets are read from the constants the demonstration uses and the answers are quoted here**,
    /// which is the one direction of that split that is not circular: what is being checked is the
    /// coordinate, so it belongs beside the code, and what is being asserted is the identity at it, so
    /// it is written out.
    ///
    /// **The fourth is the case the first three avoid**, and it is here because the demonstration's
    /// eighth step is built on it: `(20, 2)` is a cell two shapes wrote, and the answer is the one
    /// **behind**. `5` is listed after `6` and so is in front, which is why the record names `6`.
    #[test]
    fn the_four_offsets_answer_the_shapes_the_demonstration_acts_on() {
        let (diagram, _) = parse(super::DEMO).expect("the shipped description reads");
        let (origin, size) = in_the_demo_window();
        let (buffer, _) = drawn(&diagram, &glyph_catalog(), origin, size);

        for (at, expected, what) in [
            (
                super::THE_BACK_MOST_AT,
                1,
                "the back-most shape's own top-left corner",
            ),
            (
                super::THE_HUNG_FROM_AT,
                3,
                "the box the arrow hangs from, at its bottom-left corner",
            ),
            (
                super::THE_ARROW_AT,
                10,
                "the middle of the arrow's own horizontal run",
            ),
            (
                super::THE_CROSSING_AT,
                6,
                "a crossing two shapes wrote, and the record names the one behind",
            ),
        ] {
            assert_eq!(
                buffer.owner(at),
                Some(identity(expected)),
                "the offset ({}, {}) is {what}, and the record names {expected}",
                at.x,
                at.y
            );
        }
    }

    /// The demonstration asks the picture for its three identities rather than writing them out, so
    /// it follows whatever the record names rather than whatever the file happens to call them.
    ///
    /// **A description where the three answers differ from the three names**, which is the only way
    /// to tell asking from writing: an extra box is added at the origin on top of `1`, so the cell
    /// at `(0, 0)` is decided by the newcomer rather than by the entry the demonstration used to
    /// name. The demonstration's first four pictures are then **about the newcomer** — it is the
    /// shape the picture names at `(0, 0)`, so it is what gets moved and what gets taken out.
    ///
    /// Pinned as a **difference from the shipped run rather than as a picture of its own**: what is
    /// claimed is that adding a shape changes which shape the demonstration acts on, and comparing
    /// the two runs says that in one comparison. Under the identities written out by hand the first
    /// four pictures would come out byte for byte what they are in the shipped run, because `1`
    /// would still be the shape found there.
    #[test]
    fn the_demonstration_asks_the_picture_which_shapes_to_act_on() {
        let mut value: serde_json::Value =
            serde_json::from_str(super::DEMO).expect("the embedded description is well-formed");
        value["shapes"]
            .as_array_mut()
            .expect("the description lists its shapes")
            .push(serde_json::json!({
                "kind": "box", "id": 27, "at": { "x": 0, "y": 0 },
                "size": { "width": 4, "height": 3 }, "stroke": "light", "fill": "▓"
            }));

        let (first, second, third, fourth, ..) = demonstrated_pictures(&value.to_string());
        let (shipped_first, shipped_second, shipped_third, shipped_fourth, ..) =
            demonstrated_pictures(super::DEMO);

        // The newcomer stands in front of the entry the demonstration used to name, so the picture
        // has changed and none of the four pictures is the shipped one.
        assert_ne!(first, shipped_first, "the extra box is not on the screen");
        assert_ne!(second, shipped_second);
        assert_ne!(third, shipped_third);
        assert_ne!(fourth, shipped_fourth);

        // **And the fourth is the first with the newcomer gone**, which is the claim: it was the
        // shape at `(0, 0)`, so it is the one taken out. Under identities written out by hand the
        // newcomer would still be standing there and `1` would be the one gone.
        assert_eq!(
            the_glyph_at(&fourth, (1, 1)),
            '░',
            "the fill of the entry the demonstration no longer names is back where it was"
        );
        assert_ne!(
            the_glyph_at(&first, (1, 1)),
            the_glyph_at(&fourth, (1, 1)),
            "and the newcomer that stood over it is gone"
        );
    }

    /// The three arguments this binary reads, as one call each, and what each of them becomes.
    ///
    /// **The window is a pair and the pair is `None` or it is not.** `None` is the whole of what a
    /// caller that named no window says, and it is what leaves the description's own `canvas` in
    /// charge while the format still carries one; the pair is what a caller that named one gets.
    #[test]
    fn the_arguments_are_a_path_and_a_window_and_neither_is_required() {
        let args = |words: &[&str]| {
            words
                .iter()
                .map(|word| (*word).to_owned())
                .collect::<Vec<String>>()
        };

        assert_eq!(
            parse_args(&args(&[])).expect("a bare run is a run"),
            (None, None),
            "no argument at all is the demonstration"
        );
        assert_eq!(
            parse_args(&args(&["x.json"])).expect("a path is a path"),
            (Some("x.json".to_owned()), None),
            "a path alone names no window"
        );
        assert_eq!(
            parse_args(&args(&["--size", "20x7", "x.json"])).expect("a size is a window"),
            (
                Some("x.json".to_owned()),
                Some((
                    Pos { x: 0, y: 0 },
                    Size {
                        width: 20,
                        height: 7
                    }
                ))
            ),
            "a size alone is a window at the origin every marker but four uses"
        );
        assert_eq!(
            parse_args(&args(&["--size", "7x5", "--origin", "-3,-2", "x.json"]))
                .expect("both halves are a window"),
            (
                Some("x.json".to_owned()),
                Some((
                    Pos { x: -3, y: -2 },
                    Size {
                        width: 7,
                        height: 5
                    }
                ))
            ),
            "an origin may be negative, which is what `specs/086` draws at"
        );
        assert_eq!(
            parse_args(&args(&["--origin", "-3,-2", "--size", "7x5", "x.json"]))
                .expect("the order on the command line is the caller's"),
            parse_args(&args(&["--size", "7x5", "--origin", "-3,-2", "x.json"]))
                .expect("the same window either way"),
            "a flag after the path is a flag, not a second path"
        );
    }

    /// Anything the parser cannot read into a window or a path is refused with the usage beside it,
    /// and the refusal quotes what was written rather than saying what was expected.
    ///
    /// **The first two are the shapes a mistyped flag takes** and the third is the shape this
    /// binary's own markers never take, which is why a marker writes `at -3,-2` and not
    /// `--origin -3 -2`. The rest are refusals no caller in this repository makes, and they are
    /// here because a parser that reads them as something is a parser with no boundary.
    #[test]
    fn arguments_this_binary_cannot_read_are_refused_with_what_was_written() {
        let args = |words: &[&str]| {
            words
                .iter()
                .map(|word| (*word).to_owned())
                .collect::<Vec<String>>()
        };

        for (words, said) in [
            (
                &["--size", "20-7"][..],
                "`--size` takes `<width>x<height>` and not `20-7`",
            ),
            (
                &["--origin", "-3-2"][..],
                "`--origin` takes `<x>,<y>` and not `-3-2`",
            ),
            (&["--width", "20"][..], "`--width` is not an option"),
            (&["--size"][..], "`--size` was given no size"),
            (&["--origin"][..], "`--origin` was given no origin"),
            (
                &["--size", "4x3", "--size", "6x4"][..],
                "`--size` was given twice",
            ),
            (
                &["--origin", "0,0", "--origin", "1,1"][..],
                "`--origin` was given twice",
            ),
            (&["a.json", "b.json"][..], "`b.json` is a second path"),
        ] {
            let error = parse_args(&args(words))
                .err()
                .unwrap_or_else(|| panic!("{words:?} should have been refused"));

            assert!(error.starts_with(said), "expected `{said}`, got: {error}");
            assert!(
                error.ends_with(super::USAGE),
                "every refusal carries the usage beside it: {error}"
            );
        }
    }

    /// Rule 10 — the demonstration's eight pictures before the walk back are unchanged by this
    /// change.
    ///
    /// **Pinned as a snapshot rather than as eight literals**, because the eight are fifty columns
    /// and thirteen rows each and nobody reads them as text. What a snapshot buys is that a move is
    /// reported rather than argued, and the report is the whole of what "unchanged" means here: the
    /// eight are the demonstration's own, and this change contributes nothing to them but the
    /// window they are drawn at.
    ///
    /// It is a contract test and sits in the directory of its own, beside the characterization: the
    /// eight pictures are a decision about what the demonstration shows, not a record of a range.
    #[test]
    fn the_demonstrations_eight_pictures_are_unchanged() {
        let (first, second, third, fourth, fifth, sixth, seventh, eighth) =
            demonstrated_pictures(super::DEMO);
        let eight = [first, second, third, fourth, fifth, sixth, seventh, eighth].join("\n\n");

        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/snapshots/contract"
        ));
        settings.bind(|| {
            insta::assert_snapshot!("the_demonstrations_eight_pictures_are_unchanged", eight);
        });
    }

    /// Rule 11 — the offsets `specs/086` cites resolve to the same cells they do today.
    ///
    /// **The one place in the repository where the origin is load-bearing.** That spec's first
    /// picture is drawn at `(-3, -2)`, and its table counts each row from that corner: `(-2, -1)` is
    /// the offset `(1, 1)`, and the box at `(0, 0)` is two rows and three columns inside the window.
    /// A window that defaulted to `(0, 0)` everywhere would mean translating the figures and
    /// rewriting the prose around them, so this pins the whole table rather than one row of it.
    ///
    /// **The last row is the one that says the window is a window.** `(4, -2)` is outside it, so
    /// the offset `(7, 0)` reaches a cell nothing holds and the record answers `None` — which is
    /// rules 7 and 8 of that spec, and the reason the table has a row with no cell in it.
    #[test]
    fn specs_offsets_resolve_to_the_cells_they_do_today() {
        let json = r#"{
            "canvas": { "origin": { "x": -3, "y": -2 }, "size": { "width": 7, "height": 5 } },
            "next_id": 4,
            "shapes": [
                { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
                  "stroke": "light" },
                { "kind": "box", "id": 2, "at": { "x": -2, "y": -1 }, "size": { "width": 4, "height": 3 },
                  "stroke": "double", "fill": "░" },
                { "kind": "box", "id": 3, "at": { "x": 6, "y": 3 }, "size": { "width": 4, "height": 3 },
                  "stroke": "heavy" } ]
        }"#;
        let (diagram, _) = parse(json).expect("the description reads");
        let origin = Pos { x: -3, y: -2 };
        let size = Size {
            width: 7,
            height: 5,
        };
        let (buffer, picture) = drawn(&diagram, &glyph_catalog(), origin, size);

        for (asked, offset, cell, owner) in [
            ((-3, -2), (0, 0), ' ', None),
            ((-3, 0), (0, 2), ' ', None),
            ((-2, -1), (1, 1), '╔', Some(2)),
            ((0, 0), (3, 2), '░', Some(2)),
            ((1, 0), (4, 2), '╟', Some(2)),
            ((2, 2), (5, 4), '─', Some(1)),
            ((3, 1), (6, 3), '│', Some(1)),
        ] {
            let at = Offset {
                x: offset.0,
                y: offset.1,
            };
            assert_eq!(
                buffer.owner(at),
                owner.map(identity),
                "the offset {at:?}, asked for ({}, {}), is {owner:?}",
                asked.0,
                asked.1
            );
            assert_eq!(
                the_glyph_at(&picture, (offset.0 as usize, offset.1 as usize)),
                cell,
                "the cell at {at:?} renders {cell:?}"
            );
        }

        assert_eq!(
            buffer.owner(Offset { x: 7, y: 0 }),
            None,
            "the offset past the window's right edge reaches a cell nothing holds"
        );
    }
}
