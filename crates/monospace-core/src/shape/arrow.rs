//! The arrow: two endpoints, two heads and a derived route between them. See _The initial set_
//! and _The route of an arrow_ in [`docs/model.md`](../../../docs/model.md), and research.md Q5
//! in `specs/039-draw-shapes-instead-of-individual-cells/`.

use crate::shape::fragment::head::Head;
use crate::shape::route::Route;
use crate::{Direction, Glyph, Orientation, Pos, Shape, Stroke, Surface};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Where an arrow ends: a position, the direction it leaves in, and the glyph of the head that
/// sits there. A head points opposite to `leaving`.
pub struct Endpoint {
    /// The endpoint's position. The head occupies this position itself.
    pub at: Pos,
    /// The direction the arrow leaves this endpoint in. The route's starting position is one
    /// step from `at` in this direction.
    pub leaving: Direction,
    /// The glyph the head at this endpoint is drawn as — the caller's choice, per ADR-0029 and
    /// FR-027, since no glyph set holds a rule that points.
    pub head: Glyph,
}

/// An arrow: two endpoints and a stroke.
///
/// A **complete** shape in the sense of _Complete and fragment_ in
/// [`docs/model.md`](../../../docs/model.md). The route between the two endpoints is derived from
/// their positions and leaving directions alone — FR-016 — and is never described by the caller.
pub struct Arrow {
    /// One endpoint of the arrow.
    pub from: Endpoint,
    /// The other endpoint of the arrow.
    pub to: Endpoint,
    /// The stroke the route between the two endpoints is drawn in. The two heads are drawn in
    /// their own glyphs, not in this stroke.
    pub stroke: Stroke,
}

impl Shape for Arrow {
    fn draw(&self, surface: &mut dyn Surface) {
        Head {
            at: self.from.at,
            glyph: self.from.head.clone(),
        }
        .draw(surface);
        Head {
            at: self.to.at,
            glyph: self.to.head.clone(),
        }
        .draw(surface);

        if let Some(positions) =
            derive_path(self.from.at, self.from.leaving, self.to.at, self.to.leaving)
        {
            Route {
                from: self.from.at,
                to: self.to.at,
                positions,
                stroke: self.stroke.clone(),
            }
            .draw(surface);
        }
    }
}

/// One step from `at` in `dir`, or `None` if that would overflow `i32`.
fn offset(at: Pos, dir: Direction) -> Option<Pos> {
    match dir {
        Direction::Up => at.y.checked_sub(1).map(|y| Pos { x: at.x, y }),
        Direction::Down => at.y.checked_add(1).map(|y| Pos { x: at.x, y }),
        Direction::Left => at.x.checked_sub(1).map(|x| Pos { x, y: at.y }),
        Direction::Right => at.x.checked_add(1).map(|x| Pos { x, y: at.y }),
    }
}

fn opposite(dir: Direction) -> Direction {
    match dir {
        Direction::Up => Direction::Down,
        Direction::Down => Direction::Up,
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
    }
}

/// Every position from `from` to `to` inclusive, in that order. The two must share exactly one
/// coordinate.
fn positions_between(from: Pos, to: Pos) -> Vec<Pos> {
    if from.x == to.x {
        let (lo, hi) = (from.y.min(to.y), from.y.max(to.y));
        let mut positions: Vec<Pos> = (lo..=hi).map(|y| Pos { x: from.x, y }).collect();
        if from.y > to.y {
            positions.reverse();
        }
        positions
    } else {
        let (lo, hi) = (from.x.min(to.x), from.x.max(to.x));
        let mut positions: Vec<Pos> = (lo..=hi).map(|x| Pos { x, y: from.y }).collect();
        if from.x > to.x {
            positions.reverse();
        }
        positions
    }
}

/// Every position between the waypoints of a chosen candidate, each pair's shared boundary point
/// written once.
fn expand_waypoints(waypoints: &[Pos]) -> Vec<Pos> {
    if waypoints.len() == 1 {
        return vec![waypoints[0]];
    }

    let mut positions = Vec::new();
    for pair in waypoints.windows(2) {
        let segment = positions_between(pair[0], pair[1]);
        if positions.is_empty() {
            positions.extend(segment);
        } else {
            positions.extend(segment.into_iter().skip(1));
        }
    }
    positions
}

/// The smallest rectangle containing both starting positions — _The route of an arrow_ in
/// [`docs/model.md`](../../../docs/model.md).
struct RouteRectangle {
    x_min: i32,
    x_max: i32,
    y_min: i32,
    y_max: i32,
}

impl RouteRectangle {
    fn spanning(s: Pos, t: Pos) -> Self {
        Self {
            x_min: s.x.min(t.x),
            x_max: s.x.max(t.x),
            y_min: s.y.min(t.y),
            y_max: s.y.max(t.y),
        }
    }

    /// One value per axis: the cell halfway along that axis's span, taken as the one nearer
    /// `anchor` — the `from` endpoint's starting position — where the span holds an even number
    /// of cells and the halfway point falls between two. [ADR-0044](
    /// ../../../../docs/decisions/0044-let-the-endpoint-order-break-a-tied-route.md).
    fn middle(&self, anchor: Pos) -> Pos {
        Pos {
            x: Self::midpoint(self.x_min, self.x_max, anchor.x),
            y: Self::midpoint(self.y_min, self.y_max, anchor.y),
        }
    }

