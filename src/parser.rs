use eframe::egui;
use crate::utils::ColorSet;

pub struct TerminalOutput {
    pub text: String,
    pub color: egui::Color32,
    pub bold: bool,
}

pub fn parse_ansi_output(output: &str, color_set: &ColorSet, default_color: egui::Color32) -> Vec<TerminalOutput> {
    let mut segments = Vec::new();
    let mut current_color = default_color;
    let mut current_text = String::new();
    let mut bold = false;
    
    let mut chars = output.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Save current segment before processing escape sequence
            if !current_text.is_empty() {
                segments.push(TerminalOutput {
                    text: current_text.clone(),
                    color: current_color,
                    bold,
                });
                current_text.clear();
            }
            
            // Check what type of escape sequence this is
            match chars.peek() {
                Some(&'[') => {
                    // CSI (Control Sequence Introducer) - most common
                    chars.next(); // consume '['
                    let mut code = String::new();
                    
                    // Read until a letter (command character)
                    while let Some(&ch) = chars.peek() {
                        if ch.is_ascii_alphabetic() || ch == 'm' {
                            chars.next(); // consume the command character
                            break;
                        }
                        code.push(chars.next().unwrap());
                    }
                    
                    // Only parse color codes (those ending with 'm' or in our code string)
                    if code.chars().all(|c| c.is_ascii_digit() || c == ';') {
                        // Parse SGR (Select Graphic Rendition) codes
                        for part in code.split(';') {
                            match part {
                                "0" | "00" => {
                                    current_color = default_color;
                                    bold = false;
                                }
                                "1" | "01" => bold = true,
                                "31" => current_color = color_set.alert,       // Red -> alert
                                "32" => current_color = color_set.primary,     // Green -> primary
                                "33" => current_color = color_set.warning,     // Yellow -> warning
                                "34" => current_color = color_set.alternate_1,     // Blue -> alternate_1
                                "35" => current_color = color_set.alternate_2,       // Magenta -> alternate_2
                                "36" => current_color = color_set.alternate_3,     // Cyan -> alternate_3
                                _ => {} // Ignore unknown codes
                            }
                        }
                    }
                    // All other CSI sequences are ignored (cursor movement, etc.)
                }
                Some(&']') => {
                    // OSC (Operating System Command) - like window title
                    chars.next(); // consume ']'
                    
                    // Read until BEL (\x07) or ST (ESC \)
                    while let Some(ch) = chars.next() {
                        if ch == '\x07' {
                            break;
                        }
                        if ch == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next(); // consume '\'
                            break;
                        }
                    }
                }
                _ => {
                    // Other escape sequences - consume next character
                    chars.next();
                }
            }
        } else {
            current_text.push(ch);
        }
    }
    
    // Add final segment
    if !current_text.is_empty() {
        segments.push(TerminalOutput {
            text: current_text,
            color: current_color,
            bold,
        });
    }
    
    segments
}