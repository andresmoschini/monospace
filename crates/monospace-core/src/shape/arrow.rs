//! The arrow: two endpoints, two heads and a derived route between them. See _The initial set_
//! and _The route of an arrow_ in [`docs/model.md`](../../../docs/model.md), and research.md Q5
//! in `specs/039-draw-shapes-instead-of-individual-cells/`.
//!
//! # Design notes
//!
//! Nothing outside this module observes the rule below beyond the picture it produces. It decides
//! which of several paths — every one of them satisfying _The route of an arrow_ — this
//! implementation draws, so changing it is an ordinary `feat` or `fix`: it amends no model
//! document and it takes no ADR
//! ([the constitution](../../../.specify/memory/constitution.md), principle VI). The reasoning
//! below was recorded first in ADR-0044, ADR-0048 and ADR-0049, each of them now absorbed into
//! this file.
//!
//! The route is the path ranked first by [`Cost`], whose fields are the four terms of the ranking
//! in order. What the code cannot say is why each term is there.
//!
//! **1, the fewest bends, and 2, the shortest.** These two are the rule; the other two only break
//! its ties. Together they replaced a bound — a route was once required to stay inside the
//! rectangle its two starting positions span — because the shortest path is what that bound was
//! trying to name. Dropping it cost the model a construct and gained 134 arrangements that had
//! drawn two disconnected heads, since a rectangle one cell thick holds no alternating path at
//! all. Term 2 decides nothing on a monotone path, where every candidate is the same length, so
//! it is invisible across most of the characterization and easy to break without a picture
//! moving. [`RouteRectangle`] survives only as the span term 3 measures its middle within, and
//! bounds nothing.
//!
//! **3, nearest the middle.** The first two leave many candidates level, because a free run may
//! sit anywhere, so something has to choose and the middle is the choice that reads as centred.
//! Where a span holds an even number of cells that middle falls between two and names no winner;
//! it is taken nearer the endpoint the arrow leaves from, because an arrow runs from its `from`
//! to its `to` and that is the only thing in the arrangement that tells the two candidates apart.
//! The cost is that the same arrow described from its other end turns at the other of the two.
//! Both pictures are right, which is what makes this a tie-break rather than a defect.
//!
//! **4, to the right of the arrow's own travel.** Once nothing bounds a route, an arrangement
//! whose two starting positions share a row or a column has two candidates that are exact mirror
//! images about that line — equidistant from the middle by construction, so term 3 cannot reach
//! them. Of the 1856 renderings in the characterization, 84 are this one shape. It is phrased as
//! a handedness rather than as "the smaller coordinate" so that it is the same principle as term
//! 3 instead of a second, unrelated one; which of the two mirrors counts as the right is
//! arbitrary, was picked by looking at both rendered, and nothing distinguishes it from the
//! other. Coordinates grow rightward and downward, so leaving `Up` or `Right` puts the larger
//! coordinate to the right — [`is_right_of_travel`] says it in those words, because a handedness
//! under a `y` that grows downward is what a reader gets backwards.
//!
//! Two endpoints at one position put both heads on one cell and only one of them can be seen. It
//! is the `to` endpoint's, drawn over the other, for the same reason term 3 rounds toward the
//! `from`.
//!
//! **Why a search rather than a construction.** [`derive_path`] was once a list of route shapes,
//! and a list has to be argued complete. That argument failed twice, both times by omission and
//! both times silently — a missing shape does not error, it draws the second-best picture or
//! nothing. A ranking is a cost function, so the implementation that reads against it is the one
//! that minimizes it, and completeness stops being an argument. [`Lattice`] is what keeps the
//! cost fixed instead of proportional to the distance between the endpoints, which this crate
//! needs because the core compiles to WebAssembly. Its own claim — that a route turns only on
//! those lines — is confirmed against an unrestricted search rather than proved, and a term added
//! to [`Cost`] that the lattice knows nothing about is exactly how that confirmation goes stale.
//! Re-run it against any new term.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use crate::shape::fragment::head::Head;
use crate::shape::route::Route;
use crate::{Direction, Glyph, Orientation, Pos, Shape, Stroke, Surface};

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

