use egui;
use std::cmp::{max, min};

use super::common::{Component, InterfaceAction};

pub struct GrabBox {
    rect: Option<egui::Rect>,
    anchor: Option<egui::Pos2>,
    dragging: bool,
    resizing: Option<usize>,
}

impl Default for GrabBox {
    fn default() -> Self {
        Self {
            rect: None,
            anchor: None,
            dragging: false,
            resizing: None,
        }
    }
}

impl Component for GrabBox {
    fn ui(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd_to(
            egui::ViewportId::ROOT,
            egui::ViewportCommand::MousePassthrough(false),
        );

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::from_black_alpha(128)))
            .show(ctx, |ui| {
                let (response, painter) =
                    ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::drag());
                let pointer_pos = response.hover_pos().unwrap_or_default();

                if response.drag_started() {
                    self.dragging = true;
                    self.rect = Some(egui::Rect::from_min_size(pointer_pos, egui::Vec2::ZERO));
                    self.anchor = Some(pointer_pos);
                }

                if self.dragging {
                    if let (Some(rect), Some(anchor)) = (&mut self.rect, &self.anchor) {
                        let top_left = egui::Pos2::new(
                            min(anchor.x as u16, pointer_pos.x as u16) as f32,
                            min(anchor.y as u16, pointer_pos.y as u16) as f32,
                        );
                        let top_right = egui::Pos2::new(
                            max(anchor.x as u16, pointer_pos.x as u16) as f32,
                            max(anchor.y as u16, pointer_pos.y as u16) as f32,
                        );

                        rect.max = top_right;
                        rect.min = top_left;
                    }
                }

                if response.drag_stopped() {
                    self.dragging = false;
                }

                if let Some(rect) = self.rect {
                    let border_color = egui::Color32::GREEN;
                    let fill_color = egui::Color32::from_rgba_unmultiplied(0, 255, 0, 64);

                    painter.rect_stroke(rect, 0.0, egui::Stroke::new(5.0, border_color));
                    painter.rect_filled(rect, 0.0, fill_color);

                    let corner_radius = 5.0;
                    let corners = [
                        rect.left_top(),
                        rect.right_top(),
                        rect.right_bottom(),
                        rect.left_bottom(),
                    ];
                    for (i, &corner) in corners.iter().enumerate() {
                        let corner_rect = egui::Rect::from_center_size(
                            corner,
                            egui::Vec2::splat(corner_radius * 2.0),
                        );
                        painter.circle_filled(corner, corner_radius, border_color);

                        // if self.resizing.is_none() && corner_rect.contains(pointer_pos) {
                        //     if response.drag_started() {
                        //         self.resizing = Some(i);
                        //     }
                        // }
                    }
                }

                if let Some(corner_index) = self.resizing {
                    if response.dragged() {
                        if let Some(rect) = &mut self.rect {
                            match corner_index {
                                0 => rect.min = pointer_pos,
                                1 => {
                                    rect.max.x = pointer_pos.x;
                                    rect.min.y = pointer_pos.y;
                                }
                                2 => rect.max = pointer_pos,
                                3 => {
                                    rect.min.x = pointer_pos.x;
                                    rect.max.y = pointer_pos.y;
                                }
                                _ => {}
                            }
                            rect.min = rect.min.min(rect.max);
                            rect.max = rect.max.max(rect.min);
                        }
                    }

                    if response.drag_stopped() {
                        self.resizing = None;
                    }
                }
            });

        ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
    }
}
