use eframe::egui;

use crate::utils::{self, ColorSet, get_set_from_hue};

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
        }
    }
}

impl Header {
    pub fn new(hue: f32) -> Self {
        Self {
            title: "Untitled Terminal".to_string(),
            emoji_picker_open: false,
            color_picker_open: false,
            color_set: utils::get_set_from_hue(hue),
            color_mode: ColorMode::Dark,
            is_editing_title: false,
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

    pub fn get_terminal_bg_color(&mut self) -> egui::Color32 {
        match self.color_mode {
            ColorMode::Dark => self.color_set.dark,
            ColorMode::Light => self.color_set.light,
        }
    }

    pub fn get_terminal_text_color(&mut self) -> egui::Color32 {
        match self.color_mode {
            ColorMode::Dark => self.color_set.on_dark,
            ColorMode::Light => self.color_set.on_light,
        }
    }

    pub fn get_primary_color(&mut self) -> egui::Color32 {
        self.color_set.primary
    }

    pub fn set_color_set(&mut self, hue: f32) {
        self.color_set = utils::get_set_from_hue(hue);
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        let mut header_was_clicked = false;
        
        egui::Frame::default()
            .fill(self.color_set.primary)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_width(ui.available_width());
                    
                    if self.is_editing_title {
                        // Show text edit when editing
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.title)
                                .desired_width(ui.available_width())
                                .font(egui::TextStyle::Heading)
                                .text_color(self.color_set.on_primary)
                        );
                        
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
                        // Show label when not editing - use interact for better click detection
                        let text_rect = ui.available_rect_before_wrap();
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
                        ui.allocate_space(egui::vec2(ui.available_width(), 20.0));
                        
                        // Start editing on click
                        if response.clicked() {
                            self.is_editing_title = true;
                            header_was_clicked = true;
                        }
                        
                        // Mark as interacted if hovered to prevent terminal activation
                        if response.hovered() {
                            header_was_clicked = true;
                        }
                    }
                });
            });
        
        header_was_clicked
    }
}

