use std::{collections::{HashMap, HashSet}, hash::Hasher};

use palette::IntoColor;


#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64
}

impl Point {
    pub fn len(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn unit(self) -> Self {
        let len = self.len();
        if len < 1e-10 {
            return Point { x: 0.0, y: 0.0 }; // Avoid division by zero
        }
        Point { x: self.x / len, y: self.y / len }
    }
    pub fn cross(self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }
    pub fn angle(self) -> f64 {
        self.y.atan2(self.x)
    }
}
impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Point { x: self.x - other.x, y: self.y - other.y }
    }
}
impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Point { x: self.x + other.x, y: self.y + other.y }
    }
}
impl std::ops::Mul<f64> for Point {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Point { x: self.x * scalar, y: self.y * scalar }
    }
}

#[derive(Debug, Clone)]
pub struct Path(Vec<Point>);
impl Path {
    pub fn new() -> Self {
        Path(Vec::new())
    }
    pub fn push(&mut self, point: Point) {
        self.0.push(point);
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn last(&self) -> Option<&Point> {
        self.0.last()
    }

    #[allow(unused)]
    pub fn close(&mut self) {
        if let Some(first) = self.0.first() {
            if let Some(last) = self.0.last() {
                if *first != *last {
                    self.0.push(*first);
                }
            }
        }
    }
    #[allow(unused)]
    pub fn unclose(&mut self) {
        if let Some(last) = self.0.last() {
            if let Some(first) = self.0.first() {
                if *last == *first {
                    self.0.pop();
                }
            }
        }
    }
}
impl std::ops::Index<usize> for Path {
    type Output = Point;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

// == for Point has a tolerance
// Technically, this breaks the contract of PartialEq, but it works out fine.
// Perhaps it would instead be better to use fixed-position coordinates, but this is fine for now.
impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < 1e-10 && (self.y - other.y).abs() < 1e-10
    }
}
impl Eq for Point {}

impl std::hash::Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Round x and y to 1e-10 to avoid floating point precision issues
        let x_rounded = (self.x * 1e10).round() / 1e10;
        let y_rounded = (self.y * 1e10).round() / 1e10;
        // Hash the rounded values
        x_rounded.to_bits().hash(state);
        y_rounded.to_bits().hash(state);
    }
}

#[derive(Debug, PartialEq)]
enum RectanglePolarity {
    FilledInward,
    FilledOutward
}

#[derive(Debug)]
pub struct Rectangle {
    center: Point,
    width: f64,
    height: f64,
    polarity: RectanglePolarity
}

#[allow(unused)]
impl Rectangle {
    pub fn filled_outward(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        return Rectangle {
            center: Point { x: (x0 + x1) / 2., y: (y0 + y1) / 2. },
            height: y1 - y0,
            width: x1 - x0,
            polarity: RectanglePolarity::FilledOutward
        }
    }
    pub fn filled_inward(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        return Rectangle {
            center: Point { x: (x0 + x1) / 2., y: (y0 + y1) / 2. },
            height: y1 - y0,
            width: x1 - x0,
            polarity: RectanglePolarity::FilledInward
        }
    }
    pub fn filled_inward_center(center_x: f64, center_y: f64, width: f64, height: f64) -> Self {
        return Rectangle {
            center: Point { x: center_x, y: center_y },
            width,
            height,
            polarity: RectanglePolarity::FilledInward
        };
    }
    pub fn filled_outward_center(center_x: f64, center_y: f64, width: f64, height: f64) -> Self {
        return Rectangle {
            center: Point { x: center_x, y: center_y },
            width,
            height,
            polarity: RectanglePolarity::FilledOutward
        };
    }

    fn get_corners(&self) -> (Point, Point, Point, Point) {
        return (
            Point { x: self.center.x - self.width / 2., y: self.center.y + self.height / 2. },
            Point { x: self.center.x + self.width / 2., y: self.center.y + self.height / 2. },
            Point { x: self.center.x + self.width / 2., y: self.center.y - self.height / 2. },
            Point { x: self.center.x - self.width / 2., y: self.center.y - self.height / 2. }
        );
    }
}

#[derive(Debug)]
struct LineSegments {
    segments: Vec<(Point, Point)>
}