    /// The cell halfway between `min` and `max` (inclusive), nearer `anchor` — which is always
    /// one of the two, since it is one of the two positions the rectangle spans.
    fn midpoint(min: i32, max: i32, anchor: i32) -> i32 {
        let half = (max - min) / 2;
        if anchor == min {
            min + half
        } else {
            max - half
        }
    }
}

/// The orientation a direction moves along: `Left`/`Right` are horizontal, `Up`/`Down` vertical.
fn direction_orientation(dir: Direction) -> Orientation {
    match dir {
        Direction::Left | Direction::Right => Orientation::Horizontal,
        Direction::Up | Direction::Down => Orientation::Vertical,
    }
}

/// The side the arrow's own travel puts to its right. Coordinates grow rightward and downward,
/// so leaving `Up` puts the larger `x` to the right and leaving `Right` the larger `y`. Extends
/// [ADR-0044](../../../../docs/decisions/0044-let-the-endpoint-order-break-a-tied-route.md) to
/// the routes it leaves level: where two mirror each other about the line the two starting
/// positions share, neither is nearer the endpoint the arrow leaves from, and this is what
/// decides between them.
fn right_hand_takes_the_larger(leaving: Direction) -> bool {
    matches!(leaving, Direction::Up | Direction::Right)
}

/// The two directions a run may turn into: the perpendicular ones, since a reversal is not a
/// turn and a run may not double back on itself.
fn turns_from(dir: Direction) -> [Direction; 2] {
    match direction_orientation(dir) {
        Orientation::Horizontal => [Direction::Up, Direction::Down],
        Orientation::Vertical => [Direction::Left, Direction::Right],
    }
}

/// What the model ranks candidate routes by, in order — _The route of an arrow_ in
/// [`docs/model.md`](../../../docs/model.md). Every term is charged as a route is walked, so a
/// search that minimizes this yields the route the model names without a second pass over
/// candidates.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Cost {
    /// How many times the route changes direction.
    bends: u32,
    /// How many cells it travels.
    length: u32,
    /// How far, summed, the runs the bends leave free sit from the middle.
    from_middle: u32,
    /// How far those runs sit from the side the arrow's own travel puts to its right.
    hand: u32,
}

impl Cost {
    fn plus(self, other: Self) -> Self {
        Self {
            bends: self.bends + other.bends,
            length: self.length + other.length,
            from_middle: self.from_middle + other.from_middle,
            hand: self.hand + other.hand,
        }
    }
}

/// The lines a route can turn on.
///
/// A route turns at a coordinate one of the rule's terms speaks of and nowhere else, so the
/// search runs over those coordinates rather than over every cell — which is what keeps its cost
/// independent of how far apart the two endpoints are. Per axis: the line each starting position
/// pins, the line beside each of them — where an endpoint's own cell blocks the way, the route
/// passes next to it — the middle, and one line outside the rectangle the two starting positions
/// span, which is as far out as a route ever reaches.
struct Lattice {
    xs: Vec<i32>,
    ys: Vec<i32>,
    mid: Pos,
    /// Whether the `from` endpoint leaves along a column, which is the axis the hand speaks of.
    leaves_vertically: bool,
    /// Whether the hand prefers the larger coordinate on that axis.
    hand_takes_the_larger: bool,
}

impl Lattice {
    fn spanning(s: Pos, t: Pos, da: Direction) -> Self {
        let rectangle = RouteRectangle::spanning(s, t);
        let mid = rectangle.middle(s);
        Self {
            xs: Self::axis(rectangle.x_min, rectangle.x_max, s.x, t.x, mid.x),
            ys: Self::axis(rectangle.y_min, rectangle.y_max, s.y, t.y, mid.y),
            mid,
            leaves_vertically: direction_orientation(da) == Orientation::Vertical,
            hand_takes_the_larger: right_hand_takes_the_larger(da),
        }
    }

    fn axis(min: i32, max: i32, first: i32, second: i32, middle: i32) -> Vec<i32> {
        let mut values = vec![
            min.saturating_sub(1),
            first.saturating_sub(1),
            first,
            first.saturating_add(1),
            middle,
            second.saturating_sub(1),
            second,
            second.saturating_add(1),
            max.saturating_add(1),
        ];
        values.sort_unstable();
        values.dedup();
        values
    }

    fn at(&self, column: usize, row: usize) -> Pos {
        Pos {
            x: self.xs[column],
            y: self.ys[row],
        }
    }

    fn index_of(&self, at: Pos) -> Option<(usize, usize)> {
        Some((
            self.xs.iter().position(|&x| x == at.x)?,
            self.ys.iter().position(|&y| y == at.y)?,
        ))
    }

    fn states(&self) -> usize {
        self.xs.len() * self.ys.len() * 4
    }

    fn state(&self, column: usize, row: usize, heading: Direction) -> usize {
        (column * self.ys.len() + row) * 4 + heading_index(heading)
    }

    /// The cell, and the heading it was reached on, that a state stands for.
    fn decode(&self, state: usize) -> (usize, usize, Direction) {
        let node = state / 4;
        (
            node / self.ys.len(),
            node % self.ys.len(),
            heading_of(state % 4),
        )
    }

