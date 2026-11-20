mod sort;

use eframe::egui;
use image::RgbImage;
use sort::{Settings, SortKey};

struct App {
    original: RgbImage,
    sorted: RgbImage,
    settings: Settings,
    /// Settings the current `sorted` image was made with.
    rendered: Option<Settings>,
    /// Lines only depend on angle and image size, so keep them between
    /// renders instead of rebuilding them on every slider move.
    lines: Vec<Vec<usize>>,
    lines_angle: Option<f32>,
    original_tex: egui::TextureHandle,
    sorted_tex: egui::TextureHandle,
    show_original: bool,
    status: String,
}

fn to_texture(ctx: &egui::Context, name: &str, img: &RgbImage) -> egui::TextureHandle {
    let size = [img.width() as usize, img.height() as usize];
    let color = egui::ColorImage::from_rgb(size, img.as_raw());
    ctx.load_texture(name, color, egui::TextureOptions::LINEAR)
}

impl App {
    fn new(ctx: &egui::Context, img: RgbImage) -> Self {
        App {
            original_tex: to_texture(ctx, "original", &img),
            sorted_tex: to_texture(ctx, "sorted", &img),
            sorted: img.clone(),
            original: img,
            settings: Settings::default(),
            rendered: None,
            lines: Vec::new(),
            lines_angle: None,
            show_original: false,
            status: String::new(),
        }
    }

    fn set_image(&mut self, ctx: &egui::Context, img: RgbImage) {
        self.original_tex = to_texture(ctx, "original", &img);
        self.original = img;
        self.lines_angle = None; // size changed, lines must be rebuilt
        self.rendered = None;
    }

    fn render(&mut self, ctx: &egui::Context) {
        let s = self.settings;
        if self.lines_angle != Some(s.angle) {
            let (w, h) = self.original.dimensions();
            self.lines = sort::lines_at_angle(w, h, s.angle);
            self.lines_angle = Some(s.angle)
        }

        self.sorted = self.original.clone();
        sort::sort_image(&mut self.sorted, &self.lines, &s);
        self.sorted_tex = to_texture(ctx, "sorted", &self.sorted);
        self.rendered = Some(s);
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        let s = &mut self.settings;
        ui.heading("Pixel sort");
        ui.add_space(8.0);

        ui.add(
            egui::Slider::new(&mut s.angle, 0.0..=360.0)
                .text("angle")
                .suffix("°"),
        );
        ui.add(egui::Slider::new(&mut s.low, 0..=255).text("low"));
        ui.add(egui::Slider::new(&mut s.high, 0..=255).text("high"));

        egui::ComboBox::from_label("sort by")
            .selected_text(format!("{:?}", s.key))
            .show_ui(ui, |ui| {
                for key in SortKey::ALL {
                    ui.selectable_value(&mut s.key, key, format!("{key:?}"));
                }
            });
        ui.checkbox(&mut s.reverse, "reverse");

        ui.separator();
        ui.checkbox(&mut self.show_original, "show original");
        if ui.button("Reset").clicked() {
            self.settings = Settings::default();
        }
        if ui.button("Save PNG").clicked() {
            let path = concat!(env!("CARGO_MANIFEST_DIR"), "/out.png");
            self.status = match self.sorted.save(path) {
                Ok(()) => format!("Saved to {path}"),
                Err(e) => format!("Save failed: {e}"),
            };
        }

        ui.separator();
        ui.label("Drop an image on the window to load it.");
        ui.label(&self.status);
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        let dropped = ctx.input(|i| i.raw.dropped_files.first().map(|f| f.path().to_path_buf()));
        if let Some(path) = dropped {
            match image::open(&path) {
                Ok(img) => {
                    self.set_image(&ctx, img.to_rgb8());
                    self.status = format!("Loaded {}", path.display());
                }
                Err(e) => self.status = format!("Can't open {}: {e}", path.display()),
            }
        }

        egui::Panel::left("controls").show(ui, |ui| self.controls(ui));

        // Only re-sort when something actually changed.
        if self.rendered != Some(self.settings) {
            self.render(&ctx);
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let tex = if self.show_original {
                &self.original_tex
            } else {
                &self.sorted_tex
            };
            ui.centered_and_justified(|ui| ui.add(egui::Image::new(tex).shrink_to_fit()));
        });
    }
}

fn main() -> eframe::Result {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mona-lisa.jpeg");
    let img = image::open(path).unwrap().to_rgb8();

    eframe::run_native(
        "Pixel sort",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(&cc.egui_ctx, img)))),
    )
}