impl LineSegments {
    fn new() -> LineSegments {
        return Self {
            segments: vec![]
        }
    }
    fn add_rectangle(&mut self, rect: &Rectangle) {
        let corners = rect.get_corners();
        self.segments.push((corners.0, corners.1));
        self.segments.push((corners.1, corners.2));
        self.segments.push((corners.2, corners.3));
        self.segments.push((corners.3, corners.0));
    }
    /// Prune line segments that are completely inside the given rectangle if filled inward,
    /// or completely outside it if the rectangle is filled outward.
    /// This probably has like O(n^1000) complexity or something stupid, but it's fast enough for our purposes.
    fn prune_segments(&mut self, rect: &Rectangle) {
        let (x0, y0, x1, y1) = (
            rect.center.x - rect.width / 2.,
            rect.center.y - rect.height / 2.,
            rect.center.x + rect.width / 2.,
            rect.center.y + rect.height / 2.
        );

        self.segments.retain(|&(p1, p2)| {
            if rect.polarity == RectanglePolarity::FilledInward {
                // Prune segments that are inside the rectangle
                let inside = |p: &Point| p.x > x0 && p.x < x1 && p.y > y0 && p.y < y1;
                if inside(&p1) || inside(&p2) {
                    return false; // Segment is completely inside, prune it
                } else {
                    return true; // Keep segment that intersects or is outside
                }
            } else {
                // Prune segments that are outside the rectangle
                let outside = |p: &Point| p.x < x0 || p.x > x1 || p.y < y0 || p.y > y1;
                if outside(&p1) || outside(&p2) {
                    return false; // Segment is completely outside, prune it
                } else {
                    return true; // Keep segment that intersects or is inside
                }
            }
        });

        // Step 2: We recognize that any corner with three line segments touching is invalid.
        // The line that must be pruned is one of the parallel segments
        // (which must exist since we have axis-aligned rectangles). Specifically,
        // the segment that is inside ourself should be removed.
        // This only applies to rectangles that are filled inward.
        if rect.polarity != RectanglePolarity::FilledInward {
            return; // Only apply this pruning for inward rectangles
        }

        let endpoint_counts: HashMap<Point, usize> = self.segments.iter()
            .flat_map(|&(p1, p2)| vec![p1, p2])
            .fold(HashMap::new(), |mut acc, p| {
                *acc.entry(p).or_insert(0) += 1;
                acc
            });

        let mut prunable_segments: HashSet<(Point, Point)> = HashSet::new();
        for (endpoint, count) in endpoint_counts.iter() {
            if *count >= 3 {
                let mut parallel_segments = vec![];
                for &(p1, p2) in &self.segments {
                    if p1 == *endpoint {
                        parallel_segments.push((*endpoint, p2));
                    }
                    if p2 == *endpoint {
                        parallel_segments.push((*endpoint, p1));
                    }
                }

                // Only keep segments collinear with another segment. p1 will always be the endpoint.
                for i in 0..parallel_segments.len() {
                    let (_, p2_first) = parallel_segments[i];
                    // If any other segment's p2 is collinear with this one, keep it
                    let mut is_collinear = false;
                    for j in (0)..parallel_segments.len() {
                        if i == j {
                            continue; // Skip self
                        }
                        let (_, p2_other) = parallel_segments[j];
                        // Check if p2_first and p2_other are collinear with the endpoint
                        if (p2_first.x - endpoint.x).abs() < 1e-10 && (p2_other.x - endpoint.x).abs() < 1e-10 ||
                           (p2_first.y - endpoint.y).abs() < 1e-10 && (p2_other.y - endpoint.y).abs() < 1e-10 {
                            is_collinear = true;
                            break;
                        }
                    }
                    if is_collinear {
                        prunable_segments.insert(parallel_segments[i]);
                    }
                }
            }
        }

        // For debugging: return the prunable segments
        // let prunable = prunable_segments.iter().cloned().collect::<Vec<_>>();
        // self.segments = prunable;

        // For each prunable segment, remove ones that are inside the rectangle
        self.segments.retain(|&(p1, p2)| {
            if prunable_segments.contains(&(p1, p2)) || prunable_segments.contains(&(p2, p1)) {
                // If this segment is prunable, check if it's inside the rectangle
                let is_inside = |p: &Point| p.x >= x0 && p.x <= x1 && p.y >= y0 && p.y <= y1;
                return !is_inside(&p1) || !is_inside(&p2);
            }
            true
        });
    }
    fn reset(&mut self) {
        self.segments.clear();
    }
    fn split_at_intersections(&mut self) {
        // Helper to check intersection and return intersection point (excluding endpoints)
        fn segments_intersect(a: &(Point, Point), b: &(Point, Point)) -> Option<Point> {
            let (p, r) = (a.0, Point { x: a.1.x - a.0.x, y: a.1.y - a.0.y });
            let (q, s) = (b.0, Point { x: b.1.x - b.0.x, y: b.1.y - b.0.y });

            let cross = r.x * s.y - r.y * s.x;
            if cross.abs() < 1e-10 {
                // Parallel or colinear
                return None;
            }

            let qp = Point { x: q.x - p.x, y: q.y - p.y };
            let t = (qp.x * s.y - qp.y * s.x) / cross;
            let u = (qp.x * r.y - qp.y * r.x) / cross;

            // Allow intersection at a single endpoint, but not both
            let is_endpoint = |v: f64| v.abs() < 1e-10 || (v - 1.0).abs() < 1e-10;

            let t_in = t >= -1e-10 && t <= 1.0 + 1e-10;
            let u_in = u >= -1e-10 && u <= 1.0 + 1e-10;

            if t_in && u_in {
                let t_is_ep = is_endpoint(t);
                let u_is_ep = is_endpoint(u);
                // Exclude both endpoints (intersection at both endpoints means overlap, not intersection)
                if t_is_ep && u_is_ep {
                    return None;
                }
                return Some(Point { x: p.x + t * r.x, y: p.y + t * r.y });
            }
            None
        }

        let n = self.segments.len();
        // For each segment, collect all intersection points (including endpoints)
        let mut split_points: Vec<Vec<Point>> = vec![vec![]; n];

        // Add endpoints for each segment
        for (i, (p0, p1)) in self.segments.iter().enumerate() {
            split_points[i].push(*p0);
            split_points[i].push(*p1);
        }

        // Find all intersections and add to split_points
        for i in 0..n {
            for j in (i + 1)..n {
                if let Some(pt) = segments_intersect(&self.segments[i], &self.segments[j]) {
                    split_points[i].push(pt);
                    split_points[j].push(pt);
                }
            }
        }

        // Sort and deduplicate points along each segment, then create new segments
        let mut new_segments = Vec::new();
        for (idx, points) in split_points.iter_mut().enumerate() {
            let (p0, _p1) = self.segments[idx];
            // Sort points along the segment
            points.sort_by(|a, b| {
                let da = ((a.x - p0.x).powi(2) + (a.y - p0.y).powi(2)).sqrt();
                let db = ((b.x - p0.x).powi(2) + (b.y - p0.y).powi(2)).sqrt();
                da.partial_cmp(&db).unwrap()
            });
            // Remove duplicates (within tolerance)
            points.dedup_by(|a, b| (a.x - b.x).abs() < 1e-10 && (a.y - b.y).abs() < 1e-10);
            // Create segments between consecutive points
            for pair in points.windows(2) {
                if pair[0] != pair[1] {
                    new_segments.push((pair[0], pair[1]));
                }
            }
        }

        self.segments = new_segments;
    }

