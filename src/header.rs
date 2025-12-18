use eframe::egui;

use crate::{header, utils::{self, ColorSet, get_set_from_hue, window_button}};

// Header action signals
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeaderAction {
    None,
    CloseTerminal,
    MaximizeTerminal,
    MinimizeTerminal
}

// Emoji Picker =======================================

pub struct EmojiPicker {

}

impl Default for EmojiPicker {
    fn default() -> Self {
        Self {

        }
    }
}
// Color Picker =======================================

pub struct ColorPicker {
    pub color_sets: Vec<ColorSet>,
    pub selected_index: usize
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self {
            color_sets: vec![
                get_set_from_hue(180.0),
                get_set_from_hue(105.0),
                get_set_from_hue(57.0),
                get_set_from_hue(280.0)
            ],
            selected_index: 0
        }
    }
}


// Header =============================================
#[derive(Clone, PartialEq)]
pub enum ColorMode {
    Light, 
    Dark
}

#[derive(Clone)]
pub struct Header {
    emoji_picker_open: bool,
    color_picker_open: bool,
    title: String,
    pub color_set: ColorSet,
    pub color_mode: ColorMode,
    is_editing_title: bool,
    hue: f32,  // Store current hue value
    is_maximized: bool
}

impl Default for Header {
    fn default() -> Self {
        Self{
            title: "Untitled Terminal".to_string(),
            emoji_picker_open: false,
            color_picker_open: false,
            color_set: ColorSet::default(),
            color_mode: ColorMode::Dark,
            is_editing_title: false,
            hue: 180.0,
            is_maximized: false
        }
    }
}

impl Header {
    pub fn new(hue: f32, is_maximized: bool) -> Self {
        Self {
            title: "Untitled Terminal".to_string(),
            emoji_picker_open: false,
            color_picker_open: false,
            color_set: utils::get_set_from_hue(hue),
            color_mode: ColorMode::Dark,
            is_editing_title: false,
            hue,
            is_maximized: is_maximized
        }
    }
    pub fn set_dark_mode(&mut self, dark_mode: bool) {
        self.color_mode = if dark_mode {ColorMode::Dark} else {ColorMode::Light};
    }
    
    pub fn is_editing_title(&self) -> bool {
        self.is_editing_title
    }
    
    pub fn stop_editing_title(&mut self) {
        self.is_editing_title = false;
    }
    
    pub fn toggle_emoji_picker(&mut self) {
        self.emoji_picker_open = !self.emoji_picker_open;
    }

    pub fn get_terminal_bg_color_imm(&self) -> egui::Color32 {
        match self.color_mode {
            ColorMode::Dark => self.color_set.dark,
            ColorMode::Light => self.color_set.light,
        }
    }

    pub fn get_terminal_text_color_imm(&self) -> egui::Color32 {
        match self.color_mode {
            ColorMode::Dark => self.color_set.on_dark,
            ColorMode::Light => self.color_set.on_light,
        }
    }

    pub fn get_primary_color(&mut self) -> egui::Color32 {
        self.color_set.primary
    }

