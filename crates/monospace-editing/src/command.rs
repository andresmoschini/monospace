//! What a caller means to do to a diagram: one thing, with the fields that go with it, and the
//! diagram's own operation underneath. See the _Vocabulary_ row for `Command` and §2 of
//! [`docs/editing-model.md`](../../../docs/editing-model.md).

use monospace_diagram::{Anchor, Delta, Diagram, Endpoint, Position, Reference, Shape, ShapeId};

/// One thing a caller means to do to a diagram.
///
/// **It carries the meaning and not the finished figure, and each one says which of the diagram's
/// five operations it becomes.** `Replace(ShapeId, Shape)` would restate the diagram's own
/// operation and throw away what the caller knew — a delta is what a person reading a list of
/// these recognizes, and what a line of a script has to be written in either way — and the
/// decision of which operation a meaning becomes is the work of this layer rather than of every
/// caller that has one.
///
/// **The set is open and a command added to it is additive.** A session already does whatever a
/// command says, so one more meaning is one more arm here and nothing else. The four below are
/// what a first caller needed and are no promise that four is the number.
///
/// The identities are compared by value, so one issued by another diagram is a well-formed value
/// matching nothing here — the same answer the diagram gives everything that names a figure it
/// does not hold. **A command naming nothing is a step like any other**: see
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
    /// The `from` end of a connector hangs from one of `from`'s four sides.
    Hang {
        /// The connector whose end is being hung.
        id: ShapeId,
        /// The figure the end hangs from.
        from: ShapeId,
        /// Which of that figure's four sides it hangs from.
        anchor: Anchor,
    },
    /// The figure goes one place toward the front of the order.
    Forward {
        /// The figure to move forward.
        id: ShapeId,
    },
    /// The figure is taken out.
    Remove {
        /// The figure to take out.
        id: ShapeId,
    },
}

impl Command {
    /// The one of the diagram's five operations this command is, carried out on `diagram`.
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
    pub(crate) fn apply_to(&self, diagram: &Diagram) -> Diagram {
        match *self {
            Self::Move { id, by } => {
                // Read, then write, so the figure's own displacement decides what goes back: a
                // delta of nothing gives back the same figure, and `replace` on an identity this
                // diagram does not hold changes nothing.
                if let Some(moved) = diagram.get(id).map(|shape| shape.displaced_by(by)) {
                    diagram.replace(id, moved)
                } else {
                    diagram.clone()
                }
            }
            Self::Hang { id, from, anchor } => {
                // **Only the end's position changes.** The way it leaves and the terminal it
                // writes are the caller's and stay what they were, because a command carries no
                // field for either: deriving a direction from a side is
                // [#89](https://github.com/andresmoschini/monospace/issues/89), and hanging an end
                // again is not the caller saying which way it should leave.
                if let Some(Shape::Connector {
                    from: end,
                    to,
                    stroke,
                    ..
                }) = diagram.get(id).cloned()
                {
                    diagram.replace(
                        id,
                        Shape::Connector {
                            from: Endpoint {
                                at: Position::Reference(Reference::new(
                                    from,
                                    anchor,
                                    Delta { dx: 0, dy: 0 },
                                    0,
                                )),
                                leaving: end.leaving,
                                terminal: end.terminal,
                            },
                            to,
                            stroke,
                        },
                    )
                } else {
                    diagram.clone()
                }
            }
            Self::Forward { id } => diagram.forward(id),
            Self::Remove { id } => diagram.remove(id),
        }
    }
}
