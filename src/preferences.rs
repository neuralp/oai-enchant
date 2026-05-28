use egui::Slider;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Preferences {
    pub scroll_speed: f32,
    pub acceleration_enabled: bool,
    /// Per-frame velocity decay factor. 0.80 = quick stop, 0.98 = long coast.
    pub deceleration: f32,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            scroll_speed: 1.0,
            acceleration_enabled: false,
            deceleration: 0.90,
        }
    }
}

pub fn show_dialog(ctx: &egui::Context, prefs: &mut Preferences, show: &mut bool) {
    if !*show {
        return;
    }
    egui::Window::new("Preferences")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(340.0);

            ui.label(egui::RichText::new("Scrolling").strong());
            ui.separator();
            ui.add_space(4.0);

            egui::Grid::new("prefs_grid")
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Scroll Speed:");
                    ui.add(
                        Slider::new(&mut prefs.scroll_speed, 0.25..=5.0)
                            .step_by(0.25)
                            .suffix("×"),
                    );
                    ui.end_row();

                    ui.label("");
                    ui.checkbox(
                        &mut prefs.acceleration_enabled,
                        "Enable swipe acceleration with deceleration",
                    );
                    ui.end_row();

                    if prefs.acceleration_enabled {
                        ui.label("Deceleration:");
                        ui.horizontal(|ui| {
                            // Store factor directly; friction = 1 - factor.
                            // Slider over friction so left = low friction (long coast).
                            let mut friction = 1.0 - prefs.deceleration;
                            if ui
                                .add(
                                    Slider::new(&mut friction, 0.02..=0.25)
                                        .show_value(false)
                                        .clamping(egui::SliderClamping::Always),
                                )
                                .changed()
                            {
                                prefs.deceleration = 1.0 - friction;
                            }
                            let desc = if prefs.deceleration >= 0.95 {
                                "Low  (long coast)"
                            } else if prefs.deceleration >= 0.87 {
                                "Medium"
                            } else {
                                "High  (quick stop)"
                            };
                            ui.label(desc);
                        });
                        ui.end_row();
                    }
                });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(concat!("OAI Enchant  #", env!("GIT_SHORT_HASH")))
                        .weak()
                        .size(16.0),
                );
            });
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    *show = false;
                }
                if ui.button("Reset to Defaults").clicked() {
                    *prefs = Preferences::default();
                }
            });
        });
}
