//! The diagram: an ordered set of shapes, drawable into a buffer the caller gives it. See
//! [`docs/diagram-model.md`](../../../docs/diagram-model.md) and
//! [ADR-0042](../../../docs/decisions/0042-draw-a-diagram-front-to-back-into-a-given-window.md).

use std::fmt;

use monospace_core::{Buffer, Layer, StampMode};

use crate::Shape;

/// A shape's identity. The identities **the diagram issues** through [`Diagram::add`] are unique
/// within it (FR-001, FR-002), and one supplied through [`Diagram::add_under`] is **not checked**:
/// two shapes may carry the same identity, and the second is then a shape no identity names.
/// [`ShapeId::new`] builds one directly from its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapeId(String);

impl ShapeId {
    /// Builds an identity directly from its text — `"#1"`, `"#2"`, and so on.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}

impl fmt::Display for ShapeId {
    /// Writes the identity as `#1`, `#2`, and so on.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One entry of a diagram's order: a shape and the identity the diagram gave it.
struct Placed {
    id: ShapeId,
    shape: Shape,
}

/// A diagram: an ordered set of shapes, and nothing else — no buffer, no glyph catalog, no
/// rendered picture (FR-005).
pub struct Diagram {
    /// The order the shapes draw in. The **last** element is the front of the order (research.md
    /// Q2): it is drawn first and decides a shared cell before anything behind it.
    shapes: Vec<Placed>,
    /// The ordinal the next `add` takes. It is the ordinal itself rather than the last one issued,
    /// so seeding it is a plain assignment and no public method carries a subtraction (Q1).
    next: u32,
}

impl Default for Diagram {
    /// An empty diagram whose next `add` takes `#1`, which is what `new` hands a caller with no
    /// opinion. Written out rather than derived because a derived `Default` would leave `next` at
    /// the `0` a zeroed field gives, and this slice changed what that field means.
    fn default() -> Self {
        Self {
            shapes: Vec::new(),
            next: 1,
        }
    }
}

impl Diagram {
    /// An empty diagram, holding no shapes, whose first addition is handed `#1`.
    #[must_use]
    pub fn new() -> Self {
        Self::numbered_from(1)
    }

    /// An empty diagram whose next `add` takes the ordinal `next`.
    #[must_use]
    pub fn numbered_from(next: u32) -> Self {
        Self {
            shapes: Vec::new(),
            next,
        }
    }

    /// Puts `shape` at the front of the order, in front of everything already there, and returns
    /// the identity the diagram gave it (FR-002, FR-006).
    pub fn add(&mut self, shape: Shape) -> ShapeId {
        let id = ShapeId(format!("#{}", self.next));
        self.next += 1;
        self.shapes.push(Placed {
            id: id.clone(),
            shape,
        });
        id
    }

    /// Puts `shape` at the front of the order, under the identity `id`.
    ///
    /// `add` with the caller's name in place of the diagram's, and it hands back nothing: two
    /// shapes may carry one identity, both are held, and neither is an error. It does **not** touch
    /// the counter, so on a diagram numbered from 3 the next `add` is still handed `#4` whatever a
    /// caller writes here.
    pub fn add_under(&mut self, id: ShapeId, shape: Shape) {
        self.shapes.push(Placed { id, shape });
    }

    /// The place in the order the shape named by `id` stands, or `None` when this diagram holds
    /// no such shape.
    ///
    /// The one search every method naming a shape by its identity goes through. `ShapeId`s are
    /// compared by value, so an identity issued by another diagram is a well-formed value that
    /// matches nothing here — the same answer the model's _Positions_ gives a reference to a shape
    /// that is not there.
    fn find(&self, id: &ShapeId) -> Option<usize> {
        self.shapes.iter().position(|placed| &placed.id == id)
    }

    /// Moves the shape named by `id` one place toward the front of the order. Does nothing when
    /// `id` names no shape here, or when it is already the front-most (FR-007, FR-009, FR-010,
    /// FR-011).
    pub fn forward(&mut self, id: &ShapeId) {
        if let Some(index) = self.find(id)
            && index + 1 < self.shapes.len()
        {
            self.shapes.swap(index, index + 1);
        }
    }

    /// Moves the shape named by `id` one place toward the back of the order. Does nothing when
    /// `id` names no shape here, or when it is already the back-most (FR-008, FR-009, FR-010,
    /// FR-011).
    pub fn backward(&mut self, id: &ShapeId) {
        if let Some(index) = self.find(id)
            && index > 0
        {
            self.shapes.swap(index, index - 1);
        }
    }

    /// The figure named by `id`, or `None` when this diagram holds no such shape.
    ///
    /// The crate's first reader, and the only one: one query by an identity the caller already
    /// holds, and nothing coming back the other way. There is no listing of a diagram's
    /// identities, no count, no order, and no way to ask which shape decided a position.
    ///
    /// It borrows, and there is no `cloned()` beside it. What comes back is the bare figure the
    /// caller added, with the identity and the place in the order held here in the diagram rather
    /// than inside the figure, so nothing about what is returned says where it sits.
    #[must_use]
    pub fn get(&self, id: &ShapeId) -> Option<&Shape> {
        self.find(id).map(|index| &self.shapes[index].shape)
    }

    /// Takes the shape named by `id` out of the diagram, changing nothing else.
    ///
    /// An identity this diagram does not hold changes nothing, with no error, no report and no
    /// panic, which is the same answer the model's _Positions_ gives a reference to a shape that
    /// is not there. The gap is closed, so what remains of the order keeps the order it had: a
    /// removal changes the holding and nothing else.
    ///
    /// It hands back nothing. There is no history and nothing to undo, so putting a figure back
    /// means building it again. The counter is not touched either, so an identity is never handed
    /// out a second time.
    pub fn remove(&mut self, id: &ShapeId) {
        if let Some(index) = self.find(id) {
            self.shapes.remove(index);
        }
    }

    /// Puts `shape` where the shape named by `id` stands, in the same place in the order.
    ///
    /// The identity and the place in the order survive, and everything the previous figure owned
    /// is the new figure's: its kind, its parameters and its position. The diagram never reaches
    /// into a box and widens it, and a replacement may change the kind outright, a box becoming a
    /// line, with nothing of the previous figure surviving it.
    ///
    /// An identity this diagram does not hold changes nothing, and the shape handed in is not
    /// added either, so there is no way to name a shape into existence. It takes the shape by
    /// value and hands back nothing, for the reason [`Diagram::remove`] does.
    pub fn replace(&mut self, id: &ShapeId, shape: Shape) {
        if let Some(index) = self.find(id) {
            self.shapes[index].shape = shape;
        }
    }

    /// Draws every shape into `buffer`, front to back, stamping every cell with
    /// [`StampMode::Below`] (FR-010 to FR-012). Drawing changes nothing about this diagram
    /// (FR-015), so two drawings into equal windows produce equal buffers.
    ///
    /// The diagram is what a connector's endpoints resolve through, which is why it is the one that
    /// hands itself to each figure. A connector with an endpoint that does not resolve — an identity
    /// this diagram does not hold, or a kind that answers no such anchor — is **not drawn at all**,
    /// and every other shape draws exactly what it drew. There is no error, no report and no way
    /// to ask which figures were dropped; see
    /// [#88](https://github.com/andresmoschini/monospace/issues/88).
    pub fn draw(&self, buffer: &mut Buffer) {
        let mut layer = Layer::new(buffer, StampMode::Below);
        for placed in self.shapes.iter().rev() {
            placed.shape.draw(&mut layer, self);
        }
    }
}

#[cfg(test)]
mod tests {
    use monospace_core::{
        BoxShape, Buffer, Cell, Connector, Direction, Glyph, GlyphCatalog, Layer, Line,
        Orientation, Pos, Shape as CoreShape, Size, StampMode, Stroke, Terminal, render,
    };

    use super::{Diagram, ShapeId};
    use crate::{Anchor, Delta, Endpoint, Position, Reference, Shape};

    fn light() -> Stroke {
        Stroke::from("light")
    }

    /// Collects every cell of `buffer` over the window `origin`/`size`, so two buffers can be
    /// compared by value even though `Buffer` derives no `PartialEq` (research.md Q7).
    fn cells(buffer: &Buffer, origin: Pos, size: Size) -> Vec<Option<Cell>> {
        (0..size.height)
            .flat_map(|dy| (0..size.width).map(move |dx| (dx, dy)))
            .map(|(dx, dy)| {
                let at = Pos {
                    x: origin.x + i32::try_from(dx).expect("width fits i32"),
                    y: origin.y + i32::try_from(dy).expect("height fits i32"),
                };
                buffer.cell(at).cloned()
            })
            .collect()
    }

    /// TE-003, scenarios 1 and 2: a diagram holding a box and a line, drawn twice into equal
    /// windows, produces equal buffers.
    #[test]
    fn drawing_the_same_diagram_twice_produces_equal_buffers() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 10,
            height: 5,
        };
        let mut diagram = Diagram::new();
        diagram.add(Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: None,
        });
        diagram.add(Shape::Line {
            at: Pos { x: 0, y: 4 },
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        });

        let mut first = Buffer::new(origin, size);
        diagram.draw(&mut first);
        let mut second = Buffer::new(origin, size);
        diagram.draw(&mut second);

