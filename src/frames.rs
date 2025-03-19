use crate::data::*;

use eframe::{egui, App};

use egui::{CentralPanel, Image, SidePanel, TopBottomPanel, Widget};

use tokio::fs;
use std::fs as std_fs;

pub struct Main {
    pub config: structs::Config,
}

impl Main {
    pub fn new(ctx: &egui::Context) -> Self {
        egui_extras::install_image_loaders(ctx);
        
        Self {
            config: structs::Config::new(),
        }
    }

    fn load_config(&mut self) {
        let config_dir = &dirs::config_dir().unwrap().join("rs-postgres");
        if !config_dir.exists() {
            std_fs::create_dir_all(config_dir).unwrap();
        }

        let config_path = &config_dir.join("config.json");
        if !config_path.exists() {
            std_fs::write(
                config_path,
                serde_json::to_string(&structs::Config::new()).unwrap()
            ).unwrap();
        }

        let config_file = std_fs::read_to_string(config_path).unwrap();
        let config: structs::Config = serde_json::from_str(&config_file).unwrap();
        self.config = config;
    }
}

impl App for Main {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.load_config();

        let server_icon = egui::Image::new(icons::SERVER)
            .max_size(egui::vec2(24.0, 24.0));
        let database_icon = egui::Image::new(icons::DATABASE)
            .max_size(egui::vec2(24.0, 24.0));

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Rs-Postgres");
            })
        });

        SidePanel::left("left_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                server_icon.ui(ui);
                ui.label("Servers");
            });

            for server in &self.config.servers {
                ui.horizontal(|ui| {
                    database_icon.clone().ui(ui);
                    ui.label(
                        format!("{} ({}:{})", server.alias, server.ip, server.port)
                    );
                });
            }

            if ui.button("Add server").clicked() {
                println!("Clicked button.");
                // TODO
            }
        });
    }
}
