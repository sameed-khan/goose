use eframe::egui;

#[derive(Default)]
pub struct MyApp {
    // Add your app state here
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        Self::default()
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello, world!");
            // Add more UI elements here
        });
    }
}
