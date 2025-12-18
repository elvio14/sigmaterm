use eframe::egui;

use crate::terminal::Terminal;

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
            show_all: false,
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
        let top_terminal_width: f32 = available_width / self.top_row_terminals.len().max(1) as f32;
        let top_terminal_height: f32 = if self.bottom_row_terminals.len() > 0 { 
            available_height / 2.0
        } else {
            available_height
        };
        let bottom_terminal_width: f32 = available_width / self.bottom_row_terminals.len().max(1) as f32;
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
            let mut terminal = Terminal::new(id, 100.0, 100.0, self.last_hue);
            
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
            
            // If we removed the active terminal, activate the first one
            if self.active_terminal_id == Some(index) {
                self.active_terminal_id = None;
                if !self.terminals.is_empty() {
                    self.set_active_terminal(0);
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

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.style_mut().spacing.item_spacing.y = 0.0;
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing.x = 0.0;
                for &idx in &self.top_row_terminals.clone() {
                    if let Some(terminal) = self.terminals.get_mut(idx) {
                        let was_clicked = terminal.render(ui);
                        if was_clicked {
                            self.set_active_terminal(idx);
                        }
                    }
                }
            });
            
            if self.bottom_row_terminals.len() > 0 {
                ui.style_mut().spacing.item_spacing.x = 0.0;
                ui.horizontal(|ui| {
                    for &idx in &self.bottom_row_terminals.clone() {
                        if let Some(terminal) = self.terminals.get_mut(idx) {
                            let was_clicked = terminal.render(ui);
                            if was_clicked {
                                self.set_active_terminal(idx);
                            }
                        }
                    }
                });
            }
        });
    }
}