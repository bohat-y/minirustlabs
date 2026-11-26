use eframe::{
    egui::{self, Color32, Pos2, Rect, Sense, Stroke, pos2},
    epaint::color,
};

const POINT_RADIUS: f32 = 6.0;
const POINT_COLOR: Color32 = Color32::WHITE;
const HOT_COLOR: Color32 = Color32::RED;

struct App {
    // control points
    points: Vec<Pos2>,
    show_polygon: bool,
}

impl App {
    fn new() -> Self {
        App {
            points: vec![
                pos2(100.0, 400.0),
                pos2(250.0, 120.0),
                pos2(450.0, 450.0),
                pos2(650.0, 150.0),
            ],
            show_polygon: true,
        }
    }
    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Spline lab");
        ui.add_space(8.0);
        ui.checkbox(&mut self.show_polygon, "control polygons");

        ui.separator();
        ui.label(format!("{} points", self.points.len()));

        if ui.button("Reset points").clicked() {
            self.points = App::new().points;
        }
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
        let origin = response.rect.min.to_vec2();
        let bounds = Rect::from_min_size(Pos2::ZERO, response.rect.size());

        let mut hot = None;
        let mut remove = None;

        for (i, point) in self.points.iter_mut().enumerate() {
            let hit =
                Rect::from_center_size(*point + origin, egui::Vec2::splat(POINT_RADIUS * 3.0));
            let r = ui.interact(hit, response.id.with(i), Sense::click_and_drag());
            if r.dragged() {
                *point = bounds.clamp(*point + r.drag_delta());
            }
            if r.secondary_clicked() {
                remove = Some(i);
            }
            if r.hovered() || r.dragged() {
                hot = Some(i);
            }
        }

        if let Some(i) = remove {
            self.points.remove(i);
        }

        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            self.points.push(pos - origin);
        }

        let screen: Vec<Pos2> = self.points.iter().map(|p| *p + origin).collect();
        if self.show_polygon && screen.iter().len() >= 2 {
            painter.add(egui::Shape::line(
                screen.clone(),
                Stroke::new(1.0, Color32::GRAY),
            ));
        }
        for (i, point) in screen.iter().enumerate() {
            let color = if hot == Some(i) {
                HOT_COLOR
            } else {
                POINT_COLOR
            };
            painter.circle_filled(*point, POINT_RADIUS, color);
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::Panel::left("contwols").show(ui, |ui| self.controls(ui));
        egui::CentralPanel::default().show(ui, |ui| self.canvas(ui));
    }
}
fn main() -> eframe::Result {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native("spline lab", opts, Box::new(|_cc| Ok(Box::new(App::new()))))
}
