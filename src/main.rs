#![windows_subsystem = "windows"]

use eframe::egui::{self, FontId, Pos2};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tfr::{
    extract_text,
    persistence::SavedState,
    ui_additions::{ThemePreset, segmented_progress_bar},
};

fn main() -> Result<(), eframe::Error> {
    let icon = load_custom_icon();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 500.0])
            .with_active(true)
            .with_decorations(false)
            .with_icon(icon),

        ..Default::default()
    };

    eframe::run_native(
        "Trick's Fast Reader",
        options,
        Box::new(|_| Box::new(RsvpApp::default())),
    )
}

fn load_custom_icon() -> egui::IconData {
    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for _ in 0..(width * height) {
        rgba.push(255);
        rgba.push(140);
        rgba.push(0);
        rgba.push(255);
    }

    egui::IconData {
        rgba,
        width,
        height,
    }
}

struct RsvpApp {
    words: Vec<String>,
    current_path: Option<PathBuf>,
    current_index: usize,
    wpm: u32,
    is_playing: bool,
    last_tick: Instant,

    theme: ThemePreset,
    font_size: f32,
    last_mouse_move: Instant,
    mouse_pos_cache: Option<Pos2>,
}
impl Default for RsvpApp {
    fn default() -> Self {
        let saved = SavedState::load();

        let mut words = Vec::new();
        let mut loaded_path = None;

        if let Some(ref path) = saved.last_file_path
            && path.exists()
            && let Ok(text) = extract_text(path)
        {
            words = text.split_whitespace().map(|s| s.to_string()).collect();
            loaded_path = Some(path.clone());
        }

        let restored_index = if !words.is_empty() {
            saved.current_index.min(words.len() - 1)
        } else {
            0
        };

        Self {
            words,
            current_path: loaded_path,
            current_index: restored_index,
            wpm: saved.wpm,
            is_playing: false,
            last_tick: Instant::now(),

            theme: saved.last_theme,
            font_size: 54.0,
            last_mouse_move: Instant::now(),
            mouse_pos_cache: None,
        }
    }
}
impl RsvpApp {
    fn save_current_state(&self) {
        let state = SavedState {
            last_file_path: self.current_path.clone(),
            current_index: self.current_index,
            wpm: self.wpm,
            last_theme: self.theme,
        };
        state.save();
    }
    fn calculate_orp(word: &str) -> usize {
        let len = word.chars().count();
        if len <= 1 {
            0
        } else {
            ((len - 1) as f32 * 0.35).floor() as usize
        }
    }
    fn load_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Documents",
                &["txt", "md", "markdown", "pdf", "docx", "epub"],
            )
            .pick_file()
            && let Ok(text) = extract_text(&path)
        {
            self.words = text.split_whitespace().map(|s| s.to_string()).collect();
            self.current_path = Some(path);
            self.current_index = 0;
            self.is_playing = false;
            self.save_current_state();
        }
    }
    fn calculate_word_duration(&self, word: &str) -> Duration {
        let base_millis = 60_000.0 / self.wpm as f32;
        let clean_word = word.trim();

        let multiplier = if clean_word.ends_with(['.', '!', '?']) {
            2.25
        } else if clean_word.ends_with([',', ';', ':', '—', '-']) {
            1.50
        } else if clean_word.len() > 10 {
            1.20
        } else {
            1.00
        };

        Duration::from_secs_f32((base_millis * multiplier) / 1000.0)
    }
    fn estimated_time_remaining(&self) -> Duration {
        if self.words.is_empty() || self.current_index >= self.words.len() {
            return Duration::ZERO;
        }

        let remaining_words = &self.words[self.current_index..];
        let total_secs = remaining_words
            .iter()
            .map(|w| self.calculate_word_duration(w).as_secs_f32())
            .sum();

        Duration::from_secs_f32(total_secs)
    }
    fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let mins = total_secs / 60;
        let secs = total_secs % 60;

        format!("{:02}:{:02}", mins, secs)
    }
}
impl eframe::App for RsvpApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_current_state();
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.is_playing = !self.is_playing;
                self.last_tick = Instant::now();
                self.save_current_state();
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                self.wpm = (self.wpm + 25).min(1500);
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                self.wpm = self.wpm.saturating_sub(25).max(25);
            }
            if !self.is_playing && !self.words.is_empty() {
                if i.key_pressed(egui::Key::ArrowLeft) {
                    self.current_index = self.current_index.saturating_sub(1);
                }
                if i.key_pressed(egui::Key::ArrowRight) {
                    self.current_index = (self.current_index + 1).min(self.words.len() - 1);
                }
            }
        });

        if let Some(pos) = ctx.pointer_latest_pos()
            && self.mouse_pos_cache != Some(pos)
        {
            self.mouse_pos_cache = Some(pos);
            self.last_mouse_move = Instant::now()
        }

        let show_controls = !self.is_playing || self.last_mouse_move.elapsed().as_secs_f32() < 2.0;
        let ui_opacity = if show_controls { 1.0 } else { 0.0 };

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = self.theme.bg_color();
        visuals.window_fill = self.theme.bg_color();
        visuals.override_text_color = Some(self.theme.text_color());
        ctx.set_visuals(visuals);

        if self.is_playing
            && !self.words.is_empty()
            && let Some(current_word) = self.words.get(self.current_index)
        {
            let interval = self.calculate_word_duration(current_word);

            if self.last_tick.elapsed() >= interval {
                if self.current_index < self.words.len() - 1 {
                    self.current_index += 1;
                    self.last_tick = Instant::now();
                } else {
                    self.is_playing = false;
                }
            }
            ctx.request_repaint_after(interval.saturating_sub(self.last_tick.elapsed()));
        }

        if ui_opacity > 0.0 {
            egui::TopBottomPanel::top("top_panel")
                .frame(
                    egui::Frame::none()
                        .fill(self.theme.bg_color())
                        .inner_margin(4.0),
                )
                .show(ctx, |ui| {
                    ui.set_opacity(ui_opacity);
                    ui.horizontal(|ui| {
                        if ui.button("Open Document").clicked() {
                            self.load_file();
                        }
                        if ui
                            .button(if self.is_playing { "Pause" } else { "Play" })
                            .clicked()
                        {
                            self.is_playing = !self.is_playing;
                            self.last_tick = Instant::now();
                        }

                        ui.add_space(10.0);
                        ui.label("WPM:");
                        ui.add(
                            egui::DragValue::new(&mut self.wpm)
                                .clamp_range(100..=1500)
                                .speed(10),
                        );

                        ui.add_space(10.0);
                        ui.label("Text Size:");
                        ui.add(egui::Slider::new(&mut self.font_size, 32.0..=96.0).suffix("px"));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.menu_button("Theme", |ui| {
                                ui.spacing_mut().menu_margin = egui::Margin::symmetric(0.0, 0.0);
                                ui.set_max_width(60.0);
                                ui.set_min_width(0.0);
                                ui.radio_value(&mut self.theme, ThemePreset::WarmSepia, "Sepia");
                                ui.radio_value(&mut self.theme, ThemePreset::DeepAmber, "Amber");
                                ui.radio_value(&mut self.theme, ThemePreset::OledBlack, "OLED");
                            });
                        });
                    });
                });
        }

        if ui_opacity > 0.0 {
            egui::TopBottomPanel::bottom("bottom_panel")
                .frame(
                    egui::Frame::none()
                        .fill(self.theme.bg_color())
                        .inner_margin(4.0),
                )
                .show(ctx, |ui| {
                    ui.set_opacity(ui_opacity);
                    let total_words = self.words.len();
                    let current_pos = if total_words > 0 {
                        self.current_index + 1
                    } else {
                        0
                    };
                    let progress = if total_words > 0 {
                        current_pos as f32 / total_words as f32
                    } else {
                        0.0
                    };
                    let panel_rect = ui.available_rect_before_wrap();
                    let center_x = panel_rect.center().x;

                    ui.horizontal(|ui| {
                        ui.label(format!("Word: {} / {}", current_pos, total_words));

                        let num_segments = 100;
                        let segment_height = 14.0;
                        let segment_width = 1.0;
                        let segment_gap = 2.0;
                        let corner_radius = 0.0;

                        let bar_width = (num_segments as f32 * segment_width)
                            + ((num_segments - 1) as f32 * segment_gap);
                        let bar_left_x = center_x - (bar_width / 2.0);
                        let current_x = ui.cursor().min.x;

                        if bar_left_x > current_x {
                            ui.add_space(bar_left_x - current_x);
                        }

                        let response = segmented_progress_bar(
                            ui,
                            progress,
                            num_segments,
                            segment_width,
                            segment_height,
                            segment_gap,
                            corner_radius,
                        );

                        if (response.dragged() || response.clicked())
                            && let Some(mouse_pos) = response.interact_pointer_pos()
                        {
                            let rect = response.rect;
                            let new_progress =
                                ((mouse_pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                            self.current_index = ((total_words as f32 * new_progress) as usize)
                                .saturating_sub(1)
                                .min(total_words.saturating_sub(1));
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let time_left = self.estimated_time_remaining();
                            ui.label(format!("Remaining: {}", Self::format_duration(time_left)));
                        });
                    });
                });
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(self.theme.bg_color()))
            .show(ctx, |ui| {
                if let Some(word) = self.words.get(self.current_index) {
                    let orp_idx = Self::calculate_orp(word);
                    let chars: Vec<char> = word.chars().collect();

                    let left_part: String = chars[..orp_idx].iter().collect();
                    let anchor_char: String = chars
                        .get(orp_idx)
                        .cloned()
                        .map(|c| c.to_string())
                        .unwrap_or_default();
                    let right_part: String = if orp_idx + 1 < chars.len() {
                        chars[orp_idx + 1..].iter().collect()
                    } else {
                        String::new()
                    };

                    let rect = ui.available_rect_before_wrap();
                    let painter = ui.painter_at(rect);

                    let font_id = FontId::monospace(self.font_size);

                    let center_x = rect.center().x;
                    let center_y = rect.center().y;

                    let anchor_galley = painter.layout_no_wrap(
                        anchor_char.clone(),
                        font_id.clone(),
                        self.theme.anchor_color(),
                    );
                    let left_galley = painter.layout_no_wrap(
                        left_part.clone(),
                        font_id.clone(),
                        self.theme.text_color(),
                    );
                    let right_galley = painter.layout_no_wrap(
                        right_part.clone(),
                        font_id.clone(),
                        self.theme.text_color(),
                    );

                    let anchor_width = anchor_galley.rect.width();

                    let tick_color = self.theme.text_color().linear_multiply(0.2);

                    let line_height = anchor_galley.rect.height();
                    let tick_length = (self.font_size * 0.25).max(6.0);
                    let gap = 6.0;
                    let stroke_width = (self.font_size * 0.04).clamp(1.5, 3.5);

                    let top_tick_bottom = center_y - (line_height / 2.0) - gap;
                    let top_tick_top = top_tick_bottom - tick_length;

                    let bottom_tick_top = center_y + (line_height / 2.0) + gap;
                    let bottom_tick_bottom = bottom_tick_top + tick_length;

                    painter.line_segment(
                        [
                            Pos2::new(center_x, top_tick_top),
                            Pos2::new(center_x, top_tick_bottom),
                        ],
                        (stroke_width, tick_color),
                    );
                    painter.line_segment(
                        [
                            Pos2::new(center_x, bottom_tick_top),
                            Pos2::new(center_x, bottom_tick_bottom),
                        ],
                        (stroke_width, tick_color),
                    );

                    let anchor_pos = Pos2::new(
                        center_x - (anchor_width / 2.0),
                        center_y - (anchor_galley.rect.height() / 2.0),
                    );
                    painter.galley(anchor_pos, anchor_galley, self.theme.anchor_color());

                    let left_pos = Pos2::new(anchor_pos.x - left_galley.rect.width(), anchor_pos.y);
                    painter.galley(left_pos, left_galley, self.theme.text_color());

                    let right_pos = Pos2::new(anchor_pos.x + anchor_width, anchor_pos.y);
                    painter.galley(right_pos, right_galley, self.theme.text_color());
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() / 2.0);
                        ui.label("Load a document to start reading.");
                    });
                }
            });
    }
}