    pub fn get_primary_color_imm(&self) -> egui::Color32 {
        self.color_set.primary
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn set_color_set(&mut self, hue: f32) {
        self.color_set = utils::get_set_from_hue(hue);
    }

    pub fn set_maximized(&mut self, is_maximized: bool) {
        self.is_maximized = is_maximized;
    }

    pub fn render(&mut self, ui: &mut egui::Ui, is_active: bool) -> HeaderAction {
        let mut header_action: HeaderAction = HeaderAction::None;
        let slider_width: f32 = 200.0;  // Increased to fit slider + buttons
        
        egui::Frame::default()
            .fill(self.color_set.primary)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_width(ui.available_width());
                    
                    // Check if header is being hovered (only when active)
                    let is_header_hovered = is_active && ui.rect_contains_pointer(ui.max_rect());
                    
                    // Only show the frame if not editing
                    let show_frame = is_header_hovered && !self.is_editing_title;
                    
                    if self.is_editing_title {
                        // Show text edit when editing (always full width)
                        let text_edit = egui::TextEdit::singleline(&mut self.title)
                            .desired_width(ui.available_width())
                            .font(egui::TextStyle::Heading);
                        
                        // Style the text edit with white background
                        let response = egui::Frame::NONE
                            .fill(egui::Color32::WHITE)
                            .inner_margin(4.0)
                            .show(ui, |ui| {
                                ui.add(text_edit)
                            })
                            .inner;
                        
                        // Auto-focus the text edit
                        response.request_focus();
                        
                        // Check for Enter or Escape to stop editing
                        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                        let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));
                        
                        // Stop editing on Enter or lost focus
                        if response.lost_focus() || enter_pressed || escape_pressed {
                            self.is_editing_title = false;
                            
                            // Consume the Enter key event so terminal doesn't process it
                            if enter_pressed {
                                ui.input_mut(|i| {
                                    i.events.retain(|e| {
                                        !matches!(e, egui::Event::Key { key: egui::Key::Enter, pressed: true, .. })
                                    });
                                });
                            }
                        }
                    } else {
                        // Show label when not editing
                        let text_rect = ui.available_rect_before_wrap();
                        let text_width = if show_frame {
                            ui.available_width() - slider_width
                        } else {
                            ui.available_width()
                        };
                        let text_rect = egui::Rect::from_min_size(
                            text_rect.min,
                            egui::vec2(text_width, text_rect.height())
                        );
                        let response = ui.interact(text_rect, ui.id().with("title_label"), egui::Sense::click());
                        
                        // Draw the title text
                        ui.painter().text(
                            text_rect.left_center(),
                            egui::Align2::LEFT_CENTER,
                            &self.title,
                            egui::FontId::proportional(20.0),
                            self.color_set.on_primary,
                        );
                        
                        // Allocate space for the text
                        ui.allocate_space(egui::vec2(text_width, 20.0));
                        
                        // Start editing on click
                        if response.clicked() {
                            self.is_editing_title = true;
                        }
                    }
                    
                    if show_frame {
                        // Calculate the rect for the right-side frame
                        let available_rect = ui.available_rect_before_wrap();
                        let frame_rect = egui::Rect::from_min_size(
                            egui::pos2(available_rect.max.x - slider_width, available_rect.min.y),
                            egui::vec2(slider_width, 20.0)
                        );
                        
                        // Allocate and render the frame at the right edge
                        let ui_builder = egui::UiBuilder::new().max_rect(frame_rect);
                        ui.scope_builder(ui_builder, |ui| {
                            egui::Frame::default()
                                .fill(self.color_set.primary)
                                .show(ui, |ui| {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if window_button(ui, "×", self.color_set.light, self.color_set.on_primary) {
                                            header_action = HeaderAction::CloseTerminal;
                                        }

                                        ui.add_space(10.0);

                                        let maximize_icon = if self.is_maximized { "_" } else { "□" };
                                        if window_button(ui, maximize_icon, self.color_set.light, self.color_set.on_primary) {
                                            // Handle maximize/restore
                                            header_action = if self.is_maximized {
                                                self.is_maximized = false;
                                                HeaderAction::MinimizeTerminal
                                            } else {
                                                self.is_maximized = true;
                                                HeaderAction::MaximizeTerminal
                                            };
                                        }

                                        ui.add_space(10.0);

                                        // Add hue slider (leftmost in this group)
                                        let slider_response = ui.add(
                                            egui::Slider::new(&mut self.hue, 0.0..=360.0)
                                                .show_value(false)  // Hide the value display
                                        );
                                        
                                        // Update color set when hue changes
                                        if slider_response.changed() {
                                            self.color_set = utils::get_set_from_hue(self.hue);
                                        }
                                        
                                        ui.add_space(10.0);
                                    });
                                });
                        });
                    }
                });
            });
            
        header_action
    }
}

