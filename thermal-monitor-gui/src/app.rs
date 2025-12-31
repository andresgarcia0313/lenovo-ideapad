//! Application state and main loop for thermal monitor GUI
//!
//! Implements eframe::App trait for egui integration.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

use crate::system::{Mode, ThermalState, ThermalZone, set_mode, set_fan_boost, apply_thermal_control};

/// Update interval in seconds
const UPDATE_INTERVAL_SECS: f32 = 2.0;

/// History capacity (2 minutes at 2-second intervals)
const HISTORY_CAPACITY: usize = 60;

/// Get localized app description (max 8 words)
/// Supports: English, Spanish, Chinese, Portuguese, German
fn get_localized_description() -> &'static str {
    let lang = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .unwrap_or_default();

    let lang_code = lang.split('.').next().unwrap_or("en");
    let lang_prefix = lang_code.split('_').next().unwrap_or("en");

    match lang_prefix {
        "es" => "Monitorea y controla temperatura CPU",
        "zh" => "监控和控制CPU温度",
        "pt" => "Monitore e controle temperatura CPU",
        "de" => "CPU-Temperatur überwachen und steuern",
        _ => "Monitor and control CPU temperature",
    }
}

/// Temperature history buffer
#[derive(Debug)]
pub struct TemperatureHistory {
    cpu_temps: VecDeque<f32>,
    kbd_temps: VecDeque<f32>,
    capacity: usize,
}

impl Default for TemperatureHistory {
    fn default() -> Self {
        Self::new(HISTORY_CAPACITY)
    }
}

impl TemperatureHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            cpu_temps: VecDeque::with_capacity(capacity),
            kbd_temps: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, cpu: f32, kbd: f32) {
        if self.cpu_temps.len() >= self.capacity {
            self.cpu_temps.pop_front();
            self.kbd_temps.pop_front();
        }
        self.cpu_temps.push_back(cpu);
        self.kbd_temps.push_back(kbd);
    }

    /// Get CPU temperature points for plotting
    pub fn cpu_points(&self) -> PlotPoints {
        PlotPoints::new(
            self.cpu_temps
                .iter()
                .enumerate()
                .map(|(i, &t)| [i as f64, t as f64])
                .collect(),
        )
    }

    /// Get keyboard temperature points for plotting
    pub fn kbd_points(&self) -> PlotPoints {
        PlotPoints::new(
            self.kbd_temps
                .iter()
                .enumerate()
                .map(|(i, &t)| [i as f64, t as f64])
                .collect(),
        )
    }

    pub fn len(&self) -> usize {
        self.cpu_temps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cpu_temps.is_empty()
    }
}

/// Main application state
pub struct ThermalApp {
    state: ThermalState,
    history: TemperatureHistory,
    last_update: Instant,
    status_message: Option<(String, Instant)>,
    target_temp: f32,
    auto_control: bool,
    fan_boost_manual: bool,
}

impl Default for ThermalApp {
    fn default() -> Self {
        let state = ThermalState::read();
        let mut history = TemperatureHistory::default();
        history.push(state.cpu_temp, state.keyboard_temp);

        Self {
            state,
            history,
            last_update: Instant::now(),
            status_message: None,
            target_temp: 55.0,
            auto_control: false,
            fan_boost_manual: false,
        }
    }
}

