use eframe::egui;
use egui::{menu, Button};

#[derive(Default)]
pub struct MyApp {
    draw_state: bool,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        Self::default()
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::from_black_alpha(128)))
            .show(ctx, |ui| {
                ui.label("Hello, world!");
            });

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("draw_overlay"),
            egui::ViewportBuilder::default().with_inner_size(egui::vec2(800.0, 30.0)),
            // .with_decorations(false)
            // .with_maximized(true),
            |ctx, _| {
                egui::TopBottomPanel::top("menu").show(ctx, |ui| {
                    menu::bar(ui, |ui| {
                        if ui.checkbox(&mut self.draw_state, "Draw").changed() {
                            println!("Draw state: {}", self.draw_state);
                        }

                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd_to(
                                egui::ViewportId::ROOT,
                                egui::ViewportCommand::Close,
                            );
                        }
                    });
                });

                if ctx.input(|i| i.viewport().close_requested()) {
                    ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
                }
            },
        );

        ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(self.draw_state));
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}
