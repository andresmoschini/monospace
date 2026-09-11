//! The arrow: two endpoints, two heads and a derived route between them. See _The initial set_
//! and _The route of an arrow_ in [`docs/model.md`](../../../docs/model.md), and research.md Q5
//! in `specs/039-draw-shapes-instead-of-individual-cells/`.

use crate::shape::fragment::head::Head;
use crate::shape::route::Route;
use crate::{Direction, Glyph, Pos, Shape, Stroke, Surface};

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

/// Derives an arrow's route: the path between its two starting positions, per _The route of an
/// arrow_ and research.md Q5. `None` means no candidate exists and the arrow is its two heads
/// alone.
fn derive_path(a: Pos, da: Direction, b: Pos, db: Direction) -> Option<Vec<Pos>> {
    let start_a = offset(a, da)?;
    let start_b = offset(b, db)?;

    let x_min = start_a.x.min(start_b.x);
    let x_max = start_a.x.max(start_b.x);
    let y_min = start_a.y.min(start_b.y);
    let y_max = start_a.y.max(start_b.y);
    let mid_x = x_min + (x_max - x_min) / 2;
    let mid_y = y_min + (y_max - y_min) / 2;

    let mut xs = [start_a.x, start_b.x, mid_x];
    xs.sort_unstable();
    let mut ys = [start_a.y, start_b.y, mid_y];
    ys.sort_unstable();

    let mut lattice = Vec::with_capacity(9);
    for &x in &xs {
        for &y in &ys {
            let point = Pos { x, y };
            if point != a && point != b && !lattice.contains(&point) {
                lattice.push(point);
            }
        }
    }

    let start_idx = lattice.iter().position(|&p| p == start_a)?;
    let exit_dir = opposite(db);

    let mut candidates = Vec::new();
    let mut visited = vec![false; lattice.len()];
    visited[start_idx] = true;
    let mut path = vec![start_a];
    search(
        &lattice,
        &mut visited,
        start_idx,
        da,
        start_b,
        &mut path,
        &mut candidates,
    );

    let mid = Pos { x: mid_x, y: mid_y };
    candidates
        .into_iter()
        .filter(|waypoints| is_valid(waypoints, da, exit_dir))
        .map(|waypoints| {
            let bends = count_bends(&waypoints, da, exit_dir);
            let closeness: i32 = waypoints
                .iter()
                .map(|p| (p.x - mid.x).abs() + (p.y - mid.y).abs())
                .sum();
            (bends, closeness, waypoints)
        })
        .min_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then(left.1.cmp(&right.1))
                .then_with(|| compare_lexicographically(&left.2, &right.2))
        })
        .map(|(_, _, waypoints)| expand_waypoints(&waypoints))
}

/// Depth-first search over the lattice, from `current` (already in `path`) toward `start_b`,
/// respecting `current_dir` as the direction just traveled and forbidding a reversal. Every
/// completed path — reaching `start_b`, by whatever route — is recorded in `candidates`.
fn search(
    lattice: &[Pos],
    visited: &mut [bool],
    current: usize,
    current_dir: Direction,
    start_b: Pos,
    path: &mut Vec<Pos>,
    candidates: &mut Vec<Vec<Pos>>,
) {
    if lattice[current] == start_b {
        candidates.push(path.clone());
        return;
    }
    for next in 0..lattice.len() {
        if visited[next] {
            continue;
        }
        let Some(dir) = direction_between(lattice[current], lattice[next]) else {
            continue;
        };
        if dir == opposite(current_dir) {
            continue;
        }
        visited[next] = true;
        path.push(lattice[next]);
        search(lattice, visited, next, dir, start_b, path, candidates);
        path.pop();
        visited[next] = false;
    }
}

/// Whether `waypoints` (from `start_a` to `start_b`) can validly exit toward `exit_dir` without a
/// reversal at the last point.
fn is_valid(waypoints: &[Pos], da: Direction, exit_dir: Direction) -> bool {
    let last_incoming = if waypoints.len() == 1 {
        da
    } else {
        let n = waypoints.len();
        direction_between(waypoints[n - 2], waypoints[n - 1])
            .expect("consecutive waypoints share one coordinate")
    };
    last_incoming != opposite(exit_dir)
}

/// The number of positions along `waypoints` where the direction of travel actually changes,
/// counting the fixed entry direction `da` and exit direction `exit_dir` as part of the path.
fn count_bends(waypoints: &[Pos], da: Direction, exit_dir: Direction) -> u32 {
    let n = waypoints.len();
    let mut bends = 0;
    for i in 0..n {
        let incoming = if i == 0 {
            da
        } else {
            direction_between(waypoints[i - 1], waypoints[i])
                .expect("consecutive waypoints share one coordinate")
        };
        let outgoing = if i + 1 < n {
            direction_between(waypoints[i], waypoints[i + 1])
                .expect("consecutive waypoints share one coordinate")
        } else {
            exit_dir
        };
        if incoming != outgoing {
            bends += 1;
        }
    }
    bends
}

fn compare_lexicographically(left: &[Pos], right: &[Pos]) -> std::cmp::Ordering {
    left.iter()
        .map(|p| (p.x, p.y))
        .cmp(right.iter().map(|p| (p.x, p.y)))
}

#[cfg(test)]
mod tests {
    use super::{Arrow, Endpoint};
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
}
