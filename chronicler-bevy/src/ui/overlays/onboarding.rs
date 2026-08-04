//! Onboarding overlay.

use bevy_egui::egui;

use crate::state::{ActiveOverlay, AppState, OnboardingState};

/// Render the onboarding modal. Returns true when onboarding is complete.
pub fn render_onboarding(
    ctx: &egui::Context,
    onboarding: &mut OnboardingState,
    app_state: &mut AppState,
) -> bool {
    let mut completed = false;

    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.85).clamp(350.0, 500.0);
    let height = (screen.height() * 0.7).clamp(320.0, 450.0);

    egui::Window::new("Welcome to D&D AI")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([width, height])
        .show(ctx, |ui| {
            // Page indicator
            ui.horizontal(|ui| {
                for i in 0..3 {
                    let color = if i == onboarding.current_page {
                        egui::Color32::from_rgb(218, 165, 32) // Gold for current
                    } else {
                        egui::Color32::DARK_GRAY
                    };
                    ui.label(egui::RichText::new("o").color(color));
                }
            });

            ui.separator();
            ui.add_space(10.0);

            match onboarding.current_page {
                0 => render_onboarding_page_welcome(ui),
                1 => render_onboarding_page_how_to_play(ui),
                2 => render_onboarding_page_good_to_know(ui),
                _ => {}
            }

            ui.add_space(10.0);
            ui.separator();

            // Navigation buttons
            ui.horizontal(|ui| {
                // Back button (not on first page)
                if onboarding.current_page > 0 && ui.button("< Back").clicked() {
                    app_state.play_click();
                    onboarding.current_page -= 1;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if onboarding.current_page < 2 {
                        if ui
                            .button(
                                egui::RichText::new("Next >")
                                    .color(egui::Color32::from_rgb(218, 165, 32)),
                            )
                            .clicked()
                        {
                            app_state.play_click();
                            onboarding.current_page += 1;
                        }
                    } else if ui
                        .button(
                            egui::RichText::new("Begin Adventure!")
                                .color(egui::Color32::from_rgb(100, 200, 100))
                                .strong(),
                        )
                        .clicked()
                    {
                        app_state.play_click();
                        onboarding.complete();
                        app_state.overlay = ActiveOverlay::None;
                        completed = true;
                    }

                    // Skip link on any page
                    if !completed
                        && ui
                            .small_button(
                                egui::RichText::new("Skip")
                                    .color(egui::Color32::GRAY)
                                    .small(),
                            )
                            .clicked()
                    {
                        app_state.play_click();
                        onboarding.complete();
                        app_state.overlay = ActiveOverlay::None;
                        completed = true;
                    }
                });
            });
        });

    completed
}

/// Page 1: Welcome
fn render_onboarding_page_welcome(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.heading(
            egui::RichText::new("Welcome, Adventurer!")
                .color(egui::Color32::from_rgb(218, 165, 32))
                .size(26.0),
        );
    });

    ui.add_space(15.0);

    ui.label(
        egui::RichText::new(
            "You're about to play D&D 5th Edition with an AI Dungeon Master powered by your chosen LLM.",
        )
        .size(16.0),
    );

    ui.add_space(10.0);

    ui.group(|ui| {
        ui.label(egui::RichText::new("What is this?").strong());
        ui.add_space(4.0);
        ui.label("- A solo D&D experience with a real AI running the game");
        ui.label("- Full D&D 5e rules - dice rolls, combat, spells, and more");
        ui.label("- Your choices shape the story");
    });

    ui.add_space(10.0);

    ui.group(|ui| {
        ui.label(egui::RichText::new("What you'll need:").strong());
        ui.add_space(4.0);
        ui.label("- Your imagination");
        ui.label("- A willingness to explore");
        ui.label("- That's it! The AI handles the rules");
    });
}

/// Page 2: How to Play
fn render_onboarding_page_how_to_play(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.heading(
            egui::RichText::new("How to Play")
                .color(egui::Color32::from_rgb(218, 165, 32))
                .size(26.0),
        );
    });

    ui.add_space(15.0);

    ui.label(egui::RichText::new("Just type what you want to do in natural language:").size(16.0));

    ui.add_space(10.0);

    ui.group(|ui| {
        ui.label(egui::RichText::new("Example commands:").strong());
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("  \"I look around the room\"")
                .color(egui::Color32::from_rgb(150, 200, 150))
                .italics(),
        );
        ui.label(
            egui::RichText::new("  \"I attack the goblin with my sword\"")
                .color(egui::Color32::from_rgb(150, 200, 150))
                .italics(),
        );
        ui.label(
            egui::RichText::new("  \"I try to persuade the guard to let us pass\"")
                .color(egui::Color32::from_rgb(150, 200, 150))
                .italics(),
        );
        ui.label(
            egui::RichText::new("  \"I cast Fireball at the enemies\"")
                .color(egui::Color32::from_rgb(150, 200, 150))
                .italics(),
        );
    });

    ui.add_space(10.0);

    ui.group(|ui| {
        ui.label(egui::RichText::new("The AI Dungeon Master will:").strong());
        ui.add_space(4.0);
        ui.label("- Roll dice automatically when needed");
        ui.label("- Enforce D&D rules fairly");
        ui.label("- Describe what happens based on your rolls");
        ui.label("- Remember important story events");
    });
}

/// Page 3: Good to Know
fn render_onboarding_page_good_to_know(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.heading(
            egui::RichText::new("Good to Know")
                .color(egui::Color32::from_rgb(218, 165, 32))
                .size(26.0),
        );
    });

    ui.add_space(15.0);

    ui.group(|ui| {
        ui.label(
            egui::RichText::new("Response Time")
                .strong()
                .color(egui::Color32::from_rgb(100, 180, 255)),
        );
        ui.add_space(4.0);
        ui.label("The AI takes a few seconds to respond - this is normal!");
        ui.label("Complex actions with multiple dice rolls take longer.");
    });

    ui.add_space(8.0);

    ui.group(|ui| {
        ui.label(
            egui::RichText::new("Auto-Save")
                .strong()
                .color(egui::Color32::from_rgb(100, 200, 100)),
        );
        ui.add_space(4.0);
        ui.label("Press Ctrl+S (Cmd+S on Mac) to save anytime.");
        ui.label("Your game is saved to the 'saves' folder.");
    });

    ui.add_space(8.0);

    ui.group(|ui| {
        ui.label(
            egui::RichText::new("Useful Shortcuts")
                .strong()
                .color(egui::Color32::from_rgb(218, 165, 32)),
        );
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("I").strong());
            ui.label("- Inventory");
            ui.add_space(20.0);
            ui.label(egui::RichText::new("C").strong());
            ui.label("- Character");
        });
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("F1").strong());
            ui.label("- Help");
            ui.add_space(20.0);
            ui.label(egui::RichText::new("Up/Down").strong());
            ui.label("- Command history");
        });
    });

    ui.add_space(10.0);

    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("Ready to begin your adventure?")
                .size(16.0)
                .italics()
                .color(egui::Color32::from_rgb(200, 190, 170)),
        );
    });
}
