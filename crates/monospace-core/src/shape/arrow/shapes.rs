//! **A spike, not a second implementation.** The route the previous project drew, ported shape for
//! shape, so that its pictures can be put beside the ones [`derive_path`](super::derive_path)
//! draws and the two read against each other.
//!
//! The previous project is private and unpublished, so nothing here comes from a dependency: this
//! is a transcription of `src/domain/base_shapes/` and `src/built_in/connector_shapes.rs` in
//! that project, read one function at a time. Every function below names the one it came from and
//! keeps its order of writes, its guards and its arithmetic, including the two guards that can
//! never fail and the one intermediate coordinate that turns out not to reach the picture.
//!
//! Two things about the original do not survive the port, and both are recorded where they
//! happen rather than repaired:
//!
//! - **It wrote cells, not a path.** Each of its six route shapes composes a corner and one or
//!   two runs, and the composition is what merges a shared cell's arms. A [`Corner`] and a
//!   [`Segment`] merge the same way here, so the pictures match cell for cell — including where
//!   the original wrote one cell from two fragments, which this does too.
//! - **Its six shapes are not exhaustive, and its own dispatch is what shows it.**
//!   [`draw_route`] keys on the pair of chased directions, and the four pairs of equal directions
//!   match none of the six arms. The corners at the two endpoints are drawn either way, so the
//!   original leaves an arrow as two fragments with a gap between them rather than as a route.
//!   The comparison reports how many arrangements land there, because that number is part of what
//!   is being compared.
//!
//! The terminals are not ported. The comparison draws the same two heads under both routes, so
//! that the route is the only thing that differs.
//!
//! One consequence of not porting them is worth naming, because it looks like a defect of the port
//! and is not: the ported route writes a corner onto an endpoint's own cell in 85 of the grid's
//! 1856 renderings, and the head there is gone. The original lost it in the same arrangements. Its
//! terminal was a `Cell::Char` and its route a `CellObject`, and `Cell::merge` answers
//! `(CellObject, _)` with the front one — a corner beats a character, exactly as
//! `Buffer::stamp` answers a stroke cell over a literal here. The two projects agree on which side
//! wins; the port inherits the consequence rather than inventing it.

use std::cmp::Ordering;

use crate::cell::Side;
use crate::shape::fragment::corner::Corner;
use crate::shape::fragment::segment::Segment;
use crate::{Direction, Orientation, Pos, Shape, Stroke, Surface};

/// A position and the direction a stroke leaves it in: the original's `DirectedPos`, which names
/// the two together everywhere and is never one without the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Step {
    /// Where the step lands.
    pub(super) at: Pos,
    /// Which way it travels.
    pub(super) dir: Direction,
}

/// One step from `at` in `dir`, or `None` where that would leave `i32`.
///
/// The original's `DirectedPos::get_next` used unchecked arithmetic. The sweep never comes near
/// the edge of the plane and neither does the comparison, so the difference between refusing and
/// wrapping is a return type and nothing a picture can show.
pub(super) fn step(at: Pos, dir: Direction) -> Option<Step> {
    let at = match dir {
        Direction::Up => Pos {
            x: at.x,
            y: at.y.checked_sub(1)?,
        },
        Direction::Right => Pos {
            x: at.x.checked_add(1)?,
            y: at.y,
        },
        Direction::Down => Pos {
            x: at.x,
            y: at.y.checked_add(1)?,
        },
        Direction::Left => Pos {
            x: at.x.checked_sub(1)?,
            y: at.y,
        },
    };
    Some(Step { at, dir })
}

fn opposite(dir: Direction) -> Direction {
    match dir {
        Direction::Up => Direction::Down,
        Direction::Right => Direction::Left,
        Direction::Down => Direction::Up,
        Direction::Left => Direction::Right,
    }
}

/// The side of a cell that a stroke leaving it in `dir` opens toward.
fn side_ahead(dir: Direction) -> Side {
    match dir {
        Direction::Up => Side::Top,
        Direction::Right => Side::Right,
        Direction::Down => Side::Bottom,
        Direction::Left => Side::Left,
    }
}