        assert_eq!(cells(&first, origin, size), cells(&second, origin, size));
    }

    /// Scenario 3: an empty diagram leaves its buffer exactly as it was.
    #[test]
    fn an_empty_diagram_leaves_its_buffer_untouched() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 3,
            height: 3,
        };
        let mut buffer = Buffer::new(origin, size);
        Diagram::new().draw(&mut buffer);

        assert_eq!(
            cells(&buffer, origin, size),
            vec![None; (size.width * size.height) as usize]
        );
    }

    /// TE-002, scenario 4: a box partly outside the window draws what falls inside and nothing
    /// else, with no error.
    #[test]
    fn a_box_partly_outside_the_window_draws_only_what_falls_inside() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 3,
            height: 3,
        };
        let mut diagram = Diagram::new();
        diagram.add(Shape::Box {
            at: Pos { x: 1, y: 1 },
            size: Size {
                width: 4,
                height: 4,
            },
            stroke: light(),
            fill: None,
        });

        let mut buffer = Buffer::new(origin, size);
        diagram.draw(&mut buffer);

        assert!(buffer.cell(Pos { x: 1, y: 1 }).is_some());
        assert!(buffer.cell(Pos { x: 4, y: 4 }).is_none());
    }

    /// TE-005, scenario 5: a box drawn through a diagram matches the same `BoxShape` drawn
    /// directly.
    #[test]
    fn a_box_shape_matches_the_core_box_drawn_directly() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 3,
        };
        let mut diagram = Diagram::new();
        diagram.add(Shape::Box {
            at: origin,
            size,
            stroke: light(),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        });
        let mut actual = Buffer::new(origin, size);
        diagram.draw(&mut actual);

        let mut expected = Buffer::new(origin, size);
        BoxShape {
            at: origin,
            size,
            stroke: light(),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        }
        .draw(&mut Layer::new(&mut expected, StampMode::Above));

        assert_eq!(
            render(&actual, &GlyphCatalog::light(), origin, size),
            render(&expected, &GlyphCatalog::light(), origin, size)
        );
    }

    /// TE-005: a line drawn through a diagram matches the same `Line` drawn directly.
    #[test]
    fn a_line_shape_matches_the_core_line_drawn_directly() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 5,
            height: 1,
        };
        let mut diagram = Diagram::new();
        diagram.add(Shape::Line {
            at: origin,
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        });
        let mut actual = Buffer::new(origin, size);
        diagram.draw(&mut actual);

        let mut expected = Buffer::new(origin, size);
        Line {
            at: origin,
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        }
        .draw(&mut Layer::new(&mut expected, StampMode::Above));

        assert_eq!(
            render(&actual, &GlyphCatalog::light(), origin, size),
            render(&expected, &GlyphCatalog::light(), origin, size)
        );
    }

    /// TE-005: a connector drawn through a diagram matches the same `Connector` drawn directly, both
    /// endpoints included.
    #[test]
    fn a_connector_shape_matches_the_core_connector_drawn_directly() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 5,
            height: 4,
        };
        let from = || Endpoint {
            at: Pos { x: 0, y: 0 }.into(),
            leaving: Direction::Down,
            terminal: Terminal::Glyph {
                glyph: Glyph::new("▼").expect("one glyph"),
            },
        };
        let to = || Endpoint {
            at: Pos { x: 4, y: 3 }.into(),
            leaving: Direction::Left,
            terminal: Terminal::Glyph {
                glyph: Glyph::new("►").expect("one glyph"),
            },
        };

        let mut diagram = Diagram::new();
        diagram.add(Shape::Connector {
            from: from(),
            to: to(),
            stroke: light(),
        });
        let mut actual = Buffer::new(origin, size);
        diagram.draw(&mut actual);

        let mut expected = Buffer::new(origin, size);
        Connector {
            from: monospace_core::Endpoint {
                at: Pos { x: 0, y: 0 },
                leaving: Direction::Down,
                terminal: Terminal::Glyph {
                    glyph: Glyph::new("▼").expect("one glyph"),
                },
            },
            to: monospace_core::Endpoint {
                at: Pos { x: 4, y: 3 },
                leaving: Direction::Left,
                terminal: Terminal::Glyph {
                    glyph: Glyph::new("►").expect("one glyph"),
                },
            },
            stroke: light(),
        }
        .draw(&mut Layer::new(&mut expected, StampMode::Above));

        assert_eq!(
            render(&actual, &GlyphCatalog::light(), origin, size),
            render(&expected, &GlyphCatalog::light(), origin, size)
        );
    }

    /// B2.1, SC-003: a diagram told where its numbering resumes hands the next `add` **that**
    /// ordinal, and a further addition a different one. Asked by the identity's own text, because
    /// this is the one rule in the slice no picture can show.
    ///
    /// A counter holding the *last issued* ordinal would hand back `#4` in the first half below,
    /// which is the off-by-one `numbered_from` exists to make impossible rather than to document.
    #[test]
    fn a_seeded_diagram_hands_back_the_ordinal_it_was_seeded_with_and_then_a_different_one() {
        let a_line = || Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 1,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };

        let mut diagram = Diagram::numbered_from(3);
        let first = diagram.add(a_line());
        let second = diagram.add(a_line());

        assert_eq!(first.to_string(), "#3");
        assert_eq!(second.to_string(), "#4");
        assert_ne!(first, second);
    }

    /// 081's rule, run against a diagram that was told where its numbering resumes rather than
    /// one that started at `#1`: taking `#10` out of a diagram seeded at 10 still leaves the next
    /// addition as `#12`, so a seeded counter resumes rather than restarts.
    #[test]
    fn a_seeded_diagram_never_hands_out_an_identity_it_issued_before() {
        let a_line = || Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 1,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };

        let mut diagram = Diagram::numbered_from(10);
        let first = diagram.add(a_line());
        diagram.add(a_line());
        assert_eq!(first.to_string(), "#10");

        diagram.remove(&first);
        let after_the_removal = diagram.add(a_line());

        assert_eq!(after_the_removal.to_string(), "#12");
    }

    /// B1.3, SC-002: a shape put under a chosen identity is the one every change that names that
    /// identity acts on, and by **any other** identity it is not a shape this diagram holds.
    ///
    /// The second half is the one that is easy to leave out, and the one that keeps "unique" a
    /// claim about what the diagram issues rather than a promise it keeps on a caller's behalf.
    #[test]
    fn a_chosen_identity_is_found_by_that_identity_and_by_no_other_one() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 14,
            height: 3,
        };
        let a_box_at_zero = || Shape::Box {
            at: origin,
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: None,
        };

        let mut diagram = Diagram::new();
        let chosen = ShapeId::new("chosen");
        let other = ShapeId::new("other");
        diagram.add_under(chosen.clone(), a_box_at_zero());

        // `get` names the shape it was given.
        assert!(matches!(diagram.get(&chosen), Some(Shape::Box { .. })));

        // An identity this diagram holds for a *different* shape changes nothing, and neither does
        // one it holds for no shape at all: naming it draws nothing and adds nothing.
        let mut with_both = Diagram::new();
        with_both.add_under(chosen.clone(), a_box_at_zero());
        with_both.add_under(other.clone(), a_box_at_zero());
        assert!(matches!(with_both.get(&other), Some(Shape::Box { .. })));
        assert!(with_both.get(&ShapeId::new("absent")).is_none());
        let before_naming = cells(&draw_of(&with_both, origin, size), origin, size);
        with_both.forward(&ShapeId::new("absent"));
        with_both.backward(&ShapeId::new("absent"));
        with_both.remove(&ShapeId::new("absent"));
        with_both.replace(&ShapeId::new("absent"), a_box_at_zero());
        assert_eq!(
            cells(&draw_of(&with_both, origin, size), origin, size),
            before_naming,
            "an identity naming nothing must change no cell of the picture"
        );

        // `remove` takes out the shape the identity named and nothing else.
        let mut to_remove_from = Diagram::new();
        to_remove_from.add_under(chosen.clone(), a_box_at_zero());
        to_remove_from.add(a_box_at_zero());
        to_remove_from.remove(&chosen);
        assert!(to_remove_from.get(&chosen).is_none());
        assert!(matches!(
            to_remove_from.get(&ShapeId::new("#1")),
            Some(Shape::Box { .. })
        ));

        // `replace` puts the shape back under the identity it was given, and the identity it did
        // not name is untouched.
        let mut to_replace_in = Diagram::new();
        to_replace_in.add_under(chosen.clone(), a_box_at_zero());
        let issued = to_replace_in.add(a_box_at_zero());
        to_replace_in.replace(&chosen, a_box_at_zero());
        assert!(matches!(
            to_replace_in.get(&chosen),
            Some(Shape::Box { .. })
        ));
        assert!(matches!(
            to_replace_in.get(&issued),
            Some(Shape::Box { .. })
        ));

        // `forward` and `backward` move the shape the identity names, one place at a time, and
        // the picture says which one moved. Three **filled** boxes overlapping one another, so
        // which is in front decides the cells they share and the order is visible in the drawing
        // rather than only in the vector: `forward` on `chosen` puts it in front of the next one
        // along, and `backward` puts it back.
        //
        // The fill is what makes the order visible. Unfilled boxes that share only their borders
        // compose to the same junction whichever is in front, so a drawing of them cannot tell
        // the two orders apart — measured, not assumed.
        let a_filled_box = |x: i32, fill: &'static str| Shape::Box {
            at: Pos { x, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: Some(Glyph::new(fill).expect("one grapheme cluster")),
        };
        // `add_under` pushes onto the back of the vector and the back is the **front** of the
        // order, so the last one added is the front-most and the first is the back-most. Added in
        // this order, `chosen` is the back-most and a `forward` has somewhere to go.
        //
        // Two shapes, not three, and that is measured rather than chosen for tidiness: with three
        // overlapping boxes the orders "other, chosen, third" and "other, third, chosen" draw
        // **the same picture**, so a drawing cannot tell a chosen identity one place forward from
        // two places forward. With two there is only one place to move, and the two orders are two
        // different drawings.
        let mut to_move = Diagram::new();
        to_move.add_under(chosen.clone(), a_filled_box(0, "█"));
        to_move.add_under(other.clone(), a_filled_box(2, "░"));
        let at_the_back = cells(&draw_of(&to_move, origin, size), origin, size);

        to_move.forward(&chosen);
        let one_forward = cells(&draw_of(&to_move, origin, size), origin, size);
        assert_ne!(
            one_forward, at_the_back,
            "forward on a chosen identity must move that shape"
        );

        // The one place it moved is the one a diagram built with `chosen` in front draws — which
        // is the claim, stated as a drawing rather than as a count of places.
        let mut with_it_in_front = Diagram::new();
        with_it_in_front.add_under(other.clone(), a_filled_box(2, "░"));
        with_it_in_front.add_under(chosen.clone(), a_filled_box(0, "█"));
        assert_eq!(
            one_forward,
            cells(&draw_of(&with_it_in_front, origin, size), origin, size),
            "one forward on a chosen identity moves it exactly one place"
        );

        to_move.backward(&chosen);
        assert_eq!(
            cells(&draw_of(&to_move, origin, size), origin, size),
            at_the_back,
            "forward then backward restores the drawing"
        );

        // A `forward` on the front-most shape has nowhere to go, and changes no cell.
        to_move.forward(&other);
        assert_eq!(
            cells(&draw_of(&to_move, origin, size), origin, size),
            at_the_back,
            "forward on the front-most shape changes nothing"
        );
    }

    /// D2: `add_under` does not move the counter, and does not check the name.
    ///
    /// Both halves are the accepted cost pinned as a **behavior rather than a bug**, so a later
    /// slice that decides to report either has to say so rather than discover it.
    #[test]
    fn add_under_does_not_move_the_counter_and_does_not_check_the_name() {
        let a_box = || Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: None,
        };

        // Nothing a caller writes moves the numbering.
        let mut diagram = Diagram::numbered_from(3);
        diagram.add_under(ShapeId::new("#7"), a_box());
        let next = diagram.add(a_box());
        assert_eq!(next.to_string(), "#3");

        // Two shapes under one identity are both held, and `get` returns the first — the second is
        // a shape nobody can name until the first is taken out.
        let mut repeated = Diagram::new();
        repeated.add_under(ShapeId::new("#1"), a_box());
        repeated.add_under(ShapeId::new("#1"), a_box());
        assert!(matches!(
            repeated.get(&ShapeId::new("#1")),
            Some(Shape::Box { .. })
        ));
        repeated.remove(&ShapeId::new("#1"));
        assert!(
            matches!(repeated.get(&ShapeId::new("#1")), Some(Shape::Box { .. })),
            "the second shape under a repeated identity is still there after the first is taken out"
        );
    }

    /// TE-001: adding three shapes to one diagram yields three identities that differ from one
    /// another and read as `#1`, `#2`, `#3` in the order added.
    #[test]
    fn adding_three_shapes_yields_three_identities_in_order() {
        let mut diagram = Diagram::new();
        let first = diagram.add(Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 1,
            orientation: Orientation::Horizontal,
            stroke: light(),
        });
        let second = diagram.add(Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 1,
            orientation: Orientation::Horizontal,
            stroke: light(),
        });
        let third = diagram.add(Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 1,
            orientation: Orientation::Horizontal,
            stroke: light(),
        });

        assert_eq!(first.to_string(), "#1");
        assert_eq!(second.to_string(), "#2");
        assert_eq!(third.to_string(), "#3");
        assert_ne!(first, second);
        assert_ne!(second, third);
        assert_ne!(first, third);
    }

    /// Spec.md US1 scenario 3: adding the same shape value twice yields two different identities.
    #[test]
    fn adding_the_same_shape_value_twice_yields_two_different_identities() {
        let shape = || Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 1,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };
        let mut diagram = Diagram::new();
        let first = diagram.add(shape());
        let second = diagram.add(shape());

        assert_ne!(first, second);
    }

    /// Two overlapping boxes, each filled with its own glyph, so which one is front-most decides
    /// the shared cells rather than leaving both interior sides open (as two unfilled boxes with
    /// the same stroke do, order-independently).
    fn overlapping_boxes() -> (Shape, Shape) {
        (
            Shape::Box {
                at: Pos { x: 0, y: 0 },
                size: Size {
                    width: 4,
                    height: 3,
                },
                stroke: light(),
                fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
            },
            Shape::Box {
                at: Pos { x: 2, y: 1 },
                size: Size {
                    width: 4,
                    height: 3,
                },
                stroke: light(),
                fill: Some(Glyph::new("▓").expect("\"▓\" is one glyph")),
            },
        )
    }

    fn overlapping_core_boxes() -> (BoxShape, BoxShape) {
        (
            BoxShape {
                at: Pos { x: 0, y: 0 },
                size: Size {
                    width: 4,
                    height: 3,
                },
                stroke: light(),
                fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
            },
            BoxShape {
                at: Pos { x: 2, y: 1 },
                size: Size {
                    width: 4,
                    height: 3,
                },
                stroke: light(),
                fill: Some(Glyph::new("▓").expect("\"▓\" is one glyph")),
            },
        )
    }

    /// TE-001, scenario 1, SC-005: two overlapping boxes drawn front to back with `Below` produce
    /// the buffer that stamping the same two core shapes back to front with `Above` produces.
    #[test]
    fn front_to_back_with_below_equals_back_to_front_with_above() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let (a, b) = overlapping_boxes();
        let mut diagram = Diagram::new();
        diagram.add(a);
        diagram.add(b);
        let mut actual = Buffer::new(origin, size);
        diagram.draw(&mut actual);

        let (core_a, core_b) = overlapping_core_boxes();
        let mut expected = Buffer::new(origin, size);
        core_a.draw(&mut Layer::new(&mut expected, StampMode::Above));
        core_b.draw(&mut Layer::new(&mut expected, StampMode::Above));

        assert_eq!(cells(&actual, origin, size), cells(&expected, origin, size));
    }

    /// TE-004, scenarios 2 and 5: the same two overlapping boxes in opposite orders produce
    /// different buffers, and in each the front-most shape's stroke decides the shared cells.
    #[test]
    fn opposite_orders_produce_different_buffers() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let (a, b) = overlapping_boxes();
        let mut a_then_b = Diagram::new();
        a_then_b.add(a);
        a_then_b.add(b);
        let mut a_then_b_buffer = Buffer::new(origin, size);
        a_then_b.draw(&mut a_then_b_buffer);

        let (a, b) = overlapping_boxes();
        let mut b_then_a = Diagram::new();
        b_then_a.add(b);
        b_then_a.add(a);
        let mut b_then_a_buffer = Buffer::new(origin, size);
        b_then_a.draw(&mut b_then_a_buffer);

        assert_ne!(
            cells(&a_then_b_buffer, origin, size),
            cells(&b_then_a_buffer, origin, size)
        );
    }

    /// Scenarios 3 and 4: a crossing is composition, not occlusion — a horizontal and a vertical
    /// line that cross make a junction glyph, and a filled box in front of a line hides it
    /// wherever the box's interior covers it, while the box's border still composes with the
    /// line into a junction where the two meet.
    #[test]
    fn a_crossing_composes_and_a_filled_box_occludes() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 5,
            height: 5,
        };

        let mut crossing = Diagram::new();
        crossing.add(Shape::Line {
            at: Pos { x: 0, y: 2 },
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        });
        crossing.add(Shape::Line {
            at: Pos { x: 2, y: 0 },
            len: 5,
            orientation: Orientation::Vertical,
            stroke: light(),
        });
        let mut crossing_buffer = Buffer::new(origin, size);
        crossing.draw(&mut crossing_buffer);
        assert_eq!(
            render(&crossing_buffer, &GlyphCatalog::light(), origin, size),
            "  │  \n  │  \n──┼──\n  │  \n  │  \n"
        );

        let mut occluding = Diagram::new();
        occluding.add(Shape::Line {
            at: Pos { x: 2, y: 0 },
            len: 5,
            orientation: Orientation::Vertical,
            stroke: light(),
        });
        occluding.add(Shape::Box {
            at: Pos { x: 1, y: 1 },
            size: Size {
                width: 3,
                height: 3,
            },
            stroke: light(),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        });
        let mut occluding_buffer = Buffer::new(origin, size);
        occluding.draw(&mut occluding_buffer);
        assert_eq!(
            render(&occluding_buffer, &GlyphCatalog::light(), origin, size),
            "  │  \n ┌┴┐ \n │░│ \n └┬┘ \n  │  \n"
        );
    }

    /// TE-002: two partially overlapping opaque boxes, drawn before and after the back one moves
    /// forward, produce different buffers, and the second equals what the same two boxes added in
    /// the opposite order produce.
    #[test]
    fn moving_the_back_one_forward_matches_adding_them_in_the_opposite_order() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let (a, b) = overlapping_boxes();
        let mut diagram = Diagram::new();
        let back = diagram.add(a);
        diagram.add(b);
        let mut before = Buffer::new(origin, size);
        diagram.draw(&mut before);

        diagram.forward(&back);
        let mut after = Buffer::new(origin, size);
        diagram.draw(&mut after);

        assert_ne!(cells(&before, origin, size), cells(&after, origin, size));

        let (a, b) = overlapping_boxes();
        let mut opposite = Diagram::new();
        opposite.add(b);
        opposite.add(a);
        let mut opposite_buffer = Buffer::new(origin, size);
        opposite.draw(&mut opposite_buffer);

        assert_eq!(
            cells(&after, origin, size),
            cells(&opposite_buffer, origin, size)
        );
    }

    /// TE-003: the same expected buffer comes from moving the front one backward instead, against
    /// the same two boxes.
    #[test]
    fn moving_the_front_one_backward_matches_moving_the_back_one_forward() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let (a, b) = overlapping_boxes();
        let mut diagram = Diagram::new();
        diagram.add(a);
        let front = diagram.add(b);
        diagram.backward(&front);
        let mut buffer = Buffer::new(origin, size);
        diagram.draw(&mut buffer);

        let (a, b) = overlapping_boxes();
        let mut opposite = Diagram::new();
        opposite.add(b);
        opposite.add(a);
        let mut opposite_buffer = Buffer::new(origin, size);
        opposite.draw(&mut opposite_buffer);

        assert_eq!(
            cells(&buffer, origin, size),
            cells(&opposite_buffer, origin, size)
        );
    }

    /// TE-004: moving the front-most forward, and moving the back-most backward, each leave the
    /// drawn buffer unchanged.
    #[test]
    fn moving_the_front_most_forward_or_the_back_most_backward_changes_nothing() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let (a, b) = overlapping_boxes();
        let mut diagram = Diagram::new();
        let back = diagram.add(a);
        let front = diagram.add(b);
        let mut before = Buffer::new(origin, size);
        diagram.draw(&mut before);

        diagram.forward(&front);
        let mut after_forward = Buffer::new(origin, size);
        diagram.draw(&mut after_forward);
        assert_eq!(
            cells(&before, origin, size),
            cells(&after_forward, origin, size)
        );

        diagram.backward(&back);
        let mut after_backward = Buffer::new(origin, size);
        diagram.draw(&mut after_backward);
        assert_eq!(
            cells(&before, origin, size),
            cells(&after_backward, origin, size)
        );
    }

    /// TE-004, edge case: a diagram holding one shape is unchanged by either move, since that
    /// shape is both front-most and back-most.
    #[test]
    fn a_diagram_holding_one_shape_is_unchanged_by_either_move() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 4,
            height: 3,
        };
        let mut diagram = Diagram::new();
        let only = diagram.add(Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        });
        let mut before = Buffer::new(origin, size);
        diagram.draw(&mut before);

        diagram.forward(&only);
        diagram.backward(&only);
        let mut after = Buffer::new(origin, size);
        diagram.draw(&mut after);

        assert_eq!(cells(&before, origin, size), cells(&after, origin, size));
    }

    /// TE-005, edge case: an identity kept from another diagram changes nothing through either
    /// method, with no panic.
    #[test]
    fn an_identity_from_another_diagram_changes_nothing_and_does_not_panic() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let (a, b) = overlapping_boxes();
        let mut diagram = Diagram::new();
        diagram.add(a);
        diagram.add(b);
        let mut before = Buffer::new(origin, size);
        diagram.draw(&mut before);

        let mut other = Diagram::new();
        let foreign = other.add(Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 1,
            orientation: Orientation::Horizontal,
            stroke: light(),
        });

        diagram.forward(&foreign);
        diagram.backward(&foreign);
        let mut after = Buffer::new(origin, size);
        diagram.draw(&mut after);

        assert_eq!(cells(&before, origin, size), cells(&after, origin, size));
    }

    /// TE-005, edge case: an empty diagram is unchanged by either method, with no panic.
    #[test]
    fn an_empty_diagram_is_unchanged_by_either_move() {
        let mut other = Diagram::new();
        let foreign = other.add(Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 1,
            orientation: Orientation::Horizontal,
            stroke: light(),
        });

        let mut diagram = Diagram::new();
        diagram.forward(&foreign);
        diagram.backward(&foreign);

        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 3,
            height: 3,
        };
        let mut buffer = Buffer::new(origin, size);
        diagram.draw(&mut buffer);

        assert_eq!(
            cells(&buffer, origin, size),
            vec![None; (size.width * size.height) as usize]
        );
    }

    /// TE-006: a shape moved forward and then backward by the same identity draws exactly what it
    /// drew at the start.
    #[test]
    fn moving_a_shape_forward_then_backward_restores_the_original_drawing() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let (a, b) = overlapping_boxes();
        let mut diagram = Diagram::new();
        let back = diagram.add(a);
        diagram.add(b);
        let mut before = Buffer::new(origin, size);
        diagram.draw(&mut before);

        diagram.forward(&back);
        diagram.backward(&back);
        let mut after = Buffer::new(origin, size);
        diagram.draw(&mut after);

        assert_eq!(cells(&before, origin, size), cells(&after, origin, size));
    }

    /// Edge case: two unfilled boxes with the same stroke whose shared cells every side leaves
    /// `Unset` — an inner box entirely inside an outer box's interior, so their border cells never
    /// coincide — draw identically before and after a reorder.
    #[test]
    fn two_unfilled_boxes_with_no_shared_cell_draw_identically_regardless_of_order() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 5,
        };
        let outer = || Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 6,
                height: 5,
            },
            stroke: light(),
            fill: None,
        };
        let inner = || Shape::Box {
            at: Pos { x: 2, y: 2 },
            size: Size {
                width: 2,
                height: 2,
            },
            stroke: light(),
            fill: None,
        };

        let mut outer_then_inner = Diagram::new();
        outer_then_inner.add(outer());
        outer_then_inner.add(inner());
        let mut a = Buffer::new(origin, size);
        outer_then_inner.draw(&mut a);

        let mut inner_then_outer = Diagram::new();
        inner_then_outer.add(inner());
        inner_then_outer.add(outer());
        let mut b = Buffer::new(origin, size);
        inner_then_outer.draw(&mut b);

        assert_eq!(cells(&a, origin, size), cells(&b, origin, size));
    }

    // ------------------------------------------------------ a figure displaced by a delta

    /// Draws `shapes` into a fresh window, in the order given, so that a figure is observed the
    /// only way this crate lets it be observed. Both sides of every comparison below arrive
    /// through here: the figure displaced, and the same figure added where it landed.
    fn drawn(shapes: Vec<Shape>, origin: Pos, size: Size) -> Buffer {
        let mut diagram = Diagram::new();
        for shape in shapes {
            diagram.add(shape);
        }
        draw_of(&diagram, origin, size)
    }

    /// Draws a diagram the caller already holds, for a claim about what a change to it did.
    fn draw_of(diagram: &Diagram, origin: Pos, size: Size) -> Buffer {
        let mut buffer = Buffer::new(origin, size);
        diagram.draw(&mut buffer);
        buffer
    }

    /// The positions where two buffers of one window differ, so that a claim about which cells a
    /// change reached can be made by coordinates rather than by reading two pictures side by side.
    fn differing(before: &Buffer, after: &Buffer, origin: Pos, size: Size) -> Vec<Pos> {
        (0..size.height)
            .flat_map(|dy| (0..size.width).map(move |dx| (dx, dy)))
            .map(|(dx, dy)| Pos {
                x: origin.x + i32::try_from(dx).expect("width fits i32"),
                y: origin.y + i32::try_from(dy).expect("height fits i32"),
            })
            .filter(|at| before.cell(*at) != after.cell(*at))
            .collect()
    }

    /// Three filled boxes that all overlap one another, so which of them is front-most decides the
    /// cells they share and a change of order changes the picture. The first is the back-most and
    /// the last the front-most, and the second overlaps both.
    fn three_overlapping_boxes() -> (Shape, Shape, Shape) {
        let filled_box = |at: Pos, fill: &str| Shape::Box {
            at,
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: Some(Glyph::new(fill).expect("one glyph")),
        };
        (
            filled_box(Pos { x: 0, y: 0 }, "░"),
            filled_box(Pos { x: 2, y: 1 }, "▓"),
            filled_box(Pos { x: 0, y: 1 }, "▒"),
        )
    }

    /// An identity no diagram in these tests holds, and the one the specification's cases mean by
    /// an identity from another diagram.
    ///
    /// It is the **third** identity another diagram issued rather than its first, because
    /// `ShapeId` is a string: an identity from elsewhere matches nothing here only because its
    /// number differs, and a diagram that issued one shape issues `#1`, which is exactly the
    /// identity a diagram holding two shapes has already used.
    fn a_foreign_identity() -> ShapeId {
        let a_line = || Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 1,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };
        let mut other = Diagram::new();
        other.add(a_line());
        other.add(a_line());
        other.add(a_line())
    }

    /// User Story 1, spec's B3.1 scenario, SC-004: a box displaced two cells right draws exactly
    /// what the same box added at the displaced position draws.
    ///
    /// The expected picture is built by adding the box where it landed rather than pinned as text,
    /// so what is asserted is where a displacement puts a figure and not how a box draws. A
    /// displacement that quietly did nothing would draw the box where it already stood, and the two
    /// pictures would differ.
    #[test]
    fn a_displaced_box_draws_where_the_same_box_at_that_position_would_draw() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 9,
            height: 3,
        };
        let box_at = |x: i32| Shape::Box {
            at: Pos { x, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: None,
        };

        let moved = box_at(1).displaced_by(Delta { dx: 2, dy: 0 });

        assert_eq!(
            cells(&drawn(vec![moved], origin, size), origin, size),
            cells(&drawn(vec![box_at(3)], origin, size), origin, size)
        );
    }

    /// User Story 1, spec's B3.2 scenario, SC-004: a connector displaced two cells down draws
    /// exactly what the same connector added with **both** endpoints at `y + 2` draws.
    ///
    /// Both endpoints is the whole claim, and it is why the expected picture is built from the two
    /// positions rather than pinned as text. A displacement that moved one endpoint draws a
    /// connector between two rows, and one that moved neither is the silent no-op the previous
    /// system shipped along with a `// TODO: implement it`. Either passes a comparison against a
    /// picture of a connector that never moved, so this is the one neither passes.
    #[test]
    fn a_displaced_connector_draws_with_both_endpoints_moved() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 7,
            height: 5,
        };
        let connector = |from: Pos, to: Pos| Shape::Connector {
            from: Endpoint {
                at: from.into(),
                leaving: Direction::Right,
                terminal: Terminal::Glyph {
                    glyph: Glyph::new("◄").expect("one glyph"),
                },
            },
            to: Endpoint {
                at: to.into(),
                leaving: Direction::Left,
                terminal: Terminal::Glyph {
                    glyph: Glyph::new("►").expect("one glyph"),
                },
            },
            stroke: light(),
        };

        let moved =
            connector(Pos { x: 1, y: 1 }, Pos { x: 5, y: 1 }).displaced_by(Delta { dx: 0, dy: 2 });

        assert_eq!(
            cells(&drawn(vec![moved], origin, size), origin, size),
            cells(
                &drawn(
                    vec![connector(Pos { x: 1, y: 3 }, Pos { x: 5, y: 3 })],
                    origin,
                    size
                ),
                origin,
                size
            )
        );
    }

    /// User Story 1, spec's B3.3 scenario: displacing a figure a diagram holds changes no cell of
    /// it. A displacement builds a value; putting that value back under an identity is what changes
    /// anything. The second half of the rule is T026, which needs `replace` to exist.
    ///
    /// The `assert_ne!` is what keeps the test honest: without it a `displaced_by` that moved
    /// nothing would satisfy "changed no cell" by doing exactly that, and the rule would go
    /// unchecked.
    #[test]
    fn displacing_a_figure_the_diagram_holds_changes_no_cell_of_it() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let (a, b) = overlapping_boxes();
        let mut diagram = Diagram::new();
        diagram.add(a.clone());
        diagram.add(b);
        let mut before = Buffer::new(origin, size);
        diagram.draw(&mut before);

        let displaced = a.displaced_by(Delta { dx: 1, dy: 2 });
        let mut after = Buffer::new(origin, size);
        diagram.draw(&mut after);

        assert_eq!(cells(&before, origin, size), cells(&after, origin, size));
        assert_ne!(displaced, a);
    }

    // ------------------------------------------- reading a figure back, and the two changes

    /// User Story 2, spec's B4.1 scenario: `get` on the identity `add` handed back returns a figure
    /// equal by value to the one added.
    ///
    /// Compared by value rather than by picture, because this is the only rule in the slice no
    /// picture can show. The widened derives are what make it sayable at all, and what comes back
    /// is the bare figure: the identity and the place in the order are the diagram's, held beside
    /// the figure rather than inside it.
    #[test]
    fn get_returns_the_figure_the_addition_named() {
        let (a, _b) = overlapping_boxes();
        let mut diagram = Diagram::new();
        let id = diagram.add(a.clone());

        assert_eq!(diagram.get(&id), Some(&a));
    }

    /// User Story 2, spec's B4.2 scenario, SC-003: `get` on an identity this diagram does not hold
    /// gives nothing, and the picture it draws is the one it drew before.
    ///
    /// The identity is one another diagram issued, which is the case the specification means.
    #[test]
    fn get_on_an_identity_from_another_diagram_gives_nothing_and_changes_nothing() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let (a, b) = overlapping_boxes();
        let mut diagram = Diagram::new();
        diagram.add(a);
        diagram.add(b);
        let before = draw_of(&diagram, origin, size);

        assert_eq!(diagram.get(&a_foreign_identity()), None);
        assert_eq!(
            cells(&before, origin, size),
            cells(&draw_of(&diagram, origin, size), origin, size)
        );
    }

    /// User Story 3, spec's B1.1 scenario, SC-001: a diagram of several figures, drawn before and
    /// after one is taken out, produces different buffers, and the figures that stayed draw exactly
    /// what they drew on their own.
    ///
    /// Three overlapping boxes rather than a row, and the one taken out is the first of the three
    /// rather than a figure in the middle. That is what makes the test say something a swap with
    /// the last entry could not pass: closing the gap leaves the survivors in the order they were,
    /// so the one that was in front is still in front, and swapping the removed entry with the
    /// back-most would hand the diagram the other two pictures instead.
    #[test]
    fn taking_a_shape_out_changes_the_picture_and_leaves_the_rest_drawing_what_they_drew() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let (first, second, third) = three_overlapping_boxes();

        let mut diagram = Diagram::new();
        let taken_out = diagram.add(first);
        diagram.add(second.clone());
        diagram.add(third.clone());
        let before = draw_of(&diagram, origin, size);

        diagram.remove(&taken_out);
        let after = draw_of(&diagram, origin, size);

        assert_ne!(cells(&before, origin, size), cells(&after, origin, size));
        assert_eq!(
            cells(&after, origin, size),
            cells(&drawn(vec![second, third], origin, size), origin, size)
        );
    }

    /// User Story 3, spec's B1.2 scenario, SC-003: taking out an identity this diagram does not
    /// hold leaves the buffer exactly as it was, with no error, no report and no panic. The
    /// identity is one another diagram issued, which is the case worth running: a well-formed
    /// value that matches nothing here.
    #[test]
    fn taking_out_an_identity_from_another_diagram_changes_nothing_and_does_not_panic() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let (a, b) = overlapping_boxes();
        let mut diagram = Diagram::new();
        diagram.add(a);
        diagram.add(b);
        let before = draw_of(&diagram, origin, size);

        diagram.remove(&a_foreign_identity());

        assert_eq!(
            cells(&before, origin, size),
            cells(&draw_of(&diagram, origin, size), origin, size)
        );
    }

    /// User Story 3, spec's B1.3 scenario, SC-005: take `#1` out, add a figure, and the identity
    /// handed back is `#3` rather than `#1`.
    ///
    /// Asserted by the identity's own text rather than by a picture, and it needs no code beyond
    /// the absence of a decrement, because `add` already increments before use.
    #[test]
    fn an_identity_is_never_handed_out_again_after_a_removal() {
        let a_line = || Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 1,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };

        let mut diagram = Diagram::new();
        let first = diagram.add(a_line());
        diagram.add(a_line());
        assert_eq!(first.to_string(), "#1");

        diagram.remove(&first);
        let after_the_removal = diagram.add(a_line());

        assert_eq!(after_the_removal.to_string(), "#3");
    }

    /// Edge case: the last shape taken out leaves an empty diagram, and drawing an empty diagram
    /// leaves the buffer as it was. The rule `an_empty_diagram_leaves_its_buffer_untouched` already
    /// pins, reached here through a removal rather than through a diagram never given anything.
    #[test]
    fn taking_out_the_last_shape_leaves_an_empty_diagram() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let (a, _b) = overlapping_boxes();
        let mut diagram = Diagram::new();
        let only = diagram.add(a);

        diagram.remove(&only);

        assert_eq!(
            cells(&draw_of(&diagram, origin, size), origin, size),
            vec![None; (size.width * size.height) as usize]
        );
    }

    /// User Story 4, spec's B2.1 scenario, SC-002: a box put back as a wider box draws exactly what
    /// that wider box added on its own produces.
    ///
    /// Pinned against the wider box's own picture rather than against the box it replaced, which is
    /// what makes the claim about the figure handed in and not about a difference between two.
    #[test]
    fn a_box_put_back_as_a_wider_box_draws_that_wider_box() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 9,
            height: 3,
        };
        let box_of = |width: u32| Shape::Box {
            at: Pos { x: 1, y: 0 },
            size: Size { width, height: 3 },
            stroke: light(),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        };

        let mut diagram = Diagram::new();
        let id = diagram.add(box_of(4));
        diagram.replace(&id, box_of(7));

        assert_eq!(
            cells(&draw_of(&diagram, origin, size), origin, size),
            cells(&drawn(vec![box_of(7)], origin, size), origin, size)
        );
    }

    /// User Story 4, spec's B2.2 scenario, SC-002: a box put back as a line draws the line, kind
    /// included, and nothing of the previous figure survives.
    ///
    /// The pair the specification draws by hand and calls hypothetical, each side pinned against
    /// the figure handed in rather than against the other, which is what turns "nothing of the
    /// previous figure survives" into a checked claim rather than a comparison of two pictures.
    #[test]
    fn a_box_put_back_as_a_line_draws_the_line() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 4,
            height: 3,
        };
        let the_box = || Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        };
        let the_line = || Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 4,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };

        let mut diagram = Diagram::new();
        let id = diagram.add(the_box());
        diagram.replace(&id, the_line());

        assert_eq!(
            cells(&draw_of(&diagram, origin, size), origin, size),
            cells(&drawn(vec![the_line()], origin, size), origin, size)
        );
    }

    /// User Story 4, spec's B2.3 scenario: a figure overlapping another, put back under its own
    /// identity unchanged, resolves the overlap as it did.
    ///
    /// This is what shows a replacement is not a reorder, and the second assertion says so
    /// directly: it holds up the order a remove-and-add would have produced as the picture that
    /// replacement must **not** draw. Two figures would not have been enough, since removing the
    /// front-most of two and adding it back leaves the order it was.
    #[test]
    fn a_figure_put_back_unchanged_resolves_its_overlap_as_it_did() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let (a, b, c) = three_overlapping_boxes();
        let mut diagram = Diagram::new();
        diagram.add(a.clone());
        let middle = diagram.add(b.clone());
        diagram.add(c.clone());
        let before = draw_of(&diagram, origin, size);

        diagram.replace(&middle, b.clone());
        let after = draw_of(&diagram, origin, size);

        assert_eq!(cells(&before, origin, size), cells(&after, origin, size));
        assert_ne!(
            cells(&after, origin, size),
            cells(&drawn(vec![a, c, b], origin, size), origin, size)
        );
    }

    /// User Story 4, spec's B2.4 scenario, SC-003: a shape put under an identity this diagram does
    /// not hold leaves the picture alone **and adds nothing**.
    ///
    /// The figure handed in is placed where it would be plainly visible, so a `replace` that fell
    /// back to removing the old entry and adding the new one would fail on the picture rather than
    /// on a count. The identity assertion is the other half: `#1` still names the figure it always
    /// named, so there is no way to name a shape into existence.
    #[test]
    fn a_shape_put_under_a_foreign_identity_is_neither_drawn_nor_added() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 4,
        };
        let (a, b) = overlapping_boxes();
        let mut diagram = Diagram::new();
        diagram.add(a.clone());
        diagram.add(b);
        let before = draw_of(&diagram, origin, size);

        diagram.replace(
            &a_foreign_identity(),
            Shape::Box {
                at: Pos { x: 1, y: 1 },
                size: Size {
                    width: 3,
                    height: 3,
                },
                stroke: light(),
                fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
            },
        );
        let after = draw_of(&diagram, origin, size);

        assert_eq!(cells(&before, origin, size), cells(&after, origin, size));
        assert_eq!(diagram.get(&ShapeId::new("#1")), Some(&a));
    }

    /// User Story 1, spec's B3.3 scenario, second half: draw, displace, `replace`, and the picture
    /// is the one the same three figures draw with the middle one standing where the displacement
    /// put it.
    ///
    /// It lands beside the replacement rather than beside the displacement because it is the half
    /// that needs `replace` to exist. The reach is then pinned by coordinates: the cells the
    /// figure held before and the cells it holds now, and not one more. A change reaching a cell
    /// of either box, which is what a move implemented as a removal and an addition would do, is
    /// outside that set.
    #[test]
    fn putting_a_displaced_figure_back_reaches_only_the_cells_that_figure_holds() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 11,
            height: 3,
        };
        let filled_box = |x: i32, fill: &str| Shape::Box {
            at: Pos { x, y: 0 },
            size: Size {
                width: 3,
                height: 3,
            },
            stroke: light(),
            fill: Some(Glyph::new(fill).expect("one glyph")),
        };
        let the_line = |x: i32| Shape::Line {
            at: Pos { x, y: 1 },
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };

        let mut diagram = Diagram::new();
        diagram.add(filled_box(0, "░"));
        let middle = diagram.add(the_line(3));
        diagram.add(filled_box(8, "▓"));
        let before = draw_of(&diagram, origin, size);

        let moved = the_line(3).displaced_by(Delta { dx: 2, dy: 0 });
        diagram.replace(&middle, moved.clone());
        let after = draw_of(&diagram, origin, size);

        assert_eq!(
            cells(&after, origin, size),
            cells(
                &drawn(
                    vec![filled_box(0, "░"), moved, filled_box(8, "▓")],
                    origin,
                    size
                ),
                origin,
                size
            )
        );

        let held_by_the_line = |at: Pos| at.y == 1 && (3..=9).contains(&at.x);
        let reached = differing(&before, &after, origin, size);
        assert!(
            !reached.is_empty(),
            "the displacement reached nothing at all"
        );
        assert!(
            reached.iter().all(|at| held_by_the_line(*at)),
            "the change reached outside the displaced figure: {reached:?}"
        );
    }

    // ----------------------------------------- an endpoint that hangs from another figure's side

    /// The box every case below hangs an endpoint from: four by three at the origin, so its right
    /// side center is `{3, 1}` — the point the shipped demonstration's connector already holds, and
    /// the one the model shows an endpoint attached at.
    const THE_SIDE_CENTRE: Pos = Pos { x: 3, y: 1 };

    fn the_box() -> Shape {
        Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: None,
        }
    }

    /// A connector with both ends as arms and the given positions, so a picture's difference is the
    /// route and the two cells it starts and ends on, and nothing else about how a terminal draws.
    ///
    /// The directions are the caller's rather than the anchor's (§6 _Attachment_), so a case that
    /// puts the reference in the other slot has to name them the other way round rather than reuse
    /// these: a connector leaving rightward from its far end takes a different route, and one that
    /// did is testing the direction rather than the anchor.
    fn arm_connector(from: Position, to: Position) -> Shape {
        connector_leaving(from, Direction::Right, to, Direction::Left)
    }

    fn connector_leaving(
        from: Position,
        leaving: Direction,
        to: Position,
        arriving: Direction,
    ) -> Shape {
        Shape::Connector {
            from: Endpoint {
                at: from,
                leaving,
                terminal: Terminal::Arm,
            },
            to: Endpoint {
                at: to,
                leaving: arriving,
                terminal: Terminal::Arm,
            },
            stroke: light(),
        }
    }

    /// A second box that has nothing to do with either end, placed clear of everything else so that
    /// "this other figure drew exactly what it drew" is readable.
    fn the_unrelated_box() -> Shape {
        Shape::Box {
            at: Pos { x: 9, y: 0 },
            size: Size {
                width: 3,
                height: 3,
            },
            stroke: light(),
            fill: Some(Glyph::new("░").expect("one glyph")),
        }
    }

    /// The window the cases below are drawn in.
    fn the_window() -> (Pos, Size) {
        (
            Pos { x: 0, y: 0 },
            Size {
                width: 13,
                height: 4,
            },
        )
    }

    /// User Story 2, spec's B2.1, SC-001: a connector whose `from` hangs from a box's right side
    /// draws exactly what the same connector with `from` at that point draws.
    ///
    /// Each side is pinned against the point rather than against the other, which is what keeps a
    /// reference resolving to the wrong place from passing on a pair that are wrong together. So the
    /// coordinates are asserted first and the picture second: `resolve` has to be `{3, 1}` on its
    /// own account, and only then do the two drawings have to agree.
    #[test]
    fn a_hanging_endpoint_draws_what_the_point_it_resolves_to_draws() {
        let (origin, size) = the_window();

        let mut diagram = Diagram::new();
        let box_id = diagram.add(the_box());
        let hanging = arm_connector(
            Position::Reference(Reference {
                id: box_id.clone(),
                anchor: Anchor::Right,
                offset: Delta { dx: 0, dy: 0 },
            }),
            Pos { x: 8, y: 1 }.into(),
        );
        diagram.add(hanging.clone());

        // Pinned by coordinates first: a reference that resolved anywhere else fails here.
        let Shape::Connector { from, .. } = &hanging else {
            unreachable!("the figure above is a connector")
        };
        assert_eq!(from.at.resolve(&diagram), Some(THE_SIDE_CENTRE));

        let mut absolute = Diagram::new();
        absolute.add(the_box());
        absolute.add(arm_connector(
            THE_SIDE_CENTRE.into(),
            Pos { x: 8, y: 1 }.into(),
        ));

        assert_eq!(
            cells(&draw_of(&diagram, origin, size), origin, size),
            cells(&draw_of(&absolute, origin, size), origin, size)
        );
    }

    /// User Story 1, spec's B1.1, B1.2, SC-001 and SC-003: a reference standing two cells clear of a
    /// side draws exactly what the same connector standing at the point it resolves to draws.
    ///
    /// The partner of the zero-offset test above rather than a replacement of it, and the reason is
    /// what each one catches. That test's offset is zero on both axes, so a `resolve` that added
    /// nothing at all would satisfy it; this one's offset is two cells on the x, so the same failure
    /// draws its route from `{3, 1}` instead of `{5, 1}` and the two pictures stop agreeing.
    ///
    /// The coordinates are asserted **before** the picture, each against the absolute point: `resolve`
    /// has to be `{5, 1}` on its own account before the two drawings are asked to agree, so a
    /// reference that resolved to the wrong place cannot pass on a pair that is wrong together.
    #[test]
    fn an_offset_endpoint_draws_what_the_point_it_resolves_to_draws() {
        let (origin, size) = the_window();
        let the_far_end = Pos { x: 8, y: 1 };

        let mut diagram = Diagram::new();
        let box_id = diagram.add(the_box());
        let offset = Delta { dx: 2, dy: 0 };
        let hanging = arm_connector(
            Position::Reference(Reference {
                id: box_id,
                anchor: Anchor::Right,
                offset,
            }),
            the_far_end.into(),
        );
        diagram.add(hanging.clone());

        // Pinned by coordinates first: `{3, 1}` is the side centre and the offset is two cells right
        // of it, so a `resolve` that ignored the field fails here rather than in the picture below.
        let Shape::Connector { from, .. } = &hanging else {
            unreachable!("the figure above is a connector")
        };
        assert_eq!(from.at.resolve(&diagram), Some(Pos { x: 5, y: 1 }));
        assert_ne!(from.at.resolve(&diagram), Some(THE_SIDE_CENTRE));
        assert_ne!(from.at, Pos { x: 5, y: 1 }.into());

        let mut absolute = Diagram::new();
        absolute.add(the_box());
        absolute.add(arm_connector(Pos { x: 5, y: 1 }.into(), the_far_end.into()));

        assert_eq!(
            cells(&draw_of(&diagram, origin, size), origin, size),
            cells(&draw_of(&absolute, origin, size), origin, size)
        );
    }

    /// User Story 2, spec's B2.1 and SC-002: the offset is a gap **from the side**, so displacing the
    /// figure a reference hangs from carries the endpoint with it and the gap does not change.
    ///
    /// The expected picture is built from the two positions — the box where it landed and the
    /// connector starting at the point the reference resolves to *there* — rather than pinned as
    /// text, so what is claimed is where a displacement puts a figure and not how a connector draws.
    /// With the box four cells right its right side centre is `{7, 1}`, and the unchanged offset of
    /// two carries the endpoint to `{9, 1}`: the same two cells of gap as before, not the same two
    /// columns of the canvas.
    ///
    /// The `assert_ne!`s are what make the picture comparison mean anything here. A `resolve` that
    /// ignored the field would still draw a valid connector in this window, because `{7, 1}` is
    /// still a cell a route can start from — the picture alone would let it pass.
    #[test]
    fn a_displaced_box_carries_an_offset_endpoint_with_it_and_the_gap_does_not_change() {
        let (origin, size) = the_window();
        let the_far_end = Pos { x: 11, y: 1 };
        let offset = Delta { dx: 2, dy: 0 };

        let mut diagram = Diagram::new();
        let box_id = diagram.add(the_box());
        let hanging = arm_connector(
            Position::Reference(Reference {
                id: box_id.clone(),
                anchor: Anchor::Right,
                offset,
            }),
            the_far_end.into(),
        );
        diagram.add(hanging);

        // Which endpoint the hanging connector resolves to, asked the way the drawing asks it.
        let the_hanging_end = |d: &Diagram| {
            let Shape::Connector { from, .. } = d
                .get(&crate::ShapeId::new("#2"))
                .expect("the connector is the second figure")
            else {
                unreachable!("the figure under test is a connector")
            };
            from.at.resolve(d)
        };
        assert_eq!(the_hanging_end(&diagram), Some(Pos { x: 5, y: 1 }));
        assert_ne!(the_hanging_end(&diagram), Some(THE_SIDE_CENTRE));
        let before = draw_of(&diagram, origin, size);

        let moved = the_box().displaced_by(Delta { dx: 4, dy: 0 });
        diagram.replace(&box_id, moved);
        let after = draw_of(&diagram, origin, size);

        // The same two figures with the box where it landed and the endpoint at the point the
        // reference resolves to now: `{7, 1}` plus the unchanged two cells.
        let mut expected = Diagram::new();
        expected.add(the_box().displaced_by(Delta { dx: 4, dy: 0 }));
        expected.add(arm_connector(Pos { x: 9, y: 1 }.into(), the_far_end.into()));

        assert_eq!(
            cells(&after, origin, size),
            cells(&draw_of(&expected, origin, size), origin, size)
        );

        // The gap, pinned by coordinate rather than by eye: the endpoint moved by the same four
        // cells the box did, and it is not where the side centre alone would leave it.
        assert_eq!(the_hanging_end(&diagram), Some(Pos { x: 9, y: 1 }));
        assert_ne!(the_hanging_end(&diagram), Some(Pos { x: 7, y: 1 }));

        let reached = differing(&before, &after, origin, size);
        assert!(
            !reached.is_empty(),
            "the displacement reached nothing at all"
        );
        assert!(
            reached.iter().all(|at| at.x <= the_far_end.x),
            "the change reached past the endpoint that did not move: {reached:?}"
        );
    }

    /// User Story 2, spec's B2.3: a box **replaced by a line** under an offset adds it to the line's
    /// own side middle, not to the place the box stood.
    ///
    /// It takes a non-zero `dy` to tell the two apart. A horizontal line read as a flat box is one
    /// cell tall, so its top and bottom centres are **the same point asked twice** — the offset is
    /// added to that one answer and not to two — and with `dy: 0` the endpoint would land on the
    /// line's own row, where a `resolve` that had used the *box's* old bottom centre `{1, 2}` would
    /// differ. So the two candidate answers are `{2, 1}` and `{1, 2}`: different cells, neither the
    /// other, and the assertion cannot pass on a pair that is wrong together.
    #[test]
    fn an_offset_on_a_line_is_added_to_the_lines_own_middle_not_where_the_box_stood() {
        let (origin, size) = the_window();
        let the_far_end = Pos { x: 8, y: 2 };
        let offset = Delta { dx: 0, dy: 1 };
        let the_box_s_own_bottom = Pos { x: 1, y: 2 };
        let the_line = Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };

        // The line's own bottom centre is `{2, 0}`, and the offset makes it `{2, 1}`.
        assert_eq!(the_line.anchor(Anchor::Bottom), Some(Pos { x: 2, y: 0 }));
        assert_eq!(the_line.anchor(Anchor::Top), Some(Pos { x: 2, y: 0 }));
        let the_line_endpoint = offset.apply(
            the_line
                .anchor(Anchor::Bottom)
                .expect("a line answers every anchor"),
        );
        assert_eq!(the_line_endpoint, Pos { x: 2, y: 1 });
        assert_ne!(the_line_endpoint, the_box_s_own_bottom);

        // The same line, three ways, and no two of them against each other: the reference a line
        // holds, the point that reference resolves to, and the diagram where a box was replaced.
        let with_the_reference = || {
            let mut d = Diagram::new();
            let id = d.add(the_line.clone());
            d.add(arm_connector(
                Position::Reference(Reference {
                    id,
                    anchor: Anchor::Bottom,
                    offset,
                }),
                the_far_end.into(),
            ));
            d
        };
        let mut with_the_point = Diagram::new();
        with_the_point.add(the_line.clone());
        with_the_point.add(arm_connector(the_line_endpoint.into(), the_far_end.into()));

        let mut replaced = Diagram::new();
        let identity = replaced.add(the_box());
        replaced.add(arm_connector(
            Position::Reference(Reference {
                id: identity.clone(),
                anchor: Anchor::Bottom,
                offset,
            }),
            the_far_end.into(),
        ));
        // `replace` swaps the kind under the identity the reference already names, so the offset is
        // held unchanged across a change of what stands there.
        replaced.replace(&identity, the_line.clone());

        assert_eq!(
            cells(&draw_of(&with_the_reference(), origin, size), origin, size),
            cells(&draw_of(&with_the_point, origin, size), origin, size),
            "a reference on a line drew something other than the point it resolves to"
        );
        assert_eq!(
            cells(&draw_of(&replaced, origin, size), origin, size),
            cells(&draw_of(&with_the_point, origin, size), origin, size),
            "a replaced box left the offset on the box's old side middle"
        );
    }

    /// User Story 2, spec's B2.2, SC-004, and the spec's edge case: a large offset on a reference
    /// that resolves to nothing is still nothing — asked twice, and by drawing.
    ///
    /// Twice because there are two different ways not to resolve and they are not the same code: an
    /// identity no `add` ever issued, and an anchor a kind does not answer — here a connector, which
    /// is the answer that keeps a chain of references one link long. Each carries an offset big
    /// enough to be somewhere, because an offset of nothing is the case the zero-offset tests above
    /// already hold.
    ///
    /// Each asserts two things together and the second is what makes the first mean anything: the
    /// connector is **absent from the output**, and every other figure's cells are unchanged. That
    /// second half is what distinguishes "the connector is not drawn" from "the drawing stopped", and
    /// a connector drawn partly passes the first alone.
    #[test]
    fn a_large_offset_on_a_reference_that_resolves_to_nothing_draws_nothing_either() {
        let (origin, size) = the_window();
        let large = Delta { dx: 6, dy: 2 };
        let the_other_end: Position = Pos { x: 8, y: 1 }.into();
        let unrelated = the_unrelated_box();

        let cases = [
            (
                "an identity nothing holds",
                Position::Reference(Reference {
                    id: a_foreign_identity(),
                    anchor: Anchor::Right,
                    offset: large,
                }),
            ),
            (
                "a kind that answers no anchor",
                Position::Reference(Reference {
                    id: crate::ShapeId::new("#3"),
                    anchor: Anchor::Right,
                    offset: large,
                }),
            ),
        ];

        for (what, hanging) in cases {
            let mut with_it = Diagram::new();
            with_it.add(the_box());
            with_it.add(unrelated.clone());
            // A connector, so that a reference naming it asks a kind that answers no anchor.
            with_it.add(arm_connector(
                Pos { x: 0, y: 3 }.into(),
                Pos { x: 12, y: 3 }.into(),
            ));
            with_it.add(arm_connector(hanging, the_other_end.clone()));

            let mut without_it = Diagram::new();
            without_it.add(the_box());
            without_it.add(unrelated.clone());
            without_it.add(arm_connector(
                Pos { x: 0, y: 3 }.into(),
                Pos { x: 12, y: 3 }.into(),
            ));

            assert_eq!(
                cells(&draw_of(&with_it, origin, size), origin, size),
                cells(&draw_of(&without_it, origin, size), origin, size),
                "{what} drew something, or drew something else"
            );
        }

        // The other figure's cells are still there, which is the half that tells a whole figure
        // missing from a drawing that stopped.
        let drawing = draw_of(
            &{
                let mut d = Diagram::new();
                d.add(the_box());
                d.add(unrelated.clone());
                d.add(arm_connector(
                    Pos { x: 0, y: 3 }.into(),
                    Pos { x: 12, y: 3 }.into(),
                ));
                d.add(arm_connector(
                    Position::Reference(Reference {
                        id: a_foreign_identity(),
                        anchor: Anchor::Right,
                        offset: large,
                    }),
                    the_other_end,
                ));
                d
            },
            origin,
            size,
        );
        assert!(
            drawing
                .cell(
                    the_unrelated_box()
                        .anchor(Anchor::Top)
                        .expect("a box answers every anchor")
                )
                .is_some(),
            "the drawing stopped rather than the connector being skipped"
        );
    }

    /// User Story 2, spec's edge case: a box **one cell wide** with an offset — its two coincident
    /// side centres get the offset added once, and the offset is what separates them afterwards.
    ///
    /// The degenerate figure is what an implementation that special-cased the ordinary box gets
    /// wrong, and 082's `a_box_one_cell_wide_or_one_cell_tall_answers_the_same_rule` in `position.rs`
    /// is why the general rule is the claim rather than the coincidence. At width 1 the left and
    /// right centres are the same cell, so a `resolve` that added the offset once per *side* rather
    /// than once per *reference* would land two cells out; and an implementation that measured the
    /// offset against the side it is named for would find two sides and no way to tell them apart.
    #[test]
    fn a_box_one_cell_wide_takes_the_offset_once_and_the_offset_separates_its_two_sides() {
        let (origin, size) = the_window();
        let the_far_end = Pos { x: 8, y: 2 };
        let offset = Delta { dx: 3, dy: 1 };
        let one_wide = Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 3,
            },
            stroke: light(),
            fill: None,
        };

        // The two centres coincide before the offset, and the offset is added to that one answer.
        assert_eq!(one_wide.anchor(Anchor::Left), Some(Pos { x: 0, y: 1 }));
        assert_eq!(one_wide.anchor(Anchor::Right), Some(Pos { x: 0, y: 1 }));
        assert_eq!(offset.apply(Pos { x: 0, y: 1 }), Pos { x: 3, y: 2 });

        for (what, anchor) in [("left", Anchor::Left), ("right", Anchor::Right)] {
            let mut diagram = Diagram::new();
            let box_id = diagram.add(one_wide.clone());
            let hanging = arm_connector(
                Position::Reference(Reference {
                    id: box_id,
                    anchor,
                    offset,
                }),
                the_far_end.into(),
            );
            diagram.add(hanging.clone());

            let Shape::Connector { from, .. } = &hanging else {
                unreachable!("the figure above is a connector")
            };
            assert_eq!(
                from.at.resolve(&diagram),
                Some(Pos { x: 3, y: 2 }),
                "the {what} side of a one-cell-wide box resolves to the offset once, not twice"
            );

            let mut absolute = Diagram::new();
            absolute.add(one_wide.clone());
            absolute.add(arm_connector(Pos { x: 3, y: 2 }.into(), the_far_end.into()));

            assert_eq!(
                cells(&draw_of(&diagram, origin, size), origin, size),
                cells(&draw_of(&absolute, origin, size), origin, size),
                "the {what} side drew something else"
            );
        }
    }

    /// User Story 2, spec's B2.2, SC-002 and SC-005: displacing the box four cells right takes the
    /// hanging end with it, re-routes the connector to the end that did not move, and touches
    /// nothing else.
    ///
    /// The expected picture is built from the two positions rather than pinned as text, so what is
    /// claimed is where a displacement puts a figure and not how a connector draws. The reach is
    /// then pinned by coordinates: no column past the far endpoint moved, and that endpoint's own
    /// cell is byte-identical before and after. A resolution cached when the diagram was built
    /// rather than asked when it is drawn passes the first comparison and fails both of these.
    #[test]
    fn a_displaced_box_takes_the_endpoint_hanging_from_it_along() {
        let (origin, size) = the_window();
        let the_far_end = Pos { x: 11, y: 1 };

        let mut diagram = Diagram::new();
        let box_id = diagram.add(the_box());
        diagram.add(arm_connector(
            Position::Reference(Reference {
                id: box_id.clone(),
                anchor: Anchor::Right,
                offset: Delta { dx: 0, dy: 0 },
            }),
            the_far_end.into(),
        ));
        let before = draw_of(&diagram, origin, size);

        let moved = the_box().displaced_by(Delta { dx: 4, dy: 0 });
        diagram.replace(&box_id, moved);
        let after = draw_of(&diagram, origin, size);

        // The same two figures with the box where it landed and the endpoint at the point the
        // reference now resolves to: `{7, 1}`, four cells right of where it resolved before.
        let mut expected = Diagram::new();
        expected.add(Shape::Box {
            at: Pos { x: 4, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: None,
        });
        expected.add(arm_connector(Pos { x: 7, y: 1 }.into(), the_far_end.into()));

        assert_eq!(
            cells(&after, origin, size),
            cells(&draw_of(&expected, origin, size), origin, size)
        );

        let reached = differing(&before, &after, origin, size);
        assert!(
            !reached.is_empty(),
            "the displacement reached nothing at all"
        );
        assert!(
            reached.iter().all(|at| at.x <= the_far_end.x),
            "the change reached past the endpoint that did not move: {reached:?}"
        );
        assert_eq!(
            before.cell(the_far_end),
            after.cell(the_far_end),
            "the endpoint that did not move was drawn differently"
        );
    }

    /// User Story 2, spec's B2.3: a connector with one endpoint absolute and one hanging, each
    /// placed by its own rule.
    ///
    /// Both orders, because the two positions are the same field with different rules and a match
    /// written to suit one of them is the failure this case is for. Nothing here is pinned against
    /// the other arrangement: each is pinned against the same connector with both points absolute.
    #[test]
    fn a_connector_places_each_of_its_ends_by_its_own_rule() {
        let (origin, size) = the_window();
        // A box whose **bottom** side center is `{7, 2}`, so the two ends of the route below are
        // `{0, 2}` and `{7, 2}` and neither is the other.
        let the_held_box = || Shape::Box {
            at: Pos { x: 6, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: None,
        };
        let the_far_end = Pos { x: 0, y: 2 };
        let the_hanging_end = Pos { x: 7, y: 2 };

        // Each case is pinned against the same route with both ends named as points, and never
        // against the other case: a connector with its reference in the other slot leaves and
        // arrives the other way round, so the two are different routes and saying so is the point.
        let expected = |origin: Pos, size: Size| {
            let mut diagram = Diagram::new();
            diagram.add(the_held_box());
            diagram.add(arm_connector(the_far_end.into(), the_hanging_end.into()));
            cells(&draw_of(&diagram, origin, size), origin, size)
        };
        let expected_reversed = |origin: Pos, size: Size| {
            let mut diagram = Diagram::new();
            diagram.add(the_held_box());
            diagram.add(arm_connector(the_hanging_end.into(), the_far_end.into()));
            cells(&draw_of(&diagram, origin, size), origin, size)
        };

        let mut hanging_to = Diagram::new();
        let box_id = hanging_to.add(the_held_box());
        hanging_to.add(arm_connector(
            the_far_end.into(),
            Position::Reference(Reference {
                id: box_id,
                anchor: Anchor::Bottom,
                offset: Delta { dx: 0, dy: 0 },
            }),
        ));

        let mut hanging_from = Diagram::new();
        let box_id = hanging_from.add(the_held_box());
        hanging_from.add(arm_connector(
            Position::Reference(Reference {
                id: box_id,
                anchor: Anchor::Bottom,
                offset: Delta { dx: 0, dy: 0 },
            }),
            the_far_end.into(),
        ));

        assert_eq!(
            cells(&draw_of(&hanging_to, origin, size), origin, size),
            expected(origin, size)
        );
        assert_eq!(
            cells(&draw_of(&hanging_from, origin, size), origin, size),
            expected_reversed(origin, size)
        );
    }

    /// User Story 2, spec's B2.4, SC-003, and the three non-resolutions ADR-0041 enumerates: a
    /// reference to an identity this diagram does not hold, a reference to a kind that answers no
    /// anchor, and a connector with one endpoint that resolves and one that does not.
    ///
    /// Each case asserts two things together, and the second is what makes the first mean anything.
    /// The connector is **absent from the output**, and every other figure's cells are unchanged —
    /// which is what distinguishes "the connector is not drawn" from "the drawing stopped". The
    /// third case is the one a connector drawn partly would pass on the first half alone: half an
    /// arm is a route with no head, and only the whole-picture comparison rules it out.
    ///
    /// The three figures the endpoints do not touch are the same in every case, and the one the
    /// third case's unresolvable reference names is a connector, which answers no anchor at all.
    fn a_diagram_of_three_figures_and_maybe_a_fourth(under_test: Option<Shape>) -> Diagram {
        let mut diagram = Diagram::new();
        diagram.add(the_box());
        diagram.add(the_unrelated_box());
        // A connector, so that a reference naming it asks a kind that answers no anchor.
        diagram.add(arm_connector(
            Pos { x: 0, y: 3 }.into(),
            Pos { x: 12, y: 3 }.into(),
        ));
        if let Some(shape) = under_test {
            diagram.add(shape);
        }
        diagram
    }

    #[test]
    fn a_connector_with_an_endpoint_that_does_not_resolve_is_not_drawn_at_all() {
        let (origin, size) = the_window();
        let unresolvable = Position::Reference(Reference {
            id: a_foreign_identity(),
            anchor: Anchor::Right,
            offset: Delta { dx: 0, dy: 0 },
        });
        let answers_nothing = Position::Reference(Reference {
            id: crate::ShapeId::new("#3"),
            anchor: Anchor::Right,
            offset: Delta { dx: 0, dy: 0 },
        });
        let the_other_end: Position = Pos { x: 8, y: 1 }.into();

        let cases = [
            (
                "an identity nothing holds",
                unresolvable.clone(),
                the_other_end.clone(),
            ),
            (
                "a kind that answers no anchor",
                Pos { x: 5, y: 2 }.into(),
                answers_nothing,
            ),
            (
                "one end that resolves and one that does not",
                unresolvable,
                the_other_end,
            ),
        ];

        for (what, from, to) in cases {
            let with_it =
                a_diagram_of_three_figures_and_maybe_a_fourth(Some(arm_connector(from, to)));
            let without_it = a_diagram_of_three_figures_and_maybe_a_fourth(None);

            assert_eq!(
                cells(&draw_of(&with_it, origin, size), origin, size),
                cells(&draw_of(&without_it, origin, size), origin, size),
                "{what} drew something, or drew something else"
            );
        }
    }

    /// User Story 2, spec's B2.5: a reference to an identity nothing holds yet, and then a figure
    /// added under it — three kinds, because a kind answers anchors differently.
    ///
    /// The identity is spelled rather than read back, and **the connector is the first figure
    /// added** so that the figure under test is the one the counter names next. That is what makes
    /// the case reachable at all: `add_under` is `pub` and takes the caller's name rather than the
    /// counter's, so a spelled identity is the only way to hold one the counter has not reached yet
    /// — which `#2` is exactly here, since the connector took `#1` first. Nothing holds it at
    /// first and the hanging connector draws nothing; then a figure arrives and the same reference
    /// finds it.
    ///
    /// **This paragraph used to claim a diagram offers no way to name a shape into existence, and
    /// that is what `add_under` falsified** — it is that way, and it is `pub`, which is what the
    /// case above is built to reach (see #148).
    ///
    /// A `Box` answers, a `Line` answers as a flat box, and a `Connector` does not answer at all —
    /// which is the same picture as a reference to a shape that was never there.
    #[test]
    fn a_figure_added_under_a_spelled_identity_is_what_a_hanging_endpoint_finds() {
        let (origin, size) = the_window();
        let spelled = crate::ShapeId::new("#2");
        let the_far_end: Position = Pos { x: 11, y: 1 }.into();
        let the_filler = || arm_connector(Pos { x: 0, y: 0 }.into(), Pos { x: 1, y: 0 }.into());

        let hanging = |anchor| {
            arm_connector(
                Position::Reference(Reference {
                    id: spelled.clone(),
                    anchor,
                    offset: Delta { dx: 0, dy: 0 },
                }),
                the_far_end.clone(),
            )
        };

        // Nothing holds `#2` yet, so the hanging connector draws nothing at all.
        let mut nothing_under_it = Diagram::new();
        nothing_under_it.add(hanging(Anchor::Right));
        nothing_under_it.add(the_filler());

        let mut just_the_filler = Diagram::new();
        just_the_filler.add(the_filler());
        assert_eq!(
            cells(&draw_of(&nothing_under_it, origin, size), origin, size),
            cells(&draw_of(&just_the_filler, origin, size), origin, size),
            "a reference to nothing drew something"
        );

        // A box takes `#2`, and the connector hangs from its right side.
        let mut with_a_box = Diagram::new();
        with_a_box.add(hanging(Anchor::Right));
        with_a_box.add(the_box());
        with_a_box.add(the_filler());

        let mut absolute_after_a_box = Diagram::new();
        absolute_after_a_box.add(the_filler());
        absolute_after_a_box.add(the_box());
        absolute_after_a_box.add(arm_connector(THE_SIDE_CENTRE.into(), the_far_end.clone()));
        assert_eq!(
            cells(&draw_of(&with_a_box, origin, size), origin, size),
            cells(&draw_of(&absolute_after_a_box, origin, size), origin, size),
            "a box under a spelled identity is not where a reference finds it"
        );

        // A line takes `#2` instead, and the connector lands on the line's own far end — read as a
        // box one cell thick, a five-cell line's right side center is its last cell.
        let the_line = || Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };
        let mut with_a_line = Diagram::new();
        with_a_line.add(hanging(Anchor::Right));
        with_a_line.add(the_line());
        with_a_line.add(the_filler());

        let mut absolute_after_a_line = Diagram::new();
        absolute_after_a_line.add(the_filler());
        absolute_after_a_line.add(the_line());
        absolute_after_a_line.add(arm_connector(
            Pos { x: 4, y: 0 }.into(),
            the_far_end.clone(),
        ));
        assert_eq!(
            cells(&draw_of(&with_a_line, origin, size), origin, size),
            cells(&draw_of(&absolute_after_a_line, origin, size), origin, size),
            "a line under a spelled identity is not where a reference finds it"
        );

        // A connector takes `#2`, and the hanging connector stops drawing — the same answer as
        // before there was anything under the identity at all.
        let mut with_a_connector = Diagram::new();
        with_a_connector.add(hanging(Anchor::Right));
        with_a_connector.add(the_filler());

        assert_eq!(
            cells(&draw_of(&with_a_connector, origin, size), origin, size),
            cells(&draw_of(&just_the_filler, origin, size), origin, size),
            "a connector under a spelled identity still answered an anchor"
        );
    }

    /// User Story 2, and the claim the conversion's own test used to carry: a connector with a glyph
    /// terminal and a connector with an arm, each drawn through a diagram, produce the buffer the
    /// same core `Connector` drawn directly produces.
    ///
    /// A stronger pin than the conversion was, because it goes through the four lines that built it
    /// rather than naming them: a terminal or a direction dropped on the way would change the
    /// drawing, and the drawing is what is compared.
    #[test]
    fn a_connector_with_either_terminal_reaches_the_core_whole() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 8,
            height: 3,
        };
        let glyph = |text: &str| Terminal::Glyph {
            glyph: Glyph::new(text).expect("one glyph"),
        };

        for (from_terminal, to_terminal) in
            [(glyph("◄"), glyph("►")), (Terminal::Arm, Terminal::Arm)]
        {
            let endpoint = |at: Pos, leaving: Direction, terminal: Terminal| Endpoint {
                at: at.into(),
                leaving,
                terminal,
            };
            let from = endpoint(Pos { x: 0, y: 0 }, Direction::Down, from_terminal.clone());
            let to = endpoint(Pos { x: 7, y: 2 }, Direction::Up, to_terminal.clone());

            let mut diagram = Diagram::new();
            diagram.add(Shape::Connector {
                from: from.clone(),
                to: to.clone(),
                stroke: light(),
            });
            let mut actual = Buffer::new(origin, size);
            diagram.draw(&mut actual);

            let mut expected = Buffer::new(origin, size);
            Connector {
                from: monospace_core::Endpoint {
                    at: Pos { x: 0, y: 0 },
                    leaving: Direction::Down,
                    terminal: from_terminal,
                },
                to: monospace_core::Endpoint {
                    at: Pos { x: 7, y: 2 },
                    leaving: Direction::Up,
                    terminal: to_terminal,
                },
                stroke: light(),
            }
            .draw(&mut Layer::new(&mut expected, StampMode::Above));

            assert_eq!(cells(&actual, origin, size), cells(&expected, origin, size));
        }
    }

    /// User Story 3, spec's B3.1, SC-004: taking the referenced box out leaves the connector drawing
    /// nothing and leaves the figure that had nothing to do with either drawing exactly what it drew
    /// **on its own**.
    ///
    /// The second half is pinned against that box's own picture rather than against the first one,
    /// which is what makes the claim about the figures that stayed rather than a difference between
    /// two drawings. The reference is not rewritten, not reported and does not panic: taking the
    /// box out is 081's verb and this is what it now means.
    #[test]
    fn taking_the_referenced_figure_out_stops_the_connector_and_changes_nothing_else() {
        let (origin, size) = the_window();

        let mut diagram = Diagram::new();
        let box_id = diagram.add(the_box());
        diagram.add(arm_connector(
            Position::Reference(Reference {
                id: box_id.clone(),
                anchor: Anchor::Right,
                offset: Delta { dx: 0, dy: 0 },
            }),
            Pos { x: 8, y: 1 }.into(),
        ));
        diagram.add(the_unrelated_box());
        let before = draw_of(&diagram, origin, size);

        diagram.remove(&box_id);
        let after = draw_of(&diagram, origin, size);

        assert_ne!(cells(&before, origin, size), cells(&after, origin, size));
        assert_eq!(
            cells(&after, origin, size),
            cells(
                &drawn(vec![the_unrelated_box()], origin, size),
                origin,
                size
            )
        );
    }

    /// User Story 3, spec's B3.2, SC-004: a figure put back under the referenced identity makes the
    /// connector draw again, hanging from the **new** figure's side rather than where the old one
    /// stood.
    ///
    /// The case is put through `replace` rather than through `remove` followed by an addition, and
    /// the reason is the model's rather than a convenience: `remove` frees an identity permanently
    /// **as far as `add` is concerned** — `remove` does not touch the counter, so the next `add`
    /// hands out the next ordinal and never the removed one — while `add_under` puts a figure back
    /// under that very name and the picture returns byte for byte, which is the spec's B1.2 and
    /// `a_figure_put_back_under_the_removed_identity_draws_again`'s claim. What `replace` gives is
    /// the same resolution from the other side: the identity is found, the kind answers the anchor,
    /// and the connector is drawn from wherever the figure now stands. **That half is a reason about
    /// resolution rather than about removals, and it is the half that survives** — the half this
    /// paragraph used to give, about there being no `add_under`, did not.
    /// [#142](https://github.com/andresmoschini/monospace/issues/142) is where what a removal
    /// should do to a reference is answered.
    ///
    /// A **line** is what goes back, not a second box, so a kind change is covered by the same case:
    /// read as a box one cell thick, a five-cell line's right side center is its last cell, which is
    /// neither where the removed box answered nor the connector's own far end.
    #[test]
    fn a_figure_put_back_under_the_referenced_identity_draws_the_connector_again() {
        let (origin, size) = the_window();
        let the_far_end = Pos { x: 11, y: 1 };

        let mut diagram = Diagram::new();
        let identity = diagram.add(the_box());
        diagram.add(arm_connector(
            Position::Reference(Reference {
                id: identity.clone(),
                anchor: Anchor::Right,
                offset: Delta { dx: 0, dy: 0 },
            }),
            the_far_end.into(),
        ));
        let before = draw_of(&diagram, origin, size);

        let the_line = Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };
        diagram.replace(&identity, the_line.clone());
        let after = draw_of(&diagram, origin, size);

        let mut expected = Diagram::new();
        expected.add(the_line);
        expected.add(arm_connector(Pos { x: 4, y: 0 }.into(), the_far_end.into()));

        assert_eq!(
            cells(&after, origin, size),
            cells(&draw_of(&expected, origin, size), origin, size)
        );
        assert_ne!(
            cells(&before, origin, size),
            cells(&after, origin, size),
            "the replacement did not reach the picture at all"
        );
    }

    // ------------------------------- taking a shape out, and the three ways a reference can miss

    /// The window the removal cases below are drawn in: **twelve by three at the origin**, which is
    /// the canvas the specification's own hand-drawn picture is measured on, rather than the one
    /// `the_window()` above uses.
    fn the_arrangement_window() -> (Pos, Size) {
        (
            Pos { x: 0, y: 0 },
            Size {
                width: 12,
                height: 3,
            },
        )
    }

    /// The second box of the arrangement below: three by three at `{8, 0}`, clear of the first box
    /// and of everything between them.
    ///
    /// **It is the whole reason this arrangement is not `the_arrangement()`'s.** A removal is only
    /// legible if some figure stays, and this one is the figure that survives it — the sole answer
    /// to "every other figure is drawn exactly as it would have been". `the_unrelated_box()` would
    /// do as a bystander and cannot do here: it is filled, it stands at `{9, 0}` rather than
    /// `{8, 0}`, and `{8, 1}` is where the arrow's own far end is.
    fn the_far_box() -> Shape {
        Shape::Box {
            at: Pos { x: 8, y: 0 },
            size: Size {
                width: 3,
                height: 3,
            },
            stroke: light(),
            fill: None,
        }
    }

    /// The arrow of the arrangement below: its `from` naming `#1`'s right side with a gap of
    /// nothing, and its `to` the plain point `{8, 1}`. Both ends arms, so what a picture shows is
    /// the route and nothing about how a terminal draws.
    fn the_hanging_connector(box_id: ShapeId) -> Shape {
        arm_connector(
            Position::Reference(Reference {
                id: box_id,
                anchor: Anchor::Right,
                offset: Delta { dx: 0, dy: 0 },
            }),
            Pos { x: 8, y: 1 }.into(),
        )
    }

    /// The two boxes and the identity of the first — the arrangement minus its arrow, so that a
    /// case which leaves the arrow out is the same two figures rather than a second arrangement
    /// spelled from scratch.
    fn the_two_boxes() -> (Diagram, ShapeId) {
        let mut diagram = Diagram::new();
        let the_one_taken_out = diagram.add(the_box());
        diagram.add(the_far_box());
        (diagram, the_one_taken_out)
    }

    /// The arrangement as `data-model.md` spells it: `#1` the four-by-three box at the origin, `#2`
    /// the far box, and `#3` the arrow between them.
    ///
    /// Returns `#1` — the identity every removal below takes out — and `#3`.
    fn the_arrangement_with_a_survivor() -> (Diagram, ShapeId, ShapeId) {
        let (mut diagram, the_one_taken_out) = the_two_boxes();
        let the_arrow = diagram.add(the_hanging_connector(the_one_taken_out.clone()));
        (diagram, the_one_taken_out, the_arrow)
    }

    /// The same two boxes with **no arrow in the arrangement at all** — the baseline the arrow's own
    /// footprint is read as a difference against.
    ///
    /// A count of the arrow's cells taken off either picture would count the two boxes along with
    /// it; this third arrangement is what makes the difference mean what it says.
    fn the_same_arrangement_without_the_arrow() -> Diagram {
        the_two_boxes().0
    }

    /// The connector that stands where the box stood in route C, under `#1`.
    ///
    /// It writes **its own two cells** at `{0, 0}` and `{1, 0}` — both endpoints included, so the
    /// far end is a cell along and not the third one — which is what makes route C *not* the same
    /// buffer as the other two. And it answers no anchor at all, which is §4's table's second row
    /// reached without a removal.
    fn the_connector_under_the_box() -> Shape {
        arm_connector(Pos { x: 0, y: 0 }.into(), Pos { x: 1, y: 0 }.into())
    }

    /// The spec's route C: the same arrangement with **that connector** under the box's identity, so
    /// `#1` is still held and the arrow's reference names a figure that cannot answer.
    fn the_same_arrangement_with_a_connector_under_the_box() -> (Diagram, ShapeId, ShapeId) {
        let mut diagram = Diagram::new();
        diagram.add_under(ShapeId::new("#1"), the_connector_under_the_box());
        diagram.add(the_far_box());
        let the_arrow = diagram.add(the_hanging_connector(ShapeId::new("#1")));
        (diagram, ShapeId::new("#1"), the_arrow)
    }

    /// An identity the arrangement above never issues and no `add` here is holding out: the
    /// arrangement stops issuing at `#4`, and `#1`, `#2` and `#3` are all held.
    ///
    /// Spelled rather than read back, because the point is that **nothing can be read back for
    /// it**. A reference to a shape that is *coming* is a normal state rather than a fault, so the
    /// arrival has to be one a reader can see is absent.
    fn an_identity_nothing_holds() -> ShapeId {
        ShapeId::new("#7")
    }

    /// The spec's route A: the arrangement with **nothing at all where the near box stood** — it was
    /// never added — so the arrow's reference names an identity no figure is held under.
    ///
    /// **The near box is absent rather than present-but-unreferenced, and that is the whole of
    /// route A.** A reference to nothing while the figure still stands would draw a picture the
    /// model calls route A's neighbor rather than route A itself: §4's table is about a position
    /// that does not resolve, and a box nobody references is a box that draws. Here the far box
    /// holds `#1`, the arrow is `#2`, and the identity the arrow names is one nothing holds.
    fn the_same_arrangement_with_the_near_box_never_added() -> (Diagram, ShapeId) {
        let mut diagram = Diagram::new();
        diagram.add(the_far_box());
        let the_arrow = diagram.add(the_hanging_connector(an_identity_nothing_holds()));
        (diagram, the_arrow)
    }

    /// User Story 1, spec's B1.1, and SC-003: taking the box out draws exactly the far box, and the
    /// picture is **byte for byte** what a reference naming an identity nothing ever held draws.
    ///
    /// **Two comparisons rather than one, and the order is the claim.** Putting the removal beside
    /// the missing identity is §11's second question in the only form a test can ask it; the
    /// comparison against `the_far_box()` drawn on its own is what says **no other figure moved**,
    /// since two pictures agreeing is a weaker claim than a picture being a known one.
    ///
    /// **`#2` is the only figure in this arrangement that survives the removal**, which is what makes
    /// the removal legible at all — without it, a removal and an empty diagram would draw the same
    /// thing and the case would say nothing about the figures that stayed.
    ///
    /// **The `get` answers come first, and they are why this test exists rather than the
    /// comparison.** `taking_the_referenced_figure_out_stops_the_connector_and_changes_nothing_else`
    /// at `diagram.rs:2512` already pins this very picture, so what is new here is the **reasons**
    /// beside a second assertion of the same buffer: that a removal and an identity nothing holds
    /// are indistinguishable to a reader, and that the diagram stopped holding `#1` rather than
    /// merely stopped drawing it. Route A here is the specification's B3.1 — the **near box was
    /// never added at all**, not present and unreferenced.
    ///
    /// **One claim in this file is accepted here with nothing to verify it, and it is the one this
    /// test's second comparison leans on**: that a removal leaves **every** other figure byte for
    /// byte. What is asserted is that for the three figures this arrangement names. That is a claim
    /// about the whole diagram rather than about the figure that went, and no single fixture can
    /// establish it for every diagram — a fixture with more figures in it would be a different
    /// test rather than a stronger one.
    #[test]
    fn a_removal_and_a_missing_identity_draw_the_same_thing() {
        let (origin, size) = the_arrangement_window();

        let (mut diagram, the_one_taken_out, the_arrow) = the_arrangement_with_a_survivor();
        diagram.remove(&the_one_taken_out);
        assert_eq!(
            diagram.get(&the_one_taken_out),
            None,
            "the box is still held under the identity that was removed"
        );
        assert_eq!(
            diagram.get(&the_arrow),
            Some(&the_hanging_connector(the_one_taken_out.clone())),
            "the removal took the arrow with it rather than only what the arrow hung from"
        );
        let after_the_removal = draw_of(&diagram, origin, size);

        let (never_added, _) = the_same_arrangement_with_the_near_box_never_added();
        let after_a_missing_identity = draw_of(&never_added, origin, size);

        assert_eq!(
            cells(&after_the_removal, origin, size),
            cells(&after_a_missing_identity, origin, size),
            "a removal and a reference naming an identity nothing holds drew different things"
        );
        assert_eq!(
            cells(&after_the_removal, origin, size),
            cells(&drawn(vec![the_far_box()], origin, size), origin, size),
            "something other than the surviving figure is still drawn"
        );
    }

    /// User Story 1, spec's B1.2, and SC-003: a figure put back **under the removed identity**
    /// draws the whole picture again, byte for byte — the claim nothing in the crate stated before,
    /// so this is a measurement rather than a restatement of a rule.
    ///
    /// §9 says re-adding a shape with the same identity would make the references resolve again,
    /// and this is the test that finds out whether it can. `add_under` is the only way to spell the
    /// identity at all; `add_hands_back_an_identity_no_shape_holds_after_a_removal` is what the
    /// ordinary way does instead, and the reason this is a separate test rather than the same one.
    ///
    /// The comparison is the whole buffer and not "the arrow is there", because "the arrow is there"
    /// is what a rule drawing a fragment of the route would satisfy too.
    #[test]
    fn a_figure_put_back_under_the_removed_identity_draws_again() {
        let (origin, size) = the_arrangement_window();
        let (mut diagram, the_one_taken_out, _) = the_arrangement_with_a_survivor();
        let before = draw_of(&diagram, origin, size);

        diagram.remove(&the_one_taken_out);
        diagram.add_under(the_one_taken_out.clone(), the_box());

        assert_eq!(
            diagram.get(&the_one_taken_out),
            Some(&the_box()),
            "the figure went back under a different identity"
        );
        assert_eq!(
            cells(&draw_of(&diagram, origin, size), origin, size),
            cells(&before, origin, size),
            "the picture did not come back byte for byte"
        );
    }

    /// User Story 1, spec's B1.3, and SC-003: after a removal `add` hands back an identity **no
    /// shape holds** — `#4`, where `#1` was taken out — and the arrow is still not drawn. A caller
    /// walking into the hole.
    ///
    /// **The last half is the one that is not arithmetic**, and it is where the value of this test
    /// is: that `add` skips `#1` falls out of `Diagram`'s own counter rather than out of a rule.
    /// `remove` does not touch `next`, so the counter never reissues, and `add_under` is the only
    /// way to spell an identity. **Nothing repairs the hole**, and a caller who re-adds without
    /// naming the identity sees a diagram that looks the same as before and hangs from nothing —
    /// which is what the last assertion is: **the two boxes and not the arrow**, a picture its
    /// author would take for a reference that still works.
    ///
    /// **This is the second of the three figures `a_removal_and_a_missing_identity_draw_the_same_
    /// thing` names**, and it carries the same accepted claim that one does — that a removal leaves
    /// every other figure byte for byte. It is named here too rather than in one place and assumed
    /// at the other, because this test is where a caller would look for what a re-add does to the
    /// picture around it.
    #[test]
    fn add_hands_back_an_identity_no_shape_holds_after_a_removal() {
        let (origin, size) = the_arrangement_window();
        let (mut diagram, the_one_taken_out, _) = the_arrangement_with_a_survivor();

        diagram.remove(&the_one_taken_out);
        let handed_back = diagram.add(the_box());

        assert_eq!(
            handed_back.to_string(),
            "#4",
            "the counter reissued an identity a shape had already given up"
        );
        assert_eq!(
            diagram.get(&the_one_taken_out),
            None,
            "the removed identity is held again"
        );
        assert_eq!(
            diagram.get(&handed_back),
            Some(&the_box()),
            "the identity the caller was handed holds nothing"
        );
        assert_eq!(
            cells(&draw_of(&diagram, origin, size), origin, size),
            cells(
                &drawn(vec![the_box(), the_far_box()], origin, size),
                origin,
                size
            ),
            "the re-added shape hung something back from nothing"
        );
    }

    /// The arrow's own footprint in the arrangement above: **the six cells** the specification's
    /// B3.4 names, as coordinates rather than as two pictures to read side by side.
    fn the_arrows_six_cells() -> [Pos; 6] {
        [
            Pos { x: 3, y: 1 },
            Pos { x: 4, y: 1 },
            Pos { x: 5, y: 1 },
            Pos { x: 6, y: 1 },
            Pos { x: 7, y: 1 },
            Pos { x: 8, y: 1 },
        ]
    }

    /// User Story 2, spec's B3.1 to B3.4, and SC-003: the three routes by which a figure's position
    /// fails to resolve all reach **one picture**, and nothing in the diagram records which one
    /// happened.
    ///
    /// **This is the first test in the repository that can fail on a decision nobody has taken.** An
    /// implementation that answered a removal differently from an identity nothing holds would draw
    /// two of the three routes and fail — which is the whole point of putting all three in one test
    /// rather than one test per route: a test per route asserts nothing the single comparison does
    /// not, and three tests can each be satisfied by three different answers.
    #[test]
    fn the_three_routes_to_one_picture() {
        let (origin, size) = the_arrangement_window();

        // The six is **measured against the no-arrow baseline rather than quoted**: the difference
        // between the arrangement and the same arrangement with no connector in it at all. Three
        // routes compared against each other could not find a count that is wrong in all three of
        // them, and the count is one of the things this slice amends.
        let (with_the_arrow, _, _) = the_arrangement_with_a_survivor();
        let without_the_arrow = the_same_arrangement_without_the_arrow();
        assert_eq!(
            differing(
                &draw_of(&without_the_arrow, origin, size),
                &draw_of(&with_the_arrow, origin, size),
                origin,
                size,
            ),
            the_arrows_six_cells().to_vec(),
            "the arrow's own footprint is not the six cells B3.4 names"
        );

        // Route A — the shape was never there, so the arrow names an identity nothing holds.
        let (a, a_arrow) = the_same_arrangement_with_the_near_box_never_added();

        // Route B — the shape was there and was taken out.
        let (mut b, b_box, _) = the_arrangement_with_a_survivor();
        b.remove(&b_box);
        assert_eq!(
            b.get(&b_box),
            None,
            "route B is reached by a removal and the removal did not happen"
        );

        // Route C — the identity is still held, by a figure that answers no side.
        let (c, c_box, _) = the_same_arrangement_with_a_connector_under_the_box();
        assert_eq!(
            c.get(&c_box),
            Some(&the_connector_under_the_box()),
            "route C holds the identity and the diagram does not find it"
        );

        // **The comparison is the arrow's footprint and not the whole buffer, and the reason is
        // written here at the comparison rather than only in the doc comment above**: route C is
        // *not* the same buffer — it carries the replacement's own two cells at `{0, 0}` and
        // `{1, 0}` — so a whole-buffer comparison would fail it for that difference and not for
        // the one being claimed.
        //
        // **The three are compared against each other rather than against the baseline above**,
        // and that is deliberate: the baseline still holds the near box, so its `{3, 1}` is the
        // border `│` while every route's is blank. A route can only be told from the baseline by
        // a comparison that already knows the box went away, and each route says so in its own
        // arrangement rather than in the picture.
        let a_picture = draw_of(&a, origin, size);
        let a_six: Vec<Option<Cell>> = the_arrows_six_cells()
            .iter()
            .map(|at| a_picture.cell(*at).cloned())
            .collect();
        for (route, diagram) in [("B", &b), ("C", &c)] {
            let by_route = draw_of(diagram, origin, size);
            let route_six: Vec<Option<Cell>> = the_arrows_six_cells()
                .iter()
                .map(|at| by_route.cell(*at).cloned())
                .collect();
            assert_eq!(
                route_six, a_six,
                "route {route} and route A are not the same in the arrow's six cells"
            );
        }

        // **Comparing the three against each other cannot say the arrow drew nothing** — three
        // answers that all drew it would agree with one another — so each route is also compared
        // against **its own figures with no connector among them at all**. That is what rules out
        // a rule that answered all three routes the same wrong way.
        assert_eq!(
            cells(&a_picture, origin, size),
            cells(&drawn(vec![the_far_box()], origin, size), origin, size),
            "route A is not the far box alone, so something else is drawn"
        );
        assert_eq!(
            cells(&draw_of(&b, origin, size), origin, size),
            cells(&a_picture, origin, size),
            "a removal and an identity nothing holds are not the same picture"
        );
        assert_eq!(
            cells(&draw_of(&c, origin, size), origin, size),
            cells(
                &drawn(
                    vec![the_connector_under_the_box(), the_far_box()],
                    origin,
                    size
                ),
                origin,
                size
            ),
            "route C carries more than the far box and the replacement's own two cells"
        );

        // **Nothing records which route happened.** The identity names three different ways and
        // the pictures are two, which is §11's second question in the only form a test can put it.
        assert_eq!(
            a_arrow.to_string(),
            "#2",
            "route A's arrow is not issued where this arrangement issues it"
        );
        assert_eq!(
            differing(
                &draw_of(&b, origin, size),
                &draw_of(&c, origin, size),
                origin,
                size
            ),
            vec![Pos { x: 0, y: 0 }, Pos { x: 1, y: 0 }],
            "the three routes are not the two pictures they are"
        );
    }

    /// User Story 1, spec's B1.1 and B1.2: a connector with one endpoint absolute and one hanging,
    /// displaced, moves **both** of them — and what the hanging one moved is its gap.
    ///
    /// **This test used to say the opposite, and it was not among the four places the
    /// specification named.** It read `…_leaves_its_hanging_one` and asserted that the cell the
    /// hanging end stood on was byte-identical before and after, which is true of the rule this
    /// slice reverses and false of the rule it lands. It escaped the inventory because its own
    /// citation is 082's B4.2 and it never names this issue, so `grep -rn "143"` — the measurement
    /// behind that inventory — did not return it. The count was four and the truth is five.
    ///
    /// **It is rewritten rather than deleted, and the case is kept** because this is the arrangement
    /// its sibling does not reach: the reference sits in the **`to`** slot with an absolute in
    /// `from`, where the test beside it puts it in `from`. A `match` written to suit one slot and
    /// read wrongly in the other passes each of them alone, which is the reason both exist.
    ///
    /// The two halves of the claim are separate assertions rather than one picture. The gap grew by
    /// the delta — the hanging end stands at `{3, 3}` while the side it hangs from is still `{3, 1}`
    /// — and the figure that side belongs to did not move, which is what makes it a gap growing
    /// rather than a route following. The `assert_ne!` is what keeps it honest: a `displaced_by` that
    /// changed nothing at all would satisfy a comparison of before against after by doing exactly
    /// what the old rule did.
    #[test]
    fn a_displaced_connector_moves_its_absolute_end_and_its_hanging_one_too() {
        let (origin, size) = the_window();

        let mut diagram = Diagram::new();
        let box_id = diagram.add(the_box());
        diagram.add(arm_connector(
            Pos { x: 0, y: 1 }.into(),
            Position::Reference(Reference {
                id: box_id.clone(),
                anchor: Anchor::Right,
                offset: Delta { dx: 0, dy: 0 },
            }),
        ));

        // Which endpoint the hanging one resolves to, asked the way the drawing asks it. Before the
        // displacement it stands on the border, and the side it is measured from is there too.
        let the_hanging_end = |d: &Diagram| {
            let Shape::Connector { to, .. } = d
                .get(&crate::ShapeId::new("#2"))
                .expect("the connector is the second figure")
            else {
                unreachable!("the figure under test is a connector")
            };
            to.at.resolve(d)
        };
        assert_eq!(the_hanging_end(&diagram), Some(THE_SIDE_CENTRE));
        let before = draw_of(&diagram, origin, size);

        let id = crate::ShapeId::new("#2");
        let moved = diagram
            .get(&id)
            .expect("the connector is in the diagram")
            .displaced_by(Delta { dx: 0, dy: 2 });
        assert_ne!(
            moved,
            diagram.get(&id).expect("the connector is held").clone()
        );
        diagram.replace(&id, moved);
        let after = draw_of(&diagram, origin, size);

        // The same two figures with the connector standing where the rule put it: the absolute end
        // at `{0, 3}` and the hanging one at `{3, 1}` plus the two cells its gap grew by. Built
        // from the two positions rather than pinned as text, so what is claimed is where a
        // displacement puts a figure and not how a connector draws.
        let mut expected = Diagram::new();
        expected.add(the_box());
        expected.add(arm_connector(
            Pos { x: 0, y: 3 }.into(),
            Pos { x: 3, y: 3 }.into(),
        ));

        assert_eq!(
            cells(&after, origin, size),
            cells(&draw_of(&expected, origin, size), origin, size)
        );

        // The gap grew by the delta, and the figure the gap is measured from is exactly where it
        // was: the two are separate claims, and only the pair says the endpoint slid rather than
        // followed.
        assert_eq!(the_hanging_end(&diagram), Some(Pos { x: 3, y: 3 }));
        assert_ne!(the_hanging_end(&diagram), Some(THE_SIDE_CENTRE));
        assert_eq!(
            the_box().anchor(Anchor::Right),
            Some(THE_SIDE_CENTRE),
            "the box the endpoint hangs from moved, so this would be a following endpoint"
        );

        // The cell the old rule pinned — the one the hanging end stood on before — is no longer
        // written, which is the assertion this test used to make the other way round.
        assert_ne!(
            before.cell(THE_SIDE_CENTRE),
            after.cell(THE_SIDE_CENTRE),
            "the hanging end did not move off the border it hangs from"
        );
    }

    /// User Story 4, spec's B4.3: displacing a box or a line takes all four of its side centers with
    /// it, and a connector hanging from any of them goes with them.
    ///
    /// The other side of B4 from the rule above, and what makes a displacement a property of a
    /// position rather than of a figure. All four anchors rather than one, each with a destination
    /// of its own so that no two arms share a cell and no wrong answer can hide behind a right one:
    /// an implementation that moved three centers and left the fourth would draw three correct
    /// routes and one that goes nowhere.
    #[test]
    fn a_displaced_figure_takes_every_anchor_and_every_hanging_end_with_it() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 14,
            height: 6,
        };
        let by = Delta { dx: 2, dy: 0 };
        let the_anchors = [Anchor::Top, Anchor::Right, Anchor::Bottom, Anchor::Left];
        let the_destinations = [
            Pos { x: 12, y: 0 },
            Pos { x: 13, y: 1 },
            Pos { x: 13, y: 2 },
            Pos { x: 12, y: 5 },
        ];

        let a_box_at = |x: i32| Shape::Box {
            at: Pos { x, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: None,
        };
        let a_line_at = |x: i32| Shape::Line {
            at: Pos { x, y: 0 },
            len: 6,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };

        for (what, figure_at) in [
            ("a box", &a_box_at as &dyn Fn(i32) -> Shape),
            ("a line", &a_line_at as &dyn Fn(i32) -> Shape),
        ] {
            let mut diagram = Diagram::new();
            let identity = diagram.add(figure_at(0));
            for (anchor, to) in the_anchors.into_iter().zip(the_destinations) {
                diagram.add(arm_connector(
                    Position::Reference(Reference {
                        id: identity.clone(),
                        anchor,
                        offset: Delta { dx: 0, dy: 0 },
                    }),
                    to.into(),
                ));
            }
            let before = draw_of(&diagram, origin, size);

            let moved = figure_at(0).displaced_by(by);
            diagram.replace(&identity, moved.clone());
            let after = draw_of(&diagram, origin, size);

            // The same five figures with every anchor named as the point it resolved to before the
            // displacement, and again as the point it resolves to after.
            let mut expected_before = Diagram::new();
            expected_before.add(figure_at(0));
            for (anchor, to) in the_anchors.into_iter().zip(the_destinations) {
                expected_before.add(arm_connector(
                    figure_at(0)
                        .anchor(anchor)
                        .expect("a box and a line answer every anchor")
                        .into(),
                    to.into(),
                ));
            }
            assert_eq!(
                cells(&before, origin, size),
                cells(&draw_of(&expected_before, origin, size), origin, size),
                "{what} before"
            );

            let mut expected_after = Diagram::new();
            expected_after.add(moved.clone());
            for (anchor, to) in the_anchors.into_iter().zip(the_destinations) {
                expected_after.add(arm_connector(
                    moved
                        .anchor(anchor)
                        .expect("a box and a line answer every anchor")
                        .into(),
                    to.into(),
                ));
            }
            assert_eq!(
                cells(&after, origin, size),
                cells(&draw_of(&expected_after, origin, size), origin, size),
                "{what} after"
            );
        }
    }

    // -------------------------- displacing the figure that holds a reference, rather than the one

    /// The arrangement the specification's scenarios are about, with the two identities it holds:
    /// the four-by-three box at the origin, and the connector whose `from` hangs from that box's
    /// right side with a gap of nothing and reaches `{7, 1}`. Both terminals are arms and both
    /// directions are the caller's, so what a picture shows is the route and nothing else.
    ///
    /// A fresh diagram per call rather than one shared, because every case below displaces a figure
    /// **in place** and the two figures are the only things that can be displaced — a case that
    /// started from a diagram the case before had already changed would be asking about both at
    /// once, which is what the last test below is for and what the rest are not.
    fn the_arrangement() -> (Diagram, ShapeId, ShapeId) {
        let mut diagram = Diagram::new();
        let box_id = diagram.add(the_box());
        let connector_id = diagram.add(arm_connector(
            Position::Reference(Reference {
                id: box_id.clone(),
                anchor: Anchor::Right,
                offset: Delta { dx: 0, dy: 0 },
            }),
            Pos { x: 7, y: 1 }.into(),
        ));
        (diagram, box_id, connector_id)
    }

    /// The arrangement the two directions are asked in, carrying **a gap of two cells** and a free
    /// end well clear of the border — which the specification's own arrangement does not, since
    /// there displacing the box four cells right lands the endpoint exactly on the free end and the
    /// connector becomes degenerate, and a degenerate case cannot tell the two rules apart.
    ///
    /// The two directions are the two directions this rule has: displacing the box (which the
    /// reference hangs from) carries the endpoint and leaves the gap, and displacing the connector
    /// (which holds the reference) slides the endpoint and grows the gap. The helper exists so the
    /// two halves of `both_directions_move_the_endpoint_differently` cannot drift apart by
    /// construction — they are the same arrangement by definition, not by agreement.
    fn the_arrangement_with_a_gap(gap: Delta, far_end: Pos) -> (Diagram, ShapeId, ShapeId) {
        let mut diagram = Diagram::new();
        let box_id = diagram.add(the_box());
        let connector_id = diagram.add(arm_connector(
            Position::Reference(Reference {
                id: box_id.clone(),
                anchor: Anchor::Right,
                offset: gap,
            }),
            far_end.into(),
        ));
        (diagram, box_id, connector_id)
    }

    /// The middle of a box's right side, asked through the crate-private query the drawing itself
    /// goes through, so a test cannot disagree with the picture about where the border is.
    fn the_right_side(d: &Diagram, box_id: &ShapeId) -> Option<Pos> {
        d.get(box_id).and_then(|shape| shape.anchor(Anchor::Right))
    }

    /// The gap a connector's `from` holds from the side it hangs from, read off the reference
    /// rather than measured off the drawing — the offset is the gap, and the drawing can only show
    /// where the two ended up rather than what was between them.
    fn the_gap(d: &Diagram, connector: &ShapeId) -> Delta {
        let Shape::Connector { from, .. } = d.get(connector).expect("the connector is held") else {
            unreachable!("the figure named is a connector")
        };
        match &from.at {
            Position::Reference(reference) => reference.offset,
            Position::Absolute(_) => unreachable!("the `from` above is a reference"),
        }
    }

    /// Where a connector's two endpoints stand right now, asked the way the drawing asks it. Both
    /// halves separately rather than as a pair against each other, so a `resolve` that answered
    /// both wrongly cannot pass on the two being wrong together.
    fn the_ends(d: &Diagram, connector: &ShapeId) -> (Option<Pos>, Option<Pos>) {
        let Shape::Connector { from, to, .. } = d.get(connector).expect("the connector is held")
        else {
            unreachable!("the figure named is a connector")
        };
        (from.at.resolve(d), to.at.resolve(d))
    }

    /// User Story 1, spec's B1.1 and B1.2, SC-001: displacing the connector grows the reference's
    /// offset and moves the absolute end, **asked by value and then drawn** — which is the spec's
    /// own order and the order the two halves are only worth anything in.
    ///
    /// The value half catches a rule that grew the wrong field: `id` and `anchor` are held equal to
    /// what went in, and each is asked on its own rather than as a pair. The drawn half catches one
    /// that grew nothing, which the value half would also catch but that a reader cannot see, and it
    /// is the half that says the figure draws as a translation of itself rather than bending its
    /// route to reach a side that stayed put.
    ///
    /// **The expected picture is built from the two positions the rule yields** — a reference to the
    /// same side carrying the grown offset, and a `to` at the moved point — rather than pinned as
    /// text and rather than read back out of the displaced value, so what the two drawings are asked
    /// to agree about is the rule and not itself.
    #[test]
    fn a_displacement_grows_a_references_offsets() {
        let (origin, size) = the_window();
        let (mut diagram, box_id, connector_id) = the_arrangement();
        let as_written = diagram
            .get(&connector_id)
            .expect("the connector is held")
            .clone();

        let by = Delta { dx: 0, dy: 2 };
        let moved = as_written.displaced_by(by);

        let Shape::Connector { from, to, .. } = &moved else {
            unreachable!("the figure above is a connector")
        };
        let Shape::Connector {
            from: was_from,
            to: was_to,
            ..
        } = &as_written
        else {
            unreachable!("the figure above is a connector")
        };
        let Position::Reference(grown) = &from.at else {
            unreachable!("the `from` above is a reference")
        };
        let Position::Reference(was) = &was_from.at else {
            unreachable!("the `from` above is a reference")
        };

        // The one field that grew, on its own, and not equal to what it was.
        assert_eq!(grown.offset, by);
        assert_ne!(grown.offset, was.offset);

        // The two that name, each equal to what went in — and the identity is the one `add` issued,
        // so a rule that reached some other figure's side cannot pass on the anchor being right.
        assert_eq!(grown.id, was.id);
        assert_eq!(grown.id, box_id);
        assert_eq!(grown.anchor, Anchor::Right);
        assert_eq!(grown.anchor, was.anchor);

        // The absolute end moved by the same delta, and the figure as a whole is not what it was.
        assert_eq!(to.at, Pos { x: 7, y: 3 }.into());
        assert_ne!(to.at, was_to.at);
        assert_ne!(moved, as_written);

        // And then the drawing: the same two figures with the connector standing where the rule put
        // it, reached by a second diagram that never displaced anything.
        diagram.replace(&connector_id, moved);
        let after = draw_of(&diagram, origin, size);

        let mut expected = Diagram::new();
        let expected_box = expected.add(the_box());
        expected.add(arm_connector(
            Position::Reference(Reference {
                id: expected_box,
                anchor: Anchor::Right,
                offset: by,
            }),
            Pos { x: 7, y: 3 }.into(),
        ));

        assert_eq!(
            cells(&after, origin, size),
            cells(&draw_of(&expected, origin, size), origin, size),
            "the displaced connector drew something other than a translation of itself"
        );

        // The displacement reached cells, which is the half that says the two pictures above are two
        // pictures rather than one drawn twice.
        let as_written_diagram = the_arrangement().0;
        let reached = differing(
            &draw_of(&as_written_diagram, origin, size),
            &after,
            origin,
            size,
        );
        assert!(
            !reached.is_empty(),
            "the displacement reached nothing at all"
        );
    }

    /// User Story 1, spec's B1.3, SC-001: a figure holding **two** references grows both offsets by
    /// the same amount and is translated rigidly — the rule above, twice, and one test.
    ///
    /// A diagram of **two** boxes rather than one, because a shape holding a single reference cannot
    /// ask this and an implementation that moved only the first endpoint would pass the test above.
    /// The two offsets start out **different** from each other — one cell out and one cell in — so
    /// "grew by the same amount" is a claim about the delta rather than two equal values that happen
    /// to add up, and the two identities are different figures rather than the same one twice.
    ///
    /// The drawn half is what "rigidly" means as cells: the same two boxes and the same connector
    /// with both offsets written at their grown values, built by a diagram that never displaced
    /// anything, so the comparison is against the values rather than against the value under test.
    #[test]
    fn a_displacement_grows_both_offsets_of_one_connector() {
        let (origin, size) = the_window();
        let by = Delta { dx: 3, dy: 2 };

        // Box A's right side center is `{3, 1}` and box B's left side center is `{9, 1}`, so the two
        // hanging ends start on cells six apart and neither answer can be the other.
        let second_box = || Shape::Box {
            at: Pos { x: 9, y: 0 },
            size: Size {
                width: 3,
                height: 3,
            },
            stroke: light(),
            fill: None,
        };
        let hanging_from_both = |from_offset: Delta, to_offset: Delta| {
            let mut diagram = Diagram::new();
            let a = diagram.add(the_box());
            let b = diagram.add(second_box());
            diagram.add(arm_connector(
                Position::Reference(Reference {
                    id: a,
                    anchor: Anchor::Right,
                    offset: from_offset,
                }),
                Position::Reference(Reference {
                    id: b,
                    anchor: Anchor::Left,
                    offset: to_offset,
                }),
            ));
            diagram
        };
        let written = Delta { dx: 1, dy: 0 };
        let inward = Delta { dx: -1, dy: 0 };

        let mut diagram = hanging_from_both(written, inward);
        let connector_id = crate::ShapeId::new("#3");
        let before = draw_of(&diagram, origin, size);
        assert_eq!(the_gap(&diagram, &connector_id), written);

        let moved = diagram
            .get(&connector_id)
            .expect("the connector is held")
            .displaced_by(by);
        diagram.replace(&connector_id, moved);
        let after = draw_of(&diagram, origin, size);

        // Both grew by the delta, on both axes, and each is not what it was. The second offset is
        // asserted as a value of its own rather than read off the drawing below, because "both" is
        // the claim and reading one of them off a picture is how a test stops being about it.
        assert_eq!(the_gap(&diagram, &connector_id), Delta { dx: 4, dy: 2 });
        assert_ne!(the_gap(&diagram, &connector_id), written);

        let Shape::Connector { from, to, .. } =
            diagram.get(&connector_id).expect("the connector is held")
        else {
            unreachable!("the figure above is a connector")
        };
        let (Position::Reference(from), Position::Reference(to)) = (&from.at, &to.at) else {
            unreachable!("both ends of this connector are references")
        };
        assert_eq!(from.offset, Delta { dx: 4, dy: 2 });
        assert_ne!(from.offset, written);
        assert_eq!(to.offset, Delta { dx: 2, dy: 2 });
        assert_ne!(to.offset, inward);
        assert_eq!(from.anchor, Anchor::Right);
        assert_eq!(to.anchor, Anchor::Left);
        assert_ne!(from.id, to.id, "both ends hang from the same figure");
        assert_ne!(
            from.offset, to.offset,
            "the two offsets were the same to begin with"
        );

        // The two resolved ends, each against the point its own reference and anchor name, so a
        // `resolve` that answered both wrongly cannot pass on the two being wrong together.
        assert_eq!(
            the_ends(&diagram, &connector_id),
            (Some(Pos { x: 7, y: 3 }), Some(Pos { x: 11, y: 3 }))
        );

        assert_eq!(
            cells(&after, origin, size),
            cells(
                &draw_of(
                    &hanging_from_both(Delta { dx: 4, dy: 2 }, Delta { dx: 2, dy: 2 }),
                    origin,
                    size
                ),
                origin,
                size
            ),
            "the connector was not translated rigidly: both offsets did not grow together"
        );

        // Neither box moved, which is the other half of "rigidly": every cell either one holds is
        // byte for byte what it was. Their footprints are quoted rather than derived, and the route
        // is excluded from the claim on purpose — it ran along row 1 before and row 3 after, so
        // those cells are the connector's and the claim is about the two boxes alone.
        let a_box_holds = |at: &Pos| {
            let in_the_first = (0..=3).contains(&at.x) && (0..=2).contains(&at.y);
            let in_the_second = (9..=11).contains(&at.x) && (0..=2).contains(&at.y);
            in_the_first || in_the_second
        };
        let reached = differing(&before, &after, origin, size);
        assert!(
            !reached.is_empty(),
            "the displacement reached nothing at all"
        );
        assert!(
            !reached.iter().any(a_box_holds),
            "a box moved when only the connector was displaced: {reached:?}"
        );
    }

    /// The first of the specification's two **derived** arrangements, derived from the rule rather
    /// than decided by it, and pinned so that a later slice which changes it has to say so.
    ///
    /// The arithmetic is three assertions and all three are about the value. The drawing is a fourth
    /// and it is the half that is **measured rather than derived**, because the specification's own
    /// sentence about it turned out to be false on this branch: it claims that "an absolute position
    /// that saturates draws nothing and an offset that saturating draws a very far away endpoint",
    /// and that the two differ. Built and drawn, **the two draw the same thing** — the route is
    /// clipped to whatever of it falls inside the window, whichever of the two positions was
    /// saturated. What does differ from a saturated coordinate is an **unresolved** reference, which
    /// takes the whole figure out of the output, and that is the assertion this test holds.
    #[test]
    fn an_offset_that_saturated_stays_saturated() {
        let (origin, size) = the_window();
        let the_other_end: Position = Pos { x: 8, y: 1 }.into();
        let at_the_end = Delta {
            dx: i32::MAX,
            dy: 0,
        };

        // The figure that has nothing to do with either end is in this diagram from the start, so
        // the three drawings compared below differ only in the connector and not in what is beside
        // it — otherwise "the connector is still there" would be the same claim as "the drawing
        // stopped".
        let hanging_from = |offset: Delta| {
            let mut diagram = Diagram::new();
            let box_id = diagram.add(the_box());
            diagram.add(the_unrelated_box());
            let connector_id = diagram.add(arm_connector(
                Position::Reference(Reference {
                    id: box_id,
                    anchor: Anchor::Right,
                    offset,
                }),
                the_other_end.clone(),
            ));
            (diagram, connector_id)
        };

        let (mut diagram, connector_id) = hanging_from(at_the_end);
        assert_eq!(the_gap(&diagram, &connector_id), at_the_end);

        // One displacement too large for the room left: the offset is already at the end of the
        // coordinates, so it stays there. The wrapped value is named rather than left implicit,
        // because wrapping is exactly what a plain `+` would do here and it would land back inside
        // a window a caller could hold.
        let moved = diagram
            .get(&connector_id)
            .expect("the connector is held")
            .displaced_by(Delta { dx: 5, dy: 0 });
        diagram.replace(&connector_id, moved);
        assert_eq!(the_gap(&diagram, &connector_id), at_the_end);
        assert_ne!(
            the_gap(&diagram, &connector_id),
            Delta {
                dx: i32::MIN + 4,
                dy: 0
            }
        );

        // And a displacement back does **not** undo it, because the addition that saturated is not
        // remembered. Four of the five cells are gone, so one back leaves the offset one cell short
        // of the end rather than at the end: a displacement of nothing, or of a delta the offset
        // had room for, is the only way the value comes back to where it started.
        let back = diagram
            .get(&connector_id)
            .expect("the connector is held")
            .displaced_by(Delta { dx: -1, dy: 0 });
        diagram.replace(&connector_id, back);
        assert_eq!(
            the_gap(&diagram, &connector_id),
            Delta {
                dx: i32::MAX - 1,
                dy: 0
            }
        );
        assert_ne!(
            the_gap(&diagram, &connector_id),
            at_the_end,
            "the displacement back undid a saturating one, so the arithmetic kept a memory"
        );
        // Drawn from a diagram **built** at that offset rather than from the one the three
        // displacements above ran on, and the reason is worth stating: a displacement moves **both**
        // endpoints, so the free end has walked `+5`, `-1`, `+1` and now stands at `{13, 1}` — one
        // column past the right edge of a thirteen-wide window. What the drawing below is about is
        // what an offset *at the end of the coordinates* draws, and the value assertions above
        // already say this arrangement holds one. Drawing the mutated figure would have measured
        // the free end's drift as well, and measured it as though it were about the offset.
        let (built_at_the_end, _) = hanging_from(at_the_end);
        let with_the_offset = draw_of(&built_at_the_end, origin, size);

        // The same two figures and the same connector with the far end named outright, so the only
        // difference between the two drawings is whether the saturated point was reached through an
        // offset or written as a point.
        let mut with_the_point = Diagram::new();
        with_the_point.add(the_box());
        with_the_point.add(the_unrelated_box());
        with_the_point.add(arm_connector(
            Pos { x: i32::MAX, y: 1 }.into(),
            the_other_end.clone(),
        ));

        assert_eq!(
            cells(&with_the_offset, origin, size),
            cells(&draw_of(&with_the_point, origin, size), origin, size),
            "a saturated offset drew something other than the same connector with an absolute \
             endpoint at the saturated point"
        );

        // And the distinction that is real: a saturated offset is not a reference that resolves to
        // nothing. The figure stays in the output, where an unresolved one would be gone from it.
        let mut without_the_connector = Diagram::new();
        without_the_connector.add(the_box());
        without_the_connector.add(the_unrelated_box());
        assert_ne!(
            cells(&with_the_offset, origin, size),
            cells(&draw_of(&without_the_connector, origin, size), origin, size),
            "a saturated offset took the connector out of the output, which is what an unresolved \
             reference does and not what a saturated coordinate does"
        );
    }

    /// User Story 2, spec's B2.1, B2.2 and B2.3, SC-002: the two directions are distinguishable
    /// from outside, and **both of them are asked in one test**.
    ///
    /// One test is the specification's own reason and it is the whole of the reason: an
    /// implementation that reached the same place in both directions — by rewriting the shape a
    /// reference names, say, so that displacing the box moved the endpoint and displacing the
    /// connector moved the box — would satisfy each half on its own and draw neither picture. The
    /// contrast is therefore drawn **across** the two halves rather than inside either.
    ///
    /// **The arrangement carries a gap of two cells and its free end is well clear of the border**,
    /// which the specification's own arrangement does not: there, displacing the box four cells
    /// right lands the endpoint exactly on the free end and the connector becomes degenerate, and a
    /// degenerate case cannot tell the two rules apart. The gap is read off the reference rather
    /// than measured off the drawing, because the drawing can only show where the two ends landed
    /// and not what was between them and the border.
    #[test]
    fn both_directions_move_the_endpoint_differently() {
        let (origin, size) = the_window();
        let gap = Delta { dx: 2, dy: 0 };
        let far_end = Pos { x: 11, y: 1 };
        let the_arrangement = || the_arrangement_with_a_gap(gap, far_end);

        // ---- the figure the reference hangs from: the endpoint follows and the gap does not move
        let (mut box_moved, box_id, connector_id) = the_arrangement();
        let ends_before = the_ends(&box_moved, &connector_id);
        let before_box = draw_of(&box_moved, origin, size);

        let moved_box = box_moved
            .get(&box_id)
            .expect("the box is held")
            .displaced_by(Delta { dx: 4, dy: 0 });
        assert_ne!(
            moved_box,
            box_moved.get(&box_id).expect("the box is held").clone()
        );
        box_moved.replace(&box_id, moved_box);
        let after_box = draw_of(&box_moved, origin, size);

        assert_eq!(
            the_right_side(&box_moved, &box_id),
            Some(Pos { x: 7, y: 1 })
        );
        assert_eq!(
            the_ends(&box_moved, &connector_id),
            (Some(Pos { x: 9, y: 1 }), Some(far_end)),
            "the endpoint did not follow the side it hangs from"
        );
        assert_ne!(the_ends(&box_moved, &connector_id), ends_before);
        assert_eq!(
            the_gap(&box_moved, &connector_id),
            gap,
            "displacing the figure the reference hangs from changed the gap"
        );

        let mut expected_box = Diagram::new();
        expected_box.add(the_box().displaced_by(Delta { dx: 4, dy: 0 }));
        expected_box.add(arm_connector(Pos { x: 9, y: 1 }.into(), far_end.into()));
        assert_eq!(
            cells(&after_box, origin, size),
            cells(&draw_of(&expected_box, origin, size), origin, size)
        );

        // ---- the figure that holds the reference: the endpoint slides and the box stands still
        let (mut connector_moved, box_id, connector_id) = the_arrangement();
        let side_before = the_right_side(&connector_moved, &box_id);
        let ends_before = the_ends(&connector_moved, &connector_id);
        let before_connector = draw_of(&connector_moved, origin, size);

        let moved_connector = connector_moved
            .get(&connector_id)
            .expect("the connector is held")
            .displaced_by(Delta { dx: 0, dy: 2 });
        assert_ne!(
            moved_connector,
            connector_moved
                .get(&connector_id)
                .expect("the connector is held")
                .clone()
        );
        connector_moved.replace(&connector_id, moved_connector);
        let after_connector = draw_of(&connector_moved, origin, size);

        assert_eq!(
            the_right_side(&connector_moved, &box_id),
            side_before,
            "the box moved when only the connector was displaced"
        );
        assert_eq!(
            the_ends(&connector_moved, &connector_id),
            (Some(Pos { x: 5, y: 3 }), Some(Pos { x: 11, y: 3 }))
        );
        assert_ne!(the_ends(&connector_moved, &connector_id), ends_before);
        assert_eq!(
            the_gap(&connector_moved, &connector_id),
            Delta { dx: 2, dy: 2 },
            "the gap did not grow, so the endpoint moved with the figure rather than sliding off it"
        );

        let mut expected_connector = Diagram::new();
        expected_connector.add(the_box());
        expected_connector.add(arm_connector(
            Pos { x: 5, y: 3 }.into(),
            Pos { x: 11, y: 3 }.into(),
        ));
        assert_eq!(
            cells(&after_connector, origin, size),
            cells(&draw_of(&expected_connector, origin, size), origin, size)
        );

        // ---- and the contrast, across the two halves rather than inside either. **The two gaps
        // are not the same number**, and that is the claim: the same arrangement, the same
        // connector, two different figures displaced, and the gap is untouched in one and grown in
        // the other. An implementation that reached one place in both directions would have them
        // equal.
        assert_ne!(
            the_gap(&connector_moved, &connector_id),
            the_gap(&box_moved, &connector_id),
            "the two halves left the same gap, so the two directions are one rule"
        );
        assert_ne!(
            the_ends(&connector_moved, &connector_id).0,
            the_ends(&box_moved, &connector_id).0,
            "the endpoint landed in the same cell whichever figure was displaced"
        );
        assert!(
            !differing(&before_box, &after_box, origin, size).is_empty()
                && !differing(&before_connector, &after_connector, origin, size).is_empty(),
            "one of the two displacements reached nothing at all"
        );
    }

    /// User Story 2, spec's B2.2 and SC-003, and the edge case the specification spells out: a
    /// reference that resolves to nothing still resolves to nothing after a displacement — **and
    /// its offsets grew anyway**.
    ///
    /// That second half is the assertion that makes this test worth writing, and it is why the
    /// test is not simply the one beside it with a displacement added. A `displaced_by` that grew
    /// nothing would satisfy "still resolves to nothing" by doing exactly what the code did before
    /// this rule, and the first half alone cannot tell the two apart.
    ///
    /// **Two cases, and they are two different pieces of code.** An identity this diagram does not
    /// hold, and an anchor whose kind does not answer — a connector, which is the answer that keeps
    /// a chain of references one link long. The identity in the first case is **spelled** rather than
    /// taken from `a_foreign_identity()`, and the reason is worth recording: that helper hands back
    /// `#3`, which a diagram of four figures *does* hold, so naming it would have been the second
    /// case twice and the first would never have run.
    ///
    /// **What this does not check, named rather than described as tested.** It compares the whole
    /// picture against a diagram of the three figures it did not name, so for *those three* it does
    /// check that a displaced figure leaves every other shape byte for byte. What it does **not**
    /// check is the general claim — that a displacement reaches no shape the caller did not name —
    /// which is a claim about the whole diagram rather than about the value that moved, and no
    /// single fixture can establish it for every diagram. The same is true of an endpoint pushed
    /// outside the window by the gap growing: it is clipped without a report, which is the model's
    /// own rule for any figure and nothing this slice adds, and it is left where §4 and §9 state it.
    #[test]
    fn a_reference_that_resolves_to_nothing_still_does() {
        let (origin, size) = the_window();
        let large = Delta { dx: 6, dy: 2 };
        let by = Delta { dx: 1, dy: 1 };
        let the_other_end: Position = Pos { x: 8, y: 1 }.into();
        let the_figure_under_test = crate::ShapeId::new("#4");

        let cases = [
            (
                "an identity nothing holds",
                Position::Reference(Reference {
                    id: crate::ShapeId::new("#9"),
                    anchor: Anchor::Right,
                    offset: large,
                }),
            ),
            (
                "a kind that answers no anchor",
                Position::Reference(Reference {
                    id: crate::ShapeId::new("#3"),
                    anchor: Anchor::Right,
                    offset: large,
                }),
            ),
        ];

        for (what, hanging) in cases {
            let mut diagram = a_diagram_of_three_figures_and_maybe_a_fourth(Some(arm_connector(
                hanging,
                the_other_end.clone(),
            )));
            let without_it = a_diagram_of_three_figures_and_maybe_a_fourth(None);
            // Sanity: the figure under test resolves to nothing before the displacement too, or the
            // case is asking about something other than what it says.
            assert_eq!(
                the_ends(&diagram, &the_figure_under_test).0,
                None,
                "{what} resolved before the displacement, so this is not that case"
            );
            let before = draw_of(&diagram, origin, size);

            let moved = diagram
                .get(&the_figure_under_test)
                .expect("the figure under test is held")
                .displaced_by(by);
            diagram.replace(&the_figure_under_test, moved);
            let after = draw_of(&diagram, origin, size);

            // The offsets grew, on both axes, and this one is asked by value rather than by
            // drawing because a figure that draws nothing cannot show that anything changed.
            assert_eq!(
                the_gap(&diagram, &the_figure_under_test),
                Delta { dx: 7, dy: 3 },
                "{what}: the offsets did not grow"
            );
            assert_ne!(the_gap(&diagram, &the_figure_under_test), large);

            // Still nothing, and every other figure drew exactly what it drew — which is what
            // distinguishes "this figure is not drawn" from "the drawing stopped".
            assert_eq!(
                the_ends(&diagram, &the_figure_under_test).0,
                None,
                "{what} resolved after the displacement"
            );
            assert_eq!(
                cells(&after, origin, size),
                cells(&draw_of(&without_it, origin, size), origin, size),
                "{what} drew something, or drew something else"
            );
            assert_eq!(
                cells(&before, origin, size),
                cells(&draw_of(&without_it, origin, size), origin, size),
                "{what} drew something before the displacement either"
            );
            // The three figures the connector did not name are still in the output, cell by cell —
            // and the displacement reached **nothing at all**, which is what a figure that draws
            // nothing means. Without this the two assertions above would also be satisfied by a
            // drawing that stopped halfway, so it is what makes them mean what they say.
            assert_eq!(
                differing(&before, &after, origin, size),
                Vec::new(),
                "a figure that draws nothing changed the drawing, so something else did"
            );
        }
    }

    /// User Story 2, and the **second** derived arrangement: the box displaced two cells down and
    /// then the connector displaced two cells down.
    ///
    /// **Derived from the rule rather than decided by it**, and stated so a reader is not surprised
    /// by it and so a later slice that changes it has to say so rather than discover it in a
    /// picture: no displacement moves an anchor and its holder together and keeps the gap. Moving
    /// the box carries the endpoint, and moving the endpoint grows the gap, so one caller asking
    /// for two figures gets two displacements and the arrangement that follows from them.
    ///
    /// Asserted by resolved coordinate rather than by picture, and the coordinates are written out
    /// rather than computed, because what is being claimed is arithmetic. Measured on this branch:
    /// the hanging end is at `{3, 5}` — four down from where it started, one move of the side and
    /// two cells of gap — the free end is at `{7, 3}`, two down, because the box was not the figure
    /// that was displaced and a displacement reaches one figure rather than the diagram. The box
    /// itself is at `{0, 2}`, so nothing in the arrangement cascaded.
    #[test]
    fn displacing_the_box_and_then_the_connector() {
        let (mut diagram, box_id, connector_id) = the_arrangement();
        let ends_before = the_ends(&diagram, &connector_id);
        assert_eq!(
            ends_before,
            (Some(Pos { x: 3, y: 1 }), Some(Pos { x: 7, y: 1 }))
        );

        // First the box, which the reference hangs from: the endpoint goes with it and the gap is
        // what it was.
        let moved_box = diagram
            .get(&box_id)
            .expect("the box is held")
            .displaced_by(Delta { dx: 0, dy: 2 });
        diagram.replace(&box_id, moved_box);
        assert_eq!(
            the_ends(&diagram, &connector_id),
            (Some(Pos { x: 3, y: 3 }), Some(Pos { x: 7, y: 1 }))
        );
        assert_eq!(the_gap(&diagram, &connector_id), Delta { dx: 0, dy: 0 });

        // Then the connector itself, which holds the reference: the endpoint slides off the side and
        // the free end moves with it.
        let moved_connector = diagram
            .get(&connector_id)
            .expect("the connector is held")
            .displaced_by(Delta { dx: 0, dy: 2 });
        diagram.replace(&connector_id, moved_connector);

        assert_eq!(
            the_ends(&diagram, &connector_id),
            (Some(Pos { x: 3, y: 5 }), Some(Pos { x: 7, y: 3 }))
        );
        assert_eq!(
            the_gap(&diagram, &connector_id),
            Delta { dx: 0, dy: 2 },
            "the gap did not grow by the second displacement"
        );
        assert_eq!(
            diagram
                .get(&box_id)
                .and_then(|shape| shape.anchor(Anchor::Top)),
            Some(Pos { x: 1, y: 2 }),
            "the box moved a second time, so the two displacements were not one each"
        );

        // The two displacements belonged to two figures, so each was applied once: the gap is the
        // one that grew and not the two that would have come from a cascading displacement.
        assert_ne!(the_ends(&diagram, &connector_id), ends_before);
    }
}