impl ThermalApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    /// Update state from system
    fn update_state(&mut self) {
        self.state = ThermalState::read();
        self.history.push(self.state.cpu_temp, self.state.keyboard_temp);

        // Apply automatic thermal control if enabled
        if self.auto_control {
            if let Ok(msg) = apply_thermal_control(self.state.cpu_temp, self.target_temp) {
                if msg != "On target" {
                    self.status_message = Some((msg, Instant::now()));
                }
            }
        }
    }

    /// Change CPU mode
    fn change_mode(&mut self, mode: Mode) {
        match set_mode(mode) {
            Ok(()) => {
                self.status_message = Some((
                    format!("Mode changed to {}", mode.label()),
                    Instant::now(),
                ));
                self.update_state();
            }
            Err(e) => {
                self.status_message = Some((
                    format!("Error: {}", e),
                    Instant::now(),
                ));
            }
        }
    }

    /// Set status message
    fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    /// Get zone color as egui Color32
    fn zone_color(zone: ThermalZone) -> egui::Color32 {
        let (r, g, b) = zone.color_rgb();
        egui::Color32::from_rgb(r, g, b)
    }

    /// Get mode color
    fn mode_color(mode: Mode) -> egui::Color32 {
        match mode {
            Mode::Performance => egui::Color32::from_rgb(255, 100, 100),
            Mode::Comfort => egui::Color32::from_rgb(100, 200, 255),
            Mode::Balanced => egui::Color32::from_rgb(150, 220, 100),
            Mode::Quiet => egui::Color32::from_rgb(180, 180, 220),
            Mode::Auto => egui::Color32::from_rgb(255, 200, 100),
            Mode::Unknown => egui::Color32::GRAY,
        }
    }


    /// Render temperatures - adaptive version
    fn render_temperatures_adaptive(&self, ui: &mut egui::Ui, is_medium: bool) {
        let zone = self.state.thermal_zone();
        let color = Self::zone_color(zone);
        let font_size = if is_medium { 24.0 } else { 18.0 };
        let label_size = if is_medium { 11.0 } else { 9.0 };

        ui.horizontal_wrapped(|ui| {
            // CPU
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("CPU").size(label_size).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new(format!("{:.0}°", self.state.cpu_temp))
                    .size(font_size).color(color).strong());
            });
            ui.add_space(10.0);
            // Keyboard
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("KBD").size(label_size).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new(format!("{:.0}°", self.state.keyboard_temp))
                    .size(font_size).color(color).strong());
            });
            ui.add_space(10.0);
            // Zone label
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Zone").size(label_size).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new(zone.label()).size(label_size + 2.0).color(color));
            });
        });
    }

    /// Render performance - adaptive version
    fn render_performance_adaptive(&self, ui: &mut egui::Ui, is_medium: bool) {
        let font_size = if is_medium { 20.0 } else { 16.0 };
        let label_size = if is_medium { 11.0 } else { 9.0 };
        let mode_color = Self::mode_color(self.state.mode);

        ui.horizontal_wrapped(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Perf").size(label_size).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new(format!("{}%", self.state.perf_pct))
                    .size(font_size).strong());
            });
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Freq").size(label_size).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new(format!("{:.1}G", self.state.current_freq_ghz()))
                    .size(font_size).strong());
            });
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Mode").size(label_size).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new(self.state.mode.label())
                    .size(label_size + 2.0).color(mode_color).strong());
            });
        });
    }

    /// Render controls - adaptive version with wrapping
    fn render_controls_adaptive(&mut self, ui: &mut egui::Ui, available_width: f32) {
        let button_width = if available_width > 600.0 { 90.0 } else { 70.0 };
        let button_height = if available_width > 600.0 { 28.0 } else { 24.0 };
        let font_size = if available_width > 600.0 { 11.0 } else { 9.0 };

        ui.horizontal_wrapped(|ui| {
            for mode in Mode::all() {
                let is_current = self.state.mode == *mode;
                let color = Self::mode_color(*mode);

                let button = egui::Button::new(
                    egui::RichText::new(mode.label())
                        .size(font_size)
                        .color(if is_current { egui::Color32::BLACK } else { color }),
                )
                .fill(if is_current { color } else { egui::Color32::TRANSPARENT })
                .stroke(egui::Stroke::new(1.0, color))
                .min_size(egui::vec2(button_width, button_height));

                if ui.add(button).clicked() && !is_current {
                    self.change_mode(*mode);
                }
            }
        });
    }

    /// Render target temperature - adaptive version
    fn render_target_temp_adaptive(&mut self, ui: &mut egui::Ui, is_wide: bool) {
        let slider_width = if is_wide { 120.0 } else { 80.0 };
        let font_size = if is_wide { 11.0 } else { 9.0 };

        ui.horizontal_wrapped(|ui| {
            let slider = egui::Slider::new(&mut self.target_temp, 40.0..=80.0)
                .suffix("°")
                .step_by(1.0)
                .text("");
            ui.add_sized([slider_width, 20.0], slider);

            // Auto button
            let auto_color = if self.auto_control {
                egui::Color32::from_rgb(100, 220, 100)
            } else {
                egui::Color32::GRAY
            };
            if ui.add(egui::Button::new(
                egui::RichText::new(if self.auto_control { "AUTO" } else { "OFF" })
                    .size(font_size).color(auto_color)
            ).min_size(egui::vec2(40.0, 20.0))).clicked() {
                self.auto_control = !self.auto_control;
                self.set_status(if self.auto_control { "Auto ON".into() } else { "Auto OFF".into() });
            }

            // Status
            if self.state.cpu_temp > self.target_temp {
                ui.label(egui::RichText::new(format!("+{:.0}°", self.state.cpu_temp - self.target_temp))
                    .size(font_size).color(egui::Color32::from_rgb(255, 150, 100)));
            } else {
                ui.label(egui::RichText::new("OK").size(font_size)
                    .color(egui::Color32::from_rgb(100, 220, 100)));
            }
        });
    }

    /// Render fan control - adaptive version
    fn render_fan_control_adaptive(&mut self, ui: &mut egui::Ui, is_wide: bool) {
        let font_size = if is_wide { 11.0 } else { 9.0 };
        let fan_active = self.state.fan_boost || self.fan_boost_manual;
        let fan_color = if fan_active {
            egui::Color32::from_rgb(255, 150, 100)
        } else {
            egui::Color32::GRAY
        };

        ui.horizontal_wrapped(|ui| {
            if ui.add(egui::Button::new(
                egui::RichText::new(if fan_active { "BOOST" } else { "NORMAL" })
                    .size(font_size)
                    .color(if fan_active { egui::Color32::BLACK } else { fan_color })
            )
            .fill(if fan_active { fan_color } else { egui::Color32::TRANSPARENT })
            .stroke(egui::Stroke::new(1.0, fan_color))
            .min_size(egui::vec2(60.0, 20.0))).clicked() {
                self.fan_boost_manual = !self.fan_boost_manual;
                let _ = set_fan_boost(self.fan_boost_manual);
                self.set_status(if self.fan_boost_manual { "Fan boost".into() } else { "Fan auto".into() });
            }

            if is_wide {
                ui.label(egui::RichText::new("Max cooling").size(9.0).color(egui::Color32::DARK_GRAY));
            }
        });
    }

    /// Render history graph - adaptive version
    fn render_history_adaptive(&self, ui: &mut egui::Ui, target_temp: f32, height: f32) {
        if self.history.is_empty() {
            ui.label("Collecting data...");
            return;
        }

        let cpu_line = Line::new(self.history.cpu_points())
            .name("CPU")
            .color(egui::Color32::from_rgb(255, 100, 100))
            .width(2.0);

        let kbd_line = Line::new(self.history.kbd_points())
            .name("Kbd")
            .color(egui::Color32::from_rgb(100, 200, 255))
            .width(2.0);

        let target_points: Vec<[f64; 2]> = (0..HISTORY_CAPACITY)
            .map(|i| [i as f64, target_temp as f64])
            .collect();
        let target_line = Line::new(PlotPoints::new(target_points))
            .name("Target")
            .color(egui::Color32::from_rgb(255, 200, 100))
            .width(1.0)
            .style(egui_plot::LineStyle::dashed_loose());

        Plot::new("temp_history")
            .height(height)
            .show_axes(true)
            .show_grid(true)
            .include_y(30.0)
            .include_y(80.0)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .legend(egui_plot::Legend::default().position(egui_plot::Corner::RightTop))
            .show(ui, |plot_ui| {
                plot_ui.line(cpu_line);
                plot_ui.line(kbd_line);
                plot_ui.line(target_line);
            });
    }

    /// Render status bar
    fn render_status(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Status message (auto-clear after 3 seconds)
            if let Some((msg, time)) = &self.status_message {
                if time.elapsed() < Duration::from_secs(3) {
                    ui.label(egui::RichText::new(msg).size(12.0).color(egui::Color32::YELLOW));
                } else {
                    self.status_message = None;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("Thermal Monitor v1.3.0")
                        .size(11.0)
                        .color(egui::Color32::DARK_GRAY),
                );
            });
        });
    }
}

