use eframe::egui;

fn hsl_to_egui_color32(h: f32, s: f32, l: f32) -> egui::Color32 {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    
    egui::Color32::from_rgb(    
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8)
}

#[derive(Clone)]
pub struct ColorSet {
    pub primary: egui::Color32,
    pub light: egui::Color32,
    pub dark: egui::Color32,

    pub on_primary: egui::Color32,
    pub on_light: egui::Color32,
    pub on_dark: egui::Color32,

    pub alert: egui::Color32,
    pub warning: egui::Color32,

    pub alternate_1: egui::Color32,
    pub alternate_2: egui::Color32,
    pub alternate_3: egui::Color32
}

impl Default for ColorSet {
    fn default() -> Self {
        get_set_from_hue(180.0)
    }
}

pub fn get_set_from_hue(h: f32) -> ColorSet {
    ColorSet  {
        primary: hsl_to_egui_color32(h, 0.6, 0.6),
        light: hsl_to_egui_color32((h + 10.0) % 360.0,  0.6, 0.95),
        dark: hsl_to_egui_color32((h - 10.0 + 360.0) % 360.0,  0.1, 0.15),
        on_primary: hsl_to_egui_color32(h, 0.6, 0.2),
        on_light: egui::Color32::BLACK,
        on_dark: egui::Color32::WHITE,
        alert: egui::Color32::RED,
        warning: egui::Color32::YELLOW,
        alternate_1: hsl_to_egui_color32((h + 90.0) % 360.0,  0.6, 0.6),
        alternate_2: hsl_to_egui_color32((h + 180.0) % 360.0,  0.6, 0.6),
        alternate_3: hsl_to_egui_color32((h + 270.0) % 360.0,  0.6, 0.6),
    }
}