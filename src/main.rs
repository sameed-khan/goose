mod errors;
mod gui;
mod nav;
mod utils;
mod verb;

use eframe;
use eframe::egui;
use gui::app::MyApp;

use eframe::WindowBuilder;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use std::time::Instant;
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Test App")
            .with_inner_size([800.0, 600.0])
            .with_transparent(true)
            .with_always_on_top()
            .with_mouse_passthrough(true),
        ..Default::default()
    };
    eframe::run_native(
        "Test App",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}