impl eframe::App for ThermalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update state every UPDATE_INTERVAL_SECS
        if self.last_update.elapsed() >= Duration::from_secs_f32(UPDATE_INTERVAL_SECS) {
            self.update_state();
            self.last_update = Instant::now();
        }

        // Request repaint to keep updating
        ctx.request_repaint_after(Duration::from_millis(100));

        // Dark theme
        ctx.set_visuals(egui::Visuals::dark());

        egui::CentralPanel::default().show(ctx, |ui| {
            // Get available width to determine layout
            let available_width = ui.available_width();
            let is_wide = available_width > 700.0;
            let is_medium = available_width > 500.0;

            // Adaptive spacing
            let spacing = if is_wide { 8.0 } else { 4.0 };
            ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);

            // Use ScrollArea for small windows
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Title - adaptive size
                let title_size = if is_wide { 22.0 } else if is_medium { 18.0 } else { 16.0 };
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("Thermal Monitor").size(title_size));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}", self.state.platform_profile))
                                .size(if is_wide { 12.0 } else { 10.0 })
                                .color(egui::Color32::GRAY),
                        );
                    });
                });
                // Localized description
                let desc_size = if is_wide { 12.0 } else { 10.0 };
                ui.label(
                    egui::RichText::new(get_localized_description())
                        .size(desc_size)
                        .color(egui::Color32::from_rgb(180, 180, 180))
                        .italics(),
                );
                ui.separator();

                // Temperatures and Performance - side by side on wide, stacked on narrow
                if is_wide {
                    ui.horizontal(|ui| {
                        let half_width = (available_width - 20.0) / 2.0;
                        ui.group(|ui| {
                            ui.set_width(half_width);
                            ui.label(egui::RichText::new("Temperatures").size(13.0).strong());
                            self.render_temperatures_adaptive(ui, is_medium);
                        });
                        ui.group(|ui| {
                            ui.set_width(half_width);
                            ui.label(egui::RichText::new("Performance").size(13.0).strong());
                            self.render_performance_adaptive(ui, is_medium);
                        });
                    });
                } else {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Temperatures").size(13.0).strong());
                        self.render_temperatures_adaptive(ui, is_medium);
                    });
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Performance").size(13.0).strong());
                        self.render_performance_adaptive(ui, is_medium);
                    });
                }

                // Mode Control - wrapping buttons
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Mode Control").size(13.0).strong());
                    self.render_controls_adaptive(ui, available_width);
                });

                // Target and Fan - side by side on wide, stacked on narrow
                if is_medium {
                    ui.horizontal(|ui| {
                        let half_width = (available_width - 20.0) / 2.0;
                        ui.group(|ui| {
                            ui.set_width(half_width);
                            ui.label(egui::RichText::new("Target Temp").size(13.0).strong());
                            self.render_target_temp_adaptive(ui, is_wide);
                        });
                        ui.group(|ui| {
                            ui.set_width(half_width);
                            ui.label(egui::RichText::new("Fan").size(13.0).strong());
                            self.render_fan_control_adaptive(ui, is_wide);
                        });
                    });
                } else {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Target Temp").size(13.0).strong());
                        self.render_target_temp_adaptive(ui, is_wide);
                    });
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Fan").size(13.0).strong());
                        self.render_fan_control_adaptive(ui, is_wide);
                    });
                }

                // History graph - adaptive height
                let target = self.target_temp;
                let graph_height = if is_wide { 180.0 } else if is_medium { 120.0 } else { 80.0 };
                ui.group(|ui| {
                    ui.label(egui::RichText::new("History").size(13.0).strong());
                    self.render_history_adaptive(ui, target, graph_height);
                });

                // Status bar
                self.render_status(ui);
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_capacity() {
        let mut history = TemperatureHistory::new(3);
        history.push(40.0, 35.0);
        history.push(42.0, 36.0);
        history.push(44.0, 37.0);
        assert_eq!(history.len(), 3);

        history.push(46.0, 38.0);
        assert_eq!(history.len(), 3); // Should not exceed capacity
    }

    #[test]
    fn test_history_empty() {
        let history = TemperatureHistory::new(10);
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_history_default() {
        let history = TemperatureHistory::default();
        assert!(history.is_empty());
        assert_eq!(history.capacity, HISTORY_CAPACITY);
    }

    #[test]
    fn test_history_points() {
        let mut history = TemperatureHistory::new(10);
        history.push(40.0, 35.0);
        history.push(42.0, 36.0);

        let _cpu_points = history.cpu_points();
        let _kbd_points = history.kbd_points();

        // Verify points are generated correctly
        assert!(!history.is_empty());
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_history_fifo_behavior() {
        let mut history = TemperatureHistory::new(2);
        history.push(10.0, 5.0);  // First in
        history.push(20.0, 10.0);
        history.push(30.0, 15.0); // Should push out first

        assert_eq!(history.len(), 2);
        // First value (10.0) should be gone
    }

    #[test]
    fn test_zone_colors() {
        // Verify all zones have valid colors
        for zone in [
            ThermalZone::Cool,
            ThermalZone::Comfort,
            ThermalZone::Optimal,
            ThermalZone::Warm,
            ThermalZone::Hot,
            ThermalZone::Critical,
        ] {
            let color = ThermalApp::zone_color(zone);
            assert_ne!(color, egui::Color32::TRANSPARENT);
        }
    }

    #[test]
    fn test_zone_colors_match_thermal_zone() {
        // Verify zone_color matches color_rgb from ThermalZone
        for zone in [
            ThermalZone::Cool,
            ThermalZone::Comfort,
            ThermalZone::Optimal,
            ThermalZone::Warm,
            ThermalZone::Hot,
            ThermalZone::Critical,
        ] {
            let (r, g, b) = zone.color_rgb();
            let color = ThermalApp::zone_color(zone);
            assert_eq!(color, egui::Color32::from_rgb(r, g, b));
        }
    }

    #[test]
    fn test_mode_colors() {
        // Verify all modes have colors
        for mode in Mode::all() {
            let color = ThermalApp::mode_color(*mode);
            assert_ne!(color, egui::Color32::TRANSPARENT);
        }
    }

    #[test]
    fn test_mode_color_unknown() {
        let color = ThermalApp::mode_color(Mode::Unknown);
        assert_eq!(color, egui::Color32::GRAY);
    }

    #[test]
    fn test_mode_colors_distinct() {
        // Each mode should have a distinct color
        let colors: Vec<_> = Mode::all().iter().map(|m| ThermalApp::mode_color(*m)).collect();

        // Performance should be reddish
        assert!(colors[0].r() > colors[0].b());

        // Comfort should be blueish
        assert!(colors[1].b() > colors[1].r());

        // Balanced should be greenish
        assert!(colors[2].g() > colors[2].r());
    }
}
