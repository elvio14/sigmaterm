use eframe::egui;
use egui::scroll_area::ScrollBarVisibility;
use ptyprocess::PtyProcess;
use std::process::Command;
use std::io::{Write, Read};
use std::os::unix::io::AsRawFd;

use crate::header::{Header, ColorMode};
use crate::parser::{parse_ansi_output, TerminalOutput};

// Terminal ===========================================

pub struct Terminal {
    id: usize,
    is_active: bool,  // Track if terminal is active
    header: Header,
    pub width: f32,
    pub height: f32,
    pty: Option<PtyProcess>,
    output_buffer: String,
    cursor_visible: bool,
    last_cursor_toggle: std::time::Instant,
    command_buffer: String,
    command_history: Vec<String>,
    history_index: Option<usize>,  // Current position in history (None = not browsing)
}

impl Terminal {
    pub fn new(id: usize, width: f32, height: f32, hue: f32) -> Self {
        let pty = PtyProcess::spawn(Command::new("bash")).ok();
        
        Self {
            id,
            is_active: false,
            header: Header::new(hue),
            width,
            height,
            pty,
            output_buffer: String::new(),
            cursor_visible: true,
            last_cursor_toggle: std::time::Instant::now(),
            command_buffer: "".to_string(),
            command_history: Vec::new(),
            history_index: None,
        }
    }

