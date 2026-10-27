//! Polygon operations — port of `hyp2mat/lib/polygon.cc`.
//!
//! The original uses Clipper 1 (`clipper.cpp`) on a 64-bit integer grid. This
//! module keeps the observable semantics: coordinates are snapped to the grid
//! (`floor(x / grid + 0.5)`), boolean operations are performed with the
//! non-zero fill rule on both subject and clip, and results are converted back
//! to floating point multiples of the grid.
//!
//! `Polygon` is stateful like its C++ counterpart: edges and holes are
//! accumulated with `add_edge`/`add_hole`, mutated by `offset`, `simplify` and
//! the boolean operations, and finally resolved into a `FloatPolygons` list by
//! `result`. `PolygonOps` offers the same operations on plain polygon sets.

use crate::model::{FloatPoint, FloatPoly, FloatPolygon, FloatPolygons};
use clipper2::{Clipper, FillRule, One, Paths};

/// Default grid size (10 micron), matching the C++ default.
pub const DEFAULT_GRID: f64 = 1e-5;

/// Snap a coordinate to the grid (native `AlignToGrid`).
pub fn align_to_grid(x: f64, grid: f64) -> f64 {
    (x / grid + 0.5).floor() * grid
}

/// Signed area of a polygon. Positive means clockwise in the native
/// convention (see `IsClockwise`).
pub fn signed_area(poly: &FloatPolygon) -> f64 {
    let n = poly.points.len();
    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += poly.points[i].x * poly.points[j].y - poly.points[j].x * poly.points[i].y;
    }
    area
}

/// Native `IsClockwise`: positive area is clockwise, area zero counts as
/// clockwise.
pub fn is_clockwise(poly: &FloatPolygon) -> bool {
    signed_area(poly) >= 0.0
}

/// Reverse the order of vertices (native `Reverse`).
pub fn reverse(poly: &mut FloatPolygon) {
    poly.points.reverse();
}

/// Convert a float polygon to grid-unit points, removing repeated vertices.
fn to_grid_path(poly: &FloatPolygon, grid: f64) -> Vec<(f64, f64)> {
    let mut path: Vec<(f64, f64)> = poly
        .points
        .iter()
        .map(|p| ((p.x / grid + 0.5).floor(), (p.y / grid + 0.5).floor()))
        .collect();
    path.dedup();
    while path.len() > 1 && path.first() == path.last() {
        path.pop();
    }
    path
}

fn from_grid_path(path: &[(f64, f64)], grid: f64) -> FloatPolygon {
    FloatPolygon {
        points: path
            .iter()
            .map(|&(x, y)| FloatPoint {
                x: x * grid,
                y: y * grid,
            })
            .collect(),
    }
}

