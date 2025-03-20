use crate::data::*;

use crate::database;

use eframe::{egui, App};

use egui::{
    CentralPanel, Window, SidePanel,
    Widget, Layout, Align,
    TextEdit, Color32, Button,
    CollapsingHeader, Id, Sense,
    Grid,
};

use std::fs as std_fs;

use std::collections::HashMap;

pub struct Main {
    dbs: HashMap<String, database::Database>,
    config: structs::Config,
    add_server_window: structs::AddServerWindow,
    delete_server_window: structs::DeleteServerWindow
}

impl Main {
    pub fn new(ctx: &egui::Context) -> Self {
        egui_extras::install_image_loaders(ctx);
        
        Self {
            dbs: HashMap::default(),
            config: structs::Config::default(),
            add_server_window: structs::AddServerWindow::default(),
            delete_server_window: structs::DeleteServerWindow::default(),
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
                serde_json::to_string(&structs::Config::default()).unwrap()
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

    async fn load_db(&mut self, ui: egui::Ui, id: String, server: structs::Server) {
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            server.user, server.password, server.ip, server.port, server.service_database
        );
        let database = database::Database::new(database_url).await;

        if let Ok(db) = database {
            self.dbs.insert(id, db);
        } else {
            println!("Error");
        }
    }
}

impl App for Main {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.load_config();

        let server_icon = egui::Image::new(icons::SERVER)
            .max_size(egui::vec2(32.0, 32.0));
        // let database_icon = egui::Image::new(icons::DATABASE)
        //     .max_size(egui::vec2(32.0, 32.0));
        // let plus_icon = egui::Image::new(icons::PLUS)
        //     .max_size(egui::vec2(32.0, 32.0));

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Rs-Postgres");
            })
        });

        SidePanel::left("left_panel").show(ctx, |ui| {
            CollapsingHeader::new("Servers").show(ui, |ui| {
                let server_indices: Vec<usize> = (0..self.config.servers.len()).collect();
                
                for &idx in &server_indices {
                    let server = self.config.servers[idx].clone();
                    ui.horizontal(|ui| {
                        server_icon.clone().ui(ui);

                        let id_string = format!("server:{}:{}:{}", server.ip, server.port, server.user);
                        let id = Id::new(&id_string);
                        let button = CollapsingHeader::new(
                            format!("{} ({}:{})", server.alias, server.ip, server.port)
                        ).show(ui, |_| {});
                        
                        let interact_response = ui.interact(
                            button.header_response.rect,
                            id,
                            Sense::click(),
                        );
                        if interact_response.clicked() {
                            println!("Server clicked");
                        } else if interact_response.secondary_clicked() {
                            ui.memory_mut(|mem| mem.open_popup(id));
                        }

                        if ui.memory(|mem| mem.is_popup_open(id)) {
                            interact_response.context_menu(|ui| {
                                if ui.button("Delete").clicked() {
                                    ui.memory_mut(|mem| mem.close_popup());
                                    self.delete_server_window.show = true;
                                    self.delete_server_window.server = Some(server.clone());
                                }
                            });
                        }
                    });
                }
            });

            if self.add_server_window.show {
                Window::new("New server").show(ctx, |ui| {
                    Grid::new("server_form")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Name");
                            ui.add(TextEdit::singleline(&mut self.add_server_window.name_field));
                            ui.end_row();

                            ui.label("Server address");
                            ui.add(TextEdit::singleline(&mut self.add_server_window.ip_field));
                            ui.end_row();

                            ui.label("Port");
                            let is_error: bool = self.add_server_window.port_field.parse::<u16>().is_err();
                            let mut field = TextEdit::singleline(&mut self.add_server_window.port_field);
                            if is_error {
                                field = field.text_color(Color32::from_rgb(255, 0, 0));
                            }
                            ui.add(field);
                            ui.end_row();

                            ui.label("User");
                            ui.add(TextEdit::singleline(&mut self.add_server_window.user_field));
                            ui.end_row();

                            ui.label("Password");
                            ui.add(TextEdit::singleline(&mut self.add_server_window.password_field));
                            ui.end_row();

                            ui.label("Service DB");
                            ui.add(TextEdit::singleline(&mut self.add_server_window.service_database_field));
                            ui.end_row();
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

                                self.add_server_window = structs::AddServerWindow::default();
                            }
                            if ui.button("Back").clicked() {
                                self.add_server_window = structs::AddServerWindow::default();
                            }
                        });
                    });
                });
            }

            if ui.button("Add server").clicked() {
                self.add_server_window.show = true;
            }

            if self.delete_server_window.show {
                if let Some(server) = &self.delete_server_window.server {
                    let needed_id_string = format!("server:{}:{}:{}", server.ip, server.port, server.user);

                    let mut idx_to_delete: Option<usize> = None;
                    for server_idx in 0..self.config.servers.len() {
                        let server_in_find = &self.config.servers[server_idx];

                        let id_string = format!("server:{}:{}:{}", server_in_find.ip, server_in_find.port, server_in_find.user);

                        if needed_id_string == id_string {
                            idx_to_delete = Some(server_idx);
                        }
                    }

                    if let Some(idx_to_delete) = idx_to_delete {
                        Window::new("Deleting server").show(ctx, |ui| {
                            ui.label("Are you sure?");

                            ui.horizontal(|ui| {
                                if ui.button("Yes").clicked() {
                                    self.config.servers.remove(idx_to_delete);
                                    self.save_config();
                                    self.delete_server_window = structs::DeleteServerWindow::default();
                                }
                                if ui.button("No").clicked() {
                                    self.delete_server_window = structs::DeleteServerWindow::default();
                                }
                            })
                        });
                    }
                }
            }
        });
    }
}