/// The smallest rectangle containing both starting positions. It bounds nothing; it is the span
/// term 3 of the ranking measures its middle within — the module's _Design notes_.
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
    /// of cells and the halfway point falls between two — term 3 of the ranking, and why it
    /// rounds that way, in the module's _Design notes_.
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

/// The ranking of a route as one value — fewest bends, then shortest, then nearest the middle,
/// then to the right of the travel — compared lexicographically in the fields' declared order.
/// Deriving `Ord` over them **is** the ranking; nothing here is a written comparator
/// (data-model.md). Why each term is there: the module's _Design notes_.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Cost {
    bends: u32,
    length: u32,
    from_middle: u32,
    hand: u32,
}

/// A fixed order over the four directions, used only to index arrays and hash-map keys —
/// `Direction` derives no `Hash`, and this module stays the one place that needs one.
const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];

fn direction_index(d: Direction) -> usize {
    DIRECTIONS
        .iter()
        .position(|&candidate| candidate == d)
        .expect("DIRECTIONS lists all four directions")
}

/// The lines a route may turn on, one per axis: the line each starting position pins, the line
/// beside each of them, the middle, and one line outside the route rectangle on either side —
/// deduplicated to at most 7 distinct lines (research.md Q2). A node is a pair of indices into
/// `x` and `y`, so at most 7 x 7 x 4 = 196 states whatever the arrangement (FR-008, R-8), because
/// a route turns only here — a claim the module's _Design notes_ call confirmed rather than
/// proved, and what to re-run when a term is added to [`Cost`].
struct Lattice {
    x: Vec<i32>,
    y: Vec<i32>,
}

/// A node of [`Lattice`], paired with the index into [`DIRECTIONS`] of the heading a run reaching
/// it travels along.
type State = ((usize, usize), usize);

impl Lattice {
    fn new(s: Pos, t: Pos, mid: Pos) -> Self {
        Self {
            x: Self::axis(s.x, t.x, mid.x),
            y: Self::axis(s.y, t.y, mid.y),
        }
    }

    fn axis(s_c: i32, t_c: i32, mid_c: i32) -> Vec<i32> {
        let mut values = vec![s_c - 1, s_c, s_c + 1, mid_c, t_c - 1, t_c, t_c + 1];
        values.sort_unstable();
        values.dedup();
        values
    }

    fn index_of(values: &[i32], value: i32) -> usize {
        values
            .binary_search(&value)
            .expect("value is one of axis's own candidates")
    }

    fn node(&self, pos: Pos) -> (usize, usize) {
        (
            Self::index_of(&self.x, pos.x),
            Self::index_of(&self.y, pos.y),
        )
    }

    fn pos(&self, node: (usize, usize)) -> Pos {
        Pos {
            x: self.x[node.0],
            y: self.y[node.1],
        }
    }

    /// One step from `node` along `d`, or `None` at the lattice's own edge.
    fn step(&self, node: (usize, usize), d: Direction) -> Option<(usize, usize)> {
        let (ix, iy) = node;
        match d {
            Direction::Up => iy.checked_sub(1).map(|iy| (ix, iy)),
            Direction::Down => (iy + 1 < self.y.len()).then_some((ix, iy + 1)),
            Direction::Left => ix.checked_sub(1).map(|ix| (ix, iy)),
            Direction::Right => (ix + 1 < self.x.len()).then_some((ix + 1, iy)),
        }
    }
}

/// Whether the coordinate `c` sits on the side leaving `da` puts to the travel's right, of
/// `mid_c` — term 4 of the ranking, in the module's _Design notes_. Coordinates grow rightward
/// and downward, so leaving `Up` or `Right` puts the larger coordinate to the right, and leaving
/// `Down` or `Left` the smaller.
fn is_right_of_travel(da: Direction, c: i32, mid_c: i32) -> bool {
    match da {
        Direction::Up | Direction::Right => c >= mid_c,
        Direction::Down | Direction::Left => c <= mid_c,
    }
}

