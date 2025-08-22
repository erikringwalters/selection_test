use bevy::color::Color;
use bevy::log::warn;

pub const MAX_RGB: f32 = 255.;

pub const LINE: &str = "#FFFFFF";
pub const HOVER: &str = "#66CCCC";
pub const PRESSED: &str = "#CCCC00";
pub const SQUOOSH_ORANGE: &str = "#FF6600";
pub const SUNRISE_ORANGE: &str = "#F78B17";
pub const CREAMSICLE_ORANGE: &str = "#FA821E";
pub const AMBER_ORANGE: &str = "#E49B5D";
pub const COOL_BLUE: &str = "#7B9695";
pub const SAGE_GREEN: &str = "#78997A";
pub const GOLD_YELLOW: &str = "#EBC06D";

pub fn color_from_hex(hex: &str) -> Color {
    if hex.len() < 7 || hex.len() == 8 || hex.len() > 9 || !hex.starts_with('#') {
        warn!("Invalid hex code: {:?}", hex);
        return Color::default();
    }
    let r = &hex[1..3];
    let g = &hex[3..5];
    let b = &hex[5..7];
    let a = if hex.len() > 7 { &hex[7..9] } else { "FF" };

    let mut c: [u8; 4] = [0xFF; 4];
    let w = [r, g, b, a];
    for i in 0..w.len() {
        c[i] = u8::from_str_radix(w[i], 16).unwrap();
    }

    Color::srgba(
        c[0] as f32 / MAX_RGB,
        c[1] as f32 / MAX_RGB,
        c[2] as f32 / MAX_RGB,
        c[3] as f32 / MAX_RGB,
    )
}
