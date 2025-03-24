use egui::{include_image, ImageSource};

pub const RS_POSTGRES: ImageSource = include_image!("../../assets/logo.svg");
pub const RS_POSTGRES_PNG: &[u8; 5225] = include_bytes!("../../assets/logo.png");

pub const WARNING_LIGHT: ImageSource = include_image!("../../assets/warning_light.svg");
pub const WARNING_DARK: ImageSource = include_image!("../../assets/warning_dark.svg");
