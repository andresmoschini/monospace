//! The arrow: two endpoints, two heads and a derived route between them. See _The initial set_
//! and _The route of an arrow_ in [`docs/model.md`](../../../docs/model.md), and research.md Q5
//! in `specs/039-draw-shapes-instead-of-individual-cells/`.

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

/// The direction that walks from `from` to `to`, if the two share exactly one coordinate.
fn direction_between(from: Pos, to: Pos) -> Option<Direction> {
    if from.y == to.y && from.x != to.x {
        Some(if to.x > from.x {
            Direction::Right
        } else {
            Direction::Left
        })
    } else if from.x == to.x && from.y != to.y {
        Some(if to.y > from.y {
            Direction::Down
        } else {
            Direction::Up
        })
    } else {
        None
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

fn opposite_orientation(o: Orientation) -> Orientation {
    match o {
        Orientation::Horizontal => Orientation::Vertical,
        Orientation::Vertical => Orientation::Horizontal,
    }
}

/// The two interior waypoints of a route from `s` to `t` that turns at `mid` — a run leaves `s`
/// and a run arrives at `t`, both `primary`-oriented and fixed at `mid`'s coordinate on the other
/// axis; the run between them — the one the bends leave free — is fixed at `mid`'s coordinate on
/// `primary`'s own axis. _Run_ and _Middle_ in `docs/model.md`'s _The route of an arrow_.
fn three_waypoint(s: Pos, t: Pos, mid: Pos, primary: Orientation) -> (Pos, Pos) {
    match primary {
        Orientation::Horizontal => (Pos { x: mid.x, y: s.y }, Pos { x: mid.x, y: t.y }),
        Orientation::Vertical => (Pos { x: s.x, y: mid.y }, Pos { x: t.x, y: mid.y }),
    }
}

/// The double escape `s`-`z1`-`z2`-`z3`-`z4`-`t`: where the ordinary escape's boundary runs would
/// have no room (the route rectangle is only two cells wide on `escape`'s own axis, so `mid`
/// coincides with one of `s` or `t` on it), each end instead jogs to the far endpoint's coordinate
/// on `escape`'s axis before crossing at the middle on the other axis, then jogs back — the only
/// shape research.md's sweep found needed for these arrangements.
fn zigzag(s: Pos, t: Pos, mid: Pos, escape: Orientation) -> Vec<Pos> {
    match escape {
        Orientation::Horizontal => vec![
            s,
            Pos { x: t.x, y: s.y },
            Pos { x: t.x, y: mid.y },
            Pos { x: s.x, y: mid.y },
            Pos { x: s.x, y: t.y },
            t,
        ],
        Orientation::Vertical => vec![
            s,
            Pos { x: s.x, y: t.y },
            Pos { x: mid.x, y: t.y },
            Pos { x: mid.x, y: s.y },
            Pos { x: t.x, y: s.y },
            t,
        ],
    }
}

/// `waypoints` with each repeat of the one before it dropped. A constructed shape names a turn
/// that can coincide with the point before it — the middle of a span two cells wide is one of
/// the two ends of that span — and such a shape is the same path as the one without the repeat,
/// not an invalid one.
fn without_repeats(waypoints: Vec<Pos>) -> Vec<Pos> {
    let mut kept: Vec<Pos> = Vec::with_capacity(waypoints.len());
    for point in waypoints {
        if kept.last() != Some(&point) {
            kept.push(point);
        }
    }
    kept
}

/// The runs of the path `a` → `waypoints` → `b`: one entry per maximal stretch traveled in one
/// direction, as the orientation it lies along and the coordinate it is fixed at — its row if
/// horizontal, its column if vertical.
fn path_runs(a: Pos, waypoints: &[Pos], b: Pos) -> Vec<(Orientation, i32)> {
    let mut points = Vec::with_capacity(waypoints.len() + 2);
    points.push(a);
    points.extend_from_slice(waypoints);
    points.push(b);

    let mut runs: Vec<(Orientation, i32)> = Vec::new();
    let mut traveling: Option<Direction> = None;
    for pair in points.windows(2) {
        let Some(direction) = direction_between(pair[0], pair[1]) else {
            continue;
        };
        if traveling == Some(direction) {
            continue;
        }
        traveling = Some(direction);
        runs.push(match direction_orientation(direction) {
            Orientation::Horizontal => (Orientation::Horizontal, pair[0].y),
            Orientation::Vertical => (Orientation::Vertical, pair[0].x),
        });
    }
    runs
}

/// How far the runs the bends leave free sit from the middle of the route rectangle — the
/// model's tie-break among candidates that share the fewest bends, summed over the runs it
/// speaks of. The first run is pinned by `a` and the last by `b`, so neither is free and neither
/// counts.
fn distance_from_middle(a: Pos, waypoints: &[Pos], b: Pos, mid: Pos) -> u32 {
    let runs = path_runs(a, waypoints, b);
    let interior = runs.len().saturating_sub(2);
    runs.iter()
        .skip(1)
        .take(interior)
        .map(|&(orientation, coordinate)| match orientation {
            Orientation::Horizontal => coordinate.abs_diff(mid.y),
            Orientation::Vertical => coordinate.abs_diff(mid.x),
        })
        .sum()
}

/// The bend count of a route through `waypoints`, from `s` (its first element) to `t` (its last),
/// leaving `s` toward `da` and arriving at `t` against `exit_dir` — or `None` where any run,
/// including the one leaving `s` or the one arriving at `t`, would have to reverse. A run whose
/// direction matches the boundary direction its end shares an axis with costs no bend there; any
/// other direction — necessarily perpendicular, since a reversal is already ruled out — costs one.
fn path_bends(waypoints: &[Pos], da: Direction, exit_dir: Direction) -> Option<u32> {
    let mut bends = 0;
    let mut incoming = da;
    for pair in waypoints.windows(2) {
        let outgoing = direction_between(pair[0], pair[1])?;
        if outgoing == opposite(incoming) {
            return None;
        }
        bends += u32::from(outgoing != incoming);
        incoming = outgoing;
    }
    if exit_dir == opposite(incoming) {
        return None;
    }
    Some(bends + u32::from(exit_dir != incoming))
}

/// Derives an arrow's route: the path between its two starting positions, per _The route of an
/// arrow_. `None` means no candidate exists and the arrow is its two heads alone.
///
/// Built directly from runs and fixed coordinates rather than searched for and scored
/// (research.md Q2): `s` and `t` pin the first and last run. Where they align or coincide the
/// route is the single run or point between them. Otherwise every route the two directions admit
/// is one of a handful of shapes — a corner at each of the two points a pinned run from `s` could
/// meet a pinned run into `t`; a run at the middle of the route rectangle on one axis with a
/// pinned run at each end, on the axis `da` moves along, on the axis `exit_dir` moves along, or
/// (where `da` and `exit_dir` share an axis, so neither of the last two exists) on the other axis;
/// or, where that axis has no room for a pinned run either, the double escape ([`zigzag`]). The
/// fewest-bend shape wins; where two tie, the one whose free runs sit nearest the middle does,
/// per the model's tie-break.
fn derive_path(a: Pos, da: Direction, b: Pos, db: Direction) -> Option<Vec<Pos>> {
    // `s` and `t` — each endpoint's starting position: one step from it in that endpoint's own
    // leaving direction.
    let s = offset(a, da)?;
    let t = offset(b, db)?;
    let exit_dir = opposite(db);

    if s == t {
        return (da != opposite(exit_dir)).then(|| expand_waypoints(&[s]));
    }
    if let Some(dir) = direction_between(s, t) {
        return (dir != opposite(da) && dir != opposite(exit_dir))
            .then(|| expand_waypoints(&[s, t]));
    }

    let rectangle = RouteRectangle::spanning(s, t);
    let mid = rectangle.middle(s);
    let da_orientation = direction_orientation(da);
    let exit_orientation = direction_orientation(exit_dir);

    let mut shapes: Vec<Vec<Pos>> = vec![
        vec![s, Pos { x: t.x, y: s.y }, t],
        vec![s, Pos { x: s.x, y: t.y }, t],
    ];
    let via_axis = |axis| {
        let (w1, w2) = three_waypoint(s, t, mid, axis);
        vec![s, w1, w2, t]
    };
    shapes.push(via_axis(da_orientation));
    if exit_orientation == da_orientation {
        let escape = opposite_orientation(da_orientation);
        shapes.push(via_axis(escape));
        shapes.push(zigzag(s, t, mid, escape));
    } else {
        shapes.push(via_axis(exit_orientation));
    }

    shapes
        .into_iter()
        .map(without_repeats)
        .filter_map(|waypoints| {
            let bends = path_bends(&waypoints, da, exit_dir)?;
            let expanded = expand_waypoints(&waypoints);
            // No route cell is an endpoint position: a run that would cross the *other*
            // endpoint — the one `s`/`t` were not offset from — is not a route the rectangle
            // permits, per data-model.md's invariant 2.
            if expanded.contains(&a) || expanded.contains(&b) {
                return None;
            }
            let from_middle = distance_from_middle(a, &waypoints, b, mid);
            Some((bends, from_middle, expanded))
        })
        .min_by_key(|&(bends, from_middle, _)| (bends, from_middle))
        .map(|(_, _, expanded)| expanded)
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
    /// endpoints are in line on that axis: the route rectangle is one cell thick, no alternating
    /// path fits inside it, and the route is empty — the arrow is its two heads, per _The route
    /// of an arrow_.
    #[test]
    fn identical_directions_in_line_gives_an_empty_route() {
        let text = render_arrow(
            Pos { x: 0, y: 0 },
            Size {
                width: 5,
                height: 1,
            },
            endpoint(Pos { x: 0, y: 0 }, Direction::Right),
            endpoint(Pos { x: 4, y: 0 }, Direction::Right),
        );

        assert_eq!(text, "◄   ◄\n");
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
