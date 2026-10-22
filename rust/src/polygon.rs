//! Polygon operations — port of `hyp2mat/lib/polygon.cc`.
//! Uses clipper2 for boolean operations, offset, and simplify.

use crate::model::{FloatPoint, FloatPolygon, FloatPoly, FloatPolygons};
use clipper2::{FillRule, JoinType, EndType, One, Paths};

/// Grid size for snapping coordinates (10 micron, matching C++ default).
const GRID: f64 = 1e-5;

/// Snap a coordinate to the grid.
fn align_to_grid(x: f64) -> f64 {
    (x / GRID + 0.5).floor() * GRID
}

/// Convert a FloatPoint to a clipper2 point (f64, snapped to grid).
fn to_clipper_point(p: FloatPoint) -> (f64, f64) {
    (
        align_to_grid(p.x),
        align_to_grid(p.y),
    )
}

/// Convert a clipper2 point back to a FloatPoint (unsnapped from grid).
fn from_clipper_point(p: (f64, f64)) -> FloatPoint {
    FloatPoint {
        x: p.0,
        y: p.1,
    }
}

/// Convert a FloatPolygon to a clipper2 path, removing repeated vertices.
fn to_clipper_path(poly: &FloatPolygon) -> Vec<(f64, f64)> {
    let mut path: Vec<(f64, f64)> = poly.points.iter().map(|p| to_clipper_point(*p)).collect();

    // Remove repeated consecutive vertices
    path.dedup();

    // Remove last vertex if same as first
    while path.len() > 1 && path.first() == path.last() {
        path.pop();
    }

    path
}

/// Convert a clipper2 path back to a FloatPolygon.
fn from_clipper_path(path: &[(f64, f64)]) -> FloatPolygon {
    FloatPolygon {
        points: path.iter().map(|p| from_clipper_point(*p)).collect(),
    }
}

/// Convert FloatPolygons to clipper2 Paths.
fn to_clipper_paths(polys: &FloatPolygons) -> Paths<One> {
    let mut paths: Vec<Vec<(f64, f64)>> = Vec::new();
    for poly in &polys.polys {
        let path = to_clipper_path(&poly.poly);
        if path.len() >= 3 {
            paths.push(path);
        }
    }
    paths.into()
}

/// Convert clipper2 Paths back to FloatPolygons.
/// Hole status is determined by orientation (signed area).
/// Nesting level is determined by point-in-polygon containment.
fn from_clipper_paths(paths: Paths<One>) -> FloatPolygons {
    let paths: Vec<Vec<(f64, f64)>> = paths.into();
    let mut result = FloatPolygons { polys: Vec::new() };

    // First pass: convert all paths to FloatPoly with orientation-based hole flag
    let mut polys: Vec<FloatPoly> = paths
        .into_iter()
        .filter(|p| p.len() >= 3)
        .map(|path| {
            let poly = from_clipper_path(&path);
            let is_hole = is_clockwise(&poly); // clockwise = outer, counter-clockwise = hole
            FloatPoly {
                poly,
                is_hole,
                nesting_level: 0,
            }
        })
        .collect();

    // Second pass: compute nesting levels by containment
    for i in 0..polys.len() {
        let mut level = 0;
        for j in 0..polys.len() {
            if i != j && point_in_polygon(&polys[i].poly.points[0], &polys[j].poly) {
                level += 1;
            }
        }
        polys[i].nesting_level = level;
    }

    result.polys = polys;
    result
}

/// Calculate signed area of a polygon. Positive = clockwise.
fn signed_area(poly: &FloatPolygon) -> f64 {
    let mut area = 0.0;
    let n = poly.points.len();
    for i in 0..n {
        let j = (i + 1) % n;
        area += poly.points[i].x * poly.points[j].y - poly.points[j].x * poly.points[i].y;
    }
    area / 2.0
}

/// Return true if polygon is clockwise (positive signed area).
fn is_clockwise(poly: &FloatPolygon) -> bool {
    signed_area(poly) >= 0.0
}

/// Reverse the order of vertices in a polygon.
fn reverse(poly: &mut FloatPolygon) {
    poly.points.reverse();
}

