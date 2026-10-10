//! What a caller means to do to a diagram: one thing, with the fields that go with it, and the
//! diagram's own operation underneath. See the _Vocabulary_ row for `Command` and §2 of
//! [`docs/editing-model.md`](../../../docs/editing-model.md).

use monospace_diagram::{Anchor, Delta, Diagram, Endpoint, Position, Reference, Shape, ShapeId};

use crate::Session;

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
    /// Carries this command out against `session`: records the diagram as it is as a step, and
    /// changes it by the operation this command is.
    ///
    /// **The command is given the session and not the diagram**, and the signature is where that
    /// is visible rather than in a paragraph. A session method taking a command would be the same
    /// design said from the other side, and it would put the decision back where it cannot be
    /// made: only what is asked knows whether what it was asked to do was one gesture or several,
    /// and a figure dragged across a screen is one thing to the person dragging it and many
    /// displacements to the diagram. Nothing reads that answer yet — §8 of the model says what is
    /// left to write — and the field it will be read from is the step's command, which is recorded
    /// beside the diagram from this first version.
    ///
    /// **The step is recorded whatever the command did**, including when it named a figure this
    /// diagram does not hold and changed nothing. The session does not know whether the caller
    /// meant to do something, and a history that skipped a step the caller took is worse than one
    /// longer than it needed to be.
    pub fn perform(&self, session: &mut Session) {
        session.record(self.clone());
        self.apply_to(&mut session.diagram);
    }

    /// The one of the diagram's five operations this command is, carried out on `diagram`.
    ///
    /// **The whole of this method is the layer's mapping**, and it is why a caller does not have to
    /// write it: reading the figure, rebuilding the one end that changes and keeping everything
    /// else is logic that is subtly wrong in four different ways when each caller writes it. Every
    /// operation is the diagram's and unchanged, and a command that changed the diagram by any
    /// other way is not a command.
    fn apply_to(&self, diagram: &mut Diagram) {
        match *self {
            Self::Move { id, by } => {
                // Read, then write, so the figure's own displacement decides what goes back: a
                // delta of nothing gives back the same figure, and `replace` on an identity this
                // diagram does not hold changes nothing.
                if let Some(moved) = diagram.get(id).map(|shape| shape.displaced_by(by)) {
                    diagram.replace(id, moved);
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
                    );
                }
            }
            Self::Forward { id } => diagram.forward(id),
            Self::Remove { id } => diagram.remove(id),
        }
    }
}
