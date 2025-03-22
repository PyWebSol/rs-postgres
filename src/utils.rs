use crate::data::icons;

pub fn load_icon() -> egui::IconData {
	let (icon_rgba, icon_width, icon_height) = {
		let image = image::load_from_memory(icons::RS_POSTGRES_PNG)
			.expect("Failed to open icon path")
			.into_rgba8();
		let (width, height) = image.dimensions();
		let rgba = image.into_raw();
		(rgba, width, height)
	};
	
	egui::IconData {
		rgba: icon_rgba,
		width: icon_width,
		height: icon_height,
	}
}