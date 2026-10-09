use crate::{Drawing, Line, Point, Result};
/// Cursor input is deliberately snapped; textual input never uses this path.
pub fn snap(x: f64, y: f64, grid: i64) -> Result<Point> {
    if grid <= 0
        || grid > crate::MAX_TICKS
        || !x.is_finite()
        || !y.is_finite()
        || x.abs() > (crate::MAX_TICKS / 64) as f64
        || y.abs() > (crate::MAX_TICKS / 64) as f64
    {
        return Err("Invalid or out-of-bounds cursor/grid".into());
    }
    Point::new(
        (x * 64.0 / grid as f64).round() as i64 * grid,
        (y * 64.0 / grid as f64).round() as i64 * grid,
    )
}
pub fn orthogonal(start: Point, end: Point) -> Point {
    if (i128::from(end.x) - i128::from(start.x)).abs()
        >= (i128::from(end.y) - i128::from(start.y)).abs()
    {
        Point {
            x: end.x,
            y: start.y,
        }
    } else {
        Point {
            x: start.x,
            y: end.y,
        }
    }
}
pub fn replace_drawing(drawing: &mut Drawing, lines: Vec<Line>, discard: bool) -> Result<()> {
    if drawing.dirty() && !discard {
        return Err("Unsaved drawing: save to a new DXF filename or explicitly enable discard before opening".into());
    }
    for l in &lines {
        Line::new(l.a, l.b)?;
    }
    *drawing = Drawing::default();
    for line in lines {
        drawing.add(line)?;
    }
    drawing.mark_saved();
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapped_orthogonal_and_guarded_replacement() {
        assert_eq!(
            snap(12.1, -3.9, 64).unwrap(),
            Point::new(768, -256).unwrap()
        );
        for x in [f64::NAN, f64::INFINITY, 1e20] {
            assert!(snap(x, 0.0, 64).is_err());
        }
        assert!(snap(0.0, 0.0, 0).is_err());
        let start = Point::new(0, 0).unwrap();
        assert_eq!(
            orthogonal(start, Point::new(128, 64).unwrap()),
            Point::new(128, 0).unwrap()
        );
        assert_eq!(
            orthogonal(start, Point::new(64, 128).unwrap()),
            Point::new(0, 128).unwrap()
        );
        let line = Line::new(start, Point::new(64, 0).unwrap()).unwrap();
        let mut d = Drawing::default();
        d.add(line).unwrap();
        assert!(replace_drawing(&mut d, vec![], false).is_err());
        assert_eq!(d.lines(), &[line]);
        replace_drawing(&mut d, vec![], true).unwrap();
        assert!(d.lines().is_empty());
        assert!(!d.dirty());
        d.undo();
        assert!(d.lines().is_empty());
        let bad = Line { a: start, b: start };
        assert!(replace_drawing(&mut d, vec![bad], true).is_err());
        assert!(d.lines().is_empty());
    }
}
