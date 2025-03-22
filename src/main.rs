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
    
    builder.init();

    let mut options = NativeOptions::default();
    options.viewport = egui::ViewportBuilder::default().with_min_inner_size([720.0, 480.0]).with_icon(
        egui::IconData {
            rgba: data::icons::RS_POSTGRES_ICO.to_vec(),
            width: 64,
            height: 64,
        }
    );

    eframe::run_native(
        "Rs-Postgres",
        options,
        Box::new(|cc| Ok(Box::new(frames::Main::new(&cc.egui_ctx)))),
    ).unwrap();
}