    pub fn set_dark_mode(&mut self, dark_mode: bool) {
        self.header.set_dark_mode(dark_mode);
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
        
        // If deactivating, stop title editing
        if !active {
            self.header.stop_editing_title();
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }   

    pub fn set_height(&mut self, height: f32){
        self.height = height;
    }

    pub fn read_output(&mut self) {
        if let Some(pty) = &mut self.pty {
            if let Ok(mut stream) = pty.get_raw_handle() {
                let fd = stream.as_raw_fd();
                unsafe {
                    let flags = libc::fcntl(fd, libc::F_GETFL, 0);
                    libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                }
                
                let mut buffer = [0u8; 4096];
                match stream.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        let new_output = String::from_utf8_lossy(&buffer[..n]);
                        self.output_buffer.push_str(&new_output);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn send_input(&mut self, input: &str) {
        if let Some(pty) = &mut self.pty {
            if let Ok(mut stream) = pty.get_raw_handle() {
                let _ = writeln!(stream, "{}", input);
            }
        }
    }

    // Returns true if terminal was clicked
    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        let mut was_clicked = false;
        let mut header_was_clicked = false;
        
        ui.push_id(self.id, |ui| {
            self.read_output();
            
            if self.last_cursor_toggle.elapsed().as_millis() > 500 {
                self.cursor_visible = !self.cursor_visible;
                self.last_cursor_toggle = std::time::Instant::now();
            }
            
            // visual indicator for active terminal
            let stroke = if self.is_active {
                egui::Stroke::new(2.0, self.header.get_primary_color())
            } else {
                egui::Stroke::new(1.0, egui::Color32::from_gray(100))
            };
            
            let frame_response = egui::Frame::default()
                .fill(self.header.get_terminal_bg_color())
                .stroke(stroke)  // border to show active state
                .show(ui, |ui| {
                    ui.set_width(self.width);
                    ui.set_height(self.height);
                    
                    // Allocate the full rect for the terminal
                    let rect = ui.available_rect_before_wrap();

                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui|{
                        header_was_clicked = self.header.render(ui);
                        
                        let color_set = self.header.color_set.clone();
                        let default_color = self.header.get_terminal_text_color();
                        
                        let scroll_area = egui::ScrollArea::vertical()
                            .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
                            .auto_shrink([false; 2])
                            .stick_to_bottom(true);
                        
                        scroll_area.show(ui, |ui| {
                            ui.style_mut().spacing.item_spacing.x = 0.0;
                            
                            // Add left padding by wrapping content
                            ui.horizontal(|ui| {
                                ui.add_space(8.0); // Left padding
                                
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    
                            let parsed_segments = parse_ansi_output(
                                &self.output_buffer,
                                &color_set,
                                default_color
                            );
                            
                            let mut current_line_segments: Vec<TerminalOutput> = Vec::new();
                            
                            for segment in parsed_segments {
                                let text = segment.text.replace("\r\n", "\n");
                                let lines: Vec<&str> = text.split(|c| c == '\n' || c == '\r').collect();
                                
                                for (i, line) in lines.iter().enumerate() {
                                    if i > 0 {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.spacing_mut().item_spacing.x = 0.0;
                                            if current_line_segments.is_empty() {
                                                ui.label(egui::RichText::new(" ")
                                                    .size(14.0)
                                                    .monospace()
                                                );
                                            } else {
                                                for seg in &current_line_segments {
                                                    let mut text = egui::RichText::new(&seg.text)
                                                        .size(14.0)
                                                        .color(seg.color)
                                                        .monospace();
                                                    if seg.bold {
                                                        text = text.strong();
                                                    }
                                                    ui.label(text);
                                                }
                                            }
                                        });
                                        current_line_segments.clear();
                                    }
                                    
                                    if !line.is_empty() {
                                        current_line_segments.push(TerminalOutput {
                                            text: line.to_string(),
                                            color: segment.color,
                                            bold: segment.bold,
                                        });
                                    }
                                }
                            }
                            
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                
                                for seg in &current_line_segments {
                                    let mut text = egui::RichText::new(&seg.text)
                                        .size(14.0)
                                        .color(seg.color)
                                        .monospace();
                                    if seg.bold {
                                        text = text.strong();
                                    }
                                    ui.label(text);
                                }
                                
                                // Only show command buffer and cursor if active
                                if self.is_active {
                                    if !self.command_buffer.is_empty() {
                                        ui.label(egui::RichText::new(&self.command_buffer)
                                            .size(14.0)
                                            .color(default_color)
                                            .monospace()
                                        );
                                    }
                                    
                                    if self.cursor_visible {
                                        ui.label(egui::RichText::new("█")
                                            .size(14.0)
                                            .color(default_color)
                                            .monospace()
                                        );
                                    } else {
                                        ui.label(egui::RichText::new("▂")
                                            .size(14.0)
                                            .monospace()
                                        );
                                    }
                                }
                            });
                                }); // Close vertical
                            }); // Close horizontal
                        }); // Close ScrollArea
                    });
                    
                    rect  // Return the rect from the closure
                });
            
            // Only check for terminal click if terminal is not active
            // When active, only header clicks matter (for editing title)
            if !self.is_active {
                // Make the frame clickable using its actual rect
                let response = ui.interact(
                    frame_response.inner,
                    ui.id().with("terminal_click"),
                    egui::Sense::click()
                );
                
                was_clicked = response.clicked();
            }
            
            // Only handle keyboard input if active AND not editing title
            if self.is_active && !self.header.is_editing_title() {
                self.handle_keyboard_input(ui);
            }
            
            ui.ctx().request_repaint();
        });
        
        was_clicked
    }

    fn handle_keyboard_input(&mut self, ui: &mut egui::Ui) {
        let mut command_to_send: Option<String> = None;
        let mut should_clear_buffer = false;
        let mut should_pop_char = false;
        let mut ctrl_c_pressed = false;
        
        ui.input(|i| {
            for event in &i.events {
                if let egui::Event::Text(text) = event {
                    self.command_buffer.push_str(text);
                    // Reset history browsing when typing
                    self.history_index = None;
                } else if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                    match key {
                        egui::Key::Enter => {
                            // Don't add empty commands to history
                            if !self.command_buffer.trim().is_empty() {
                                // Add to history
                                self.command_history.push(self.command_buffer.clone());
                            }
                            command_to_send = Some(self.command_buffer.clone());
                            should_clear_buffer = true;
                            // Reset history browsing
                            self.history_index = None;
                        }
                        egui::Key::Backspace => {
                            should_pop_char = true;
                            // Reset history browsing when editing
                            self.history_index = None;
                        }
                        egui::Key::ArrowUp => {
                            // Navigate backwards in history (older commands)
                            if !self.command_history.is_empty() {
                                if let Some(idx) = self.history_index {
                                    if idx > 0 {
                                        self.history_index = Some(idx - 1);
                                        self.command_buffer = self.command_history[idx - 1].clone();
                                    }
                                } else {
                                    // Start browsing from the most recent command
                                    let last_idx = self.command_history.len() - 1;
                                    self.history_index = Some(last_idx);
                                    self.command_buffer = self.command_history[last_idx].clone();
                                }
                            }
                        }
                        egui::Key::ArrowDown => {
                            // Navigate forwards in history (newer commands)
                            if let Some(idx) = self.history_index {
                                if idx < self.command_history.len() - 1 {
                                    self.history_index = Some(idx + 1);
                                    self.command_buffer = self.command_history[idx + 1].clone();
                                } else {
                                    // Reached the end, clear buffer
                                    self.history_index = None;
                                    self.command_buffer.clear();
                                }
                            }
                        }
                        egui::Key::C if modifiers.ctrl => {
                            ctrl_c_pressed = true;
                        }
                        _ => {}
                    }
                }
            }
        });
        
        if let Some(command) = command_to_send {
            self.send_input(&command);
        }
        if should_clear_buffer {
            self.command_buffer.clear();
        }
        if should_pop_char {
            self.command_buffer.pop();
        }
        if ctrl_c_pressed {
            self.send_input("\x03");
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        if let Some(mut pty) = self.pty.take() {
            match pty.exit(true) {
                Ok(_) => {
                    // PTY process successfully terminated
                }
                Err(e) => {
                    eprintln!("Warning: Failed to cleanly exit PTY process: {}", e);
                }
            }
        }
    }
}