    fn generate_path(&self) -> Path {
        let mut path = Path::new();
        if self.segments.is_empty() {
            return path; // No segments to draw
        }

        let mut unused_segments: HashSet<(Point, Point)> = self.segments.iter().cloned().collect();
        unused_segments.remove(&self.segments[0]);

        // Start at the first segment
        path.push(self.segments[0].0);
        path.push(self.segments[0].1);

        while unused_segments.len() > 0 {
            let last_point = *path.last().unwrap();
            let mut found_next = false;

            // Find the next segment that connects to the last point
            unused_segments.retain(|&(p1, p2)| {
                if p1 == last_point {
                    path.push(p2);
                    found_next = true;
                    false // Remove this segment
                } else if p2 == last_point {
                    path.push(p1);
                    found_next = true;
                    false // Remove this segment
                } else {
                    true // Keep this segment
                }
            });

            if !found_next {
                // No more segments to connect, break out
                break;
            }
        }

        path
    }

    #[allow(unused)]
    fn debug_draw(&self, cr: &cairo::Context, use_path: bool) {
        cr.set_line_width(2.0);
        if use_path {
            let path = self.generate_path();
            if !path.is_empty() {
                cr.move_to(path[0].x, path[0].y);
                for point in &path.0[1..] {
                    cr.line_to(point.x, point.y);
                }
                cr.stroke().expect("Failed to stroke path");
            }
            return;
        }

        for(i, (p1, p2)) in self.segments.iter().enumerate() {
            let (r, g, b) = IntoColor::<palette::Srgb>::into_color(palette::Hsl::new(
                (i as f32 * 360. / self.segments.len() as f32) % 360.,
                1.0,
                0.5
            )).into_components();
            cr.set_source_rgb(r as f64, g as f64, b as f64);
            cr.move_to(p1.x, p1.y);
            cr.line_to(p2.x, p2.y);
            cr.stroke().expect("Failed to stroke line segment");
        }
    }
}

#[derive(Debug)]
pub struct BorderState {
    rectangles: Vec<Rectangle>,
    line_segments: LineSegments
}

impl BorderState {
    pub fn new() -> Self {
        BorderState {
            rectangles: vec![],
            line_segments: LineSegments::new()
        }
    }

    // Temporary
    pub fn update_rectangles(&mut self, outer: Rectangle, cutin: Option<Rectangle>) {
        if let Some(cutin) = cutin {
            self.rectangles = vec![outer, cutin];
        } else {
            self.rectangles = vec![outer];
        }
    }

    pub fn compute_border_path(&mut self) -> Path {
        let start_time = std::time::Instant::now();
        self.line_segments.reset();

        for rect in &self.rectangles {
            self.line_segments.add_rectangle(rect);
        }

        self.line_segments.split_at_intersections();

        for rect in &self.rectangles {
            self.line_segments.prune_segments(rect);
        }

        let path = self.line_segments.generate_path();
        println!("Computed paths in {:?}", start_time.elapsed());
        return path;
    }

    #[allow(unused)]
    pub fn debug_draw(&self, cr: &cairo::Context, use_path: bool) {
        self.line_segments.debug_draw(cr, use_path);
    }
}