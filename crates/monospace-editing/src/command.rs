//! What a caller means to do to a diagram: one thing, with the fields that go with it, and the
//! diagram's own operation underneath. See the _Vocabulary_ row for `Command` and §2 of
//! [`docs/editing-model.md`](../../../docs/editing-model.md).

use monospace_diagram::{
    Anchor, Delta, Diagram, Direction, End, Reference, Shape, ShapeId, Stroke, Terminal, Text,
};

/// One thing a caller means to do to a diagram.
///
/// **It carries the meaning and not the finished figure, and each one says which of the diagram's
/// seven operations it becomes.** `Replace(ShapeId, Shape)` would restate the diagram's own
/// operation and throw away what the caller knew — a delta is what a person reading a list of
/// these recognizes, and what a line of a script has to be written in either way — and the
/// decision of which operation a meaning becomes is the work of this layer rather than of every
/// caller that has one.
///
/// **The set is open and a command added to it is additive.** A session already does whatever a
/// command says, so one more meaning is one more arm here and nothing else. The set below is
/// what the callers so far needed and is no promise that its size is final.
///
/// The identities are compared by value, so one issued by another diagram is a well-formed value
/// matching nothing here — the same answer the diagram gives everything that names a figure it
/// does not hold. **A command that changes nothing is not a step** (#285): the session records one only for
/// the diagram a command leaves, so one naming a figure the diagram does not hold, and one
/// whose change gives the figure back, leave the history where it was. See
/// [`Session`](crate::Session) and §5 of the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// The figure goes `by`, along each axis.
    Move {
        /// The figure to move.
        id: ShapeId,
        /// How far, along each screen axis. Negative amounts move against it.
        by: Delta,
    },
    /// The figure becomes so much wider and taller, or a line longer or shorter along its orientation.
    Resize {
        /// The figure to resize.
        id: ShapeId,
        /// How much the size changes along screen axes: for a box `dx` is width and `dy` is height;
        /// for a line the component along its orientation changes its length.
        by: Delta,
    },
    /// The line's orientation is toggled between horizontal and vertical.
    RotateLine {
        /// The line whose orientation is being toggled.
        id: ShapeId,
    },
    /// The figure's stroke is replaced with `stroke`.
    SetStroke {
        /// The figure whose stroke is being set.
        id: ShapeId,
        /// The stroke to apply.
        stroke: Stroke,
    },
    /// The box's text is replaced with `text`, or taken out when it is `None`.
    SetText {
        /// The box whose text is being set.
        id: ShapeId,
        /// The block to store, or `None` for no text at all.
        text: Option<Text>,
    },
    /// The connector's terminals are swapped.
    InvertTerminals {
        /// The connector whose terminals are being inverted.
        id: ShapeId,
    },
    /// The connector's endpoint specified by `end` goes `by` and adjusts standoff by `out`.
    MoveEndpoint {
        /// The connector whose endpoint is being moved.
        id: ShapeId,
        /// Which endpoint is targeted (`From` or `To`).
        end: End,
        /// How far to move the endpoint along screen axes.
        by: Delta,
        /// How far outward to adjust standoff.
        out: i16,
    },
    /// The connector's endpoint specified by `end` has its offset and standoff reset to zero.
    ResetEndpoint {
        /// The connector whose endpoint is being reset.
        id: ShapeId,
        /// Which endpoint is targeted (`From` or `To`).
        end: End,
    },
    /// The connector's endpoint specified by `end` has its terminal replaced by `terminal`.
    SetEndpointTerminal {
        /// The connector whose endpoint terminal is being set.
        id: ShapeId,
        /// Which endpoint is targeted (`From` or `To`).
        end: End,
        /// The terminal style to apply.
        terminal: Terminal,
    },
    /// The connector's endpoint specified by `end` is detached into an absolute canvas position.
    DetachEndpoint {
        /// The connector whose endpoint is being detached.
        id: ShapeId,
        /// Which endpoint is targeted (`From` or `To`).
        end: End,
    },
    /// The connector's detached endpoint specified by `end` has its departure direction set to `leaving`.
    SetEndpointLeaving {
        /// The connector whose endpoint leaving direction is being set.
        id: ShapeId,
        /// Which endpoint is targeted (`From` or `To`).
        end: End,
        /// The direction the connector leaves the endpoint in.
        leaving: Direction,
    },
    /// One end of a connector hangs from one of `target`'s four sides.
    Hang {
        /// The connector whose end is being hung.
        id: ShapeId,
        /// Which endpoint hangs (`From` or `To`).
        end: End,
        /// The figure the end hangs from.
        target: ShapeId,
        /// Which of that figure's four sides it hangs from.
        anchor: Anchor,
    },
    /// The figure goes one place toward the front of the order.
    Forward {
        /// The figure to move forward.
        id: ShapeId,
    },
    /// The figure goes one place toward the back of the order.
    Backward {
        /// The figure to move backward.
        id: ShapeId,
    },
    /// The figure goes to the front of the order.
    ToFront {
        /// The figure to move to the front.
        id: ShapeId,
    },
    /// The figure goes to the back of the order.
    ToBack {
        /// The figure to move to the back.
        id: ShapeId,
    },
    /// The figure is added to the front of the drawing order.
    Add {
        /// The figure to add.
        shape: Shape,
    },
    /// The figure is taken out.
    Remove {
        /// The figure to take out.
        id: ShapeId,
    },
}

