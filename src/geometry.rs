use std::collections::VecDeque;

use crate::model::{Canvas, Point};

#[must_use]
pub fn line_points(start: Point, end: Point) -> Vec<Point> {
    let mut points = Vec::new();
    let mut x0 = i32::from(start.x);
    let mut y0 = i32::from(start.y);
    let x1 = i32::from(end.x);
    let y1 = i32::from(end.y);
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        points.push(Point::new(x0 as u16, y0 as u16));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let doubled = error * 2;
        if doubled >= dy {
            error += dy;
            x0 += sx;
        }
        if doubled <= dx {
            error += dx;
            y0 += sy;
        }
    }

    points
}

#[must_use]
pub fn rectangle_points(start: Point, end: Point, filled: bool) -> Vec<Point> {
    let left = start.x.min(end.x);
    let right = start.x.max(end.x);
    let top = start.y.min(end.y);
    let bottom = start.y.max(end.y);
    let mut points = Vec::new();

    for y in top..=bottom {
        for x in left..=right {
            if filled || x == left || x == right || y == top || y == bottom {
                points.push(Point::new(x, y));
            }
        }
    }

    points
}

#[must_use]
pub fn flood_region(canvas: &Canvas, start: Point) -> Vec<Point> {
    if !canvas.contains(start) {
        return Vec::new();
    }

    let target = canvas.composite_cell(start).cloned().unwrap_or_default();
    let mut visited = vec![false; usize::from(canvas.width) * usize::from(canvas.height)];
    let mut queue = VecDeque::from([start]);
    let mut region = Vec::new();

    while let Some(point) = queue.pop_front() {
        let index = usize::from(point.y) * usize::from(canvas.width) + usize::from(point.x);
        if visited[index] {
            continue;
        }
        visited[index] = true;

        if canvas.composite_cell(point) != Some(&target) {
            continue;
        }
        region.push(point);

        if point.x > 0 {
            queue.push_back(Point::new(point.x - 1, point.y));
        }
        if point.x + 1 < canvas.width {
            queue.push_back(Point::new(point.x + 1, point.y));
        }
        if point.y > 0 {
            queue.push_back(Point::new(point.x, point.y - 1));
        }
        if point.y + 1 < canvas.height {
            queue.push_back(Point::new(point.x, point.y + 1));
        }
    }

    region
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_contains_both_endpoints() {
        let points = line_points(Point::new(1, 1), Point::new(5, 3));
        assert_eq!(points.first(), Some(&Point::new(1, 1)));
        assert_eq!(points.last(), Some(&Point::new(5, 3)));
    }

    #[test]
    fn hollow_rectangle_has_expected_corners() {
        let points = rectangle_points(Point::new(2, 2), Point::new(4, 4), false);
        for corner in [
            Point::new(2, 2),
            Point::new(4, 2),
            Point::new(2, 4),
            Point::new(4, 4),
        ] {
            assert!(points.contains(&corner));
        }
        assert!(!points.contains(&Point::new(3, 3)));
    }
}
