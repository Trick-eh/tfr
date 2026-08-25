use eframe::egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};

pub fn segmented_progress_bar(
    ui: &mut Ui,
    progress: f32,
    num_segments: usize,
    segment_width: f32,
    segment_height: f32,
    segment_gap: f32,
    corner_radius: f32,
) -> Response {
    let total_width =
        (num_segments as f32 * segment_width) + ((num_segments - 1) as f32 * segment_gap);

    let desired_size = Vec2::new(total_width, segment_height);

    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        let orange_active = Color32::from_rgb(255, 140, 0);
        let dark_inactive = Color32::from_rgb(25, 25, 25);

        let active_count = ((progress * num_segments as f32).round() as usize).min(num_segments);

        for i in 0..num_segments {
            let x_left = rect.min.x + i as f32 * (segment_width + segment_gap);
            let seg_rect = Rect::from_min_size(
                Pos2::new(x_left, rect.min.y),
                Vec2::new(segment_width, segment_height),
            );

            if i < active_count {
                painter.rect_filled(seg_rect, corner_radius, orange_active);
            } else {
                painter.rect_filled(seg_rect, corner_radius, dark_inactive);
                painter.rect_stroke(seg_rect, corner_radius, Stroke::new(1.0, dark_inactive));
            }
        }
    }

    response
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum ThemePreset {
    OledBlack,
    DeepAmber,
    WarmSepia,
}

impl ThemePreset {
    pub fn bg_color(&self) -> Color32 {
        match self {
            ThemePreset::OledBlack => Color32::BLACK,
            ThemePreset::DeepAmber => Color32::from_rgb(12, 10, 8),
            ThemePreset::WarmSepia => Color32::from_rgb(22, 18, 14),
        }
    }

    pub fn text_color(&self) -> Color32 {
        match self {
            ThemePreset::OledBlack => Color32::WHITE,
            ThemePreset::DeepAmber => Color32::from_rgb(255, 190, 120),
            ThemePreset::WarmSepia => Color32::from_rgb(230, 210, 180),
        }
    }

    pub fn anchor_color(&self) -> Color32 {
        match self {
            ThemePreset::OledBlack => Color32::from_rgb(255, 60, 60),
            ThemePreset::DeepAmber => Color32::from_rgb(255, 120, 0),
            ThemePreset::WarmSepia => Color32::from_rgb(210, 80, 40),
        }
    }
}
