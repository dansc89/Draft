use draft::{Drawing, Line, Point, parse_inches};
use draft::{
    editor::{orthogonal, replace_drawing, snap},
    pdf::encode_pdf,
    persistence::{decimal_inches, encode_dxf, read_dxf, write_new},
};
use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};
use std::path::Path;

pub struct DraftApp {
    drawing: Drawing,
    coordinates: [String; 4],
    path: String,
    error: String,
    status: String,
    discard: bool,
    ortho: bool,
    grid: i64,
    anchor: Option<Point>,
    zoom: f32,
    pan: Vec2,
    closing: bool,
    allow_close: bool,
}
impl Default for DraftApp {
    fn default() -> Self {
        Self {
            drawing: Drawing::default(),
            coordinates: ["0".into(), "0".into(), "12'".into(), "0".into()],
            path: "drawing.dxf".into(),
            error: String::new(),
            status: "Ready".into(),
            discard: false,
            ortho: true,
            grid: 64 * 12,
            anchor: None,
            zoom: 4.0,
            pan: Vec2::ZERO,
            closing: false,
            allow_close: false,
        }
    }
}
impl DraftApp {
    fn report(&mut self, result: Result<(), String>, success: &str) {
        match result {
            Ok(()) => {
                self.error.clear();
                self.status = success.into();
            }
            Err(e) => self.error = e,
        }
    }
    fn add_exact(&mut self) -> Result<(), String> {
        let a = Point::new(
            parse_inches(&self.coordinates[0])?,
            parse_inches(&self.coordinates[1])?,
        )?;
        let b = Point::new(
            parse_inches(&self.coordinates[2])?,
            parse_inches(&self.coordinates[3])?,
        )?;
        self.drawing.add(Line::new(a, b)?)?;
        self.anchor = None;
        Ok(())
    }
    fn save_dxf(&mut self) -> Result<(), String> {
        let text = encode_dxf(self.drawing.lines())?;
        write_new(Path::new(&self.path), text.as_bytes())?;
        self.drawing.mark_saved();
        Ok(())
    }
    fn open_dxf(&mut self) -> Result<(), String> {
        if self.drawing.dirty() && !self.discard {
            return Err(
                "Unsaved drawing: save it or explicitly enable discard before Open DXF".into(),
            );
        }
        let lines = read_dxf(Path::new(&self.path))?;
        replace_drawing(&mut self.drawing, lines, self.discard)?;
        self.discard = false;
        self.anchor = None;
        self.pan = Vec2::ZERO;
        Ok(())
    }
    fn export_pdf(&self) -> Result<(), String> {
        let pdf = encode_pdf(self.drawing.lines())?;
        let path = Path::new(&self.path).with_extension("pdf");
        write_new(&path, &pdf)
    }
    fn render(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        if ctx.input(|i| i.viewport().close_requested())
            && self.drawing.dirty()
            && !self.allow_close
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.closing = true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.anchor = None;
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
            if ctx.input(|i| i.modifiers.shift) {
                self.drawing.redo();
            } else {
                self.drawing.undo();
            }
            self.anchor = None;
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Y)) {
            self.drawing.redo();
            self.anchor = None;
        }
        egui::Panel::top("toolbar").show(root, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Draft");
                ui.label(if self.drawing.dirty() {
                    "● Unsaved"
                } else {
                    "Saved / unchanged"
                });
                if ui.button("Undo").clicked() {
                    self.drawing.undo();
                    self.anchor = None;
                }
                if ui.button("Redo").clicked() {
                    self.drawing.redo();
                    self.anchor = None;
                }
                ui.checkbox(&mut self.ortho, "Orthogonal cursor");
                egui::ComboBox::from_id_salt("grid")
                    .selected_text(format!("Grid {} in", decimal_inches(self.grid)))
                    .show_ui(ui, |ui| {
                        for (label, ticks) in
                            [("1 ft", 768), ("6 in", 384), ("1 in", 64), ("1/64 in", 1)]
                        {
                            ui.selectable_value(&mut self.grid, ticks, label);
                        }
                    });
                if ui.button("Reset view").clicked() {
                    self.zoom = 4.0;
                    self.pan = Vec2::ZERO;
                }
            });
            ui.horizontal(|ui| {
                ui.label("File path");
                ui.add(egui::TextEdit::singleline(&mut self.path).desired_width(360.0));
                if ui.button("Save DXF").clicked() {
                    let r = self.save_dxf();
                    self.report(r, "DXF saved to a new file");
                }
                if ui.button("Open DXF").clicked() {
                    let r = self.open_dxf();
                    self.report(r, "DXF opened");
                }
                if ui.button("Export PDF").clicked() {
                    let r = self.export_pdf();
                    self.report(r, "Vector PDF exported (path with .pdf extension)");
                }
            });
            ui.checkbox(
                &mut self.discard,
                "Discard unsaved drawing when opening (explicit consent)",
            );
        });
        egui::Panel::left("coordinates").exact_size(245.0).resizable(false).show(root,|ui| {
            ui.heading("Exact line");
            ui.label("Absolute coordinates, inches unless marked with ' for feet.");
            for (i,label) in ["Start X", "Start Y", "End X", "End Y"].iter().enumerate() {
                ui.label(*label); ui.text_edit_singleline(&mut self.coordinates[i]);
            }
            if ui.button("Add exact line").clicked() { let r = self.add_exact(); self.report(r,"Exact line added"); }
            ui.label("Examples: 12' 3 1/2\"\n1/64\"   -6.125\nExact resolution: 1/64 inch\nBounds: ±100,000 ft");
            ui.separator();
            ui.label("Click start and end on canvas. Cursor snaps to selected grid. Orthogonal mode applies only to cursor drawing; exact entries may be diagonal.");
            ui.label("Scroll: zoom\nMiddle-drag: pan\nEsc / right-click: cancel line\nCtrl+Z / Ctrl+Y: undo / redo");
            ui.separator();
            ui.label(format!("{} lines",self.drawing.lines().len()));
            ui.label("PDF: Letter landscape\n1:48 = 1/4 inch per foot\nHalf-inch margins\nNo fit-to-page. Print at 100%.\nNew filenames only; no autosave.");
        });
        egui::Panel::bottom("status").show(root, |ui| {
            if !self.error.is_empty() {
                ui.colored_label(Color32::LIGHT_RED, &self.error);
            }
            ui.label(&self.status);
        });
        egui::CentralPanel::default().show(root, |ui| self.canvas(ui));
        if self.closing {
            egui::Window::new("Unsaved drawing")
                .collapsible(false)
                .resizable(false)
                .show(&ctx, |ui| {
                    ui.label("Close and discard unsaved lines? No shutdown save is performed.");
                    if ui.button("Keep drafting").clicked() {
                        self.closing = false;
                    }
                    if ui.button("Discard and close").clicked() {
                        self.allow_close = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
        }
    }
    fn canvas(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, Color32::from_rgb(17, 22, 29));
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            self.zoom = (self.zoom * (scroll * 0.002).exp()).clamp(0.05, 64.0);
        }
        if response.dragged_by(egui::PointerButton::Middle) {
            self.pan += ui.input(|i| i.pointer.delta());
        }
        let origin = Pos2::new(
            rect.left() + 40.0 + self.pan.x,
            rect.bottom() - 40.0 + self.pan.y,
        );
        let screen = |p: Point| {
            Pos2::new(
                origin.x + p.x as f32 / 64.0 * self.zoom,
                origin.y - p.y as f32 / 64.0 * self.zoom,
            )
        };
        let spacing = self.grid as f32 / 64.0 * self.zoom;
        // Coarsen only display grid when zoomed out; cursor snap remains exact.
        let visible_spacing = spacing * (12.0 / spacing).ceil().max(1.0);
        for axis in 0..2 {
            let (lo, hi, o) = if axis == 0 {
                (rect.left(), rect.right(), origin.x)
            } else {
                (rect.top(), rect.bottom(), origin.y)
            };
            let first = ((lo - o) / visible_spacing).ceil();
            for index in 0..=((hi - lo) / visible_spacing).ceil() as usize {
                let v = o + (first + index as f32) * visible_spacing;
                let ends = if axis == 0 {
                    [Pos2::new(v, rect.top()), Pos2::new(v, rect.bottom())]
                } else {
                    [Pos2::new(rect.left(), v), Pos2::new(rect.right(), v)]
                };
                painter.line_segment(ends, Stroke::new(1.0, Color32::from_rgb(35, 43, 53)));
            }
        }
        painter.line_segment(
            [
                Pos2::new(rect.left(), origin.y),
                Pos2::new(rect.right(), origin.y),
            ],
            Stroke::new(1.0, Color32::from_rgb(88, 62, 63)),
        );
        painter.line_segment(
            [
                Pos2::new(origin.x, rect.top()),
                Pos2::new(origin.x, rect.bottom()),
            ],
            Stroke::new(1.0, Color32::from_rgb(56, 83, 74)),
        );
        for line in self.drawing.lines() {
            painter.line_segment(
                [screen(line.a), screen(line.b)],
                Stroke::new(1.8, Color32::from_rgb(224, 230, 236)),
            );
        }
        if response.secondary_clicked() {
            self.anchor = None;
        }
        if let Some(cursor) = response.hover_pos() {
            let snapped = snap(
                f64::from((cursor.x - origin.x) / self.zoom),
                f64::from((origin.y - cursor.y) / self.zoom),
                self.grid,
            );
            match snapped {
                Ok(mut point) => {
                    if let Some(start) = self.anchor {
                        if self.ortho {
                            point = orthogonal(start, point);
                        }
                        painter.line_segment(
                            [screen(start), screen(point)],
                            Stroke::new(1.5, Color32::from_rgb(105, 196, 245)),
                        );
                    }
                    let at = screen(point);
                    painter.circle_stroke(at, 4.0, Stroke::new(1.0, Color32::LIGHT_BLUE));
                    self.status = format!(
                        "X {} in  Y {} in{}",
                        decimal_inches(point.x),
                        decimal_inches(point.y),
                        if self.anchor.is_some() {
                            " — click endpoint"
                        } else {
                            " — click start"
                        }
                    );
                    if response.clicked() {
                        if let Some(start) = self.anchor {
                            match Line::new(start, point).and_then(|line| self.drawing.add(line)) {
                                Ok(()) => {
                                    self.anchor = None;
                                    self.error.clear();
                                }
                                Err(e) => self.error = e,
                            }
                        } else {
                            self.anchor = Some(point);
                            self.coordinates[0] = decimal_inches(point.x);
                            self.coordinates[1] = decimal_inches(point.y);
                        }
                    }
                }
                Err(e) => {
                    if response.clicked() {
                        self.error = e;
                    }
                }
            }
        }
    }
}
impl eframe::App for DraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.render(ui);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn frame(app: &mut DraftApp, ctx: &egui::Context, events: Vec<egui::Event>) -> egui::Rect {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1200.0, 800.0),
            )),
            events,
            ..Default::default()
        };
        let output = ctx.run_ui(input, |ui| app.render(ui));
        let rect = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::epaint::Shape::Rect(r) if r.fill == Color32::from_rgb(17, 22, 29) => {
                    Some(r.rect)
                }
                _ => None,
            })
            .expect("Canvas is painted");
        output.drop_without_applying_deltas();
        rect
    }
    fn click(app: &mut DraftApp, ctx: &egui::Context, pos: Pos2) {
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
            );
        }
    }
    #[test]
    fn canvas_clicks_create_snapped_orthogonal_line() {
        let mut app = DraftApp::default();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        let rect = frame(&mut app, &ctx, vec![]);
        assert!(
            rect.width() > 900.0 && rect.height() > 600.0,
            "Dominant canvas: {rect:?}"
        );
        let origin = Pos2::new(rect.left() + 40.0, rect.bottom() - 40.0);
        click(&mut app, &ctx, origin + egui::vec2(48.0, -48.0));
        assert!(app.anchor.is_some());
        click(&mut app, &ctx, origin + egui::vec2(144.0, -96.0));
        assert_eq!(
            app.drawing.lines(),
            &[Line::new(
                Point::new(768, 768).unwrap(),
                Point::new(2304, 768).unwrap()
            )
            .unwrap()]
        );
        assert!(app.anchor.is_none());
    }
    #[test]
    fn oversized_dxf_save_preserves_dirty_drawing_without_creating_file() {
        let dir = std::env::temp_dir().join(format!("draft-ui-size-test-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("oversized.dxf");
        let mut app = DraftApp {
            path: path.to_str().unwrap().into(),
            ..Default::default()
        };
        for x in 0..130_000 {
            app.drawing
                .add(Line::new(Point::new(x, 0).unwrap(), Point::new(x + 1, 64).unwrap()).unwrap())
                .unwrap();
        }
        assert!(app.drawing.dirty());
        let result = app.save_dxf();
        let file_created = path.exists();
        let dirty = app.drawing.dirty();
        println!("save result: {result:?}; file_created: {file_created}; dirty: {dirty}");
        std::fs::remove_dir_all(dir).unwrap();
        assert_eq!(result, Err("DXF exceeds 8 MiB limit".into()));
        assert!(!file_created, "Oversized save must not create a file");
        assert!(dirty, "Failed save must leave the drawing dirty");
        assert_eq!(app.drawing.lines().len(), 130_000);
    }
    #[test]
    fn file_controls_preserve_unsaved_state_on_failures() {
        let dir = std::env::temp_dir().join(format!("draft-ui-test-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let mut app = DraftApp {
            path: dir.join("drawing.dxf").to_str().unwrap().into(),
            ..Default::default()
        };
        app.add_exact().unwrap();
        assert!(app.drawing.dirty());
        assert!(app.open_dxf().is_err());
        app.save_dxf().unwrap();
        assert!(!app.drawing.dirty());
        app.coordinates[3] = "12'".into();
        app.add_exact().unwrap();
        assert!(app.save_dxf().is_err());
        assert!(app.drawing.dirty());
        assert!(app.open_dxf().is_err());
        assert_eq!(app.drawing.lines().len(), 2);
        app.discard = true;
        app.open_dxf().unwrap();
        assert_eq!(app.drawing.lines().len(), 1);
        assert!(!app.discard);
        app.export_pdf().unwrap();
        assert!(app.export_pdf().is_err());
        app.add_exact().unwrap();
        app.discard = true;
        app.path = dir.join("bad.dxf").to_str().unwrap().into();
        std::fs::write(&app.path, b"bad").unwrap();
        assert!(app.open_dxf().is_err());
        assert_eq!(app.drawing.lines().len(), 2);
        assert!(app.drawing.dirty());
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn exact_entry_reaches_canvas_without_native_window() {
        let mut app = DraftApp::default();
        app.add_exact().unwrap();
        assert_eq!(app.drawing.lines().len(), 1);
        assert_eq!(app.drawing.lines()[0].b.x, 9216);
        app.coordinates[2] = "0.1".into();
        assert!(app.add_exact().is_err());
        assert_eq!(app.drawing.lines().len(), 1);
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200.0, 800.0),
            )),
            ..Default::default()
        };
        let output = ctx.run_ui(input, |ui| app.render(ui));
        assert!(
            output.shapes.len() > 20,
            "UI must paint controls and canvas"
        );
        output.drop_without_applying_deltas();
    }
}
