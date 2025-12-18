use eframe::egui;
use std::sync::Arc;

mod header;
mod utils;
mod terminal;
mod manager;
mod parser;
mod window;

use header::Header;
use utils::ColorSet;
use manager::TerminalManager;
use window::WindowBar;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Sigmaterm")
            .with_inner_size([800.0, 600.0])
            .with_resizable(true)
            .with_decorations(false), // Disable native window decorations
        ..Default::default()
    };
    
    eframe::run_native(
        "Sigmaterm",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(Sigmaterm::new()))
        }),
    )
}

fn setup_fonts(ctx: &egui::Context){
    let mut fonts = egui::FontDefinitions::default();
    // JetBrains
    fonts.font_data.insert("jetbrains".to_owned(), 
        Arc::new(egui::FontData::from_static(include_bytes!("../assets/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Regular.ttf")))
    );

    fonts.font_data.insert(
        "emoji".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!("../assets/Noto_Color_Emoji/NotoColorEmoji-Regular.ttf")))
    );

    // Set up font families with fallback
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(0, "jetbrains".to_owned());
    
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("emoji".to_owned());
    
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "jetbrains".to_owned());
    
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .push("emoji".to_owned());

    ctx.set_fonts(fonts);
}

#[derive(Default)]
struct Sigmaterm {
    text: String,
    terminal_manager: TerminalManager,
    window_bar: WindowBar,
}

impl Sigmaterm {
    fn new() -> Self {
        let mut app = Self::default();
        app.terminal_manager.add_terminal(800.0, 600.0);
        app.terminal_manager.add_terminal(800.0, 600.0);
        app
    }
}

impl eframe::App for Sigmaterm {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Render the window bar at the top
        let should_add_terminal = self.window_bar.render(ctx, frame);
        let dark_mode = self.window_bar.is_dark_mode();
        
        egui::CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(0.0))
            .show(ctx, |ui| {
            // Add new terminal if the button was clicked
            if should_add_terminal {
                self.terminal_manager.add_terminal(ui.available_width(), ui.available_height());
            }
            self.terminal_manager.set_dark_mode(dark_mode);
            self.terminal_manager.update(ui, ui.available_width(), ui.available_height());
            self.terminal_manager.render(ui);
        });
    }
}   