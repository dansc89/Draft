mod ui;
use draft::{Drawing, Line, Point, parse_inches};
use draft::{
    pdf::encode_pdf,
    persistence::{encode_dxf, write_new},
};
use std::path::Path;
fn demo(dir: &Path) -> Result<(), String> {
    let w = parse_inches("12'")?;
    let h = parse_inches("8'")?;
    let points = [
        Point::new(0, 0)?,
        Point::new(w, 0)?,
        Point::new(w, h)?,
        Point::new(0, h)?,
    ];
    let mut drawing = Drawing::default();
    for i in 0..4 {
        drawing.add(Line::new(points[i], points[(i + 1) % 4])?)?;
    }
    let dxf = encode_dxf(drawing.lines())?;
    let pdf = encode_pdf(drawing.lines())?;
    std::fs::create_dir(dir).map_err(|e| format!("Demo requires a new output directory: {e}"))?;
    write_new(&dir.join("rectangle.dxf"), dxf.as_bytes())?;
    write_new(&dir.join("rectangle.pdf"), &pdf)?;
    println!(
        "Created 12-foot × 8-foot rectangle: {} and {} (Letter landscape, 1:48, vector 3 × 2 inches)",
        dir.join("rectangle.dxf").display(),
        dir.join("rectangle.pdf").display()
    );
    Ok(())
}
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let result = if args.len() == 2 && args[0] == "--demo" {
        demo(Path::new(&args[1]))
    } else if args.is_empty() {
        let options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 800.0])
                .with_min_inner_size([1000.0, 720.0]),
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        };
        eframe::run_native(
            "Draft — exact imperial line drafting",
            options,
            Box::new(|cc| {
                cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());
                Ok(Box::new(ui::DraftApp::default()))
            }),
        )
        .map_err(|e| e.to_string())
    } else {
        Err("Usage: draft [--demo <new-output-dir>]".into())
    };
    if let Err(error) = result {
        eprintln!("Draft: {error}");
        std::process::exit(1);
    }
}
