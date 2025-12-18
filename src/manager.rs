use eframe::egui;

use crate::terminal::{Terminal, TerminalResponse};

pub struct TerminalManager {
    terminals: Vec<Terminal>,
    num_terminals: usize,
    max_terminals: usize,
    top_row_terminals: Vec<usize>,
    bottom_row_terminals: Vec<usize>,
    show_all: bool,
    last_hue: f32,
    active_terminal_id: Option<usize>,  // Track active terminal
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self {
            terminals: Vec::new(),
            num_terminals: 0,
            max_terminals: 6,
            top_row_terminals: Vec::new(),
            bottom_row_terminals: Vec::new(),
            show_all: true,
            last_hue: 180.0,
            active_terminal_id: None,
        }
    }
}

impl TerminalManager {
    pub fn set_dark_mode(&mut self, dark_mode: bool) {
        for &idx in &self.top_row_terminals {
            if let Some(terminal) = self.terminals.get_mut(idx) {
                terminal.set_dark_mode(dark_mode);
            }
        }
        for &idx in &self.bottom_row_terminals {
            if let Some(terminal) = self.terminals.get_mut(idx) {
                terminal.set_dark_mode(dark_mode);
            }
        }
    }

    fn set_active_terminal(&mut self, id: usize) {
        // Deactivate all terminals
        for terminal in &mut self.terminals {
            terminal.set_active(false);
        }
        
        // Activate the clicked terminal
        if let Some(terminal) = self.terminals.get_mut(id) {
            terminal.set_active(true);
            self.active_terminal_id = Some(id);
        }
    }

    pub fn resize_terminals(&mut self, available_width: f32, available_height: f32){
        let border_width = 2.0;
        
        let top_count = self.top_row_terminals.len().max(1) as f32;
        let top_terminal_width: f32 = (available_width - (border_width * top_count)) / top_count;
        let top_terminal_height: f32 = if self.bottom_row_terminals.len() > 0 { 
            available_height / 2.0
        } else {
            available_height
        };
        
        let bottom_count = self.bottom_row_terminals.len().max(1) as f32;
        let bottom_terminal_width: f32 = if self.bottom_row_terminals.len() > 0 {
            (available_width - (border_width * bottom_count)) / bottom_count
        } else {
            available_width
        };
        let bottom_terminal_height: f32 = available_height / 2.0;

        for &idx in &self.top_row_terminals {
            if let Some(terminal) = self.terminals.get_mut(idx) {
                terminal.set_width(top_terminal_width);
                terminal.set_height(top_terminal_height);
            }
        }
        for &idx in &self.bottom_row_terminals {
            if let Some(terminal) = self.terminals.get_mut(idx) {
                terminal.set_width(bottom_terminal_width);
                terminal.set_height(bottom_terminal_height);
            }
        }
    }
    
    pub fn rearrange_terminals(&mut self) {
        self.top_row_terminals.clear();
        self.bottom_row_terminals.clear();

        if self.num_terminals <= 2 {
            self.top_row_terminals = (0..self.num_terminals).collect();
        } else {
            let mid = self.num_terminals / 2;
            self.top_row_terminals = (0..mid).collect();
            self.bottom_row_terminals = (mid..self.num_terminals).collect();
        }
    }

    pub fn add_terminal(&mut self, available_width: f32, available_height: f32) -> Option<usize> {
        if self.num_terminals + 1 > 6 {
            None
        } else {
            let id = self.num_terminals;
            let mut terminal = Terminal::new(id, 100.0, 100.0, self.last_hue, !self.show_all);
            
            // Make first terminal active by default
            if self.num_terminals == 0 {
                terminal.set_active(true);
                self.active_terminal_id = Some(id);
            }
            
            self.terminals.push(terminal);
            self.num_terminals += 1;
            self.last_hue += 55.0;
            self.rearrange_terminals();
            self.resize_terminals(available_width, available_height);
            Some(self.num_terminals - 1)
        }
    }