    /// The step from a lattice node to the next line along `heading`, or `None` past its edge.
    fn step(&self, column: usize, row: usize, heading: Direction) -> Option<(usize, usize)> {
        match heading {
            Direction::Up => Some((column, row.checked_sub(1)?)),
            Direction::Down => (row + 1 < self.ys.len()).then_some((column, row + 1)),
            Direction::Left => Some((column.checked_sub(1)?, row)),
            Direction::Right => (column + 1 < self.xs.len()).then_some((column + 1, row)),
        }
    }

    /// What a run costs beyond its bend and its length: how far the line it is fixed on sits
    /// from the middle, and from the side the arrow's travel puts to its right. Only runs on the
    /// axis the `from` endpoint leaves along can mirror each other, so only those carry a hand.
    fn run_cost(&self, turn_at: Pos, heading: Direction) -> Cost {
        let vertical = direction_orientation(heading) == Orientation::Vertical;
        let (coordinate, middle, lines) = if vertical {
            (turn_at.x, self.mid.x, &self.xs)
        } else {
            (turn_at.y, self.mid.y, &self.ys)
        };
        let hand = if vertical == self.leaves_vertically {
            let outermost = if self.hand_takes_the_larger {
                lines[lines.len() - 1]
            } else {
                lines[0]
            };
            coordinate.abs_diff(outermost)
        } else {
            0
        };
        Cost {
            bends: 1,
            length: 0,
            from_middle: coordinate.abs_diff(middle),
            hand,
        }
    }
}

/// A direction's place in the state table, so a cell and the heading it was reached on index one
/// entry between them.
fn heading_index(heading: Direction) -> usize {
    match heading {
        Direction::Up => 0,
        Direction::Right => 1,
        Direction::Down => 2,
        Direction::Left => 3,
    }
}

/// The inverse of [`heading_index`].
fn heading_of(index: usize) -> Direction {
    match index {
        0 => Direction::Up,
        1 => Direction::Right,
        2 => Direction::Down,
        _ => Direction::Left,
    }
}

/// Whether `blocked` lies on the run from `from` to `to`, `from` itself excepted — whatever
/// arrived there accounted for it already.
fn run_covers(from: Pos, to: Pos, blocked: Pos) -> bool {
    if blocked == from {
        return false;
    }
    if from.x == to.x {
        blocked.x == from.x && (from.y.min(to.y)..=from.y.max(to.y)).contains(&blocked.y)
    } else {
        blocked.y == from.y && (from.x.min(to.x)..=from.x.max(to.x)).contains(&blocked.x)
    }
}

/// Derives an arrow's route: the path between its two starting positions, per _The route of an
/// arrow_. `None` means no such path exists and the arrow is its two heads alone.
///
/// The rule is a ranking — fewest bends, then shortest, then nearest the middle, then to the
/// right of the arrow's own travel — so the derivation is the search that minimizes it rather
/// than a list of shapes to score. Nothing bounds the route but the ranking itself: a path that
/// leaves the rectangle the two starting positions span is longer than one that stays inside it,
/// so it wins only where no path inside it exists at all — which is what lets two endpoints
/// facing away from each other along one line reach each other around the outside.
fn derive_path(a: Pos, da: Direction, b: Pos, db: Direction) -> Option<Vec<Pos>> {
    // `s` and `t` — each endpoint's starting position: one step from it in that endpoint's own
    // leaving direction. The route runs between them and writes neither head's cell, so an
    // endpoint standing on the other's starting position leaves no route at all.
    let s = offset(a, da)?;
    let t = offset(b, db)?;
    let exit_dir = opposite(db);
    if s == b || t == a {
        return None;
    }
    if s == t {
        return (da != opposite(exit_dir)).then(|| vec![s]);
    }

    let lattice = Lattice::spanning(s, t, da);
    let (start_column, start_row) = lattice.index_of(s)?;
    let (goal_column, goal_row) = lattice.index_of(t)?;

    let mut reached: Vec<Option<Cost>> = vec![None; lattice.states()];
    let mut came_from: Vec<Option<usize>> = vec![None; lattice.states()];
    let start = lattice.state(start_column, start_row, da);
    reached[start] = Some(Cost::default());

    let mut frontier = BinaryHeap::new();
    frontier.push(Reverse((Cost::default(), start)));
    while let Some(Reverse((cost, state))) = frontier.pop() {
        if reached[state] != Some(cost) {
            continue;
        }
        let (column, row, heading) = lattice.decode(state);
        let at = lattice.at(column, row);
        let [one, other] = turns_from(heading);
        for next_heading in [heading, one, other] {
            let Some((next_column, next_row)) = lattice.step(column, row, next_heading) else {
                continue;
            };
            let onto = lattice.at(next_column, next_row);
            if run_covers(at, onto, a) || run_covers(at, onto, b) {
                continue;
            }
            let traveled = Cost {
                length: at.x.abs_diff(onto.x) + at.y.abs_diff(onto.y),
                ..Cost::default()
            };
            let step = if next_heading == heading {
                traveled
            } else {
                traveled.plus(lattice.run_cost(at, next_heading))
            };
            let next = lattice.state(next_column, next_row, next_heading);
            let total = cost.plus(step);
            if reached[next].is_none_or(|best| total < best) {
                reached[next] = Some(total);
                came_from[next] = Some(state);
                frontier.push(Reverse((total, next)));
            }
        }
    }

    // The route arrives at `t` on some heading and then steps into `b` against `db`. That step
    // writes no route cell, but the bend it may need is the route's, so it is charged here.
    let mut best: Option<(Cost, usize)> = None;
    for heading in [
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
    ] {
        if heading == opposite(exit_dir) {
            continue;
        }
        let Some(cost) = reached[lattice.state(goal_column, goal_row, heading)] else {
            continue;
        };
        let total = if heading == exit_dir {
            cost
        } else {
            cost.plus(lattice.run_cost(t, exit_dir))
        };
        if best.is_none_or(|(so_far, _)| total < so_far) {
            best = Some((total, lattice.state(goal_column, goal_row, heading)));
        }
    }

    let (_, arrival) = best?;
    let mut waypoints = Vec::new();
    let mut state = Some(arrival);
    while let Some(current) = state {
        let (column, row, _) = lattice.decode(current);
        waypoints.push(lattice.at(column, row));
        state = came_from[current];
    }
    waypoints.reverse();
    Some(expand_waypoints(&waypoints))
}

