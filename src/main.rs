mod frames;
mod data;
mod utils;
mod database;

use eframe::NativeOptions;
use env_logger::Builder;
use log::LevelFilter;
use std::env;

fn main() {
    let mut builder = Builder::new();

    let args: Vec<String> = env::args().collect();
    if args.contains(&String::from("--debug")) {
        builder.filter_level(LevelFilter::Debug);
    } else {
        builder.filter_level(LevelFilter::Info);
    }

    builder.filter_module("zbus", LevelFilter::Error);
    builder.filter_module("tracing", LevelFilter::Error);
    builder.filter_module("winit", LevelFilter::Error);

    builder.init();

    let mut options = NativeOptions::default();
    options.viewport = egui::ViewportBuilder::default()
        .with_min_inner_size([1024.0, 768.0])
        .with_icon(utils::load_icon());

    eframe::run_native(
        format!("Rs-Postgres (v{})", env!("CARGO_PKG_VERSION")).as_str(),
        options,
        Box::new(|cc| Ok(Box::new(frames::Main::new(&cc.egui_ctx)))),
    ).unwrap();
}
