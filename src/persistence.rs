use crate::{Line, Point, Result, parse_inches};
use std::{
    io::{Read, Write},
    path::Path,
};
const HEADER: &str = "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1015\n9\n$INSUNITS\n70\n1\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n";
const END: &str = "0\nENDSEC\n0\nEOF\n";
pub fn decimal_inches(ticks: i64) -> String {
    let t = i128::from(ticks).abs();
    format!(
        "{}{}.{:06}",
        if ticks < 0 { "-" } else { "" },
        t / 64,
        t % 64 * 15625
    )
}
pub const MAX_DXF_BYTES: usize = 8 * 1024 * 1024;

pub fn encode_dxf(lines: &[Line]) -> Result<String> {
    let mut out = HEADER.to_owned();
    for l in lines {
        Line::new(l.a, l.b)?;
        let entity = format!(
            "0\nLINE\n8\n0\n10\n{}\n20\n{}\n30\n0\n11\n{}\n21\n{}\n31\n0\n",
            decimal_inches(l.a.x),
            decimal_inches(l.a.y),
            decimal_inches(l.b.x),
            decimal_inches(l.b.y)
        );
        // Reserve the closing section before accumulating each bounded LINE entity.
        if entity.len() > MAX_DXF_BYTES - END.len() - out.len() {
            return Err("DXF exceeds 8 MiB limit".into());
        }
        out.push_str(&entity);
    }
    out.push_str(END);
    Ok(out)
}
pub fn decode_dxf(text: &str) -> Result<Vec<Line>> {
    if text.len() > MAX_DXF_BYTES {
        return Err("DXF exceeds 8 MiB limit".into());
    }
    let normalized = text.replace("\r\n", "\n");
    let body = normalized
        .strip_prefix(HEADER)
        .and_then(|s| s.strip_suffix(END))
        .ok_or("Only Draft inch-unit 2D LINE DXF files are supported")?;
    let mut values = body.lines();
    let mut lines = Vec::new();
    fn expect<'a>(values: &mut impl Iterator<Item = &'a str>, s: &str) -> Result<()> {
        if values.next() == Some(s) {
            Ok(())
        } else {
            Err(format!("Unsupported or malformed DXF (expected {s})"))
        }
    }
    fn coord<'a>(values: &mut impl Iterator<Item = &'a str>, code: &str) -> Result<i64> {
        expect(values, code)?;
        parse_inches(values.next().ok_or("Missing DXF coordinate")?)
    }
    while let Some(v) = values.next() {
        if v != "0" {
            return Err("Expected DXF entity".into());
        }
        expect(&mut values, "LINE")?;
        expect(&mut values, "8")?;
        expect(&mut values, "0")?;
        let a = Point::new(coord(&mut values, "10")?, coord(&mut values, "20")?)?;
        expect(&mut values, "30")?;
        expect(&mut values, "0")?;
        let b = Point::new(coord(&mut values, "11")?, coord(&mut values, "21")?)?;
        expect(&mut values, "31")?;
        expect(&mut values, "0")?;
        lines.push(Line::new(a, b)?);
    }
    Ok(lines)
}
pub fn read_dxf(path: &Path) -> Result<Vec<Line>> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut text = String::new();
    file.take(MAX_DXF_BYTES as u64 + 1)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    decode_dxf(&text)
}
/// Atomically reserves a new name, never follows an existing symlink or overwrites.
/// A failed write removes this call's incomplete newly-created file.
pub fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| {
            format!(
                "{}: {e}; choose a new filename (no overwrite)",
                path.display()
            )
        })?;
    if let Err(e) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = std::fs::remove_file(path);
        return Err(e.to_string());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point;
    #[test]
    fn dxf_exact_roundtrip_and_rejection() {
        let lines =
            vec![Line::new(Point::new(-1, 0).unwrap(), Point::new(9216, 6144).unwrap()).unwrap()];
        let text = encode_dxf(&lines).unwrap();
        assert!(text.contains("$INSUNITS\n70\n1"));
        assert!(text.contains("-0.015625"));
        assert_eq!(decode_dxf(&text).unwrap(), lines);
        for bad in [
            text.replace("-0.015625", "NaN"),
            text.replace("-0.015625", "0.1"),
            text.replace("LINE", "CIRCLE"),
            text.replace("30\n0", "30\n1"),
            text.replace("EOF", ""),
            text.replace("70\n1", "70\n4"),
        ] {
            assert!(decode_dxf(&bad).is_err());
        }
        assert!(decode_dxf("0\nEOF\n").is_err());
        assert!(
            encode_dxf(&[Line {
                a: Point { x: 0, y: 0 },
                b: Point { x: 0, y: 0 }
            }])
            .is_err()
        );
    }
    #[test]
    fn dxf_size_limit_is_inclusive_for_encode_decode_and_read() {
        let limit = 8 * 1024 * 1024;
        let short = Line::new(Point::new(0, 0).unwrap(), Point::new(1, 0).unwrap()).unwrap();
        let longer = Line::new(Point::new(0, 0).unwrap(), Point::new(640, 0).unwrap()).unwrap();
        let overhead = encode_dxf(&[]).unwrap().len();
        let entity_bytes = encode_dxf(&[short]).unwrap().len() - overhead;
        assert_eq!(
            encode_dxf(&[longer]).unwrap().len(),
            overhead + entity_bytes + 1
        );
        let count = (limit - overhead) / entity_bytes;
        let extra = (limit - overhead) % entity_bytes;
        let mut lines = vec![short; count];
        lines[..extra].fill(longer);
        let text = encode_dxf(&lines).unwrap();
        assert_eq!(text.len(), limit);
        assert_eq!(decode_dxf(&text).unwrap(), lines);
        let path = std::env::temp_dir().join(format!("draft-size-boundary-{}", std::process::id()));
        write_new(&path, text.as_bytes()).unwrap();
        let reopened = read_dxf(&path);
        std::fs::remove_file(&path).unwrap();
        assert_eq!(reopened.unwrap(), lines);

        // One additional decimal digit makes a valid LINE DXF exactly one byte too big.
        lines[extra] = longer;
        let oversized = text.replacen("11\n0.015625", "11\n10.000000", 1);
        assert_eq!(oversized.len(), limit + 1);
        assert_eq!(
            decode_dxf(&oversized).err().as_deref(),
            Some("DXF exceeds 8 MiB limit")
        );
        write_new(&path, oversized.as_bytes()).unwrap();
        let rejected = read_dxf(&path);
        std::fs::remove_file(path).unwrap();
        assert_eq!(rejected.err().as_deref(), Some("DXF exceeds 8 MiB limit"));
        assert_eq!(
            encode_dxf(&lines).err().as_deref(),
            Some("DXF exceeds 8 MiB limit")
        );
    }
    #[test]
    fn write_never_clobbers() {
        let path = std::env::temp_dir().join(format!("draft-write-test-{}", std::process::id()));
        let _ = std::fs::remove_file(&path);
        write_new(&path, b"first").unwrap();
        assert!(write_new(&path, b"second").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"first");
        std::fs::remove_file(path).unwrap();
    }
}