/// The side of a cell that a stroke arriving along `dir` opens toward: the other one, because a
/// direction names where a stroke is going rather than where it came from.
fn side_behind(dir: Direction) -> Side {
    side_ahead(opposite(dir))
}

/// The two sides a corner writes where a stroke arrives along `into` and leaves along `out`.
///
/// The original asked this as four independent questions, one per side, each of them whether
/// either of its two directions pointed at that side. Asking it once per side is the same
/// question, and the port says so here rather than repeating all four.
fn opens(into: Direction, out: Direction) -> (Side, Side) {
    (side_behind(into), side_ahead(out))
}

/// One cell, opening toward two sides.
fn corner(at: Pos, opens: (Side, Side), stroke: &Stroke, surface: &mut dyn Surface) {
    Corner {
        at,
        opens,
        stroke: stroke.clone(),
    }
    .draw(surface);
}

/// A run of `len` cells from `from`, the far end left out.
///
/// The length convention is the original's `simple_lines`, whose `l` counts from `pos` up to but
/// excluding `pos + l`, because the corner at the far end is a separate write. Every `l` in the
/// original is a difference of two positions this file has just ordered, so a negative one cannot
/// arrive; a length of zero writes nothing.
fn segment(
    from: Pos,
    orientation: Orientation,
    len: i32,
    stroke: &Stroke,
    surface: &mut dyn Surface,
) {
    let Ok(len) = u32::try_from(len) else {
        return;
    };
    if len == 0 {
        return;
    }
    Segment {
        from,
        len,
        orientation,
        stroke: stroke.clone(),
    }
    .draw(surface);
}

/// The original wrote every run length as an `abs_diff`, which is a `u32`; this file's [`segment`]
/// takes the `i32` the original's own `l: usize` was standing in for, so that the two bridges can
/// hand it a plain difference. Converted in one place rather than at eight call sites.
fn count(difference: u32) -> i32 {
    i32::try_from(difference).unwrap_or(i32::MAX)
}

/// The two points in the order the original sorted them, left to right.
fn by_x(a: Pos, b: Pos) -> (Pos, Pos) {
    if a.x <= b.x { (a, b) } else { (b, a) }
}

/// The two points in the order the original sorted them, top to bottom.
fn by_y(a: Pos, b: Pos) -> (Pos, Pos) {
    if a.y <= b.y { (a, b) } else { (b, a) }
}

/// `bent_lines::RightDown`: the run along the upper row, the run down the right-hand column, and
/// the corner `┐` where the two meet.
fn bent_right_down(a: Pos, b: Pos, stroke: &Stroke, surface: &mut dyn Surface) {
    let (left, right) = by_x(a, b);

    if left.y > right.y {
        // The original's "Invalid scenario": the two points do not sit where this shape reaches,
        // so nothing is written.
        return;
    }

    corner(
        Pos {
            x: right.x,
            y: left.y,
        },
        (Side::Left, Side::Bottom),
        stroke,
        surface,
    );
    segment(
        Pos {
            x: left.x,
            y: left.y,
        },
        Orientation::Horizontal,
        count(right.x.abs_diff(left.x)),
        stroke,
        surface,
    );
    segment(
        Pos {
            x: right.x,
            y: left.y + 1,
        },
        Orientation::Vertical,
        count(right.y.abs_diff(left.y)),
        stroke,
        surface,
    );
}

/// `bent_lines::RightUp`: the run along the lower row, the run up the right-hand column, and the
/// corner `┘`.
fn bent_right_up(a: Pos, b: Pos, stroke: &Stroke, surface: &mut dyn Surface) {
    let (left, right) = by_x(a, b);

    if right.y > left.y {
        return;
    }

    corner(
        Pos {
            x: right.x,
            y: left.y,
        },
        (Side::Left, Side::Top),
        stroke,
        surface,
    );
    segment(
        Pos {
            x: left.x,
            y: left.y,
        },
        Orientation::Horizontal,
        count(right.x.abs_diff(left.x)),
        stroke,
        surface,
    );
    segment(
        Pos {
            x: right.x,
            y: right.y,
        },
        Orientation::Vertical,
        count(right.y.abs_diff(left.y)),
        stroke,
        surface,
    );
}