#[cfg(test)]
mod tests {
    use super::{Arrow, Endpoint, derive_path, offset};
    use crate::shape::counting::CountingSurface;
    use crate::{
        Buffer, Cell, Direction, Glyph, GlyphCatalog, Layer, Pos, Shape, Size, StampMode, Stroke,
        render,
    };

    fn light() -> Stroke {
        Stroke::from("light")
    }

    /// The head glyph FR-028 fixes for a picture-asserting test: opposite the leaving direction.
    fn head_for(leaving: Direction) -> Glyph {
        let text = match leaving {
            Direction::Up => "▼",
            Direction::Down => "▲",
            Direction::Left => "►",
            Direction::Right => "◄",
        };
        Glyph::new(text).expect("one of the four head glyphs is one glyph each")
    }

    fn endpoint(at: Pos, leaving: Direction) -> Endpoint {
        Endpoint {
            at,
            leaving,
            head: head_for(leaving),
        }
    }

    fn render_arrow(origin: Pos, size: Size, from: Endpoint, to: Endpoint) -> String {
        let mut buffer = Buffer::new(origin, size);
        Arrow {
            from,
            to,
            stroke: light(),
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        render(&buffer, &GlyphCatalog::light(), origin, size)
    }

    /// User story 3, scenario 1: perpendicular directions, one bend.
    #[test]
    fn scenario_1_perpendicular_one_bend() {
        let text = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 5,
                height: 4,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Down),
            endpoint(Pos { x: 4, y: 3 }, Direction::Left),
        );

