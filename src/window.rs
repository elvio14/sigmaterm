use eframe::egui;
use egui::Stroke;

pub struct WindowBar {
    bg_color: egui::Color32,
    button_color: egui::Color32,
    hover_color: egui::Color32,
    close_hover_color: egui::Color32,
    dark_mode: bool,
}

impl Default for WindowBar {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowBar {
    pub fn new() -> Self {
        Self {
            bg_color: egui::Color32::from_gray(30),
            button_color: egui::Color32::from_gray(180),
            hover_color: egui::Color32::from_gray(60),
            close_hover_color: egui::Color32::from_rgb(200, 50, 50),
            dark_mode: true,
        }
    }
    
    pub fn is_dark_mode(&self) -> bool {
        self.dark_mode
    }

    pub fn render(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> bool {
        let mut add_terminal: bool = false;
        
        // Add resize handles for custom window decorations
        self.render_resize_handles(ctx);
        
        egui::TopBottomPanel::top("window_bar")
            .frame(egui::Frame::default()
                .fill(self.bg_color)
                .inner_margin(8.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Left side: Add terminal button
                    if self.window_button(ui, "❮+❯", self.hover_color) {
                        add_terminal = true;
                    }
                    
                    // Allocate space for right buttons first
                    ui.allocate_ui_with_layout(
                        ui.available_size(),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            // Right side: Window control buttons (added right to left)
                            if self.window_button(ui, "✕", self.close_hover_color) {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            
                            let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                            let maximize_icon = if is_maximized { "🗗" } else { "🗖" };
                            if self.window_button(ui, maximize_icon, self.hover_color) {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                            }
                            
                            if self.window_button(ui, "🗕", self.hover_color) {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }

                            if self.dark_mode_toggle_button(ui, self.dark_mode) {
                                self.dark_mode = !self.dark_mode;
                            }
                            
                            // Center: Title with draggable area (takes remaining space)
                            let title_response = ui.allocate_response(
                                ui.available_size(),
                                egui::Sense::drag()
                            );
                            
                            // Draw title text centered
                            ui.painter().text(
                                title_response.rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "Sigmaterm",
                                egui::FontId::proportional(14.0),
                                egui::Color32::from_gray(200),
                            );
                            
                            // Enable window dragging
                            if title_response.drag_started() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                            }
                        },
                    );
                });
            });
        
        add_terminal
    }

    fn window_button(&self, ui: &mut egui::Ui, text: &str, hover_color: egui::Color32) -> bool {
        let button_size = egui::vec2(32.0, 24.0);
        let (rect, response) = ui.allocate_exact_size(button_size, egui::Sense::click());
        
        // Draw background on hover
        if response.hovered() {
            ui.painter().rect_filled(rect, 0.0, hover_color);
        }
        
        // Draw icon
        let text_color = if response.hovered() {
            egui::Color32::WHITE
        } else {
            self.button_color
        };
        
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(16.0),
            text_color,
        );
        
        response.clicked()
    }

    fn dark_mode_toggle_button(&self, ui: &mut egui::Ui, dark_mode: bool) -> bool {
        let button_size = egui::vec2(24.0, 24.0);
        let (rect, response) = ui.allocate_exact_size(button_size, egui::Sense::click());

        if dark_mode {
            ui.painter().rect_filled(rect, 12.0, self.hover_color);
        };

        let text_color = if response.hovered() {
            egui::Color32::WHITE
        } else {
            self.button_color
        };

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "⏾",
            egui::FontId::proportional(16.0),
            text_color,
        );

        response.clicked()
    }
    
    fn render_resize_handles(&self, ctx: &egui::Context) {
        let frame_rect = ctx.input(|i| {
            i.viewport().inner_rect.unwrap_or(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(800.0, 600.0)
            ))
        });
        let edge_size = 5.0; // Size of the resize edge detection area
        
        // Get mouse position
        let pointer_pos = ctx.pointer_hover_pos();
        
        if let Some(pos) = pointer_pos {
            let left = (pos.x - frame_rect.min.x).abs() < edge_size;
            let right = (pos.x - frame_rect.max.x).abs() < edge_size;
            let top = (pos.y - frame_rect.min.y).abs() < edge_size;
            let bottom = (pos.y - frame_rect.max.y).abs() < edge_size;
            
            // Determine resize direction and cursor
            let resize_direction = match (left, right, top, bottom) {
                (true, _, true, _) => Some((egui::CursorIcon::ResizeNwSe, egui::viewport::ResizeDirection::NorthWest)),
                (_, true, true, _) => Some((egui::CursorIcon::ResizeNeSw, egui::viewport::ResizeDirection::NorthEast)),
                (true, _, _, true) => Some((egui::CursorIcon::ResizeNeSw, egui::viewport::ResizeDirection::SouthWest)),
                (_, true, _, true) => Some((egui::CursorIcon::ResizeNwSe, egui::viewport::ResizeDirection::SouthEast)),
                (true, _, _, _) => Some((egui::CursorIcon::ResizeHorizontal, egui::viewport::ResizeDirection::West)),
                (_, true, _, _) => Some((egui::CursorIcon::ResizeHorizontal, egui::viewport::ResizeDirection::East)),
                (_, _, true, _) => Some((egui::CursorIcon::ResizeVertical, egui::viewport::ResizeDirection::North)),
                (_, _, _, true) => Some((egui::CursorIcon::ResizeVertical, egui::viewport::ResizeDirection::South)),
                _ => None,
            };
            
            if let Some((cursor, direction)) = resize_direction {
                ctx.set_cursor_icon(cursor);
                
                // Start resize on click
                if ctx.input(|i| i.pointer.primary_pressed()) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(direction));
                }
            }
        }
    }
}