/// `bent_lines::LeftUp`: the run down the left-hand column, the run along the lower row, and the
/// corner `└`.
fn bent_left_up(a: Pos, b: Pos, stroke: &Stroke, surface: &mut dyn Surface) {
    let (left, right) = by_x(a, b);

    if right.x < left.x {
        // Unreachable: `by_x` has just ordered them. Kept because the original checks it, so that
        // this file can be read against the original line by line.
        return;
    }

    corner(
        Pos {
            x: left.x,
            y: right.y,
        },
        (Side::Top, Side::Right),
        stroke,
        surface,
    );
    segment(
        Pos {
            x: left.x + 1,
            y: right.y,
        },
        Orientation::Horizontal,
        count(right.x.abs_diff(left.x)),
        stroke,
        surface,
    );
    segment(
        Pos {
            x: left.x,
            y: left.y,
        },
        Orientation::Vertical,
        count(right.y.abs_diff(left.y)),
        stroke,
        surface,
    );
}

/// `bent_lines::LeftDown`: the run along the upper row, the run down the left-hand column, and
/// the corner `┌`.
fn bent_left_down(a: Pos, b: Pos, stroke: &Stroke, surface: &mut dyn Surface) {
    let (left, right) = by_x(a, b);

    if left.x > right.x {
        // Unreachable, as above.
        return;
    }

    corner(
        Pos {
            x: left.x,
            y: right.y,
        },
        (Side::Bottom, Side::Right),
        stroke,
        surface,
    );
    segment(
        Pos {
            x: left.x + 1,
            y: right.y,
        },
        Orientation::Horizontal,
        count(right.x.abs_diff(left.x)),
        stroke,
        surface,
    );
    segment(
        Pos {
            x: left.x,
            y: right.y + 1,
        },
        Orientation::Vertical,
        count(left.y.abs_diff(right.y)),
        stroke,
        surface,
    );
}

/// `bridge_lines::Horizontal`: the two points run at each other along a row, and where they are
/// not on the same row a staircase at the middle of the span stands in for the row they are not
/// on.
///
/// `h1` is the original's half of the row difference, and it does not reach the picture: both of
/// the two shapes it splits the staircase into run their vertical leg on `b1.x`, so the staircase
/// turns on `b1.x` whatever `h1` says. It is transcribed rather than dropped because dropping it
/// would make this file disagree with the original about what the code computes.
fn bridge_horizontal(a: Pos, b: Pos, stroke: &Stroke, surface: &mut dyn Surface) {
    let (left, right) = by_x(a, b);
    let l = right.x - left.x + 1;

    match right.y.cmp(&left.y) {
        Ordering::Greater => {
            let w1 = l / 2;
            let h1 = (right.y - left.y) / 2;
            let b1 = Pos {
                x: left.x + w1,
                y: left.y + h1,
            };

            bent_right_down(left, b1, stroke, surface);
            bent_left_up(
                Pos {
                    x: b1.x,
                    y: b1.y + 1,
                },
                right,
                stroke,
                surface,
            );
        }
        Ordering::Less => {
            let w1 = l / 2;
            let h1 = (left.y - right.y) / 2;
            let b1 = Pos {
                x: left.x + w1,
                y: left.y - h1,
            };

            bent_right_up(left, b1, stroke, surface);
            bent_left_down(
                Pos {
                    x: b1.x,
                    y: b1.y - 1,
                },
                right,
                stroke,
                surface,
            );
        }
        Ordering::Equal => segment(
            Pos {
                x: left.x,
                y: left.y,
            },
            Orientation::Horizontal,
            l,
            stroke,
            surface,
        ),
    }
}

