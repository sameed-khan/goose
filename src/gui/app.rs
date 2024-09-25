use crate::gui::components::{
    common::{Component, InterfaceAction},
    grab_box::GrabBox,
};
use eframe::egui;
use egui::{menu, Button};

pub struct MyApp {
    action_state: Option<Box<dyn Component>>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self { action_state: None }
    }
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self::default()
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("draw_overlay"),
            egui::ViewportBuilder::default()
                .with_title("Menu Controls")
                .with_inner_size([800.0, 20.0]),
            |ctx, _| {
                egui::TopBottomPanel::top("menu").show(ctx, |ui| {
                    menu::bar(ui, |ui| {
                        if ui.button("Draw").clicked() {
                            self.action_state = Some(Box::new(GrabBox::default()));
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

        if let Some(action) = &mut self.action_state {
            action.ui(ctx);

            egui::Area::new(egui::Id::new("draw_controls"))
                .fixed_pos(egui::pos2(1800.0, 150.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("X").clicked() {
                            self.action_state = None;
                        }
                    })
                });
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(true));
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}