/// The `from_middle` and `hand` a run of `orientation`, sitting at `pos` — any point along it,
/// since a run's own fixed coordinate does not change — charges against `mid` and the arrow's own
/// travel `da`. `hand` is charged only where `orientation` matches `da`'s own, which is what makes
/// term 4's handedness the arrow's own travel rather than the coordinate system's
/// (data-model.md).
fn run_cost(orientation: Orientation, pos: Pos, mid: Pos, da: Direction) -> (u32, u32) {
    let (c, mid_c) = match orientation {
        Orientation::Horizontal => (pos.y, mid.y),
        Orientation::Vertical => (pos.x, mid.x),
    };
    let from_middle = c.abs_diff(mid_c);
    let hand = if orientation == direction_orientation(da) {
        u32::from(!is_right_of_travel(da, c, mid_c))
    } else {
        0
    };
    (from_middle, hand)
}

/// Derives an arrow's route: the path between its two starting positions, per _The route of an
/// arrow_. `None` means no path exists and the arrow is its two heads alone.
///
/// A single-source Dijkstra over states `(node, heading)`, `node` a point of [`Lattice`] and
/// `heading` the direction the run reaching it travels along, minimized by [`Cost`] — nothing
/// bounds where the search goes beyond the lattice itself. The search starts
/// at `s` with a virtual heading `da`, as though the route already arrived there leaving `a`, and
/// ends at `t` against a virtual `exit_dir`, the direction continuing from `t` into `b`; both
/// virtual steps charge a bend exactly where a real one would, and forbid reversing. `from_middle`
/// and `hand` are charged to a run exactly when a bend closes it *and* a bend opened it — an
/// interior run always opened with one, and closes with the one being charged now; the run
/// leaving `s` opened with one only if its first direction is not `da`, which is exactly when
/// `cost.bends` is still nonzero for the run about to close (data-model.md).
fn derive_path(a: Pos, da: Direction, b: Pos, db: Direction) -> Option<Vec<Pos>> {
    // Two endpoints at one position leaving the same direction: `s` and `t` coincide and the
    // only way to arrive would be to leave immediately in the opposite direction, which
    // _The route of an arrow_ declines as a path returning to where it began. A search without
    // the full path in its state cannot see that a longer alternative loops back over itself, so
    // this is ruled out directly rather than left to the search to discover.
    if a == b && da == db {
        return None;
    }
    // `s` and `t` — each endpoint's starting position: one step from it in that endpoint's own
    // leaving direction.
    let s = offset(a, da)?;
    let t = offset(b, db)?;
    let exit_dir = opposite(db);
    // No route cell is an endpoint position (data-model.md invariant 2). `a` and `b` are always
    // lattice nodes — `a`'s coordinate on `da`'s axis is `s`'s own minus or plus one, and its
    // other coordinate is `s`'s own, both always lattice candidates — so excluding them from the
    // graph excludes every path that would cross a head.
    let forbidden = |p: Pos| p == a || p == b;
    if forbidden(s) || forbidden(t) {
        return None;
    }

    let rectangle = RouteRectangle::spanning(s, t);
    let mid = rectangle.middle(s);
    let lattice = Lattice::new(s, t, mid);
    let start = (lattice.node(s), direction_index(da));
    let goal_node = lattice.node(t);

    let mut dist: HashMap<State, Cost> = HashMap::new();
    let mut prev: HashMap<State, State> = HashMap::new();
    let mut heap = BinaryHeap::new();
    dist.insert(start, Cost::default());
    heap.push(Reverse((Cost::default(), start)));

    while let Some(Reverse((cost, key))) = heap.pop() {
        if dist.get(&key) != Some(&cost) {
            continue;
        }
        let (node, heading_index) = key;
        let heading = DIRECTIONS[heading_index];
        let pos = lattice.pos(node);

        for d in DIRECTIONS {
            if d == opposite(heading) {
                continue;
            }
            let Some(neighbor) = lattice.step(node, d) else {
                continue;
            };
            let neighbor_pos = lattice.pos(neighbor);
            if forbidden(neighbor_pos) {
                continue;
            }

            let mut next = cost;
            next.length += match direction_orientation(d) {
                Orientation::Horizontal => neighbor_pos.x.abs_diff(pos.x),
                Orientation::Vertical => neighbor_pos.y.abs_diff(pos.y),
            };
            if d != heading {
                next.bends += 1;
                if cost.bends > 0 {
                    let (from_middle, hand) =
                        run_cost(direction_orientation(heading), pos, mid, da);
                    next.from_middle += from_middle;
                    next.hand += hand;
                }
            }

            let next_key = (neighbor, direction_index(d));
            if dist.get(&next_key).is_none_or(|&best| next < best) {
                dist.insert(next_key, next);
                prev.insert(next_key, key);
                heap.push(Reverse((next, next_key)));
            }
        }
    }

    let (_, winner) = DIRECTIONS
        .into_iter()
        .filter(|&h| h != opposite(exit_dir))
        .filter_map(|h| {
            let key = (goal_node, direction_index(h));
            let cost = *dist.get(&key)?;
            let mut total = cost;
            if h != exit_dir {
                total.bends += 1;
                if cost.bends > 0 {
                    let (from_middle, hand) = run_cost(direction_orientation(h), t, mid, da);
                    total.from_middle += from_middle;
                    total.hand += hand;
                }
            }
            Some((total, key))
        })
        .min_by_key(|(total, _)| *total)?;

    let mut path = Vec::new();
    let mut current = winner;
    loop {
        path.push(lattice.pos(current.0));
        if current == start {
            break;
        }
        current = prev[&current];
    }
    path.reverse();
    Some(expand_waypoints(&path))
}

