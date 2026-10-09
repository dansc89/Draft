pub mod editor;
pub mod pdf;
pub mod persistence;
pub const TICKS: i64 = 64;
pub const MAX_TICKS: i64 = 64 * 12 * 100_000;
type Result<T> = std::result::Result<T, String>;
fn integer(s: &str) -> Result<i128> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Expected unsigned digits".into());
    }
    s.parse().map_err(|_| "Number too large".into())
}
fn unsigned_inches(s: &str) -> Result<i128> {
    if let Some((whole, fraction)) = s.split_once(' ') {
        if !fraction.trim().contains('/') || fraction.trim().contains(' ') {
            return Err("Mixed inches require a whole number and fraction".into());
        }
        return integer(whole)?
            .checked_mul(64)
            .and_then(|v| v.checked_add(unsigned_inches(fraction.trim()).ok()?))
            .ok_or("Invalid mixed fraction".into());
    }
    let (n, d) = if let Some((n, d)) = s.split_once('/') {
        (integer(n)?, integer(d)?)
    } else if let Some((a, b)) = s.split_once('.') {
        let d = 10_i128
            .checked_pow(u32::try_from(b.len()).map_err(|_| "Decimal too long")?)
            .ok_or("Decimal too long")?;
        (
            integer(if a.is_empty() { "0" } else { a })?
                .checked_mul(d)
                .and_then(|n| n.checked_add(integer(b).ok()?))
                .ok_or("Number too large")?,
            d,
        )
    } else {
        (integer(s)?, 1)
    };
    let scaled = n.checked_mul(64).ok_or("Number too large")?;
    if d == 0 || scaled % d != 0 {
        return Err("Coordinate must be exactly representable in 1/64 inch ticks".into());
    }
    Ok(scaled / d)
}
pub fn parse_inches(input: &str) -> Result<i64> {
    let s = input.trim();
    let (sign, s) = if let Some(s) = s.strip_prefix('-') {
        (-1, s)
    } else {
        (1, s.strip_prefix('+').unwrap_or(s))
    };
    let s = s.strip_suffix('"').unwrap_or(s).trim();
    let ticks = if let Some((feet, inches)) = s.split_once('\'') {
        integer(feet)?
            .checked_mul(12 * 64)
            .and_then(|f| {
                f.checked_add(if inches.trim().is_empty() {
                    0
                } else {
                    unsigned_inches(inches.trim()).ok()?
                })
            })
            .ok_or("Invalid feet/inches")?
    } else {
        unsigned_inches(s)?
    };
    if ticks > i128::from(MAX_TICKS) {
        return Err("Coordinate exceeds ±100,000 feet".into());
    }
    Ok(i64::try_from(ticks).map_err(|_| "Coordinate too large")? * sign)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}
impl Point {
    pub fn new(x: i64, y: i64) -> Result<Self> {
        if !(-MAX_TICKS..=MAX_TICKS).contains(&x) || !(-MAX_TICKS..=MAX_TICKS).contains(&y) {
            Err("Out of bounds".into())
        } else {
            Ok(Self { x, y })
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line {
    pub a: Point,
    pub b: Point,
}
impl Line {
    pub fn new(a: Point, b: Point) -> Result<Self> {
        if a == b {
            Err("Zero-length line".into())
        } else {
            Point::new(a.x, a.y)?;
            Point::new(b.x, b.y)?;
            Ok(Self { a, b })
        }
    }
}
#[derive(Default)]
pub struct Drawing {
    lines: Vec<Line>,
    redo: Vec<Line>,
    saved: Vec<Line>,
}
impl Drawing {
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }
    pub fn add(&mut self, line: Line) -> Result<()> {
        Line::new(line.a, line.b)?;
        self.lines.push(line);
        self.redo.clear();
        Ok(())
    }
    pub fn undo(&mut self) {
        if let Some(line) = self.lines.pop() {
            self.redo.push(line);
        }
    }
    pub fn redo(&mut self) {
        if let Some(line) = self.redo.pop() {
            self.lines.push(line);
        }
    }
    pub fn mark_saved(&mut self) {
        self.saved = self.lines.clone();
    }
    pub fn dirty(&self) -> bool {
        self.lines != self.saved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn drawing_never_accepts_invalid_public_geometry() {
        let mut d = Drawing::default();
        assert!(
            d.add(Line {
                a: Point { x: 0, y: 0 },
                b: Point { x: 0, y: 0 }
            })
            .is_err()
        );
        assert!(d.lines().is_empty());
    }
    #[test]
    fn geometry_history_and_bounds() {
        let a = Point::new(0, 0).unwrap();
        let b = Point::new(768, 0).unwrap();
        let l = Line::new(a, b).unwrap();
        let mut d = Drawing::default();
        d.add(l).unwrap();
        assert!(d.dirty());
        d.mark_saved();
        d.undo();
        assert!(d.dirty());
        assert!(d.lines().is_empty());
        d.redo();
        assert!(!d.dirty());
        assert_eq!(d.lines(), &[l]);
        d.undo();
        d.add(Line::new(a, Point::new(0, 768).unwrap()).unwrap())
            .unwrap();
        d.redo();
        assert_eq!(d.lines().len(), 1);
        assert!(Line::new(a, a).is_err());
        assert!(Point::new(MAX_TICKS + 1, 0).is_err());
        assert!(Point::new(i64::MIN, 0).is_err());
    }
    #[test]
    fn decimal_shorthand_and_malformed_mixed_numbers() {
        assert_eq!(parse_inches(".5"), Ok(32));
        assert_eq!(parse_inches("-.015625"), Ok(-1));
        for s in ["1 2", "1 0.5", "1 1/2/3", "1..2", "1' 1'", "--1", "1e3"] {
            assert!(parse_inches(s).is_err(), "{s}");
        }
    }
    #[test]
    fn exact_imperial_parser() {
        assert_eq!(parse_inches("12' 3 1/2\""), Ok(147 * 64 + 32));
        assert_eq!(parse_inches("-1' 6\""), Ok(-18 * 64));
        assert_eq!(parse_inches("0.125"), Ok(8));
        assert_eq!(parse_inches("1/64\""), Ok(1));
        for s in [
            "0.1",
            "1/128",
            "NaN",
            "inf",
            "1/0",
            "",
            "99999999999999999",
            "1' -2\"",
        ] {
            assert!(parse_inches(s).is_err(), "{s}");
        }
    }
}
