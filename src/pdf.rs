use crate::{Line, Result};
/// Letter landscape, half-inch margins, 0.5 pt stroke. Geometry is translated
/// to the lower-left printable margin; its scale is NEVER changed.
pub fn encode_pdf(lines: &[Line]) -> Result<Vec<u8>> {
    if lines.is_empty() {
        return Err("Cannot export an empty drawing".into());
    }
    for l in lines {
        Line::new(l.a, l.b)?;
    }
    let points: Vec<_> = lines.iter().flat_map(|l| [l.a, l.b]).collect();
    let min_x = points.iter().map(|p| p.x).min().ok_or("Empty drawing")?;
    let max_x = points.iter().map(|p| p.x).max().ok_or("Empty drawing")?;
    let min_y = points.iter().map(|p| p.y).min().ok_or("Empty drawing")?;
    let max_y = points.iter().map(|p| p.y).max().ok_or("Empty drawing")?;
    // 72 points per inch / 48 = 3/2 points per real inch = 3/128 per tick.
    if (max_x - min_x) * 3 > 7195 * 128 / 10 || (max_y - min_y) * 3 > 5395 * 128 / 10 {
        return Err("Drawing would clip at 1:48 on Letter landscape (half-inch margins). No fit-to-page is performed.".into());
    }
    fn coordinate(ticks: i64) -> String {
        let n = ticks * 3 + 4640; // 36.25 pt: margin + half stroke
        format!("{}.{:07}", n / 128, n % 128 * 78125)
    }
    let mut stream = String::from("0 G\n0.5 w\n");
    for l in lines {
        stream.push_str(&format!(
            "{} {} m {} {} l S\n",
            coordinate(l.a.x - min_x),
            coordinate(l.a.y - min_y),
            coordinate(l.b.x - min_x),
            coordinate(l.b.y - min_y)
        ));
    }
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 792 612] /Resources << >> /Contents 4 0 R >>"
            .to_owned(),
        format!(
            "<< /Length {} >>\nstream\n{}endstream",
            stream.len(),
            stream
        ),
    ];
    let mut pdf = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (i, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.push_str(&format!("{} 0 obj\n{}\nendobj\n", i + 1, object));
    }
    let xref = pdf.len();
    pdf.push_str("xref\n0 5\n0000000000 65535 f \n");
    for offset in offsets {
        pdf.push_str(&format!("{offset:010} 00000 n \n"));
    }
    pdf.push_str(&format!(
        "trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n"
    ));
    Ok(pdf.into_bytes())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point;
    #[test]
    fn physical_scale_vector_and_page() {
        let lines =
            [Line::new(Point::new(0, 0).unwrap(), Point::new(9216, 6144).unwrap()).unwrap()];
        let pdf = String::from_utf8(encode_pdf(&lines).unwrap()).unwrap();
        assert!(pdf.starts_with("%PDF-1.4"));
        assert!(pdf.contains("/MediaBox [0 0 792 612]"));
        assert!(pdf.contains("36.2500000 36.2500000 m 252.2500000 180.2500000 l S"));
        assert!(!pdf.contains("/Image"));
        let offset: usize = pdf
            .split("startxref\n")
            .nth(1)
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert!(pdf[offset..].starts_with("xref\n"));
        assert!(encode_pdf(&[]).is_err());
        let oversized =
            [Line::new(Point::new(0, 0).unwrap(), Point::new(64 * 480, 0).unwrap()).unwrap()];
        assert!(encode_pdf(&oversized).is_err());
        let high =
            [Line::new(Point::new(0, 0).unwrap(), Point::new(0, 64 * 360).unwrap()).unwrap()];
        assert!(encode_pdf(&high).is_err());
        let invalid = [Line {
            a: Point { x: 0, y: 0 },
            b: Point { x: i64::MAX, y: 0 },
        }];
        assert!(encode_pdf(&invalid).is_err());
    }
}