#[cfg(test)]
mod tests {
    use super::{Arrow, Endpoint, Lattice, RouteRectangle, derive_path, offset};
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
    /// endpoints are in line on that axis: `s` and `t` sit on the same row with the `to`
    /// endpoint's own head between them, so the straight run the old bound drew nothing for is
    /// not a route the rule can draw either — it would cross a head. The ranking instead
    /// routes around the outside, per _The route of an arrow_. SC-009, research.md Q6: the only
    /// test in the workspace whose pinned picture this feature moves, renamed and re-pinned
    /// rather than deleted. Spec's User Story 1 acceptance scenario 2.
    #[test]
    fn identical_directions_in_line_are_joined_around_the_outside() {
        let text = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 6,
                height: 2,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
            endpoint(Pos { x: 4, y: 0 }, Direction::Right),
        );

        assert_eq!(text, "◄──┐◄┐\n   └─┘\n");
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

    /// User story 2, FR-005, SC-004, R-5: where the coordinate the bends leave free spans exactly
    /// two cells, the route turns at the cell nearer the endpoint the arrow leaves from. Issue
    /// 104: today the `waypoints.len() > 3` proxy discards the winning shape for naming one point
    /// twice and the opposite corner wins instead, so both orders turn the wrong way round. This
    /// test MUST pass unchanged through the Dijkstra rewrite (research.md Q3).
    #[test]
    fn fr005_a_two_cell_free_span_turns_toward_the_endpoint_the_arrow_leaves_from() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 4,
            height: 2,
        };

        let leaving_right_first = render_arrow(
            origin,
            size,
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
            endpoint(Pos { x: 3, y: 1 }, Direction::Left),
        );
        let leaving_left_first = render_arrow(
            origin,
            size,
            endpoint(Pos { x: 3, y: 1 }, Direction::Left),
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
        );

        assert_eq!(leaving_right_first, concat!("◄┐  \n", " └─►\n"));
        assert_eq!(leaving_left_first, concat!("◄─┐ \n", "  └►\n"));
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

    /// User Story 1, R-1, FR-002, spec acceptance scenarios 1 and 3: two endpoints facing away
    /// from each other on one line are joined, wrapping around the outside on the side the
    /// travel puts to its right, however far apart the two are — the same shape with longer runs.
    #[test]
    fn facing_away_are_joined_at_any_distance() {
        let near = render_arrow(
            Pos { x: 0, y: -1 },
            Size {
                width: 2,
                height: 5,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Up),
            endpoint(Pos { x: 0, y: 2 }, Direction::Down),
        );
        let far = render_arrow(
            Pos { x: 0, y: -1 },
            Size {
                width: 2,
                height: 21,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Up),
            endpoint(Pos { x: 0, y: 18 }, Direction::Down),
        );

        assert_eq!(near, "┌┐\n▼│\n │\n▲│\n└┘\n");
        assert_eq!(far, format!("┌┐\n▼│\n{}▲│\n└┘\n", " │\n".repeat(17)));
    }

    /// User Story 1, R-1, FR-002, spec acceptance scenario 6: the same family with the two
    /// starting positions on different columns takes the same four bends around the outside.
    #[test]
    fn facing_away_off_column_takes_four_bends_around_the_outside() {
        let text = render_arrow(
            Pos { x: -1, y: -1 },
            Size {
                width: 3,
                height: 5,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Up),
            endpoint(Pos { x: 1, y: 2 }, Direction::Down),
        );

        assert_eq!(text, "┌┐ \n│▼ \n│  \n│ ▲\n└─┘\n");
    }

    /// User Story 1, R-4, FR-007, SC-005, spec acceptance scenario 5: the former double escape
    /// takes four bends rather than six, and the two orders mirror each other rather than drawing
    /// the same picture — `zigzag` is gone.
    #[test]
    fn the_former_double_escape_takes_four_bends_and_the_two_orders_mirror() {
        let size = Size {
            width: 5,
            height: 3,
        };

        let leaving_left_first = render_arrow(
            Pos { x: -1, y: -1 },
            size,
            endpoint(Pos { x: 0, y: 0 }, Direction::Left),
            endpoint(Pos { x: 2, y: 1 }, Direction::Right),
        );
        let leaving_right_first = render_arrow(
            Pos { x: -1, y: 0 },
            size,
            endpoint(Pos { x: 2, y: 1 }, Direction::Right),
            endpoint(Pos { x: 0, y: 0 }, Direction::Left),
        );

        assert_eq!(leaving_left_first, "┌───┐\n└►  │\n   ◄┘\n");
        assert_eq!(leaving_right_first, "┌►   \n│  ◄┐\n└───┘\n");

        let bends = |text: &str| text.chars().filter(|c| "┌┐└┘".contains(*c)).count();
        assert_eq!(bends(&leaving_left_first), 4);
        assert_eq!(bends(&leaving_right_first), 4);
    }

    /// User Story 1, R-6, FR-006, SC-003, spec acceptance scenario 7: where two routes mirror
    /// each other about the line the two starting positions share — here, `b`'s own head stands
    /// between them, so neither can go straight through — the one drawn is the one on the side
    /// the arrow's own travel puts to its right. Leaving `down`, that is the smaller `x`.
    #[test]
    fn a_mirrored_route_passes_on_the_right_of_the_travel() {
        let text = render_arrow(
            Pos { x: -1, y: 0 },
            Size {
                width: 2,
                height: 4,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Down),
            endpoint(Pos { x: 0, y: 2 }, Direction::Down),
        );

        assert_eq!(text, " ▲\n┌┘\n│▲\n└┘\n");
    }

    /// FR-003, R-2, SC-001: the route is empty in exactly the two arrangements the model names —
    /// an endpoint standing on the cell the route would have to arrive at, and two endpoints at
    /// one position leaving the same direction.
    #[test]
    fn the_route_is_empty_only_where_the_model_says_it_is() {
        let an_endpoint_stands_on_the_arrival_cell = derive_path(
            Pos { x: 0, y: 0 },
            Direction::Up,
            Pos { x: 0, y: 1 },
            Direction::Up,
        );
        let coincident_leaving_the_same_direction = derive_path(
            Pos { x: 0, y: 0 },
            Direction::Right,
            Pos { x: 0, y: 0 },
            Direction::Right,
        );

        assert_eq!(an_endpoint_stands_on_the_arrival_cell, None);
        assert_eq!(coincident_leaving_the_same_direction, None);
    }

    /// FR-012, R-9, SC-010: of the sixteen arrangements whose two endpoints sit at one position —
    /// excluded from the sweep grid, so this is the only thing that covers them — the four that
    /// leave in the same direction draw no route, and the twelve that leave in different
    /// directions draw one.
    #[test]
    fn the_coincident_position_family_follows_the_rule() {
        const DIRECTIONS: [Direction; 4] = [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ];
        let at = Pos { x: 0, y: 0 };

        let mut same_direction = 0;
        let mut different_direction = 0;
        for da in DIRECTIONS {
            for db in DIRECTIONS {
                let route = derive_path(at, da, at, db);
                if da == db {
                    assert_eq!(route, None, "leaving {da:?} both ways draws no route");
                    same_direction += 1;
                } else {
                    assert!(route.is_some(), "leaving {da:?} then {db:?} draws a route");
                    different_direction += 1;
                }
            }
        }

        assert_eq!(same_direction, 4);
        assert_eq!(different_direction, 12);
    }

    /// FR-008, R-8, SC-007: the number of states the derivation searches — the lattice's nodes,
    /// times the four headings — is equal for two endpoints four, fifty and five hundred cells
    /// apart, and never exceeds `7 x 7 x 4 = 196`. Counted directly from [`Lattice`] rather than
    /// timed, and rather than counting `expand_waypoints` or `Route::draw`, which grow with the
    /// route's length by design (research.md Q2).
    #[test]
    fn the_state_count_does_not_grow_with_distance() {
        let states_apart = |cells_apart: i32| {
            let s = Pos { x: 0, y: -1 };
            let t = Pos {
                x: 0,
                y: cells_apart + 1,
            };
            let mid = RouteRectangle::spanning(s, t).middle(s);
            let lattice = Lattice::new(s, t, mid);
            lattice.x.len() * lattice.y.len() * 4
        };

        let four = states_apart(4);
        let fifty = states_apart(50);
        let five_hundred = states_apart(500);

        assert_eq!(four, fifty);
        assert_eq!(fifty, five_hundred);
        assert!(four <= 196);
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

    /// Whether `p` falls inside the sweep's window — the same inclusion rule as
    /// [`crate::Buffer::contains`], duplicated here because that method is private and this test
    /// asserts against positions the derivation returns, not against a buffer.
    fn window_contains(p: Pos) -> bool {
        let dx =
            p.x.checked_sub(SWEEP_ORIGIN.x)
                .and_then(|d| u32::try_from(d).ok());
        let dy =
            p.y.checked_sub(SWEEP_ORIGIN.y)
                .and_then(|d| u32::try_from(d).ok());
        matches!((dx, dy), (Some(dx), Some(dy)) if dx < SWEEP_SIZE.width && dy < SWEEP_SIZE.height)
    }

    /// R-10, FR-011: every position any sweep arrangement writes — both heads and every cell of
    /// the derived route — lies inside the window the sweep renders into, so a rendering showing
    /// two heads with a gap can only mean an empty route rather than a clipped one. research.md
    /// Q1 found the window already wide enough: the lattice a route may turn on never reaches more
    /// than one line outside the rectangle the two starting positions span, which this assertion
    /// checks mechanically rather than by that arithmetic alone.
    #[test]
    fn sweep_every_route_fits_inside_the_window() {
        for (a, da, b, db) in sweep_arrangements() {
            for (from_at, from_dir, to_at, to_dir) in [(a, da, b, db), (b, db, a, da)] {
                assert!(
                    window_contains(from_at) && window_contains(to_at),
                    "({from_at:?}, {from_dir:?}) -> ({to_at:?}, {to_dir:?}): a head outside the window"
                );
                if let Some(path) = derive_path(from_at, from_dir, to_at, to_dir) {
                    for pos in path {
                        assert!(
                            window_contains(pos),
                            "({from_at:?}, {from_dir:?}) -> ({to_at:?}, {to_dir:?}): route cell {pos:?} outside the window"
                        );
                    }
                }
            }
        }
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
