//! Settings overlay.

use bevy_egui::egui;

use crate::state::{ActiveOverlay, AppState};
use crate::window::WindowSettings;

/// Render the settings overlay. Returns true if user wants to return to main menu.
pub fn render_settings(
    ctx: &egui::Context,
    app_state: &mut AppState,
    sound_settings: Option<&mut crate::sound::SoundSettings>,
    window_settings: Option<&mut WindowSettings>,
    saves_path: &str,
) -> bool {
    let mut return_to_menu = false;

    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.75).clamp(280.0, 400.0);
    let height = (screen.height() * 0.65).clamp(280.0, 380.0);

    egui::Window::new("Settings")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([width, height])
        .max_size([500.0, 500.0])
        .show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();

            // Display section
            ui.collapsing(egui::RichText::new("Display").strong(), |ui| {
                if let Some(window) = window_settings {
                    // Fullscreen toggle
                    ui.horizontal(|ui| {
                        ui.label("Fullscreen:");
                        if ui.checkbox(&mut window.fullscreen, "").changed() {
                            window.mark_changed();
                        }
                    });

                    // Window size (only when not fullscreen)
                    ui.add_enabled_ui(!window.fullscreen, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Window size:");
                            egui::ComboBox::from_id_salt("resolution")
                                .selected_text(format!(
                                    "{}x{}",
                                    window.width as u32, window.height as u32
                                ))
                                .show_ui(ui, |ui| {
                                    let resolutions = [
                                        (1280.0, 720.0, "1280x720 (HD)"),
                                        (1280.0, 800.0, "1280x800"),
                                        (1366.0, 768.0, "1366x768"),
                                        (1600.0, 900.0, "1600x900"),
                                        (1920.0, 1080.0, "1920x1080 (Full HD)"),
                                        (2560.0, 1440.0, "2560x1440 (QHD)"),
                                    ];
                                    for (w, h, label) in resolutions {
                                        if ui
                                            .selectable_label(
                                                (window.width - w).abs() < 1.0
                                                    && (window.height - h).abs() < 1.0,
                                                label,
                                            )
                                            .clicked()
                                        {
                                            window.width = w;
                                            window.height = h;
                                            window.mark_changed();
                                        }
                                    }
                                });
                        });
                    });

                    ui.add_space(4.0);
                }

                // Character panel setting
                ui.horizontal(|ui| {
                    ui.label("Character panel:");
                    if ui
                        .selectable_label(app_state.character_panel_expanded, "Expanded")
                        .clicked()
                    {
                        app_state.character_panel_expanded = true;
                    }
                    if ui
                        .selectable_label(!app_state.character_panel_expanded, "Collapsed")
                        .clicked()
                    {
                        app_state.character_panel_expanded = false;
                    }
                });
            });

            ui.add_space(8.0);

            // Audio section
            ui.collapsing(egui::RichText::new("Audio").strong(), |ui| {
                if let Some(sound) = sound_settings {
                    // Mute toggle
                    ui.horizontal(|ui| {
                        ui.label("Sound enabled:");
                        if ui.checkbox(&mut sound.enabled, "").changed() {
                            sound.mark_changed();
                        }
                    });

                    // Volume slider (only if sound is enabled)
                    ui.horizontal(|ui| {
                        ui.label("Volume:");
                        let slider_response = ui.add_enabled(
                            sound.enabled,
                            egui::Slider::new(&mut sound.volume, 0.0..=1.0)
                                .show_value(false)
                                .clamping(egui::SliderClamping::Always),
                        );
                        if slider_response.changed() {
                            sound.mark_changed();
                        }
                        ui.label(format!("{}%", (sound.volume * 100.0) as i32));
                    });
                } else {
                    ui.label(
                        egui::RichText::new("Audio settings unavailable")
                            .color(egui::Color32::GRAY),
                    );
                }
            });

            ui.add_space(8.0);

            // Save files section
            ui.collapsing(egui::RichText::new("Save Files").strong(), |ui| {
                ui.label(format!("Save directory: {}/", saves_path));
                ui.label(format!("Character saves: {}/characters/", saves_path));

                ui.add_space(4.0);

                if ui.button("Open saves folder").clicked() {
                    app_state.play_click();
                    #[cfg(target_os = "macos")]
                    {
                        let _ = std::process::Command::new("open").arg(saves_path).spawn();
                    }
                    #[cfg(target_os = "windows")]
                    {
                        let _ = std::process::Command::new("explorer")
                            .arg(saves_path)
                            .spawn();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let _ = std::process::Command::new("xdg-open")
                            .arg(saves_path)
                            .spawn();
                    }
                }
            });

            ui.add_space(8.0);

            // Keyboard shortcuts
            ui.collapsing(egui::RichText::new("Keyboard Shortcuts").strong(), |ui| {
                ui.label("Ctrl+S / Cmd+S - Save game");
                ui.label("Ctrl+Q / Cmd+Q - Quit game");
                ui.label("I - Inventory");
                ui.label("C - Character sheet");
                ui.label("Shift+Q - Quest log");
                ui.label("F1 / ? - Help");
                ui.label("Escape - Close overlay");
            });

            ui.add_space(8.0);

            // About section
            ui.collapsing(egui::RichText::new("About").strong(), |ui| {
                ui.label(
                    egui::RichText::new("D&D: AI Dungeon Master")
                        .color(egui::Color32::from_rgb(218, 165, 32)),
                );
                ui.label("A text-based adventure powered by AI");
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Built with Rust, Bevy, and your chosen LLM")
                        .small()
                        .color(egui::Color32::GRAY),
                );
            });

            ui.add_space(16.0);
            ui.separator();

            // Game actions
            ui.horizontal(|ui| {
                if ui.button("Return to Main Menu").clicked() {
                    app_state.play_click();
                    return_to_menu = true;
                    app_state.overlay = ActiveOverlay::None;
                }

                if ui
                    .button(
                        egui::RichText::new("Quit Game")
                            .color(egui::Color32::from_rgb(200, 100, 100)),
                    )
                    .clicked()
                {
                    app_state.play_click();
                    std::process::exit(0);
                }
            });

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Press Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });

    return_to_menu
}