/// `bridge_lines::Vertical`: the same the other way round.
///
/// Its `w1` divides the plain difference where `bridge_horizontal`'s divides the difference plus
/// one, so the two staircases turn at different points across the same span. That asymmetry is the
/// original's arithmetic rather than this port's reading of it, and the comparison is what says
/// how often a picture can tell.
fn bridge_vertical(a: Pos, b: Pos, stroke: &Stroke, surface: &mut dyn Surface) {
    let (up, down) = by_y(a, b);
    let l = down.y - up.y + 1;

    match up.x.cmp(&down.x) {
        Ordering::Greater => {
            let h1 = l / 2;
            let w1 = (up.x - down.x) / 2;
            let b1 = Pos {
                x: down.x + w1,
                y: up.y + h1,
            };

            bent_right_up(b1, up, stroke, surface);
            bent_left_down(
                Pos {
                    x: b1.x - 1,
                    y: b1.y,
                },
                down,
                stroke,
                surface,
            );
        }
        Ordering::Less => {
            let h1 = l / 2;
            let w1 = (down.x - up.x) / 2;
            let b1 = Pos {
                x: up.x + w1,
                y: up.y + h1,
            };

            bent_left_up(b1, up, stroke, surface);
            bent_right_down(
                Pos {
                    x: b1.x + 1,
                    y: b1.y,
                },
                down,
                stroke,
                surface,
            );
        }
        Ordering::Equal => segment(
            Pos { x: up.x, y: up.y },
            Orientation::Vertical,
            l,
            stroke,
            surface,
        ),
    }
}

/// `connectors::Corner::new`: the step the route takes one cell after `at`, chasing `other`.
///
/// Keep going while this axis is already lined up with the other end, or while it still has
/// ground to cover on its own side of the other end; otherwise turn onto the other axis, towards
/// the other end. The preference between the two turns is `x < other.x` for a vertical run and
/// `y < other.y` for a horizontal one — always the turn that closes the gap, never a detour away
/// from it.
pub(super) fn chase(at: Pos, dir: Direction, other: Pos) -> Step {
    let Pos { x, y } = at;
    let (other_x, other_y) = (other.x, other.y);

    let chased = match dir {
        Direction::Up => {
            if x == other_x || y > other_y {
                step(at, Direction::Up)
            } else if x < other_x {
                step(at, Direction::Right)
            } else {
                step(at, Direction::Left)
            }
        }
        Direction::Down => {
            if x == other_x || y < other_y {
                step(at, Direction::Down)
            } else if x < other_x {
                step(at, Direction::Right)
            } else {
                step(at, Direction::Left)
            }
        }
        Direction::Left => {
            if y == other_y || x > other_x {
                step(at, Direction::Left)
            } else if y < other_y {
                step(at, Direction::Down)
            } else {
                step(at, Direction::Up)
            }
        }
        Direction::Right => {
            if y == other_y || x < other_x {
                step(at, Direction::Right)
            } else if y < other_y {
                step(at, Direction::Down)
            } else {
                step(at, Direction::Up)
            }
        }
    };

    chased.expect("the sweep and the comparison both stay far inside i32")
}