/// Test if a point is inside a polygon (ray casting).
pub fn point_in_polygon(p: &FloatPoint, poly: &FloatPolygon) -> bool {
    let n = poly.points.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        if (poly.points[i].y > p.y) != (poly.points[j].y > p.y) {
            let x_intersect = (poly.points[j].x - poly.points[i].x) * (p.y - poly.points[i].y)
                / (poly.points[j].y - poly.points[i].y)
                + poly.points[i].x;
            if p.x < x_intersect {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

/// Remove vertices that are exactly collinear with their neighbours, matching
/// the native Clipper 1 input/output cleaning (collinear points are dropped
/// unless they are required to keep a valid polygon).
fn remove_collinear(path: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if path.len() < 3 {
        return path.to_vec();
    }
    let mut result: Vec<(f64, f64)> = path.to_vec();
    let mut changed = true;
    while changed && result.len() > 3 {
        changed = false;
        let n = result.len();
        for i in 0..n {
            let prev = result[(i + n - 1) % n];
            let current = result[i];
            let next = result[(i + 1) % n];
            let cross = (current.0 - prev.0) * (next.1 - current.1)
                - (current.1 - prev.1) * (next.0 - current.0);
            if cross == 0.0 {
                // Only remove points that lie between their neighbours.
                let between = (current.0 - prev.0) * (next.0 - current.0)
                    + (current.1 - prev.1) * (next.1 - current.1)
                    >= 0.0;
                if between {
                    result.remove(i);
                    changed = true;
                    break;
                }
            }
        }
    }
    result
}

fn paths_to_polygons(paths: Paths<One>, grid: f64) -> FloatPolygons {
    let raw: Vec<Vec<(f64, f64)>> = paths.into();
    let mut polys: Vec<FloatPoly> = raw
        .into_iter()
        .map(|path| remove_collinear(&path))
        .filter(|path| path.len() >= 3)
        .map(|path| {
            let poly = from_grid_path(&path, grid);
            FloatPoly {
                is_hole: !is_clockwise(&poly),
                nesting_level: 0,
                poly,
            }
        })
        .collect();

    // Nesting level = number of polygons that contain this one. Boolean
    // results never overlap, so the first vertex is representative.
    for i in 0..polys.len() {
        let mut level = 0;
        for j in 0..polys.len() {
            if i == j || polys[j].poly.points.is_empty() || polys[i].poly.points.is_empty() {
                continue;
            }
            if point_in_polygon(&polys[i].poly.points[0], &polys[j].poly) {
                level += 1;
            }
        }
        polys[i].nesting_level = level;
    }

    // Native `_AddToPolygonList` makes outer edges clockwise and holes
    // counter-clockwise, whatever the clipper output orientation was.
    for poly in polys.iter_mut() {
        if poly.is_hole == is_clockwise(&poly.poly) {
            reverse(&mut poly.poly);
        }
    }

    FloatPolygons { polys }
}

/// Round like Clipper 1 (`Round(double)`): half away from zero, truncating.
fn round_half_away(value: f64) -> i64 {
    if value < 0.0 {
        (value - 0.5) as i64
    } else {
        (value + 0.5) as i64
    }
}

/// Clipper 1 `GetUnitNormal`.
fn unit_normal(from: (i64, i64), to: (i64, i64)) -> (f64, f64) {
    if from == to {
        return (0.0, 0.0);
    }
    let dx = (to.0 - from.0) as f64;
    let dy = (to.1 - from.1) as f64;
    let f = 1.0 / (dx * dx + dy * dy).sqrt();
    (dy * f, -dx * f)
}

/// Clipper 1 `BuildArc`.
fn build_arc(center: (i64, i64), a1: f64, a2: f64, r: f64, limit: f64) -> Vec<(i64, i64)> {
    let pi = std::f64::consts::PI;
    let arc_frac = (a2 - a1).abs() / (2.0 * pi);
    let denominator = (1.0 - limit / r.abs()).clamp(-1.0, 1.0).acos();
    let mut steps = if denominator > 0.0 {
        (arc_frac * pi / denominator) as i32
    } else {
        2
    };
    if steps < 2 {
        steps = 2;
    } else if steps > (222.0 * arc_frac) as i32 {
        steps = (222.0 * arc_frac) as i32;
        if steps < 2 {
            steps = 2;
        }
    }

    let mut x = a1.cos();
    let mut y = a1.sin();
    let c = ((a2 - a1) / steps as f64).cos();
    let s = ((a2 - a1) / steps as f64).sin();
    let mut result = Vec::with_capacity(steps as usize + 1);
    for _ in 0..=steps {
        result.push((
            center.0 + round_half_away(x * r),
            center.1 + round_half_away(y * r),
        ));
        let x2 = x;
        x = x * c - s * y;
        y = x2 * s + y * c;
    }
    result
}

fn offset_point(
    out: &mut Vec<(i64, i64)>,
    point: (i64, i64),
    normal_current: (f64, f64),
    normal_previous: (f64, f64),
    delta: f64,
    limit: f64,
    round_joins: bool,
) {
    let pi = std::f64::consts::PI;
    // `n_j` is the normal of the edge starting at this vertex, `n_k` the
    // normal of the edge ending here (native `normals[m_j]`/`normals[m_k]`).
    let n_j = normal_current;
    let n_k = normal_previous;
    let pt1 = (
        round_half_away(point.0 as f64 + n_k.0 * delta),
        round_half_away(point.1 as f64 + n_k.1 * delta),
    );
    let pt2 = (
        round_half_away(point.0 as f64 + n_j.0 * delta),
        round_half_away(point.1 as f64 + n_j.1 * delta),
    );
    let cross = (n_k.0 * n_j.1 - n_j.0 * n_k.1) * delta;

    if round_joins {
        out.push(pt1);
        if cross >= 0.0 {
            if n_j.0 * n_k.0 + n_j.1 * n_k.1 < 0.985 {
                let a1 = n_k.1.atan2(n_k.0);
                let mut a2 = n_j.1.atan2(n_j.0);
                if delta > 0.0 && a2 < a1 {
                    a2 += 2.0 * pi;
                } else if delta < 0.0 && a2 > a1 {
                    a2 -= 2.0 * pi;
                }
                let arc = build_arc(point, a1, a2, delta, limit);
                out.extend(arc);
            }
        } else {
            out.push(point);
        }
        out.push(pt2);
        return;
    }

    if cross >= 0.0 {
        let mut a1 = n_k.1.atan2(n_k.0);
        let a2 = (-n_j.1).atan2(-n_j.0);
        a1 = (a2 - a1).abs();
        if a1 > pi {
            a1 = 2.0 * pi - a1;
        }
        let dx = ((pi - a1) / 4.0).tan() * delta.abs();
        out.push((
            (pt1.0 as f64 - n_k.1 * dx) as i64,
            (pt1.1 as f64 + n_k.0 * dx) as i64,
        ));
        out.push((
            (pt2.0 as f64 + n_j.1 * dx) as i64,
            (pt2.1 as f64 - n_j.0 * dx) as i64,
        ));
    } else {
        out.push(pt1);
        out.push(point);
        out.push(pt2);
    }
}
/// Offset all closed subject paths, returning grid-unit paths.
///
/// `limit` is the native round-join tolerance in grid units (unused for square
/// joins); `round_joins` selects `jtRound` instead of `jtSquare`.
fn offset_closed_paths(
    subject: &[Vec<(f64, f64)>],
    delta: f64,
    limit: f64,
    round_joins: bool,
) -> Vec<Vec<(f64, f64)>> {
    if delta == 0.0 {
        return subject.to_vec();
    }
    let mut result: Vec<Vec<(f64, f64)>> = Vec::new();
    for path in subject {
        let integer: Vec<(i64, i64)> = path.iter().map(|&(x, y)| (x as i64, y as i64)).collect();
        if integer.len() == 1 {
            let arc = build_arc(integer[0], 0.0, 2.0 * std::f64::consts::PI, delta, limit);
            result.push(arc.into_iter().map(|(x, y)| (x as f64, y as f64)).collect());
            continue;
        }
        let out = offset_path_native(&integer, delta, limit, round_joins);
        if !out.is_empty() {
            result.push(out.into_iter().map(|(x, y)| (x as f64, y as f64)).collect());
        }
    }
    result
}

/// The closed-polygon branch of Clipper 1's `OffsetBuilder`.
fn offset_path_native(
    subject: &[(i64, i64)],
    delta: f64,
    limit: f64,
    round_joins: bool,
) -> Vec<(i64, i64)> {
    let mut len = subject.len();
    if len == 0 || (len < 3 && delta <= 0.0) {
        return Vec::new();
    }
    let force_close = subject[0] == subject[len - 1];
    if force_close {
        len -= 1;
    }
    if len < 2 {
        return Vec::new();
    }

    let mut normals = vec![(0.0, 0.0); len];
    for j in 0..len - 1 {
        normals[j] = unit_normal(subject[j], subject[j + 1]);
    }
    normals[len - 1] = unit_normal(subject[len - 1], subject[0]);

    let mut out: Vec<(i64, i64)> = Vec::with_capacity(len * 2);
    let mut k = len - 1;
    for j in 0..len {
        offset_point(
            &mut out,
            subject[j],
            normals[j],
            normals[k],
            delta,
            limit,
            round_joins,
        );
        k = j;
    }
    out
}

/// Union a set of grid-unit paths with the non-zero fill rule.
fn union_paths(paths: &[Vec<(f64, f64)>]) -> Vec<Vec<(f64, f64)>> {
    if paths.is_empty() {
        return Vec::new();
    }
    let subject: Paths<One> = paths.to_vec().into();
    let clipper = Clipper::<clipper2::NoSubjects, One>::new()
        .add_subject(subject)
        .add_clip(Vec::<Vec<(f64, f64)>>::new());
    clipper
        .union(FillRule::NonZero)
        .map(|paths| paths.into())
        .unwrap_or_default()
}

/// Difference of two path sets with the non-zero fill rule.
fn difference_paths(subject: &[Vec<(f64, f64)>], clip: &[Vec<(f64, f64)>]) -> Vec<Vec<(f64, f64)>> {
    let subject_paths: Paths<One> = subject.to_vec().into();
    let clip_paths: Paths<One> = clip.to_vec().into();
    let clipper = Clipper::<clipper2::NoSubjects, One>::new()
        .add_subject(subject_paths)
        .add_clip(clip_paths);
    clipper
        .difference(FillRule::NonZero)
        .map(|paths| paths.into())
        .unwrap_or_default()
}

/// A rectangular frame around all subject paths, expanded by `margin` grid
/// units on every side (native uses a frame 10 units outside the bounds).
fn frame_path(subject: &[Vec<(f64, f64)>], margin: f64) -> Vec<(f64, f64)> {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for path in subject {
        for &(x, y) in path {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }
    if !min_x.is_finite() {
        min_x = 0.0;
        max_x = 1.0;
        min_y = 0.0;
        max_y = 1.0;
    }
    vec![
        (min_x - margin, max_y + margin),
        (max_x + margin, max_y + margin),
        (max_x + margin, min_y - margin),
        (min_x - margin, min_y - margin),
    ]
}

/// Stateful polygon accumulator with native semantics.
#[derive(Debug, Clone)]
pub struct Polygon {
    subject: Vec<Vec<(f64, f64)>>,
    grid: f64,
    arc_precision: f64,
}

impl Default for Polygon {
    fn default() -> Self {
        Polygon::new(DEFAULT_GRID, 0.0)
    }
}

impl Polygon {
    pub fn new(grid: f64, arc_precision: f64) -> Self {
        Polygon {
            subject: Vec::new(),
            grid,
            arc_precision,
        }
    }

    pub fn grid(&self) -> f64 {
        self.grid
    }

    pub fn set_grid(&mut self, grid: f64) {
        self.grid = grid;
    }

    pub fn set_arc_precision(&mut self, arc_precision: f64) {
        self.arc_precision = arc_precision;
    }

    /// Number of stored edges.
    pub fn edge_count(&self) -> usize {
        self.subject.len()
    }

    /// Native `IsEmpty`.
    pub fn is_empty(&self) -> bool {
        self.subject.iter().all(|path| path.len() < 3)
    }

    fn add_edge_with_orientation(&mut self, edge: &FloatPolygon, positive: bool) {
        if edge.points.is_empty() {
            return;
        }
        let mut path = to_grid_path(edge, self.grid);
        if path.len() < 2 {
            return;
        }
        // Native: `if (!Orientation(poly) == orientation) ReversePolygon(poly)`.
        // Outer edges (positive) become clockwise (positive area) and holes
        // become counter-clockwise.
        let area: f64 = {
            let n = path.len();
            let mut area = 0.0;
            for i in 0..n {
                let j = (i + 1) % n;
                area += path[i].0 * path[j].1 - path[j].0 * path[i].1;
            }
            area
        };
        let clockwise = area >= 0.0;
        if clockwise != positive {
            path.reverse();
        }
        self.subject.push(path);
    }

    /// Native `AddEdge`.
    pub fn add_edge(&mut self, edge: &FloatPolygon) {
        self.add_edge_with_orientation(edge, true);
    }

    /// Native `AddHole`.
    pub fn add_hole(&mut self, hole: &FloatPolygon) {
        self.add_edge_with_orientation(hole, false);
    }

    /// Native `Offset`. Positive expands, negative shrinks.
    ///
    /// The offset paths are built with the native Clipper 1 join rules
    /// (`jtSquare` when the arc precision is zero, otherwise `jtRound`).
    /// Shrinking uses the native complement/erosion construction: the subject
    /// is complemented inside a frame, the complement is expanded, and the
    /// result is subtracted from the frame again.
    pub fn offset(&mut self, delta: f64) {
        if delta == 0.0 {
            return;
        }
        let d = delta / self.grid;
        let round_joins = self.arc_precision > 0.0;
        let mut limit = if round_joins {
            self.arc_precision / self.grid
        } else {
            0.0
        };
        if limit <= 0.0 {
            limit = 0.25;
        } else if limit > d.abs() {
            limit = d.abs();
        }

        let result = if d > 0.0 {
            let built = offset_closed_paths(&self.subject, d, limit, round_joins);
            union_paths(&built)
        } else {
            let frame = frame_path(&self.subject, d.abs() + 10.0);
            let complement = difference_paths(std::slice::from_ref(&frame), &self.subject);
            let dilated = union_paths(&offset_closed_paths(
                &complement,
                d.abs(),
                limit,
                round_joins,
            ));
            difference_paths(&[frame], &dilated)
        };
        self.subject = result;
    }

    /// Native `Simplify` (remove self-intersections, non-zero fill rule).
    pub fn simplify(&mut self) {
        let paths: Paths<One> = self.subject.clone().into();
        let result = clipper2::simplify(paths, 0.0, false);
        self.subject = result.into();
    }

    fn execute(&mut self, clip: &Polygon, op: Op) {
        let subject: Paths<One> = self.subject.clone().into();
        let clip_paths: Paths<One> = clip.subject.clone().into();
        let clipper = Clipper::<clipper2::NoSubjects, One>::new()
            .add_subject(subject)
            .add_clip(clip_paths);
        let result = match op {
            Op::Intersection => clipper.intersect(FillRule::NonZero),
            Op::Union => clipper.union(FillRule::NonZero),
            Op::Difference => clipper.difference(FillRule::NonZero),
            Op::Xor => clipper.xor(FillRule::NonZero),
        };
        match result {
            Ok(paths) => self.subject = paths.into(),
            Err(_) => self.subject.clear(),
        }
    }

    /// Native `Intersection`.
    pub fn intersection(&mut self, clip: &Polygon) {
        self.execute(clip, Op::Intersection);
    }

    /// Native `Union`.
    pub fn union(&mut self, clip: &Polygon) {
        self.execute(clip, Op::Union);
    }

    /// Native `Difference`.
    pub fn difference(&mut self, clip: &Polygon) {
        self.execute(clip, Op::Difference);
    }

    /// Native `Xor`.
    pub fn xor(&mut self, clip: &Polygon) {
        self.execute(clip, Op::Xor);
    }

    /// Native `Result`: union the accumulated edges, then convert to a polygon
    /// tree flattened in depth-first order (outer, holes, nested islands).
    pub fn result(&self) -> FloatPolygons {
        let subject: Paths<One> = self.subject.clone().into();
        let empty: Paths<One> = Vec::<Vec<(f64, f64)>>::new().into();
        let clipper = Clipper::<clipper2::NoSubjects, One>::new()
            .add_subject(subject)
            .add_clip(empty);
        let result = clipper.union(FillRule::NonZero);
        match result {
            Ok(paths) => paths_to_polygons(paths, self.grid),
            Err(_) => FloatPolygons { polys: Vec::new() },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Intersection,
    Union,
    Difference,
    Xor,
}

/// Functional polygon operations on whole polygon sets.
///
/// This is the original Rust API, kept for callers that do not need the
/// stateful accumulator.
#[derive(Debug, Clone, Copy)]
pub struct PolygonOps {
    grid: f64,
    arc_precision: f64,
}

impl Default for PolygonOps {
    fn default() -> Self {
        PolygonOps {
            grid: DEFAULT_GRID,
            arc_precision: 0.0,
        }
    }
}

impl PolygonOps {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_grid(&mut self, grid: f64) {
        self.grid = grid;
    }

    pub fn set_arc_precision(&mut self, arc_precision: f64) {
        self.arc_precision = arc_precision;
    }

    pub fn grid(&self) -> f64 {
        self.grid
    }

    pub fn align_to_grid(&self, x: f64) -> f64 {
        align_to_grid(x, self.grid)
    }

    /// Build a `Polygon` with this context's settings.
    pub fn polygon(&self) -> Polygon {
        Polygon::new(self.grid, self.arc_precision)
    }

    /// Convert a polygon set into a fresh accumulator, adding holes as holes.
    pub fn from_polygons(&self, polys: &FloatPolygons) -> Polygon {
        let mut polygon = self.polygon();
        for poly in &polys.polys {
            if poly.is_hole {
                polygon.add_hole(&poly.poly);
            } else {
                polygon.add_edge(&poly.poly);
            }
        }
        polygon
    }

    pub fn is_empty(&self, polys: &FloatPolygons) -> bool {
        polys.polys.is_empty()
    }

    /// Add an outer edge to a polygon set (native `Polygon::AddEdge`).
    pub fn add_edge(&self, polys: &mut FloatPolygons, edge: FloatPolygon) {
        let mut polygon = self.from_polygons(polys);
        polygon.add_edge(&edge);
        *polys = polygon.result();
    }

    /// Add a hole to a polygon set (native `Polygon::AddHole`).
    pub fn add_hole(&self, polys: &mut FloatPolygons, hole: FloatPolygon) {
        let mut polygon = self.from_polygons(polys);
        polygon.add_hole(&hole);
        *polys = polygon.result();
    }

    /// Expand/shrink polygons by a distance.
    pub fn offset(&self, polys: &FloatPolygons, delta: f64) -> FloatPolygons {
        if delta == 0.0 {
            return polys.clone();
        }
        let mut polygon = self.from_polygons(polys);
        polygon.offset(delta);
        polygon.result()
    }

    /// Remove self-intersections.
    pub fn simplify(&self, polys: &FloatPolygons) -> FloatPolygons {
        let mut polygon = self.from_polygons(polys);
        polygon.simplify();
        polygon.result()
    }

    fn boolean(&self, subject: &FloatPolygons, clip: &FloatPolygons, op: Op) -> FloatPolygons {
        let mut polygon = self.from_polygons(subject);
        let clip_polygon = self.from_polygons(clip);
        match op {
            Op::Intersection => polygon.intersection(&clip_polygon),
            Op::Union => polygon.union(&clip_polygon),
            Op::Difference => polygon.difference(&clip_polygon),
            Op::Xor => polygon.xor(&clip_polygon),
        }
        polygon.result()
    }

    pub fn intersection(&self, subject: &FloatPolygons, clip: &FloatPolygons) -> FloatPolygons {
        self.boolean(subject, clip, Op::Intersection)
    }

    pub fn union(&self, subject: &FloatPolygons, clip: &FloatPolygons) -> FloatPolygons {
        self.boolean(subject, clip, Op::Union)
    }

    pub fn difference(&self, subject: &FloatPolygons, clip: &FloatPolygons) -> FloatPolygons {
        self.boolean(subject, clip, Op::Difference)
    }

    pub fn xor(&self, subject: &FloatPolygons, clip: &FloatPolygons) -> FloatPolygons {
        self.boolean(subject, clip, Op::Xor)
    }

    /// Resolve polygon nesting: ensure outer edges are clockwise, holes
    /// counter-clockwise, and compute nesting levels.
    pub fn resolve(&self, polys: &FloatPolygons) -> FloatPolygons {
        let mut result = self.from_polygons(polys).result();
        for i in 0..result.polys.len() {
            let mut level = 0;
            for j in 0..result.polys.len() {
                if i != j
                    && !result.polys[j].poly.points.is_empty()
                    && point_in_polygon(&result.polys[i].poly.points[0], &result.polys[j].poly)
                {
                    level += 1;
                }
            }
            result.polys[i].nesting_level = level;
        }
        result
    }
}

/// Total area of a polygon set (outer polygons minus holes).
pub fn net_area(polys: &FloatPolygons) -> f64 {
    polys
        .polys
        .iter()
        .map(|p| signed_area(&p.poly).abs() / 2.0 * if p.is_hole { -1.0 } else { 1.0 })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(x: f64, y: f64, w: f64) -> FloatPolygon {
        FloatPolygon {
            points: vec![
                FloatPoint { x, y },
                FloatPoint { x: x + w, y },
                FloatPoint { x: x + w, y: y + w },
                FloatPoint { x, y: y + w },
            ],
        }
    }

    fn polys(poly: FloatPolygon) -> FloatPolygons {
        FloatPolygons {
            polys: vec![FloatPoly {
                poly,
                is_hole: false,
                nesting_level: 0,
            }],
        }
    }

    fn area_of(poly: &FloatPolygon) -> f64 {
        signed_area(poly).abs() / 2.0
    }

    #[test]
    fn test_align_to_grid() {
        let ops = PolygonOps::new();
        assert_eq!(ops.align_to_grid(0.000012), 0.00001);
        assert_eq!(ops.align_to_grid(0.000014), 0.00001);
        assert_eq!(ops.align_to_grid(0.000016), 0.00002);
    }

    #[test]
    fn test_is_clockwise() {
        let cw = square(0.0, 0.0, 1.0);
        assert!(is_clockwise(&cw));
        let mut ccw = square(0.0, 0.0, 1.0);
        ccw.points.reverse();
        assert!(!is_clockwise(&ccw));
    }

    #[test]
    fn test_point_in_polygon() {
        let sq = square(0.0, 0.0, 10.0);
        assert!(point_in_polygon(&FloatPoint { x: 5.0, y: 5.0 }, &sq));
        assert!(!point_in_polygon(&FloatPoint { x: 15.0, y: 5.0 }, &sq));
        assert!(!point_in_polygon(&FloatPoint { x: 5.0, y: 15.0 }, &sq));
    }

    #[test]
    fn union_area_is_sum_minus_overlap() {
        let ops = PolygonOps::new();
        let a = polys(square(0.0, 0.0, 1.0));
        let b = polys(square(0.5, 0.0, 1.0));
        let result = ops.union(&a, &b);
        assert_eq!(result.polys.len(), 1);
        assert!((net_area(&result) - 1.5).abs() < 1e-12);
    }

    #[test]
    fn difference_creates_a_hole() {
        let ops = PolygonOps::new();
        let a = polys(square(0.0, 0.0, 1.0));
        let b = polys(square(0.25, 0.25, 0.5));
        let result = ops.difference(&a, &b);
        assert_eq!(result.polys.len(), 2);
        let holes: Vec<&FloatPoly> = result.polys.iter().filter(|p| p.is_hole).collect();
        assert_eq!(holes.len(), 1);
        assert_eq!(holes[0].nesting_level, 1);
        assert!((net_area(&result) - 0.75).abs() < 1e-12);
        // Outer edge clockwise, hole counter-clockwise.
        assert!(is_clockwise(&result.polys[0].poly));
        assert!(!is_clockwise(&result.polys[1].poly));
    }

    #[test]
    fn intersection_area() {
        let ops = PolygonOps::new();
        let a = polys(square(0.0, 0.0, 1.0));
        let b = polys(square(0.5, 0.0, 1.0));
        let result = ops.intersection(&a, &b);
        assert_eq!(result.polys.len(), 1);
        assert!((net_area(&result) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn xor_area() {
        let ops = PolygonOps::new();
        let a = polys(square(0.0, 0.0, 1.0));
        let b = polys(square(0.5, 0.0, 1.0));
        let result = ops.xor(&a, &b);
        assert!((net_area(&result) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn disjoint_intersection_is_empty() {
        let ops = PolygonOps::new();
        let a = polys(square(0.0, 0.0, 1.0));
        let b = polys(square(2.0, 0.0, 1.0));
        assert!(ops.intersection(&a, &b).polys.is_empty());
    }

    #[test]
    fn offset_expands_by_delta() {
        let ops = PolygonOps::new();
        let a = polys(square(0.0, 0.0, 1.0));
        let result = ops.offset(&a, 0.1);
        assert_eq!(result.polys.len(), 1);
        // Native `jtSquare` offset chamfers the corners: 1.44 minus four
        // corner triangles of side tan(pi/8)*0.1.
        assert!(
            (net_area(&result) - 1.4331367672).abs() < 1e-9,
            "{}",
            net_area(&result)
        );
        let min_x = result.polys[0]
            .poly
            .points
            .iter()
            .map(|p| p.x)
            .fold(f64::MAX, f64::min);
        assert!((min_x + 0.1).abs() < 1e-9);
    }

    #[test]
    fn negative_offset_shrinks() {
        let ops = PolygonOps::new();
        let a = polys(square(0.0, 0.0, 1.0));
        let result = ops.offset(&a, -0.1);
        assert!(
            (net_area(&result) - 0.64).abs() < 1e-9,
            "{}",
            net_area(&result)
        );
    }

    #[test]
    fn offset_to_nothing_is_empty() {
        let ops = PolygonOps::new();
        let a = polys(square(0.0, 0.0, 1.0));
        assert!(ops.offset(&a, -0.6).polys.is_empty());
    }

    #[test]
    fn simplify_keeps_area_and_removes_bowtie() {
        let ops = PolygonOps::new();
        // Self-intersecting bow tie.
        let bowtie = FloatPolygon {
            points: vec![
                FloatPoint { x: 0.0, y: 0.0 },
                FloatPoint { x: 1.0, y: 1.0 },
                FloatPoint { x: 1.0, y: 0.0 },
                FloatPoint { x: 0.0, y: 1.0 },
            ],
        };
        let result = ops.simplify(&polys(bowtie));
        assert!(!result.polys.is_empty());
        let all_clockwise = result
            .polys
            .iter()
            .all(|p| p.is_hole != is_clockwise(&p.poly));
        assert!(all_clockwise);
        assert!((net_area(&result) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn simplify_removes_collinear_vertices() {
        let ops = PolygonOps::new();
        let poly = FloatPolygon {
            points: vec![
                FloatPoint { x: 0.0, y: 0.0 },
                FloatPoint { x: 0.5, y: 0.0 },
                FloatPoint { x: 1.0, y: 0.0 },
                FloatPoint { x: 1.0, y: 1.0 },
                FloatPoint { x: 0.0, y: 1.0 },
            ],
        };
        let result = ops.simplify(&polys(poly));
        assert_eq!(result.polys.len(), 1);
        assert_eq!(result.polys[0].poly.points.len(), 4);
    }

    #[test]
    fn add_hole_and_result_nesting() {
        let ops = PolygonOps::new();
        let mut polygon = ops.polygon();
        polygon.add_edge(&square(0.0, 0.0, 1.0));
        polygon.add_hole(&square(0.25, 0.25, 0.5));
        let result = polygon.result();
        assert_eq!(result.polys.len(), 2);
        assert_eq!(result.polys.iter().filter(|p| p.is_hole).count(), 1);
        assert!((net_area(&result) - 0.75).abs() < 1e-12);
    }

    #[test]
    fn island_inside_hole_has_nesting_level_two() {
        let ops = PolygonOps::new();
        let mut polygon = ops.polygon();
        polygon.add_edge(&square(0.0, 0.0, 1.0));
        polygon.add_hole(&square(0.1, 0.1, 0.8));
        polygon.add_edge(&square(0.3, 0.3, 0.4));
        let result = polygon.result();
        assert_eq!(result.polys.len(), 3);
        let island = result
            .polys
            .iter()
            .find(|p| !p.is_hole && p.nesting_level == 2)
            .expect("island nesting level 2");
        assert!((area_of(&island.poly) - 0.16).abs() < 1e-12);
    }

    /// Vertices sorted lexicographically, for set comparison against the
    /// native Clipper 1 output (which may start a cycle at a different vertex).
    fn vertex_set(poly: &FloatPolygon) -> Vec<(i64, i64)> {
        let mut set: Vec<(i64, i64)> = poly
            .points
            .iter()
            .map(|p| ((p.x / 1e-5).round() as i64, (p.y / 1e-5).round() as i64))
            .collect();
        set.sort();
        set
    }

    fn grid_vertices(points: &[(i64, i64)]) -> Vec<(i64, i64)> {
        let mut set = points.to_vec();
        set.sort();
        set
    }

    #[test]
    fn offset_vertices_match_native_clipper1() {
        // Native `polytest` output for offset(1x1, +0.1) with jtSquare.
        let ops = PolygonOps::new();
        let result = ops.offset(&polys(square(0.0, 0.0, 1.0)), 0.1);
        assert_eq!(
            vertex_set(&result.polys[0].poly),
            grid_vertices(&[
                (104142, -10000),
                (110000, -4142),
                (110000, 104142),
                (104142, 110000),
                (-4142, 110000),
                (-10000, 104142),
                (-10000, -4142),
                (-4142, -10000),
            ])
        );
    }

    #[test]
    fn shrink_vertices_match_native_clipper1() {
        // Native `polytest` output for offset(1x1, -0.1) with jtSquare.
        let ops = PolygonOps::new();
        let result = ops.offset(&polys(square(0.0, 0.0, 1.0)), -0.1);
        assert_eq!(result.polys.len(), 1);
        assert_eq!(
            vertex_set(&result.polys[0].poly),
            grid_vertices(&[
                (90000, 10000),
                (90000, 90000),
                (10000, 90000),
                (10000, 10000),
            ])
        );
    }

    #[test]
    fn union_vertices_match_native_clipper1() {
        // Native `polytest` output for union(1x1, 1x1 at x=0.5).
        let ops = PolygonOps::new();
        let result = ops.union(&polys(square(0.0, 0.0, 1.0)), &polys(square(0.5, 0.0, 1.0)));
        assert_eq!(result.polys.len(), 1);
        assert_eq!(
            vertex_set(&result.polys[0].poly),
            grid_vertices(&[(150000, 0), (150000, 100000), (0, 100000), (0, 0),])
        );
    }

    #[test]
    fn difference_vertices_match_native_clipper1() {
        // Native `polytest` output for difference(1x1, 0.5x0.5 at 0.25).
        let ops = PolygonOps::new();
        let result = ops.difference(
            &polys(square(0.0, 0.0, 1.0)),
            &polys(square(0.25, 0.25, 0.5)),
        );
        assert_eq!(result.polys.len(), 2);
        let outer = result.polys.iter().find(|p| !p.is_hole).unwrap();
        let hole = result.polys.iter().find(|p| p.is_hole).unwrap();
        assert_eq!(
            vertex_set(&outer.poly),
            grid_vertices(&[(100000, 0), (100000, 100000), (0, 100000), (0, 0)])
        );
        assert_eq!(
            vertex_set(&hole.poly),
            grid_vertices(&[
                (25000, 25000),
                (25000, 75000),
                (75000, 75000),
                (75000, 25000)
            ])
        );
        assert_eq!(hole.nesting_level, 1);
    }

    #[test]
    fn island_levels_match_native_clipper1() {
        // Native `polytest` output for square + hole + island.
        let ops = PolygonOps::new();
        let mut polygon = ops.polygon();
        polygon.add_edge(&square(0.0, 0.0, 1.0));
        polygon.add_hole(&square(0.1, 0.1, 0.8));
        polygon.add_edge(&square(0.3, 0.3, 0.4));
        let result = polygon.result();
        let levels: Vec<i32> = result.polys.iter().map(|p| p.nesting_level).collect();
        assert_eq!(levels, vec![0, 1, 2]);
        assert_eq!(result.polys.iter().filter(|p| p.is_hole).count(), 1);
    }

    #[test]
    fn round_join_offset_matches_native_clipper1() {
        // Native `polytest` output: octagon of radius 0.5 expanded by 0.05
        // with arc precision 0.001 produces 24 vertices.
        let mut ops = PolygonOps::new();
        ops.set_arc_precision(0.001);
        let mut octagon = FloatPolygon { points: Vec::new() };
        for i in 0..8 {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / 8.0;
            octagon.points.push(FloatPoint {
                x: 0.5 * angle.cos(),
                y: 0.5 * angle.sin(),
            });
        }
        let result = ops.offset(&polys(octagon), 0.05);
        assert_eq!(result.polys.len(), 1);
        assert_eq!(result.polys[0].poly.points.len(), 24);
        // The native output starts at (0, -0.55) and contains these points.
        let set = vertex_set(&result.polys[0].poly);
        for point in [(0, -55000), (55000, 0), (0, 55000), (-55000, 0)] {
            assert!(set.contains(&point), "missing {point:?}");
        }
    }

    #[test]
    fn grid_snapping_matches_native() {
        let ops = PolygonOps::new();
        let a = polys(FloatPolygon {
            points: vec![
                FloatPoint {
                    x: 0.000011,
                    y: 0.0,
                },
                FloatPoint {
                    x: 1.000014,
                    y: 0.0,
                },
                FloatPoint {
                    x: 1.000016,
                    y: 1.0,
                },
                FloatPoint { x: 0.0, y: 1.0 },
            ],
        });
        let result = ops.union(&a, &FloatPolygons { polys: Vec::new() });
        let xs: Vec<f64> = result.polys[0].poly.points.iter().map(|p| p.x).collect();
        assert!(xs
            .iter()
            .all(|x| (x / 1e-5 - (x / 1e-5).round()).abs() < 1e-9));
    }
}