/// Test if a point is inside a polygon (ray casting algorithm).
fn point_in_polygon(p: &FloatPoint, poly: &FloatPolygon) -> bool {
    let mut inside = false;
    let n = poly.points.len();
    if n < 3 {
        return false;
    }
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

/// Ensure outer edges are clockwise and holes are counter-clockwise.
fn orient_polygons(polys: &mut [FloatPoly]) {
    for poly in polys.iter_mut() {
        let cw = is_clockwise(&poly.poly);
        if poly.is_hole == cw {
            reverse(&mut poly.poly);
        }
    }
}

/// Polygon operations context — holds grid and arc precision settings.
pub struct PolygonOps {
    grid: f64,
    arc_precision: f64,
    fill_type: FillRule,
}

impl Default for PolygonOps {
    fn default() -> Self {
        PolygonOps {
            grid: GRID,
            arc_precision: 0.0,
            fill_type: FillRule::default(), // non-zero (even-odd) fill rule
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

    /// Align a coordinate to the grid.
    fn align_to_grid(&self, x: f64) -> f64 {
        (x / self.grid + 0.5).floor() * self.grid
    }

    /// Check if a FloatPolygons is empty.
    pub fn is_empty(&self, polys: &FloatPolygons) -> bool {
        polys.polys.is_empty()
    }

    /// Add an outer edge to a polygon set.
    pub fn add_edge(&self, _polys: &mut FloatPolygons, _edge: FloatPolygon) {
        // Not implemented as a standalone operation; edges are added via boolean ops.
    }

    /// Add a hole to a polygon set.
    pub fn add_hole(&self, _polys: &mut FloatPolygons, _hole: FloatPolygon) {
        // Not implemented as a standalone operation.
    }

    /// Expand/shrink polygons by a distance (positive = expand, negative = shrink).
    pub fn offset(&self, polys: &FloatPolygons, delta: f64) -> FloatPolygons {
        if delta == 0.0 {
            return polys.clone();
        }

        let paths = to_clipper_paths(polys);
        let d = delta / self.grid;
        let limit = self.arc_precision / self.grid;

        let result = if self.arc_precision == 0.0 {
            clipper2::inflate(paths, d, JoinType::Square, EndType::Butt, 0.0)
        } else {
            clipper2::inflate(paths, d, JoinType::Round, EndType::Butt, limit)
        };

        from_clipper_paths(result)
    }

    /// Remove self-intersections from polygons.
    pub fn simplify(&self, polys: &FloatPolygons) -> FloatPolygons {
        let paths = to_clipper_paths(polys);
        let result = clipper2::simplify(paths, 0.01, false);
        from_clipper_paths(result)
    }

    /// Boolean intersection of two polygon sets.
    pub fn intersection(&self, subject: &FloatPolygons, clip: &FloatPolygons) -> FloatPolygons {
        let subj_paths = to_clipper_paths(subject);
        let clip_paths = to_clipper_paths(clip);
        let result = clipper2::intersect(subj_paths, clip_paths, self.fill_type);
        match result {
            Ok(paths) => from_clipper_paths(paths),
            Err(_) => FloatPolygons { polys: Vec::new() },
        }
    }

    /// Boolean union of two polygon sets.
    pub fn union(&self, subject: &FloatPolygons, clip: &FloatPolygons) -> FloatPolygons {
        let subj_paths = to_clipper_paths(subject);
        let clip_paths = to_clipper_paths(clip);
        let result = clipper2::union(subj_paths, clip_paths, self.fill_type);
        match result {
            Ok(paths) => from_clipper_paths(paths),
            Err(_) => FloatPolygons { polys: Vec::new() },
        }
    }

    /// Boolean difference (subject minus clip).
    pub fn difference(&self, subject: &FloatPolygons, clip: &FloatPolygons) -> FloatPolygons {
        let subj_paths = to_clipper_paths(subject);
        let clip_paths = to_clipper_paths(clip);
        let result = clipper2::difference(subj_paths, clip_paths, self.fill_type);
        match result {
            Ok(paths) => from_clipper_paths(paths),
            Err(_) => FloatPolygons { polys: Vec::new() },
        }
    }

    /// Boolean XOR of two polygon sets.
    pub fn xor(&self, subject: &FloatPolygons, clip: &FloatPolygons) -> FloatPolygons {
        let subj_paths = to_clipper_paths(subject);
        let clip_paths = to_clipper_paths(clip);
        let result = clipper2::xor(subj_paths, clip_paths, self.fill_type);
        match result {
            Ok(paths) => from_clipper_paths(paths),
            Err(_) => FloatPolygons { polys: Vec::new() },
        }
    }

    /// Resolve polygon nesting: ensure outer edges are clockwise, holes counter-clockwise,
    /// and compute nesting levels.
    pub fn resolve(&self, polys: &FloatPolygons) -> FloatPolygons {
        let mut result = polys.clone();
        orient_polygons(&mut result.polys);
        for i in 0..result.polys.len() {
            let mut level = 0;
            for j in 0..result.polys.len() {
                if i != j && point_in_polygon(&result.polys[i].poly.points[0], &result.polys[j].poly) {
                    level += 1;
                }
            }
            result.polys[i].nesting_level = level;
        }
        result
    }
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

    #[test]
    fn test_align_to_grid() {
        let ops = PolygonOps::new();
        assert_eq!(ops.align_to_grid(0.000012), 0.00001);
        assert_eq!(ops.align_to_grid(0.000014), 0.00001);
        assert_eq!(ops.align_to_grid(0.000016), 0.00002);
    }

    #[test]
    fn test_is_clockwise() {
        // Clockwise square (positive signed area)
        let cw = square(0.0, 0.0, 1.0);
        assert!(is_clockwise(&cw));

        // Counter-clockwise square (reverse of above)
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
    fn test_union() {
        let ops = PolygonOps::new();
        let a = polys(square(0.0, 0.0, 10.0));
        let b = polys(square(5.0, 5.0, 10.0));
        let result = ops.union(&a, &b);
        assert!(!result.polys.is_empty());
    }

    #[test]
    fn test_difference() {
        let ops = PolygonOps::new();
        let a = polys(square(0.0, 0.0, 10.0));
        let b = polys(square(2.0, 2.0, 6.0));
        let result = ops.difference(&a, &b);
        assert!(!result.polys.is_empty());
    }

    #[test]
    fn test_intersection() {
        let ops = PolygonOps::new();
        let a = polys(square(0.0, 0.0, 10.0));
        let b = polys(square(5.0, 5.0, 10.0));
        let result = ops.intersection(&a, &b);
        assert!(!result.polys.is_empty());
    }

    #[test]
    fn test_offset() {
        let ops = PolygonOps::new();
        let a = polys(square(0.0, 0.0, 10.0));
        let result = ops.offset(&a, 1.0);
        assert!(!result.polys.is_empty());
    }

    #[test]
    fn test_simplify() {
        let ops = PolygonOps::new();
        let a = polys(square(0.0, 0.0, 10.0));
        let result = ops.simplify(&a);
        assert!(!result.polys.is_empty());
    }

    #[test]
    fn test_resolve() {
        let ops = PolygonOps::new();
        let a = polys(square(0.0, 0.0, 10.0));
        let result = ops.resolve(&a);
        assert_eq!(result.polys.len(), 1);
        assert!(!result.polys[0].is_hole);
    }
}
