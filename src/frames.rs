use crate::data::*;

use eframe::{egui, App};

use egui::{
    CentralPanel, Window, SidePanel,
    Widget, Layout, Align,
    TextEdit, Color32, Button,
    CollapsingHeader, Id, Sense,
    containers::popup
};

use std::fs as std_fs;

pub struct Main {
    config: structs::Config,
    add_server_window: structs::AddServerWindow,
}

impl Main {
    pub fn new(ctx: &egui::Context) -> Self {
        egui_extras::install_image_loaders(ctx);
        
        Self {
            config: structs::Config::new(),
            add_server_window: structs::AddServerWindow::new(),
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

    fn save_config(&mut self) {
        let config_dir = &dirs::config_dir().unwrap().join("rs-postgres");
        let config_path = &config_dir.join("config.json");

        std_fs::write(
            config_path,
            serde_json::to_string(&self.config).unwrap()
        ).unwrap();
    }
}

impl App for Main {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.load_config();

        let server_icon = egui::Image::new(icons::SERVER)
            .max_size(egui::vec2(32.0, 32.0));
        let database_icon = egui::Image::new(icons::DATABASE)
            .max_size(egui::vec2(32.0, 32.0));
        let plus_icon = egui::Image::new(icons::PLUS)
            .max_size(egui::vec2(32.0, 32.0));

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Rs-Postgres");
            })
        });

        SidePanel::left("left_panel").show(ctx, |ui| {
            CollapsingHeader::new("Servers").show(ui, |ui| {
                for server in &self.config.servers {
                    ui.horizontal(|ui| {
                        database_icon.clone().ui(ui);

                        let id = Id::new(format!("server:{}:{}:{}", server.ip, server.port, server.user));
                        let button = ui.add(Button::new(format!("{} ({}:{})", server.alias, server.ip, server.port)));
                        
                        let interact_response = ui.interact(
                            button.rect,
                            id,
                            Sense::click(),
                        );
                        if interact_response.clicked() {
                            println!("Server clicked");
                            // TODO
                        } else if interact_response.secondary_clicked() {
                            ui.memory_mut(|mem| mem.open_popup(id));
                        }

                        if ui.memory(|mem| mem.is_popup_open(id)) {
                            interact_response.context_menu(|ui| {
                                ui.label("test");
                            });
                        }
                    });
                }
            });

            if self.add_server_window.show {
                Window::new("New server").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.add(TextEdit::singleline(&mut self.add_server_window.name_field));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Server address");
                        ui.add(TextEdit::singleline(&mut self.add_server_window.ip_field));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Port");
                        let is_error: bool = self.add_server_window.port_field.parse::<u16>().is_err();
                        let mut field = TextEdit::singleline(&mut self.add_server_window.port_field);
                        if is_error {
                            field = field.text_color(
                                Color32::from_rgb(255, 0, 0)
                            );
                        }
                        ui.add(field);
                    });
                    ui.horizontal(|ui| {
                        ui.label("User");
                        ui.add(TextEdit::singleline(&mut self.add_server_window.user_field));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Password");
                        ui.add(TextEdit::singleline(&mut self.add_server_window.password_field));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Service DB");
                        ui.add(TextEdit::singleline(&mut self.add_server_window.service_database_field));
                    });

                    ui.with_layout(Layout::bottom_up(Align::RIGHT), |ui| {
                        ui.horizontal(|ui| {
                            let is_port_error: bool = self.add_server_window.port_field.parse::<u16>().is_err();

                            if ui.add_enabled(!is_port_error, Button::new("Save")).clicked() {
                                let server = structs::Server {
                                    alias: self.add_server_window.name_field.clone(),
                                    ip: self.add_server_window.ip_field.clone(),
                                    port: self.add_server_window.port_field.clone().parse::<u16>().unwrap(),
                                    user: self.add_server_window.user_field.clone(),
                                    password: self.add_server_window.password_field.clone(),
                                    service_database: self.add_server_window.service_database_field.clone(),
                                };
                                self.config.servers.push(server);
                                self.save_config();

                                self.add_server_window = structs::AddServerWindow::new();
                            }
                            if ui.button("Back").clicked() {
                                self.add_server_window = structs::AddServerWindow::new();
                            }
                        });
                    });
                });
            }

            if ui.button("Add server").clicked() {
                self.add_server_window.show = true;
            }
        });
    }
}
