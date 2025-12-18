use eframe::egui;
use egui::scroll_area::ScrollBarVisibility;
use ptyprocess::PtyProcess;
use std::process::Command;
use std::io::{Write, Read};
use std::os::unix::io::AsRawFd;

use crate::header::{Header, HeaderAction};
use crate::parser::{parse_ansi_output, TerminalOutput};

// Terminal ===========================================
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalResponse {
    None,
    WasClicked,
    CloseMe,
    MaximizeMe,
    MinimizeMe
}

pub struct Terminal {
    id: usize,
    is_active: bool,
    header: Header,
    pub width: f32,
    pub height: f32,
    pty: Option<PtyProcess>,
    output_buffer: String,
    text_size: f32,
    command_buffer: String,
    cursor_visible: bool,
    last_cursor_toggle: std::time::Instant,
    raw_mode: bool,  // True when in interactive program (SSH, vim, etc.)
    is_maximized: bool
}

impl Terminal {
    pub fn new(id: usize, width: f32, height: f32, hue: f32, is_maximized:bool) -> Self {
        let mut pty = PtyProcess::spawn(Command::new("bash")).ok();
        
        // Set initial PTY size (80 cols x 24 rows is a common default)
        if let Some(ref mut p) = pty {
            let _ = p.set_window_size(80, 24);
        }
        
        Self {
            id,
            is_active: false,
            header: Header::new(hue, is_maximized),
            width,
            height,
            pty,
            output_buffer: String::new(),
            text_size: 18.0,
            command_buffer: String::new(),
            cursor_visible: true,
            last_cursor_toggle: std::time::Instant::now(),
            raw_mode: false,
            is_maximized: is_maximized
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

    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }   

    pub fn set_height(&mut self, height: f32){
        self.height = height;
    }

    pub fn set_maximized(&mut self, is_maximized:bool){
        self.is_maximized = is_maximized;
        self.header.set_maximized(is_maximized);
    }
    pub fn get_title(&self) -> String {
        self.header.get_title().to_string()
    }

    pub fn get_primary_color(&self) -> egui::Color32 {
        self.header.get_primary_color_imm()
    }

    pub fn get_text_color(&self) -> egui::Color32 {
        self.header.get_terminal_text_color_imm()
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
                        
                        // Detect raw mode: if output contains certain escape sequences
                        // that indicate screen manipulation (alternate screen buffer, cursor positioning, etc.)
                        // NOTE: Disabled for now - vim/fullscreen apps need a proper terminal grid
                        // which is complex to implement. For now, only SSH works reasonably.
                        if false && (new_output.contains("\x1b[?1049h") || // Alternate screen buffer
                           new_output.contains("\x1b[?25l") ||   // Hide cursor (vim, ssh)
                           new_output.contains("\x1b[2J") ||     // Clear screen
                           new_output.contains("\x1b[H\x1b[2J")) { // Home + clear
                            self.raw_mode = true;
                        }
                        
                        // Exit raw mode when we see the alternate screen buffer exit
                        if new_output.contains("\x1b[?1049l") {
                            self.raw_mode = false;
                            self.output_buffer.clear(); // Clear buffer when exiting raw mode
                        }
                        
                        self.output_buffer.push_str(&new_output);
                        
                        // Keep buffer size reasonable (last 50KB of output)
                        if self.output_buffer.len() > 50000 {
                            let keep_from = self.output_buffer.len() - 50000;
                            self.output_buffer = self.output_buffer[keep_from..].to_string();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Returns true if terminal was clicked
    pub fn render(&mut self, ui: &mut egui::Ui) -> TerminalResponse {
        let mut terminal_response: TerminalResponse = TerminalResponse::None;
        let mut header_action: HeaderAction = HeaderAction::None;
        
        ui.push_id(self.id, |ui| {
            self.read_output();
            
            // Toggle cursor visibility
            if self.last_cursor_toggle.elapsed().as_millis() > 500 {
                self.cursor_visible = !self.cursor_visible;
                self.last_cursor_toggle = std::time::Instant::now();
            }
            
            let stroke = if self.is_active {
                egui::Stroke::new(2.0, self.header.get_primary_color())
            } else {
                egui::Stroke::new(2.0, egui::Color32::from_gray(100))
            };
            
            let frame_response = egui::Frame::default()
                .fill(self.header.get_terminal_bg_color_imm())
                .stroke(stroke)  // border to show active state
                .show(ui, |ui| {
                    ui.set_max_width(self.width-2.0);
                    ui.set_height(self.height-5.5);
                    
                    // Allocate the full rect for the terminal
                    let rect = ui.available_rect_before_wrap();

                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui|{
                        header_action = self.header.render(ui, self.is_active);

                        match header_action {
                            HeaderAction::CloseTerminal => terminal_response = TerminalResponse::CloseMe,
                            HeaderAction::MinimizeTerminal => terminal_response = TerminalResponse::MinimizeMe,
                            HeaderAction::MaximizeTerminal => terminal_response = TerminalResponse::MaximizeMe,
                            HeaderAction::None => {},
                        };
                        
                        let color_set = self.header.color_set.clone();
                        let default_color = self.header.get_terminal_text_color_imm();
                        
                        let scroll_area = egui::ScrollArea::vertical()
                            .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
                            .auto_shrink([false; 2])
                            .stick_to_bottom(true)
                            .max_width(self.width - 4.0); // Constrain width to prevent expansion
                        
                        scroll_area.show(ui, |ui| {
                            ui.set_max_width(self.width - 4.0); // Also constrain the inner ui
                            ui.style_mut().spacing.item_spacing.x = 0.0;
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                            
                            // Add left padding by wrapping content
                            ui.horizontal(|ui| {
                                ui.add_space(8.0); // Left padding
                                
                                ui.vertical(|ui| {
                                    ui.set_max_width(self.width - 20.0); // Constrain content width
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    
                            let parsed_segments = parse_ansi_output(
                                &self.output_buffer,
                                &color_set,
                                default_color
                            );
                            
                            if self.raw_mode {
                                // In raw mode, just show the raw text as-is in a simple label
                                // This won't be perfect but works for basic interactive programs
                                let raw_text = self.output_buffer
                                    .replace("\x1b[?1049h", "") // Remove alternate screen enter
                                    .replace("\x1b[?1049l", "") // Remove alternate screen exit
                                    .replace("\x1b[?25l", "")   // Remove hide cursor
                                    .replace("\x1b[?25h", "");  // Remove show cursor
                                
                                ui.label(egui::RichText::new(raw_text)
                                    .size(self.text_size)
                                    .color(default_color)
                                    .monospace()
                                );
                            } else {
                                // Normal mode: use the existing line-by-line rendering
                            
                            let mut current_line_segments: Vec<TerminalOutput> = Vec::new();
                            
                            for segment in parsed_segments {
                                let text = segment.text.replace("\r\n", "\n");
                                let lines: Vec<&str> = text.split(|c| c == '\n' || c == '\r').collect();
                                
                                for (i, line) in lines.iter().enumerate() {
                                    if i > 0 {
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 0.0;
                                            if current_line_segments.is_empty() {
                                                ui.label(egui::RichText::new(" ")
                                                    .size(self.text_size)
                                                    .monospace()
                                                );
                                            } else {
                                                for seg in &current_line_segments {
                                                    let mut text = egui::RichText::new(&seg.text)
                                                        .size(self.text_size)
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
                                        .size(self.text_size)
                                        .color(seg.color)
                                        .monospace();
                                    if seg.bold {
                                        text = text.strong();
                                    }
                                    ui.label(text);
                                }
                                
                                // Show command buffer and cursor if active and NOT in raw mode
                                if self.is_active && !self.raw_mode {
                                    if !self.command_buffer.is_empty() {
                                        ui.label(egui::RichText::new(&self.command_buffer)
                                            .size(self.text_size)
                                            .color(default_color)
                                            .monospace()
                                        );
                                    }
                                    
                                    // Show cursor
                                    if self.cursor_visible {
                                        ui.label(egui::RichText::new("█")
                                            .size(self.text_size)
                                            .color(default_color)
                                            .monospace()
                                        );
                                    } else {
                                        ui.label(egui::RichText::new("▂")
                                            .size(self.text_size)
                                            .monospace()
                                        );
                                    }
                                }
                            });
                            } // Close else block
                                }); // Close vertical
                            }); // Close horizontal
                        }); // Close ScrollArea
                    });
                    
                    rect 
                });
            
            if !self.is_active {
                let response = ui.interact(
                    frame_response.inner,
                    ui.id().with("terminal_click"),
                    egui::Sense::click()
                );
                
                if response.clicked() { terminal_response = TerminalResponse::WasClicked;}
            }
            
            if self.is_active && !self.header.is_editing_title() {
                self.handle_keyboard_input(ui);
            }
            
            ui.ctx().request_repaint();
        });
        
        terminal_response
    }

    fn handle_keyboard_input(&mut self, ui: &mut egui::Ui) {
        ui.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Text(text) => {
                        if self.raw_mode {
                            // In raw mode, send text directly to PTY
                            if let Some(pty) = &mut self.pty {
                                if let Ok(mut stream) = pty.get_raw_handle() {
                                    let _ = write!(stream, "{}", text);
                                    let _ = stream.flush();
                                }
                            }
                        } else {
                            // In normal mode, add to command buffer
                            self.command_buffer.push_str(text);
                        }
                    }
                    egui::Event::Key { key, pressed: true, modifiers, .. } => {
                        if self.raw_mode {
                            // In raw mode, send all keys directly to PTY
                            if let Some(pty) = &mut self.pty {
                                if let Ok(mut stream) = pty.get_raw_handle() {
                                    let key_seq = match key {
                                        egui::Key::Enter => "\r",
                                        egui::Key::Backspace => "\x7f",
                                        egui::Key::Tab => "\t",
                                        egui::Key::Escape => "\x1b",
                                        egui::Key::ArrowUp => "\x1b[A",
                                        egui::Key::ArrowDown => "\x1b[B",
                                        egui::Key::ArrowRight => "\x1b[C",
                                        egui::Key::ArrowLeft => "\x1b[D",
                                        egui::Key::Home => "\x1b[H",
                                        egui::Key::End => "\x1b[F",
                                        egui::Key::PageUp => "\x1b[5~",
                                        egui::Key::PageDown => "\x1b[6~",
                                        egui::Key::Delete => "\x1b[3~",
                                        egui::Key::C if modifiers.ctrl => "\x03",
                                        egui::Key::D if modifiers.ctrl => "\x04",
                                        egui::Key::Z if modifiers.ctrl => "\x1a",
                                        egui::Key::L if modifiers.ctrl => "\x0c",
                                        _ => "",
                                    };
                                    
                                    if !key_seq.is_empty() {
                                        let _ = write!(stream, "{}", key_seq);
                                        let _ = stream.flush();
                                    }
                                }
                            }
                        } else {
                            // In normal mode, handle keys for command buffer
                            match key {
                                egui::Key::Enter => {
                                    // Send command to PTY
                                    if let Some(pty) = &mut self.pty {
                                        if let Ok(mut stream) = pty.get_raw_handle() {
                                            let _ = write!(stream, "{}\n", self.command_buffer);
                                            let _ = stream.flush();
                                        }
                                    }
                                    self.command_buffer.clear();
                                }
                                egui::Key::Backspace => {
                                    self.command_buffer.pop();
                                }
                                egui::Key::C if modifiers.ctrl => {
                                    // Send Ctrl+C
                                    if let Some(pty) = &mut self.pty {
                                        if let Ok(mut stream) = pty.get_raw_handle() {
                                            let _ = write!(stream, "\x03");
                                            let _ = stream.flush();
                                        }
                                    }
                                    self.command_buffer.clear();
                                }
                                egui::Key::D if modifiers.ctrl => {
                                    // Send Ctrl+D
                                    if let Some(pty) = &mut self.pty {
                                        if let Ok(mut stream) = pty.get_raw_handle() {
                                            let _ = write!(stream, "\x04");
                                            let _ = stream.flush();
                                        }
                                    }
                                }
                                egui::Key::L if modifiers.ctrl => {
                                    // Send Ctrl+L (clear screen)
                                    if let Some(pty) = &mut self.pty {
                                        if let Ok(mut stream) = pty.get_raw_handle() {
                                            let _ = write!(stream, "\x0c");
                                            let _ = stream.flush();
                                        }
                                    }
                                }
                                // Send arrow keys and other special keys to PTY
                                _ => {
                                    if let Some(pty) = &mut self.pty {
                                        if let Ok(mut stream) = pty.get_raw_handle() {
                                            let key_seq = match key {
                                                egui::Key::Tab => "\t",
                                                egui::Key::Escape => "\x1b",
                                                egui::Key::ArrowUp => "\x1b[A",
                                                egui::Key::ArrowDown => "\x1b[B",
                                                egui::Key::ArrowRight => "\x1b[C",
                                                egui::Key::ArrowLeft => "\x1b[D",
                                                egui::Key::Home => "\x1b[H",
                                                egui::Key::End => "\x1b[F",
                                                egui::Key::PageUp => "\x1b[5~",
                                                egui::Key::PageDown => "\x1b[6~",
                                                egui::Key::Delete => "\x1b[3~",
                                                _ => "",
                                            };
                                            
                                            if !key_seq.is_empty() {
                                                let _ = write!(stream, "{}", key_seq);
                                                let _ = stream.flush();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
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