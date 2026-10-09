use draft::pdf::encode_pdf;
use draft::persistence::{decimal_inches, decode_dxf, encode_dxf, write_new};
use draft::{Line, MAX_TICKS, Point, parse_inches};

#[test]
fn every_fractional_tick_roundtrips_without_float_conversion() {
    for ticks in -128..=128 {
        assert_eq!(parse_inches(&decimal_inches(ticks)).unwrap(), ticks);
        let line = Line::new(
            Point::new(ticks, -ticks).unwrap(),
            Point::new(ticks + 1, -ticks + 1).unwrap(),
        )
        .unwrap();
        assert_eq!(
            decode_dxf(&encode_dxf(&[line]).unwrap()).unwrap(),
            vec![line]
        );
    }
    for ticks in [-MAX_TICKS, MAX_TICKS] {
        assert_eq!(parse_inches(&decimal_inches(ticks)).unwrap(), ticks);
    }
}

#[test]
fn pdf_translation_preserves_one_tick_scale_and_exact_fit_boundary() {
    let line = Line::new(Point::new(-64, -64).unwrap(), Point::new(-63, -63).unwrap()).unwrap();
    let text = String::from_utf8(encode_pdf(&[line]).unwrap()).unwrap();
    assert!(text.contains("36.2500000 36.2500000 m 36.2734375 36.2734375 l S"));
    // Largest tick width whose full 0.5-point stroke fits the printable area.
    let max_width = 7195 * 128 / 10 / 3;
    let max_height = 5395 * 128 / 10 / 3;
    let line = |x, y| Line::new(Point::new(0, 0).unwrap(), Point::new(x, y).unwrap()).unwrap();
    assert!(encode_pdf(&[line(max_width, max_height)]).is_ok());
    assert!(encode_pdf(&[line(max_width + 1, max_height)]).is_err());
    assert!(encode_pdf(&[line(max_width, max_height + 1)]).is_err());
}

#[cfg(unix)]
#[test]
fn writes_refuse_existing_symlinks_without_touching_target() {
    let dir = std::env::temp_dir().join(format!("draft-symlink-test-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let target = dir.join("target");
    std::fs::write(&target, b"original").unwrap();
    let link = dir.join("link");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(write_new(&link, b"overwrite").is_err());
    assert_eq!(std::fs::read(target).unwrap(), b"original");
    std::fs::remove_dir_all(dir).unwrap();
}
