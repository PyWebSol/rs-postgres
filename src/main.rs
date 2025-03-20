mod frames;
mod data;
mod utils;
mod database;

use eframe::NativeOptions;

fn main() {
    let options = NativeOptions::default();

    eframe::run_native(
        "Rs-Postgres",
        options,
        Box::new(|cc| Ok(Box::new(frames::Main::new(&cc.egui_ctx)))),
    ).unwrap();
}