        assert_eq!(text, "▲    \n│    \n│    \n└───►\n");
    }

    /// User story 3, scenario 2: the mirror of scenario 1, and scenario 3 asserts the two differ.
    #[test]
    fn scenario_2_perpendicular_one_bend_mirrored() {
        let text = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 5,
                height: 4,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
            endpoint(Pos { x: 4, y: 3 }, Direction::Up),
        );

        assert_eq!(text, "◄───┐\n    │\n    │\n    ▼\n");
    }

    /// User story 3, scenario 3, SC-002: scenarios 1 and 2 share both endpoint positions and
    /// differ only in direction, so their rendered texts must differ.
    #[test]
    fn scenario_3_direction_is_part_of_the_description() {
        let size = Size {
            width: 5,
            height: 4,
        };
        let origin = Pos { x: 0, y: 0 };
        let one = render_arrow(
            origin,
            size,
            endpoint(Pos { x: 0, y: 0 }, Direction::Down),
            endpoint(Pos { x: 4, y: 3 }, Direction::Left),
        );
        let two = render_arrow(
            origin,
            size,
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
            endpoint(Pos { x: 4, y: 3 }, Direction::Up),
        );

        assert_ne!(one, two);
    }

    /// User story 3, scenario 4: opposite directions, endpoints aligned — a straight run, no
    /// bend.
    #[test]
    fn scenario_4_opposite_aligned_no_bend() {
        let text = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 7,
                height: 1,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
            endpoint(Pos { x: 6, y: 0 }, Direction::Left),
        );

        assert_eq!(text, "◄─────►\n");
    }

    /// User story 3, scenario 5: opposite directions, endpoints not aligned — two bends, the
    /// pinned tie-break winner among three same-length candidates (research.md Q5).
    #[test]
    fn scenario_5_opposite_not_aligned_two_bends() {
        let text = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 7,
                height: 3,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
            endpoint(Pos { x: 6, y: 2 }, Direction::Left),
        );

        assert_eq!(text, "◄──┐   \n   │   \n   └──►\n");
    }

    /// User story 3, scenario 6: the same two-bend family, entirely inside the endpoint
    /// rectangle since both directions head toward the other end.
    #[test]
    fn scenario_6_two_bends_inside_the_endpoint_rectangle() {
        let text = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 9,
                height: 3,
            },
            endpoint(Pos { x: 2, y: 0 }, Direction::Right),
            endpoint(Pos { x: 8, y: 2 }, Direction::Left),
        );

        assert_eq!(text, "  ◄──┐   \n     │   \n     └──►\n");
    }

    /// User story 3, scenario 7: opposite directions, both endpoints facing away — four bends,
    /// the route rectangle one cell wider on each side.
    #[test]
    fn scenario_7_facing_away_four_bends() {
        let text = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 10,
                height: 3,
            },
            endpoint(Pos { x: 2, y: 0 }, Direction::Left),
            endpoint(Pos { x: 8, y: 2 }, Direction::Right),
        );

        assert_eq!(text, " ┌►       \n └───────┐\n        ◄┘\n");
    }

    /// User story 3, scenario 8: perpendicular directions, one endpoint heading away — the route
    /// rectangle grows on that one side only.
    #[test]
    fn scenario_8_perpendicular_one_heading_away() {
        let text = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 9,
                height: 4,
            },
            endpoint(Pos { x: 2, y: 0 }, Direction::Left),
            endpoint(Pos { x: 8, y: 2 }, Direction::Down),
        );

        assert_eq!(text, " ┌►      \n │       \n │      ▲\n └──────┘\n");
    }

    /// User story 3, scenario 9: the tightest two-bend arrangement, where the two starting
    /// positions share a column and the two bends are adjacent.
    #[test]
    fn scenario_9_tightest_two_bend_arrangement() {
        let text = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 5,
                height: 2,
            },
            endpoint(Pos { x: 2, y: 0 }, Direction::Right),
            endpoint(Pos { x: 4, y: 1 }, Direction::Left),
        );

        assert_eq!(text, "  ◄┐ \n   └►\n");
    }

    /// User story 3, scenario 10: the tightest arrangement of the facing-away family.
    #[test]
    fn scenario_10_tightest_facing_away_arrangement() {
        let text = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 5,
                height: 3,
            },
            endpoint(Pos { x: 2, y: 0 }, Direction::Left),
            endpoint(Pos { x: 3, y: 2 }, Direction::Right),
        );

        assert_eq!(text, " ┌►  \n └──┐\n   ◄┘\n");
    }

    /// User story 3, scenario 11, SC-002: the three arrows of scenarios 6, 7 and 8 share both
    /// endpoint positions and differ only in direction, so all three rendered texts must differ.
    #[test]
    fn scenario_11_the_three_shared_endpoint_arrows_all_differ() {
        let six = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 9,
                height: 3,
            },
            endpoint(Pos { x: 2, y: 0 }, Direction::Right),
            endpoint(Pos { x: 8, y: 2 }, Direction::Left),
        );
        let seven = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 9,
                height: 3,
            },
            endpoint(Pos { x: 2, y: 0 }, Direction::Left),
            endpoint(Pos { x: 8, y: 2 }, Direction::Right),
        );
        let eight = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 9,
                height: 4,
            },
            endpoint(Pos { x: 2, y: 0 }, Direction::Left),
            endpoint(Pos { x: 8, y: 2 }, Direction::Down),
        );

        assert_ne!(six, seven);
        assert_ne!(six, eight);
        assert_ne!(seven, eight);
    }

    /// The direction families table's identical-directions row, the geometry where a path fits:
    /// both endpoints leaving `Right`, one heading toward the other end and one heading away.
    /// Produced by running the code, per _Claims are measured, not assumed_ — the row is
    /// deliberately not pinned by the spec.
    #[test]
    fn identical_directions_where_a_path_fits() {
        let text = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 8,
                height: 3,
            },
            endpoint(Pos { x: 2, y: 0 }, Direction::Right),
            endpoint(Pos { x: 6, y: 2 }, Direction::Right),
        );

        assert_eq!(text, "  ◄────┐\n       │\n      ◄┘\n");
    }

    /// The direction families table's identical-directions row, the geometry where the two
    /// endpoints are in line on that axis. No path fits between the two starting positions, so
    /// the route steps outside the rectangle they span and comes back — four bends, which is
    /// fewer than any path that stayed inside could manage, there being none.
    #[test]
    fn identical_directions_in_line_route_around_the_outside() {
        let text = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 6,
                height: 2,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
            endpoint(Pos { x: 4, y: 0 }, Direction::Right),
        );

        assert_eq!(text, concat!("◄──┐◄┐\n", "   └─┘\n"));
    }

    /// Issue 103's first example: two endpoints facing away from each other and aligned on the
    /// axis they face along, however far apart, are joined rather than left as two heads with a
    /// gap between them.
    #[test]
    fn facing_away_and_aligned_is_joined_around_the_outside() {
        let origin = Pos { x: -1, y: -1 };
        let size = Size {
            width: 3,
            height: 5,
        };

        let up_first = render_arrow(
            origin,
            size,
            endpoint(Pos { x: 0, y: 0 }, Direction::Up),
            endpoint(Pos { x: 0, y: 2 }, Direction::Down),
        );
        let down_first = render_arrow(
            origin,
            size,
            endpoint(Pos { x: 0, y: 2 }, Direction::Down),
            endpoint(Pos { x: 0, y: 0 }, Direction::Up),
        );

        assert_eq!(
            up_first,
            concat!(" ┌┐\n", " ▼│\n", "  │\n", " ▲│\n", " └┘\n")
        );
        assert_eq!(
            down_first,
            concat!("┌┐ \n", "│▼ \n", "│  \n", "│▲ \n", "└┘ \n")
        );
    }

    /// Issue 103's second example, and the tie-break the two orders above show: the two routes
    /// mirror each other about the line the endpoints share, neither is nearer the endpoint the
    /// arrow leaves from, and the one to the right of its own travel wins. Leaving `Down`, the
    /// right hand is the smaller `x`.
    #[test]
    fn identical_directions_in_line_passes_to_the_right_of_its_own_travel() {
        let text = render_arrow(
            Pos { x: -1, y: 0 },
            Size {
                width: 3,
                height: 4,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Down),
            endpoint(Pos { x: 0, y: 2 }, Direction::Down),
        );

        assert_eq!(text, concat!(" ▲ \n", "┌┘ \n", "│▲ \n", "└┘ \n"));
    }

    /// The family research.md Q2 needed a seventh run — a double escape — to route inside the
    /// route rectangle. Once a route may leave that rectangle, the same arrangement is four
    /// bends around the outside rather than six inside, so the double escape is not a shape the
    /// rule can ever choose.
    #[test]
    fn the_double_escape_family_is_four_bends_around_the_outside() {
        let text = render_arrow(
            Pos { x: -1, y: -1 },
            Size {
                width: 4,
                height: 5,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Up),
            endpoint(Pos { x: 1, y: 2 }, Direction::Down),
        );

        assert_eq!(
            text,
            concat!("┌┐  \n", "│▼  \n", "│   \n", "│ ▲ \n", "└─┘ \n")
        );
    }

    /// The arrangements the rule still leaves without a route: the cell the route would have to
    /// arrive at is the other endpoint's own, so there is nowhere for it to go. In every one of
    /// them the two heads are touching, so nothing looks disconnected.
    #[test]
    fn an_endpoint_standing_on_the_others_starting_position_leaves_no_route() {
        let text = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 2,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Up),
            endpoint(Pos { x: 0, y: 1 }, Direction::Up),
        );

        assert_eq!(text, concat!("▼\n", "▼\n"));
    }

    /// The direction families table's last row and user story 3's scenario 13: an arrow whose
    /// two endpoints coincide returns normally — no error, no panic. What it draws is the general
    /// rule's business and is deliberately not asserted here.
    #[test]
    fn both_endpoints_at_the_same_position_returns_normally() {
        let _ = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
            endpoint(Pos { x: 0, y: 0 }, Direction::Left),
        );
    }

    /// User story 3, scenario 12: none of the ten pinned arrows writes any position more than
    /// once — including where a route bends, which is where a route drawn as two overlapping
    /// runs would write twice (FR-020).
    #[test]
    fn no_pinned_arrow_writes_any_position_more_than_once() {
        let pairs = [
            (
                endpoint(Pos { x: 0, y: 0 }, Direction::Down),
                endpoint(Pos { x: 4, y: 3 }, Direction::Left),
            ),
            (
                endpoint(Pos { x: 0, y: 0 }, Direction::Right),
                endpoint(Pos { x: 4, y: 3 }, Direction::Up),
            ),
            (
                endpoint(Pos { x: 0, y: 0 }, Direction::Right),
                endpoint(Pos { x: 6, y: 0 }, Direction::Left),
            ),
            (
                endpoint(Pos { x: 0, y: 0 }, Direction::Right),
                endpoint(Pos { x: 6, y: 2 }, Direction::Left),
            ),
            (
                endpoint(Pos { x: 2, y: 0 }, Direction::Right),
                endpoint(Pos { x: 8, y: 2 }, Direction::Left),
            ),
            (
                endpoint(Pos { x: 2, y: 0 }, Direction::Left),
                endpoint(Pos { x: 8, y: 2 }, Direction::Right),
            ),
            (
                endpoint(Pos { x: 2, y: 0 }, Direction::Left),
                endpoint(Pos { x: 8, y: 2 }, Direction::Down),
            ),
            (
                endpoint(Pos { x: 2, y: 0 }, Direction::Right),
                endpoint(Pos { x: 4, y: 1 }, Direction::Left),
            ),
            (
                endpoint(Pos { x: 2, y: 0 }, Direction::Left),
                endpoint(Pos { x: 3, y: 2 }, Direction::Right),
            ),
        ];

        for (index, (from, to)) in pairs.into_iter().enumerate() {
            let mut surface = CountingSurface::default();
            Arrow {
                from,
                to,
                stroke: light(),
            }
            .draw(&mut surface);

            assert!(
                surface.max_writes() <= 1,
                "pinned arrow {} wrote a position more than once",
                index + 1
            );
        }
    }

    /// SC-002: the bug report's arrow renders the same picture whichever endpoint is named
    /// first, instead of drawing the route backwards out of its starting cell.
    #[test]
    fn sc002_the_bug_report_renders_the_same_from_either_end() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 7,
            height: 1,
        };
        let expected = "◄─────►\n";

        let named_left_first = render_arrow(
            origin,
            size,
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
            endpoint(Pos { x: 6, y: 0 }, Direction::Left),
        );
        let named_right_first = render_arrow(
            origin,
            size,
            endpoint(Pos { x: 6, y: 0 }, Direction::Left),
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
        );

        assert_eq!(named_left_first, expected);
        assert_eq!(named_right_first, expected);
    }

    /// SC-003: the ADR-0044 pair's free coordinate spans an even number of cells, so the two
    /// orders draw different — and equally correct — pictures, turning at row 3 or row 4
    /// depending on which endpoint is named first.
    #[test]
    fn sc003_the_adr_0044_pair_turns_at_the_row_nearer_the_endpoint_named_first() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 3,
            height: 7,
        };

        let up_first = render_arrow(
            origin,
            size,
            endpoint(Pos { x: 0, y: 1 }, Direction::Down),
            endpoint(Pos { x: 2, y: 6 }, Direction::Up),
        );
        let down_first = render_arrow(
            origin,
            size,
            endpoint(Pos { x: 2, y: 6 }, Direction::Up),
            endpoint(Pos { x: 0, y: 1 }, Direction::Down),
        );

        assert_eq!(
            up_first,
            concat!(
                "   \n",
                "▲  \n",
                "│  \n",
                "└─┐\n",
                "  │\n",
                "  │\n",
                "  ▼\n",
            )
        );
        assert_eq!(
            down_first,
            concat!(
                "   \n",
                "▲  \n",
                "│  \n",
                "│  \n",
                "└─┐\n",
                "  │\n",
                "  ▼\n",
            )
        );
    }

    /// ADR-0044 where the free coordinate spans exactly two cells: the middle is then one of the
    /// two ends of the span, and the shape that turns there names a point twice. Scoring a
    /// candidate by how far its free runs sit from the middle — rather than by whether it has
    /// four waypoints — is what makes these two turn nearer the endpoint named first, and
    /// mirror each other, instead of both turning away from it.
    #[test]
    fn a_free_span_two_cells_wide_still_turns_nearer_the_endpoint_named_first() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 4,
            height: 2,
        };

        let right_first = render_arrow(
            origin,
            size,
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
            endpoint(Pos { x: 3, y: 1 }, Direction::Left),
        );
        let left_first = render_arrow(
            origin,
            size,
            endpoint(Pos { x: 3, y: 1 }, Direction::Left),
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
        );

        assert_eq!(right_first, concat!("◄┐  \n", " └─►\n"));
        assert_eq!(left_first, concat!("◄─┐ \n", "  └►\n"));
    }

    /// SC-004: each arrangement in User Story 2's table turns at the middle of its route
    /// rectangle rather than at an edge (`n = 4`) or at neither of the two middle cells
    /// (`n = 5`). `n = 5` names the far endpoint first so ADR-0044's tie-break picks the cell
    /// this table pins, `x = 3`.
    #[test]
    fn sc004_each_arrangement_turns_at_the_middle_of_its_route_rectangle() {
        let origin = Pos { x: 0, y: 0 };

        let n4 = render_arrow(
            origin,
            Size {
                width: 5,
                height: 4,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
            endpoint(Pos { x: 4, y: 3 }, Direction::Left),
        );
        let n5 = render_arrow(
            origin,
            Size {
                width: 6,
                height: 4,
            },
            endpoint(Pos { x: 5, y: 3 }, Direction::Left),
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
        );
        let n6 = render_arrow(
            origin,
            Size {
                width: 7,
                height: 4,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
            endpoint(Pos { x: 6, y: 3 }, Direction::Left),
        );

        assert_eq!(n4, concat!("◄─┐  \n", "  │  \n", "  │  \n", "  └─►\n"));
        assert_eq!(n5, concat!("◄──┐  \n", "   │  \n", "   │  \n", "   └─►\n"));
        assert_eq!(
            n6,
            concat!("◄──┐   \n", "   │   \n", "   │   \n", "   └──►\n")
        );
    }

    /// FR-006, SC-004's second half: the shipped demonstration's arrow turns at the middle of
    /// its route rectangle. The middle is computed here from the two endpoint positions rather
    /// than transcribed from a picture, which is what would have caught this defect had it
    /// existed in the demonstration.
    #[test]
    fn fr006_the_demonstration_turns_at_the_middle_of_its_route_rectangle() {
        let a = Pos { x: 13, y: 3 };
        let da = Direction::Right;
        let b = Pos { x: 22, y: 4 };
        let db = Direction::Down;

        let s = offset(a, da).expect("no overflow in this fixture");
        let t = offset(b, db).expect("no overflow in this fixture");
        let middle_x = i32::midpoint(s.x.min(t.x), s.x.max(t.x));
        assert_eq!(middle_x, 18);

        let path = derive_path(a, da, b, db).expect("the demonstration's arrow has a route");
        assert!(path.contains(&Pos {
            x: middle_x,
            y: s.y
        }));
        assert!(path.contains(&Pos {
            x: middle_x,
            y: t.y
        }));
    }

    /// C-6, FR-004: where both endpoints occupy one position, the glyph seen is the `to`
    /// endpoint's head — `Arrow` draws `from` then `to`, and `Above` lets the second win. Pins
    /// the behavior; does not change it.
    #[test]
    fn c6_the_to_head_wins_a_shared_cell() {
        let text = render_arrow(
            Pos { x: 2, y: 1 },
            Size {
                width: 1,
                height: 1,
            },
            endpoint(Pos { x: 2, y: 1 }, Direction::Right),
            endpoint(Pos { x: 2, y: 1 }, Direction::Left),
        );

        assert_eq!(text, "►\n");
    }

    /// The window the sweep renders into — research.md Q6 — sized to hold the six-by-five field
    /// plus the one cell of margin a leaving direction can add on each side.
    const SWEEP_ORIGIN: Pos = Pos { x: -2, y: -2 };
    const SWEEP_SIZE: Size = Size {
        width: 10,
        height: 9,
    };

    /// One group of [`sweep_arrangements_by_anchor`]: the anchor, its leaving direction, and
    /// every `(position, leaving direction)` the anchor is paired with.
    type AnchorGroup = (Pos, Direction, Vec<(Pos, Direction)>);

    /// The grid research.md Q6 defines, grouped by anchor and its leaving direction: two anchors,
    /// each leaving in four directions — eight groups of 116 arrangements each, against every
    /// position of a six-by-five field with four leaving directions each, excluding the
    /// arrangements where the second position is the anchor itself. The grouping is what lets the
    /// sweep's snapshot split into one file per group instead of one no review tool can render.
    fn sweep_arrangements_by_anchor() -> Vec<AnchorGroup> {
        const DIRECTIONS: [Direction; 4] = [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ];
        let anchors = [Pos { x: 0, y: 0 }, Pos { x: 2, y: 1 }];

        let mut groups = Vec::new();
        for anchor in anchors {
            for anchor_dir in DIRECTIONS {
                let mut others = Vec::new();
                for x in 0..6 {
                    for y in 0..5 {
                        let other = Pos { x, y };
                        if other == anchor {
                            continue;
                        }
                        for other_dir in DIRECTIONS {
                            others.push((other, other_dir));
                        }
                    }
                }
                groups.push((anchor, anchor_dir, others));
            }
        }
        groups
    }

    /// The grid research.md Q6 defines, flattened — 928 arrangements.
    fn sweep_arrangements() -> Vec<(Pos, Direction, Pos, Direction)> {
        sweep_arrangements_by_anchor()
            .into_iter()
            .flat_map(|(anchor, anchor_dir, others)| {
                others
                    .into_iter()
                    .map(move |(other, other_dir)| (anchor, anchor_dir, other, other_dir))
            })
            .collect()
    }

    /// Research.md Q6's own check on its grid definition: it reproduces the spec's 928.
    #[test]
    fn the_sweep_grid_has_928_arrangements() {
        assert_eq!(sweep_arrangements().len(), 928);
    }

    /// C-2 and C-3 over the whole grid, rendered from both ends (SC-001): both endpoint
    /// positions always render their own head, and drawing into a surface that counts writes
    /// never writes any position more than once.
    #[test]
    fn sweep_every_endpoint_renders_its_own_head_and_no_position_is_written_twice() {
        for (a, da, b, db) in sweep_arrangements() {
            for (from_at, from_dir, to_at, to_dir) in [(a, da, b, db), (b, db, a, da)] {
                let mut buffer = Buffer::new(SWEEP_ORIGIN, SWEEP_SIZE);
                Arrow {
                    from: endpoint(from_at, from_dir),
                    to: endpoint(to_at, to_dir),
                    stroke: light(),
                }
                .draw(&mut Layer::new(&mut buffer, StampMode::Above));
                assert_eq!(
                    buffer.cell(from_at),
                    Some(&Cell::Literal(head_for(from_dir))),
                    "({from_at:?}, {from_dir:?}) -> ({to_at:?}, {to_dir:?}): from's own head"
                );
                assert_eq!(
                    buffer.cell(to_at),
                    Some(&Cell::Literal(head_for(to_dir))),
                    "({from_at:?}, {from_dir:?}) -> ({to_at:?}, {to_dir:?}): to's own head"
                );

                let mut counting = CountingSurface::default();
                Arrow {
                    from: endpoint(from_at, from_dir),
                    to: endpoint(to_at, to_dir),
                    stroke: light(),
                }
                .draw(&mut counting);
                assert!(
                    counting.max_writes() <= 1,
                    "({from_at:?}, {from_dir:?}) -> ({to_at:?}, {to_dir:?}) wrote a position more than once"
                );
            }
        }
    }

    /// C-1, SC-001: the whole grid, rendered from both ends and labeled by arrangement, pinned as
    /// one reviewed snapshot per anchor and leaving direction — eight files rather than one, so a
    /// PR review tool can render each diff; a single 20,000-line file is what GitHub would not
    /// show at all — per
    /// [ADR-0045](../../../../../docs/decisions/0045-pin-every-arrow-arrangement-as-a-reviewed-snapshot.md).
    /// Each line's trailing blanks are trimmed before it goes into the snapshot — research.md
    /// Q3 — since the gate's `editorconfig-checker` step runs with `trim_trailing_whitespace` on
    /// and `render` pads every line to the window's width.
    #[test]
    fn sweep_matches_the_reviewed_snapshot() {
        use std::fmt::Write as _;

        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(concat!(env!("CARGO_MANIFEST_DIR"), "/src/snapshots"));
        settings.bind(|| {
            for (anchor, anchor_dir, others) in sweep_arrangements_by_anchor() {
                let mut rendered = String::new();
                for (other, other_dir) in others {
                    for (from_at, from_dir, to_at, to_dir) in [
                        (anchor, anchor_dir, other, other_dir),
                        (other, other_dir, anchor, anchor_dir),
                    ] {
                        let text = render_arrow(
                            SWEEP_ORIGIN,
                            SWEEP_SIZE,
                            endpoint(from_at, from_dir),
                            endpoint(to_at, to_dir),
                        );
                        let trimmed: Vec<&str> = text.lines().map(str::trim_end).collect();
                        let _ = writeln!(
                            rendered,
                            "({from_at:?}, {from_dir:?}) -> ({to_at:?}, {to_dir:?})\n{}\n",
                            trimmed.join("\n")
                        );
                    }
                }
                let name = format!(
                    "arrow_sweep_anchor_{}_{}_{anchor_dir:?}",
                    anchor.x, anchor.y
                )
                .to_lowercase();
                insta::assert_snapshot!(name, rendered);
            }
        });
    }
}