impl Command {
    /// The one of the diagram's seven operations this command is, carried out on `diagram`.
    ///
    /// **The whole of this method is the layer's mapping**, and it is why a caller does not have to
    /// write it: reading the figure, rebuilding the one end that changes and keeping everything
    /// else is logic that is subtly wrong in four different ways when each caller writes it. Every
    /// operation is the diagram's and unchanged, and a command that changed the diagram by any
    /// other way is not a command.
    ///
    /// **It takes the diagram and hands one back rather than writing into it**, which is what every
    /// operation of the diagram now does. A command naming a figure the diagram does not hold hands
    /// back the diagram it was given rather than a diagram something happened to: the two are the
    /// same, and saying so here is how a caller reads a missing figure as silence rather than as a
    /// change.
    ///
    /// **`pub(crate)` and no longer the public way in.** A caller performs a command by handing it
    /// to a session — `Session::perform` is where it is recorded, discarded-with and carried out,
    /// and only the session can do any of that. **What this method stays is what the model's
    /// "a command that changed the diagram by any other way is not a command" is about**: it is
    /// reachable from inside the crate and nowhere else, so the claim is a rule about the type
    /// rather than a request from outside it.
    ///
    /// **`None` is the diagram the command found, and it is not a failure.** A figure the diagram does
    /// not hold, a change that gives the figure back as it was, and a figure brought forward or to
    /// the front when it is already at the front, or backward or to the back when it is already at
    /// the back are one answer: nothing changed. What `Some` holds is the whole diagram the command
    /// left, and the session records it as the step.
    pub(crate) fn apply_to(&self, diagram: &Diagram) -> Option<Diagram> {
        match *self {
            Self::Add { ref shape } => Some(diagram.add(shape.clone()).0),
            Self::Move { id, by } => changed(diagram, id, |shape| shape.displaced_by(by)),
            Self::Resize { id, by } => changed(diagram, id, |shape| shape.resized_by(by)),
            Self::RotateLine { id } => changed(diagram, id, Shape::with_rotated_orientation),
            Self::SetStroke { id, ref stroke } => {
                let stroke = stroke.clone();
                changed(diagram, id, |shape| shape.with_stroke(stroke))
            }
            Self::SetText { id, ref text } => {
                let text = text.clone();
                changed(diagram, id, |shape| shape.with_text(text))
            }
            Self::InvertTerminals { id } => changed(diagram, id, Shape::with_inverted_terminals),
            Self::MoveEndpoint { id, end, by, out } => {
                changed(diagram, id, |shape| shape.with_endpoint_moved(end, by, out))
            }
            Self::ResetEndpoint { id, end } => {
                changed(diagram, id, |shape| shape.with_endpoint_reset(end))
            }
            Self::SetEndpointTerminal {
                id,
                end,
                ref terminal,
            } => {
                let terminal = terminal.clone();
                changed(diagram, id, |shape| {
                    shape.with_endpoint_terminal(end, terminal)
                })
            }
            Self::DetachEndpoint { id, end } => changed(diagram, id, |shape| {
                shape.with_endpoint_detached(end, diagram)
            }),
            Self::SetEndpointLeaving { id, end, leaving } => changed(diagram, id, |shape| {
                shape.with_endpoint_leaving(end, leaving)
            }),
            Self::Hang {
                id,
                end,
                target,
                anchor,
            } => changed(diagram, id, |shape| {
                shape.with_endpoint_attached(
                    end,
                    Reference::new(target, anchor, Delta::ZERO, 0),
                    diagram,
                )
            }),
            Self::Forward { id } => diagram.can_forward(id).then(|| diagram.forward(id)),
            Self::Backward { id } => diagram.can_backward(id).then(|| diagram.backward(id)),
            Self::ToFront { id } => diagram.can_forward(id).then(|| diagram.to_front(id)),
            Self::ToBack { id } => diagram.can_backward(id).then(|| diagram.to_back(id)),
            Self::Remove { id } => diagram.get(id).map(|_| diagram.remove(id)),
        }
    }
}

/// The diagram `change` leaves, or `None` when it leaves the figure `id` names as it found it.
///
/// **A figure that is not there and a change that gives the same figure back are one answer**: the
/// diagram a command leaves is the one it was given, and nothing above this line needs to tell the
/// two apart. `Shape` compares by value, which is the diagram's own notion of the same figure and
/// not a second one written here.
fn changed(
    diagram: &Diagram,
    id: ShapeId,
    change: impl FnOnce(&Shape) -> Shape,
) -> Option<Diagram> {
    let before = diagram.get(id)?;
    let after = change(before);

    (after != *before).then(|| diagram.replace(id, after))
}
