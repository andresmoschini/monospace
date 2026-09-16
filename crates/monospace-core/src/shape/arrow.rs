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

    /// One value per axis: the cell halfway along that axis's span. Today this rounds toward the
    /// smaller coordinate on both axes; rounding the free one toward `s` instead is ADR-0044's
    /// tie-break, added in a later step.
    fn middle(&self) -> Pos {
        Pos {
            x: self.x_min + (self.x_max - self.x_min) / 2,
            y: self.y_min + (self.y_max - self.y_min) / 2,
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

/// The bend count of the two-run corner `s`-`corner`-`t`, or `None` where either run would have
/// to reverse out of `da` or into `exit_dir`. A run whose direction matches the boundary direction
/// its end shares an axis with costs no bend there; any other direction — necessarily
/// perpendicular, since a reversal is already ruled out — costs one.
fn corner_bends(s: Pos, corner: Pos, t: Pos, da: Direction, exit_dir: Direction) -> Option<u32> {
    let entry = direction_between(s, corner)?;
    let exit = direction_between(corner, t)?;
    if entry == opposite(da) || exit == opposite(exit_dir) {
        return None;
    }
    Some(u32::from(entry != da) + 1 + u32::from(exit != exit_dir))
}

/// The bend count of the three-waypoint route `s`-`w1`-`w2`-`t`, under the same rule as
/// [`corner_bends`] applied at both ends.
fn three_waypoint_bends(
    s: Pos,
    (w1, w2): (Pos, Pos),
    t: Pos,
    da: Direction,
    exit_dir: Direction,
) -> Option<u32> {
    let entry = direction_between(s, w1)?;
    let exit = direction_between(w2, t)?;
    if entry == opposite(da) || exit == opposite(exit_dir) {
        return None;
    }
    Some(u32::from(entry != da) + 2 + u32::from(exit != exit_dir))
}

/// Derives an arrow's route: the path between its two starting positions, per _The route of an
/// arrow_. `None` means no candidate exists and the arrow is its two heads alone.
///
/// Built directly from runs and fixed coordinates rather than searched for and scored
/// (research.md Q2): `s` and `t` pin the first and last run. Where they align or coincide the
/// route is the single run or point between them. Otherwise every route the two directions admit
/// is one of five shapes — a corner at each of the two points a pinned run from `s` could meet a
/// pinned run into `t`, or a run at the middle of the route rectangle on one axis with a pinned
/// run at each end, on the axis `da` moves along, on the axis `exit_dir` moves along, or (where
/// `da` and `exit_dir` share an axis, so neither of the last two exists) on the other axis. The
/// fewest-bend shape wins; where two tie, the one with a run at the middle does, per the model's
/// tie-break.
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
    let mid = rectangle.middle();
    let da_orientation = direction_orientation(da);
    let exit_orientation = direction_orientation(exit_dir);

    let mut candidates: Vec<(u32, bool, [Pos; 2])> = Vec::new();
    for corner in [Pos { x: t.x, y: s.y }, Pos { x: s.x, y: t.y }] {
        if let Some(bends) = corner_bends(s, corner, t, da, exit_dir) {
            candidates.push((bends, false, [corner, corner]));
        }
    }
    let mut try_axis = |axis| {
        let (w1, w2) = three_waypoint(s, t, mid, axis);
        if let Some(bends) = three_waypoint_bends(s, (w1, w2), t, da, exit_dir) {
            candidates.push((bends, true, [w1, w2]));
        }
    };
    try_axis(da_orientation);
    if exit_orientation == da_orientation {
        try_axis(opposite_orientation(da_orientation));
    } else {
        try_axis(exit_orientation);
    }

    let (_, is_via_mid, waypoints) = candidates
        .into_iter()
        .min_by_key(|&(bends, is_via_mid, _)| (bends, !is_via_mid))?;
    let path = if is_via_mid {
        vec![s, waypoints[0], waypoints[1], t]
    } else {
        vec![s, waypoints[0], t]
    };
    Some(expand_waypoints(&path))
}

#[cfg(test)]
mod tests {
    use super::{Arrow, Endpoint};
    use crate::shape::counting::CountingSurface;
    use crate::{
        Buffer, Direction, Glyph, GlyphCatalog, Layer, Pos, Shape, Size, StampMode, Stroke, render,
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
}
