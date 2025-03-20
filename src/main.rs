mod frames;
mod data;
mod utils;
mod database;

use eframe::NativeOptions;

use std::env;

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    let args: Vec<String> = env::args().collect();
    log::info!("Args: {:?}", args);
    if args.contains(&String::from("--debug")) {
        log::set_max_level(log::LevelFilter::Debug);
    } else {
        log::set_max_level(log::LevelFilter::Info);
    }

    let options = NativeOptions::default();

    eframe::run_native(
        "Rs-Postgres",
        options,
        Box::new(|cc| Ok(Box::new(frames::Main::new(&cc.egui_ctx)))),
    ).unwrap();
}
