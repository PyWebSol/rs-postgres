mod frames;
mod data;
mod utils;
mod database;

use eframe::NativeOptions;

use std::env;

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    let args: Vec<String> = env::args().collect();
    if args.contains(&String::from("--debug")) {
        log::set_max_level(log::LevelFilter::Debug);
    } else {
        log::set_max_level(log::LevelFilter::Info);
    }

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