/// `bridge_lines::Free`: the whole route, in the order the original wrote it — a corner where
/// each endpoint's stroke leaves it, then one of six shapes between the two cells past those
/// corners.
///
/// The six arms are six `if`s rather than a `match` because the original wrote them that way. It
/// makes no difference to the pictures: the arms are keyed on the unordered pair of the two
/// chased directions, so exactly one of them fires whenever the two differ, and none of them
/// fires when they do not.
#[allow(clippy::too_many_lines)]
pub(super) fn draw_route(
    from: Pos,
    from_dir: Direction,
    to: Pos,
    to_dir: Direction,
    stroke: &Stroke,
    surface: &mut dyn Surface,
) {
    let Some(a) = step(from, from_dir) else {
        return;
    };
    let Some(b) = step(to, to_dir) else {
        return;
    };

    if a.at == b.at {
        // `connectors::Corner::single`: both endpoints' strokes meet on one cell, which opens
        // toward the side each of them came from. The `to` end contributes `to_dir` reversed,
        // because `Corner::new` names the direction a stroke *leaves* a cell in and this stroke
        // arrives.
        corner(a.at, opens(a.dir, opposite(b.dir)), stroke, surface);
        return;
    }

    let chased_a = chase(a.at, a.dir, b.at);
    let chased_b = chase(b.at, b.dir, a.at);

    corner(a.at, opens(a.dir, chased_a.dir), stroke, surface);
    corner(b.at, opens(b.dir, chased_b.dir), stroke, surface);

    let (na, dir_a) = (chased_a.at, chased_a.dir);
    let (nb, dir_b) = (chased_b.at, chased_b.dir);

    if (dir_a == Direction::Right && dir_b == Direction::Left && na.x <= nb.x)
        || (dir_a == Direction::Left && dir_b == Direction::Right && nb.x <= na.x)
    {
        bridge_horizontal(na, nb, stroke, surface);
    }

    if (dir_a == Direction::Down && dir_b == Direction::Up && na.y <= nb.y)
        || (dir_a == Direction::Up && dir_b == Direction::Down && nb.y <= na.y)
    {
        bridge_vertical(na, nb, stroke, surface);
    }

    if (dir_a == Direction::Down && dir_b == Direction::Left)
        || (dir_a == Direction::Left && dir_b == Direction::Down)
    {
        bent_left_up(na, nb, stroke, surface);
    }

    if (dir_a == Direction::Down && dir_b == Direction::Right)
        || (dir_a == Direction::Right && dir_b == Direction::Down)
    {
        bent_right_up(na, nb, stroke, surface);
    }

    if (dir_a == Direction::Up && dir_b == Direction::Left)
        || (dir_a == Direction::Left && dir_b == Direction::Up)
    {
        bent_left_down(na, nb, stroke, surface);
    }

    if (dir_a == Direction::Up && dir_b == Direction::Right)
        || (dir_a == Direction::Right && dir_b == Direction::Up)
    {
        bent_right_down(na, nb, stroke, surface);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Step, bent_left_down, bent_left_up, bent_right_down, bent_right_up, chase, draw_route,
        opens, step,
    };
    use crate::{Buffer, Direction, Layer, Pos, Size, StampMode, Stroke, render};

    fn light() -> Stroke {
        Stroke::from("light")
    }

    fn p(x: i32, y: i32) -> Pos {
        Pos { x, y }
    }

    fn u(width: u32, height: u32) -> Size {
        Size { width, height }
    }

    /// One shape, drawn into a window exactly as large as the original's twenty-three tests
    /// declared theirs, with every line's trailing blanks trimmed the way the original's
    /// `WorldWindow::to_string` trimmed them. The expectations below are therefore the original's
    /// own strings rather than this port's.
    fn picture(size: Size, draw_one: impl FnOnce(&mut dyn crate::Surface)) -> String {
        let at = p(0, 0);
        let mut buffer = Buffer::new(at, size);
        draw_one(&mut Layer::new(&mut buffer, StampMode::Above));
        let drawn = render(&buffer, &crate::GlyphCatalog::light(), at, size);
        let trimmed: Vec<&str> = drawn.lines().map(str::trim_end).collect();
        let mut out = trimmed.join("\n");
        out.push('\n');
        out
    }

    /// The twenty-three pictures the original's `bent_lines.tests.rs` pins, transcribed with the
    /// shape each came from.
    ///
    /// These are the only ground truth the original has for the four shapes every one of its
    /// routes is built from, so they are checked before the port is compared against anything. A
    /// disagreement here is a disagreement about the port, not about the two route models.
    ///
    /// One window is transcribed as the original wrote it and not as the original drew it: case 17
    /// declares a window two rows tall and pins a three-row picture, because the original's
    /// `WorldWindow::to_string` renders by the extent of what was written rather than by the
    /// window it was given. This port's `Buffer` clips at the window, so the window here is the
    /// three rows the picture needs — the shape, not the window, is what is under test.
    #[test]
    fn the_four_bent_lines_draw_the_twenty_three_pictures_the_original_pins() {
        type Bent = fn(Pos, Pos, &Stroke, &mut dyn crate::Surface);

        let cases: [(Bent, Size, Pos, Pos, &str); 23] = [
            (bent_right_down, u(1, 1), p(0, 0), p(0, 0), "┐\n"),
            (bent_right_down, u(1, 2), p(0, 0), p(0, 1), "┐\n│\n"),
            (bent_right_down, u(1, 3), p(0, 0), p(0, 2), "┐\n│\n│\n"),
            (bent_right_down, u(4, 1), p(0, 0), p(3, 0), "───┐\n"),
            (
                bent_right_down,
                u(4, 3),
                p(0, 0),
                p(3, 2),
                "───┐\n   │\n   │\n",
            ),
            (
                bent_right_down,
                u(4, 3),
                p(3, 2),
                p(0, 0),
                "───┐\n   │\n   │\n",
            ),
            (bent_left_up, u(1, 1), p(0, 0), p(0, 0), "└\n"),
            (bent_left_up, u(1, 2), p(0, 0), p(0, 1), "│\n└\n"),
            (bent_left_up, u(1, 3), p(0, 0), p(0, 2), "│\n│\n└\n"),
            (bent_left_up, u(4, 1), p(0, 0), p(3, 0), "└───\n"),
            (bent_left_up, u(4, 3), p(0, 0), p(3, 2), "│\n│\n└───\n"),
            (bent_left_up, u(4, 3), p(3, 2), p(0, 0), "│\n│\n└───\n"),
            (bent_left_down, u(1, 1), p(0, 0), p(0, 0), "┌\n"),
            (bent_left_down, u(1, 2), p(0, 1), p(0, 0), "┌\n│\n"),
            (bent_left_down, u(1, 3), p(0, 2), p(0, 0), "┌\n│\n│\n"),
            (bent_left_down, u(4, 1), p(0, 0), p(3, 0), "┌───\n"),
            (bent_left_down, u(4, 3), p(0, 2), p(3, 0), "┌───\n│\n│\n"),
            (bent_right_up, u(1, 1), p(0, 0), p(0, 0), "┘\n"),
            (bent_right_up, u(1, 2), p(0, 1), p(0, 0), "│\n┘\n"),
            (bent_right_up, u(1, 3), p(0, 2), p(0, 0), "│\n│\n┘\n"),
            (bent_right_up, u(4, 1), p(0, 0), p(3, 0), "───┘\n"),
            (
                bent_right_up,
                u(4, 3),
                p(0, 2),
                p(3, 0),
                "   │\n   │\n───┘\n",
            ),
            (
                bent_right_up,
                u(4, 3),
                p(3, 0),
                p(0, 2),
                "   │\n   │\n───┘\n",
            ),
        ];

        for (index, (shape, size, a, b, expected)) in cases.into_iter().enumerate() {
            let drawn = picture(size, |surface| shape(a, b, &light(), surface));
            assert_eq!(drawn, expected, "bent line case {}", index + 1);
        }
    }

    /// `chase`, spelled out: the two things that keep it going, the two turns it can make, and the
    /// rule that picks between them. Asserted because the whole comparison turns on this one
    /// function, and because a mapping that happens to be right for three of the four directions
    /// is wrong.
    #[test]
    fn chase_keeps_going_on_its_own_axis_and_turns_towards_the_other_end_off_it() {
        use Direction::{Down, Left, Right, Up};
        let chased = |at: Pos, dir: Direction, other: Pos| chase(at, dir, other);

        // Lined up on the axis it is traveling along: it keeps going.
        assert_eq!(chased(p(2, 4), Down, p(2, 0)), step(p(2, 4), Down).unwrap());
        assert_eq!(chased(p(2, 4), Up, p(2, 9)), step(p(2, 4), Up).unwrap());
        assert_eq!(
            chased(p(4, 2), Right, p(0, 2)),
            step(p(4, 2), Right).unwrap()
        );
        assert_eq!(chased(p(4, 2), Left, p(9, 2)), step(p(4, 2), Left).unwrap());

        // Off its axis, and the other end is on the near side of it: it turns towards that side.
        assert_eq!(
            chased(p(1, 1), Down, p(3, 0)),
            step(p(1, 1), Right).unwrap()
        );
        assert_eq!(chased(p(3, 1), Down, p(1, 0)), step(p(3, 1), Left).unwrap());
        assert_eq!(
            chased(p(1, 1), Right, p(0, 3)),
            step(p(1, 1), Down).unwrap()
        );
        assert_eq!(chased(p(1, 3), Right, p(0, 1)), step(p(1, 3), Up).unwrap());

        // Past the other end on its own axis and still off it: it turns rather than reversing, so
        // a route that has gone past comes back alongside rather than doubling over itself.
        assert_eq!(
            chased(p(1, 4), Down, p(3, 0)),
            step(p(1, 4), Right).unwrap()
        );
        assert_eq!(
            chased(p(4, 2), Right, p(0, 5)),
            step(p(4, 2), Down).unwrap()
        );
    }

    /// `chase` never turns onto the axis it is leaving, which is what keeps the ported route from
    /// reversing on the spot.
    #[test]
    fn chase_never_turns_back_on_itself() {
        use Direction::{Down, Left, Right, Up};
        let fields = [(0, 0, Right), (0, 0, Left), (0, 0, Up), (0, 0, Down)];

        for x in -1..3 {
            for y in -1..3 {
                for (ox, oy, _) in fields {
                    let other = Pos { x: ox, y: oy };
                    for dir in [Up, Right, Down, Left] {
                        let chased: Step = chase(Pos { x, y }, dir, other);
                        assert!(
                            chased.dir != opposite_of(dir),
                            "({x}, {y}) leaving {dir:?} towards {other:?} turned into itself"
                        );
                    }
                }
            }
        }
    }

    fn opposite_of(dir: Direction) -> Direction {
        match dir {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }

    /// `opens`, the two sides a corner writes: the side the stroke arrived from and the side it
    /// left by, which for one direction are opposites and make a straight run.
    #[test]
    fn a_corner_opens_towards_the_side_the_stroke_arrived_from_and_the_one_it_left_by() {
        use crate::cell::Side;
        use Direction::{Down, Left, Right, Up};

        assert_eq!(opens(Down, Right), (Side::Top, Side::Right));
        assert_eq!(opens(Right, Down), (Side::Left, Side::Bottom));
        assert_eq!(opens(Up, Left), (Side::Bottom, Side::Left));
        assert_eq!(opens(Left, Up), (Side::Right, Side::Top));
        assert_eq!(opens(Right, Right), (Side::Left, Side::Right));
    }

    /// The arrangement the original's own dispatch has a dedicated arm for: two endpoints one
    /// step apart along a row, leaving towards each other, so the two first cells past them are
    /// the same one. The whole of its route is that one cell, which carries two opposite arms.
    #[test]
    fn two_endpoints_one_step_apart_along_a_row_draw_one_cell_and_nothing_else() {
        let text = picture(u(4, 1), |surface| {
            draw_route(
                p(0, 0),
                Direction::Right,
                p(2, 0),
                Direction::Left,
                &light(),
                surface,
            );
        });

        assert_eq!(text, " ─\n");
    }

    /// The arrangement the comparison exists to count. Two endpoints leaving the same way chase
    /// into a pair of directions no shape of the six can take — here `Down` and `Right`, which is
    /// one of the six, but the two cells it would join are stacked rather than side by side, so
    /// the shape declines them — and what the original draws is the corner at each end and nothing
    /// between: two one-cell stubs with a gap.
    #[test]
    fn two_endpoints_leaving_the_same_way_draw_two_stubs_and_a_gap() {
        let text = picture(u(4, 4), |surface| {
            draw_route(
                p(0, 0),
                Direction::Right,
                p(0, 2),
                Direction::Down,
                &light(),
                surface,
            );
        });

        assert_eq!(text, " ┐\n\n\n└\n");
    }
}