    pub fn remove_terminal(&mut self, index: usize, available_width: f32, available_height: f32) -> Option<Terminal> {
        if index < self.terminals.len() {
            self.num_terminals -= 1;
            let removed = Some(self.terminals.remove(index));
            
            // Update IDs of all remaining terminals to match their new indices
            for (new_id, terminal) in self.terminals.iter_mut().enumerate() {
                terminal.set_id(new_id);
            }
            
            // If we removed the active terminal, activate the first one
            if self.active_terminal_id == Some(index) {
                self.active_terminal_id = None;
                if !self.terminals.is_empty() {
                    self.set_active_terminal(0);
                }
            } else if let Some(active_id) = self.active_terminal_id {
                // If the active terminal was after the removed one, adjust its ID
                if active_id > index {
                    self.active_terminal_id = Some(active_id - 1);
                }
            }
            
            self.rearrange_terminals();
            self.resize_terminals(available_width, available_height);
            removed
        } else {
            None
        }
    }

    pub fn update(&mut self, _ui: &mut egui::Ui, available_width: f32, available_height: f32){
        self.resize_terminals(available_width, available_height);
    }

    fn render_all(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.style_mut().spacing.item_spacing.y = 0.0;
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing.x = 0.0;
                for &idx in &self.top_row_terminals.clone() {
                    if let Some(terminal) = self.terminals.get_mut(idx) {
                        let terminal_response = terminal.render(ui);
                        if terminal_response == TerminalResponse::WasClicked {
                            self.set_active_terminal(idx);
                        } else if terminal_response == TerminalResponse::CloseMe { 
                            self.remove_terminal(idx, ui.available_width(), ui.available_height());
                        } else if terminal_response == TerminalResponse::MaximizeMe {
                            self.set_active_terminal(idx);
                            self.show_all = false;
                        }
                    }
                }
            });
            
            if self.bottom_row_terminals.len() > 0 {
                ui.style_mut().spacing.item_spacing.x = 0.0;
                ui.horizontal(|ui| {
                    for &idx in &self.bottom_row_terminals.clone() {
                        if let Some(terminal) = self.terminals.get_mut(idx) {
                            let terminal_response = terminal.render(ui);
                            if terminal_response == TerminalResponse::WasClicked {
                                self.set_active_terminal(idx);
                            } else if terminal_response == TerminalResponse::CloseMe { 
                                self.remove_terminal(idx, ui.available_width(), ui.available_height());
                            } else if terminal_response == TerminalResponse::MaximizeMe {
                                self.set_active_terminal(idx);
                                self.show_all = false;
                            }
                        }
                    }
                });
            }
        });
    }

    fn render_single(&mut self, ui: &mut egui::Ui) {
        // Render only the active terminal in full screen
        ui.vertical(|ui| {
            ui.style_mut().spacing.item_spacing.y = 0.0;
            
            // Get available height, reserving space for the tab bar at bottom
            let tab_bar_height = 40.0;
            let terminal_height = ui.available_height() - tab_bar_height;
            
            // Render the active terminal
            if let Some(active_id) = self.active_terminal_id {
                if let Some(terminal) = self.terminals.get_mut(active_id) {
                    // Set terminal to full width and available height
                    terminal.set_width(ui.available_width());
                    terminal.set_height(terminal_height);
                    
                    let terminal_response = terminal.render(ui);
                    if terminal_response == TerminalResponse::CloseMe {
                        self.remove_terminal(active_id, ui.available_width(), ui.available_height());
                    } else if terminal_response == TerminalResponse::MinimizeMe {
                        self.show_all = true;
                    }
                }
            }
            
            // Render tab bar at the bottom
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing.x = 5.0;
                ui.add_space(10.0);
                
                let mut clicked_terminal: Option<usize> = None;
                
                for (idx, terminal) in self.terminals.iter_mut().enumerate() {
                    let is_active = Some(idx) == self.active_terminal_id;
                    
                    let button = egui::Button::new(
                        egui::RichText::new(terminal.get_title())
                            .size(14.0)
                            .color(terminal.get_text_color())
                    )
                    .fill(if is_active {
                        terminal.get_primary_color()
                    } else {
                        egui::Color32::from_gray(60)
                    })
                    .stroke(egui::Stroke::new(
                        if is_active { 2.0 } else { 1.0 },
                        terminal.get_primary_color()
                    ));
                    
                    if ui.add(button).clicked() {
                        clicked_terminal = Some(idx);
                    }
                }
                
                // Handle click outside the loop to avoid borrow conflicts
                if let Some(idx) = clicked_terminal {
                    self.set_active_terminal(idx);
                }
            });
        });
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui) {
        if self.show_all {
            self.render_all(ui);
        } else {
            self.render_single(ui);
        }
    }
}