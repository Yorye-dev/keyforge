use crate::domain::layout::Layout;
use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};

pub fn thumbnail(ui: &mut egui::Ui, layout: &Layout) {
    let (response, painter) = ui.allocate_painter(Vec2::new(200.0, 76.0), egui::Sense::hover());
    let (min, max) = layout.bounds();
    let extent = Vec2::new(max[0] - min[0], max[1] - min[1]);
    let scale = (196.0 / extent.x).min(70.0 / extent.y);
    let origin = response.rect.min;
    for key in &layout.keys {
        let points = key.geometry.corners().map(|p| {
            origin + Vec2::new(2.0 + (p[0] - min[0]) * scale, 2.0 + (p[1] - min[1]) * scale)
        });
        painter.add(egui::Shape::convex_polygon(
            points.to_vec(),
            Color32::from_rgb(65, 100, 115),
            Stroke::new(0.5, Color32::LIGHT_BLUE),
        ));
    }
}

pub fn draw(
    ui: &mut egui::Ui,
    layout: &Layout,
    zoom: f32,
    pan: &mut Vec2,
    selected: &mut Option<String>,
    labels: &std::collections::BTreeMap<String, String>,
    changed: &std::collections::BTreeSet<String>,
) {
    let (response, painter) =
        ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
    if response.dragged() {
        *pan += response.drag_delta();
    }
    let (min, max) = layout.bounds();
    let extent = Vec2::new(max[0] - min[0], max[1] - min[1]);
    let scale = ((response.rect.width() - 48.0) / extent.x)
        .min((response.rect.height() - 48.0) / extent.y)
        .max(1.0)
        * zoom;
    let origin =
        response.rect.center() - extent * scale / 2.0 + *pan - Vec2::new(min[0], min[1]) * scale;
    let transform = |p: [f32; 2]| origin + Vec2::new(p[0], p[1]) * scale;
    for key in &layout.keys {
        let points: Vec<Pos2> = key.geometry.corners().into_iter().map(transform).collect();
        let hovered = response.hover_pos().is_some_and(|p| contains(&points, p));
        if hovered && response.clicked() {
            *selected = Some(key.id.clone());
        }
        let active = selected.as_ref() == Some(&key.id);
        let half = layout
            .halves
            .iter()
            .position(|h| h == &key.half)
            .unwrap_or(0);
        let fill = if active {
            Color32::from_rgb(30, 130, 145)
        } else if hovered {
            Color32::from_rgb(69, 82, 105)
        } else if half % 2 == 0 {
            Color32::from_rgb(39, 55, 73)
        } else {
            Color32::from_rgb(57, 46, 78)
        };
        painter.add(egui::Shape::convex_polygon(
            points,
            fill,
            Stroke::new(
                if active { 2.0 } else { 1.0 },
                if changed.contains(&key.id) {
                    Color32::from_rgb(235, 180, 75)
                } else {
                    Color32::from_gray(105)
                },
            ),
        ));
        let g = &key.geometry;
        let label = labels.get(&key.id).unwrap_or(&key.label);
        let center = transform([g.x + g.width / 2.0, g.y + g.height / 2.0]);
        let text_rect =
            egui::Rect::from_center_size(center, Vec2::new(g.width, g.height) * scale * 0.9);
        let longest_line = label
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let font_size = (scale * 0.28)
            .min(text_rect.width() / (longest_line as f32 * 0.6))
            .clamp(6.0, 24.0);
        painter
            .with_clip_rect(text_rect.intersect(response.rect))
            .text(
                center,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(font_size),
                Color32::WHITE,
            );
        if hovered {
            response
                .clone()
                .on_hover_text(format!("{}: {}", key.id, label.replace('\n', " ")));
        }
    }
    for display in &layout.displays {
        painter.add(egui::Shape::convex_polygon(
            display
                .geometry
                .corners()
                .into_iter()
                .map(transform)
                .collect(),
            Color32::from_rgb(15, 30, 34),
            Stroke::new(1.0, Color32::from_rgb(75, 190, 170)),
        ));
        let g = &display.geometry;
        painter.text(
            transform([g.x + g.width / 2.0, g.y + g.height / 2.0]),
            egui::Align2::CENTER_CENTER,
            "IPS\nDemo",
            egui::FontId::proportional(11.0),
            Color32::LIGHT_GREEN,
        );
    }
}

fn contains(points: &[Pos2], p: Pos2) -> bool {
    (0..points.len()).all(|i| {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x) >= 0.0
    })
}
