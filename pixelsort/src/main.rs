mod sort;

use eframe::egui;
use image::RgbImage;
use sort::{Settings, SortKey};

/// While dragging a slider, the image is shrunk so its longest side is at
/// most this many pixels. Sorting that is fast enough to keep up.
const PREVIEW_SIZE: u32 = 600;

/// An image plus the lines cut through it. Lines only depend on angle and
/// image size, so keep them between renders instead of rebuilding them on
/// every slider move.
struct Source {
    img: RgbImage,
    lines: Vec<Vec<usize>>,
    lines_angle: Option<f32>,
}

impl Source {
    fn new(img: RgbImage) -> Self {
        Source {
            img,
            lines: Vec::new(),
            lines_angle: None,
        }
    }

    fn sorted(&mut self, s: &Settings) -> RgbImage {
        if self.lines_angle != Some(s.angle) {
            let (w, h) = self.img.dimensions();
            self.lines = sort::lines_at_angle(w, h, s.angle);
            self.lines_angle = Some(s.angle);
        }

        let mut out = self.img.clone();
        sort::sort_image(&mut out, &self.lines, s);
        out
    }
}

/// A smaller copy of `img` for previews. Small images are just copied.
fn shrink(img: &RgbImage) -> RgbImage {
    let (w, h) = img.dimensions();
    let scale = PREVIEW_SIZE as f32 / w.max(h) as f32;
    if scale >= 1.0 {
        return img.clone();
    }
    let pw = ((w as f32 * scale) as u32).max(1);
    let ph = ((h as f32 * scale) as u32).max(1);
    image::imageops::thumbnail(img, pw, ph)
}

struct App {
    full: Source,
    preview: Source,
    /// Full-size result, kept for "Save PNG".
    sorted: RgbImage,
    settings: Settings,
    /// True while one of the sliders is being dragged.
    dragging: bool,
    /// What the shown image was made with: the settings, and whether it
    /// was the preview.
    rendered: Option<(Settings, bool)>,
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
            preview: Source::new(shrink(&img)),
            full: Source::new(img),
            settings: Settings::default(),
            dragging: false,
            rendered: None,
            show_original: false,
            status: String::new(),
        }
    }

    fn set_image(&mut self, ctx: &egui::Context, img: RgbImage) {
        self.original_tex = to_texture(ctx, "original", &img);
        self.preview = Source::new(shrink(&img));
        self.full = Source::new(img);
        self.rendered = None;
    }

    fn render(&mut self, ctx: &egui::Context) {
        let s = self.settings;
        if self.dragging {
            let small = self.preview.sorted(&s);
            self.sorted_tex = to_texture(ctx, "sorted", &small);
        } else {
            self.sorted = self.full.sorted(&s);
            self.sorted_tex = to_texture(ctx, "sorted", &self.sorted);
        }
        self.rendered = Some((s, self.dragging));
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        let s = &mut self.settings;
        ui.heading("Pixel sort");
        ui.add_space(8.0);

        let angle = ui.add(
            egui::Slider::new(&mut s.angle, 0.0..=360.0)
                .text("angle")
                .suffix("°"),
        );
        let low = ui.add(egui::Slider::new(&mut s.low, 0..=255).text("low"));
        let high = ui.add(egui::Slider::new(&mut s.high, 0..=255).text("high"));
        self.dragging = angle.dragged() || low.dragged() || high.dragged();

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

        let quit = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Q);
        if ctx.input_mut(|i| i.key_pressed(egui::Key::Escape) || i.consume_shortcut(&quit)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

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

        // Only re-sort when something changed: a setting, or a drag starting
        // or stopping (which switches between preview and full size).
        if self.rendered != Some((self.settings, self.dragging)) {
            self.render(&ctx);
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let tex = if self.show_original {
                &self.original_tex
            } else {
                &self.sorted_tex
            };
            // Tell egui the image is full size even when the texture is the
            // small preview, so the picture doesn't jump in size while dragging.
            let (w, h) = self.full.img.dimensions();
            let sized = egui::load::SizedTexture::new(tex.id(), [w as f32, h as f32]);
            ui.centered_and_justified(|ui| {
                ui.add(egui::Image::from_texture(sized).shrink_to_fit())
            });
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
