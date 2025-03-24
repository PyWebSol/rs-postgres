mod widgets;

use crate::data::*;
use crate::database;
use crate::utils;

use eframe::{egui, App};
use egui::{
    RichText, Modal, CentralPanel, Spinner, Layout, Align, TextEdit, Color32,
    Button, CollapsingHeader, Id, Grid, ScrollArea, Label,
    Key, Slider,
};
use egui_extras::{TableBuilder, Column};
use egui_file_dialog::FileDialog;
use log::{info, error};
use std::fs as std_fs;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::utils::{encrypt_string, decrypt_string};
use std::fs::File;
use serde_json;
use serde_merge;

struct DbManager {
    dbs: Arc<Mutex<HashMap<String, structs::DbState>>>,
}

pub struct Main<'a> {
    db_manager: DbManager,
    config: structs::Config,
    add_server_window: structs::AddServerWindow,
    delete_server_window: structs::DeleteServerWindow,
    edit_server_window: structs::EditServerWindow,
    sql_response_copy_window: structs::SQLResponseCopyWindow,
    settings_window: structs::SettingsWindow,
    login_window: structs::LoginWindow,
    icons: structs::Icons<'a>,
    runtime: tokio::runtime::Runtime,
    pages: structs::Pages,
    actions: Vec<structs::Action>,
    password: Option<String>,
    select_file_dialog: FileDialog,
    select_file_dialog_action: Option<structs::SelectFileDialogAction>,
    trans: translates::Translator,
}

impl Main<'_> {
    pub fn new(ctx: &egui::Context) -> Self {
        egui_extras::install_image_loaders(ctx);

        let dbs = Arc::new(Mutex::new(HashMap::new()));
        let db_manager = DbManager { dbs };

        let runtime = tokio::runtime::Runtime::new().unwrap();

        let mut main = Self {
            db_manager,
            config: structs::Config::default(),
            add_server_window: structs::AddServerWindow::default(),
            delete_server_window: structs::DeleteServerWindow::default(),
            edit_server_window: structs::EditServerWindow::default(),
            sql_response_copy_window: structs::SQLResponseCopyWindow::default(),
            login_window: structs::LoginWindow::default(),
            settings_window: structs::SettingsWindow::default(),
            icons: structs::Icons {
                warning_light: egui::Image::new(icons::WARNING_LIGHT).bg_fill(Color32::TRANSPARENT).max_size(egui::vec2(32.0, 32.0)),
                warning_dark: egui::Image::new(icons::WARNING_DARK).bg_fill(Color32::TRANSPARENT).max_size(egui::vec2(32.0, 32.0)),
                rs_postgres: egui::Image::new(icons::RS_POSTGRES).bg_fill(Color32::TRANSPARENT).max_size(egui::vec2(32.0, 32.0)),
            },
            runtime,
            pages: structs::Pages::default(),
            actions: Vec::new(),
            password: None,
            select_file_dialog: FileDialog::new(),
            select_file_dialog_action: None,
            trans: translates::Translator::new(translates::Language::English),
        };

        main.load_config();
        main
    }

    fn load_config(&mut self) {
        let config_dir = dirs::config_dir().unwrap().join("rs-postgres");
        if !config_dir.exists() {
            std_fs::create_dir_all(&config_dir).unwrap();
        }

        let config_path = config_dir.join("config.json");
        if !config_path.exists() {
            std_fs::write(
                &config_path,
                serde_json::to_string(&structs::Config::default()).unwrap(),
            ).unwrap();
        }

        let mut write_config = false;

        let config_file = std_fs::read_to_string(&config_path).unwrap();
        let mut config = match serde_json::from_str::<structs::Config>(&config_file) {
            Ok(config) => config,
            Err(_) => {
                let mut default_config = structs::Config::default();
                if let Ok(mut partial_config) = serde_json::from_str::<serde_json::Value>(&config_file) {
                    Self::merge_defaults(&mut partial_config, &serde_json::to_value(&default_config).unwrap());
                    default_config = serde_json::from_value(partial_config).unwrap_or(default_config);
                }
                write_config = true;
                default_config
            },
        };

        if !config.settings.theme.is_inited() {
            config.settings.theme = structs::Theme::Dark;
            write_config = true;
        }

        if write_config {
            std_fs::write(
                &config_path,
                serde_json::to_string_pretty(&config).unwrap(),
            ).unwrap();
        }

        self.trans.language = config.settings.language.clone();
        self.config = config;
    }

    fn merge_defaults(current: &mut serde_json::Value, default: &serde_json::Value) {
        match (current, default) {
            (serde_json::Value::Object(ref mut current_map), serde_json::Value::Object(default_map)) => {
                for (key, default_value) in default_map {
                    if !current_map.contains_key(key) {
                        current_map.insert(key.clone(), default_value.clone());
                    } else {
                        Self::merge_defaults(current_map.get_mut(key).unwrap(), default_value);
                    }
                }
            }
            _ => {}
        }
    }

    fn save_config(&mut self) {
        let config_dir = dirs::config_dir().unwrap().join("rs-postgres");
        let config_path = config_dir.join("config.json");

        self.trans.language = self.config.settings.language.clone();

        let mut config = self.config.clone();
        config.servers.iter_mut().for_each(|server| {
            server.password = encrypt_string(&server.password, self.password.as_ref().unwrap()).unwrap();
        });

        std_fs::write(
            &config_path,
            serde_json::to_string_pretty(&config).unwrap(),
        ).unwrap();
    }

    fn decrypt_passwords(&mut self) {
        for server in self.config.servers.iter_mut() {
            let encrypted_password = decrypt_string(
                &server.password,
                self.password.as_ref().unwrap_or(&"".to_string())
            );

            match encrypted_password {
                Ok(password) => {
                    server.password = password;
                },
                Err(e) => {
                    self.login_window.error = Some(format!("Incorrect password: {}", e));

                    return;
                },
            }
        }
    }

    async fn load_db(id: String, server: structs::Server, dbs: Arc<Mutex<HashMap<String, structs::DbState>>>) {
        info!("Starting to load database for server {}", server.ip);
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            server.user, server.password, server.ip, server.port, server.service_database
        );
        match database::Database::new(&database_url).await {
            Ok(db) => {
                info!("Database loaded for server {}", server.ip);
                let databases_names = db.get_databases().await;

                if let Ok(databases_names) = databases_names {
                    let mut databases: Vec<structs::LoadedDatabase> = Vec::new();
                    for name in databases_names {
                        let database_url = format!(
                            "postgres://{}:{}@{}:{}/{}",
                            server.user, server.password, server.ip, server.port, name
                        );
                        let database = database::Database::new(&database_url).await;
                        if let Ok(database) = database {
                            let tables = database.get_tables().await;

                            if let Ok(tables) = tables {
                                databases.push(
                                    structs::LoadedDatabase {
                                        name: name.clone(),
                                        database,
                                        tables,
                                    }
                                );
                            } else if let Err(e) = tables {
                                error!("Error loading tables for database {}: {}", name, e);
                            }
                        } else if let Err(e) = database {
                            error!("Error loading database for server {} ({}): {}", server.ip, name, e);
                        }
                    }

                    let mut dbs = dbs.lock().unwrap();
                    dbs.insert(id, structs::DbState::Loaded(databases));
                } else if let Err(e) = databases_names {
                    error!("Error loading database for server {}: {}", server.ip, e);
                    let mut dbs = dbs.lock().unwrap();
                    dbs.insert(id, structs::DbState::Error(e.to_string()));
                }
            }
            Err(e) => {
                error!("Error loading database for server {}: {}", server.ip, e);
                let mut dbs = dbs.lock().unwrap();
                dbs.insert(id, structs::DbState::Error(e.to_string()));
            }
        }
    }

    async fn fetch_sql_query(database: database::Database, code: &str, sql_query_execution_status: Option<Arc<Mutex<structs::SQLQueryExecutionStatusType>>>) {
        let start_time = Instant::now();
        let result = database.execute_query(&code).await;
        let execution_time = start_time.elapsed().as_millis();

        let execution_status = match result {
            Ok(result) => structs::SQLQueryExecutionStatusType::Success(structs::SQLQueryExecutionSuccess {
                result,
                execution_time: execution_time as u64,
                page_index: 0,
            }),
            Err(e) => structs::SQLQueryExecutionStatusType::Error(e.to_string()),
        };

        if let Some(sql_query_execution_status) = sql_query_execution_status {
            let mut sql_query_execution_status = sql_query_execution_status.lock().unwrap();
            *sql_query_execution_status = execution_status;
        }
    }

    fn save_code(sqlquery_page: &mut structs::SQLQueryPage) {
        if !sqlquery_page.code.ends_with("\n") {
            sqlquery_page.code = format!("{}\n", sqlquery_page.code);
        }

        let mut file = File::create(sqlquery_page.code_file_path.as_ref().unwrap()).unwrap();
        file.write_all(sqlquery_page.code.as_bytes()).unwrap();
    }

    async fn reload_server(index: usize, config: structs::Config, dbs: Arc<Mutex<HashMap<String, structs::DbState>>>) {
        let server = &config.servers[index];
        let id = format!("server:{}:{}:{}", server.ip, server.port, server.user);

        {
            let mut dbs = dbs.lock().unwrap();
            dbs.remove(&id);
        }

        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            server.user, server.password, server.ip, server.port, server.service_database
        );
        match database::Database::new(&database_url).await {
            Ok(database) => {
                let databases = database.get_databases().await;
                match databases {
                    Ok(databases) => {
                        let mut loaded_databases: Vec<structs::LoadedDatabase> = Vec::new();

                        for db_name in databases {
                            let db_url = format!(
                                "postgres://{}:{}@{}:{}/{}",
                                server.user, server.password, server.ip, server.port, db_name
                            );

                            match database::Database::new(&db_url).await {
                                Ok(db_connection) => {
                                    let tables = match db_connection.get_tables().await {
                                        Ok(tables) => tables,
                                        Err(_) => Vec::new()
                                    };

                                    loaded_databases.push(structs::LoadedDatabase {
                                        name: db_name,
                                        database: db_connection,
                                        tables,
                                    });
                                },
                                Err(_) => {
                                    loaded_databases.push(structs::LoadedDatabase {
                                        name: db_name,
                                        database: database.clone(),
                                        tables: Vec::new(),
                                    });
                                }
                            }
                        }

                        let mut dbs = dbs.lock().unwrap();
                        dbs.insert(id, structs::DbState::Loaded(loaded_databases));
                    }
                    Err(e) => {
                        error!("Error loading databases for server {}: {}", server.ip, e);
                        let mut dbs = dbs.lock().unwrap();
                        dbs.insert(id, structs::DbState::Error(e.to_string()));
                    }
                }
            }
            Err(e) => {
                error!("Error connecting to server {}: {}", server.ip, e);
                let mut dbs = dbs.lock().unwrap();
                dbs.insert(id, structs::DbState::Error(e.to_string()));
            }
        }
    }

    fn update_windows(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.add_server_window.show {
            Modal::new(Id::new("add_server_modal"))
                .show(ctx, |ui| {
                    widgets::modal_label(ui, self.trans.add_server());

                    Grid::new("server_form")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            let input_color = self.config.settings.theme.text_input_color();

                            ui.label(self.trans.name());
                            ui.add(TextEdit::singleline(&mut self.add_server_window.name_field).background_color(input_color));
                            ui.end_row();

                            ui.label(self.trans.server_address());
                            ui.add(TextEdit::singleline(&mut self.add_server_window.ip_field).background_color(input_color));
                            ui.end_row();

                            ui.label(self.trans.port());
                            let is_error = self.add_server_window.port_field.parse::<u16>().is_err();
                            let mut field = TextEdit::singleline(&mut self.add_server_window.port_field);
                            if is_error {
                                field = field.text_color(Color32::from_rgb(255, 0, 0));
                            }
                            ui.add(field.background_color(input_color));
                            ui.end_row();

                            ui.label(self.trans.user());
                            ui.add(TextEdit::singleline(&mut self.add_server_window.user_field).background_color(input_color));
                            ui.end_row();

                            ui.label(self.trans.password());
                            ui.add(TextEdit::singleline(&mut self.add_server_window.password_field).background_color(input_color));
                            ui.end_row();

                            ui.label(self.trans.service_database());
                            ui.add(TextEdit::singleline(&mut self.add_server_window.service_database_field).background_color(input_color));
                            ui.end_row();
                        });

                    let is_name_error = {
                        if self.add_server_window.name_field.is_empty() {
                            ui.label(self.trans.name_is_required());
                            true
                        } else if self.add_server_window.name_field.chars().count() > 32 {
                            ui.label(self.trans.name_must_be_less_than_32_characters());
                            true
                        } else if self.config.servers.iter().any(|server| server.alias == self.add_server_window.name_field) {
                            ui.label(self.trans.name_must_be_unique());
                            true
                        } else {
                            false
                        }
                    };
                    let is_ip_error = {
                        if self.add_server_window.ip_field.is_empty() {
                            ui.label(self.trans.ip_is_required());
                            true
                        } else {
                            false
                        }
                    };
                    let is_port_error = {
                        if self.add_server_window.port_field.parse::<u16>().is_err() {
                            ui.label(self.trans.incorrect_port_value());
                            true
                        } else {
                            false
                        }
                    };
                    let is_user_error = {
                        if self.add_server_window.user_field.is_empty() {
                            ui.label(self.trans.user_is_required());
                            true
                        } else {
                            false
                        }
                    };
                    let is_service_database_error = {
                        if self.add_server_window.service_database_field.is_empty() {
                            ui.label(self.trans.service_database_is_required());
                            true
                        } else {
                            false
                        }
                    };

                    let enable_save_button = !is_name_error && !is_ip_error && !is_port_error && !is_user_error && !is_service_database_error;

                    ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                        ui.separator();

                        ui.horizontal(|ui| {
                            if ui.add_enabled(enable_save_button, Button::new(self.trans.save())).clicked() {
                                let server = structs::Server {
                                    alias: self.add_server_window.name_field.clone(),
                                    ip: self.add_server_window.ip_field.clone(),
                                    port: self.add_server_window.port_field.parse::<u16>().unwrap(),
                                    user: self.add_server_window.user_field.clone(),
                                    password: self.add_server_window.password_field.clone(),
                                    service_database: self.add_server_window.service_database_field.clone(),
                                };
                                self.config.servers.push(server);
                                self.save_config();
                                self.add_server_window = structs::AddServerWindow::default();
                            }
                            if ui.button(self.trans.back()).clicked() {
                                self.add_server_window = structs::AddServerWindow::default();
                            }
                        });
                    });
                });
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
                    Modal::new(Id::new("delete_server_modal")).show(ctx, |ui| {
                        widgets::modal_label(ui, self.trans.delete_server());

                        ui.label(self.trans.delete_server_confirmation());

                        ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                            ui.separator();

                            ui.horizontal(|ui| {
                                if ui.button(self.trans.yes()).clicked() {
                                    self.config.servers.remove(idx_to_delete);
                                    self.save_config();
                                    self.delete_server_window = structs::DeleteServerWindow::default();
                                }
                                if ui.button(self.trans.no()).clicked() {
                                    self.delete_server_window = structs::DeleteServerWindow::default();
                                }
                            });
                        });
                    });
                }
            }
        }

        if self.edit_server_window.show {
            Modal::new(Id::new("edit_server_modal")).show(ctx, |ui| {
                widgets::modal_label(ui, self.trans.edit_server());

                    Grid::new("server_form")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            let input_color = self.config.settings.theme.text_input_color();

                            ui.label(self.trans.name());
                            ui.add(TextEdit::singleline(&mut self.edit_server_window.name_field).background_color(input_color));
                            ui.end_row();

                            ui.label(self.trans.server_address());
                            ui.add(TextEdit::singleline(&mut self.edit_server_window.ip_field).background_color(input_color));
                            ui.end_row();

                            ui.label(self.trans.port());
                            let is_error = self.edit_server_window.port_field.parse::<u16>().is_err();
                            let mut field = TextEdit::singleline(&mut self.edit_server_window.port_field);
                            if is_error {
                                field = field.text_color(Color32::from_rgb(255, 0, 0));
                            }
                            ui.add(field.background_color(input_color));
                            ui.end_row();

                            ui.label(self.trans.user());
                            ui.add(TextEdit::singleline(&mut self.edit_server_window.user_field).background_color(input_color));
                            ui.end_row();

                            ui.label(self.trans.password());
                            ui.add(TextEdit::singleline(&mut self.edit_server_window.password_field).background_color(input_color));
                            ui.end_row();

                            ui.label(self.trans.service_database());
                            ui.add(TextEdit::singleline(&mut self.edit_server_window.service_database_field).background_color(input_color));
                            ui.end_row();
                        });

                    let is_name_error = {
                        if self.edit_server_window.name_field.is_empty() {
                            ui.label(self.trans.name_is_required());
                            true
                        } else if self.edit_server_window.name_field.chars().count() > 32 {
                            ui.label(self.trans.name_must_be_less_than_32_characters());
                            true
                        } else if self.config.servers.iter().any(|server| server.alias == self.edit_server_window.name_field && server.alias != self.edit_server_window.original_server.as_ref().unwrap().alias) {
                            ui.label(self.trans.name_must_be_unique());
                            true
                        } else {
                            false
                        }
                    };
                    let is_ip_error = {
                        if self.edit_server_window.ip_field.is_empty() {
                            ui.label(self.trans.ip_is_required());
                            true
                        } else {
                            false
                        }
                    };
                    let is_port_error = {
                        if self.edit_server_window.port_field.is_empty() {
                            ui.label(self.trans.port_is_required());
                            true
                        } else if self.edit_server_window.port_field.parse::<u16>().is_err() {
                            ui.label(self.trans.incorrect_port_value());
                            true
                        } else {
                            false
                        }
                    };
                    let is_user_error = {
                        if self.edit_server_window.user_field.is_empty() {
                            ui.label(self.trans.user_is_required());
                            true
                        } else {
                            false
                        }
                    };

                    let is_service_database_error = {
                        if self.add_server_window.service_database_field.is_empty() {
                            ui.label(self.trans.service_database_is_required());
                            true
                        } else {
                            false
                        }
                    };

                    let enable_save_button = !is_name_error && !is_ip_error && !is_port_error && !is_user_error && !is_service_database_error;

                    ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                        ui.separator();

                        ui.horizontal(|ui| {
                            if ui.add_enabled(enable_save_button, Button::new(self.trans.save())).clicked() {
                                let server = structs::Server {
                                    alias: self.edit_server_window.name_field.clone(),
                                    ip: self.edit_server_window.ip_field.clone(),
                                    port: self.edit_server_window.port_field.parse::<u16>().unwrap(),
                                    user: self.edit_server_window.user_field.clone(),
                                    password: self.edit_server_window.password_field.clone(),
                                    service_database: self.edit_server_window.service_database_field.clone(),
                                };
                                let mut original_server_index: Option<usize> = None;

                                let original_server = self.edit_server_window.original_server.clone().unwrap();
                                let original_server_id = format!("server:{}:{}:{}", original_server.ip, original_server.port, original_server.user);

                                for server_idx in 0..self.config.servers.len() {
                                    let server_in_find = &self.config.servers[server_idx];
                                    let id_string = format!("server:{}:{}:{}", server_in_find.ip, server_in_find.port, server_in_find.user);

                                    if original_server_id == id_string {
                                        original_server_index = Some(server_idx);
                                    }
                                }

                                self.config.servers[original_server_index.unwrap()] = server;
                                self.save_config();
                                self.edit_server_window = structs::EditServerWindow::default();

                                let dbs = self.db_manager.dbs.clone();
                                let config = self.config.clone();

                                self.runtime.spawn(async move {
                                    Self::reload_server(original_server_index.unwrap(), config, dbs).await;
                                });
                            }
                            if ui.button(self.trans.back()).clicked() {
                                self.edit_server_window = structs::EditServerWindow::default();
                            }
                        });
                    });
            });
        }

        if self.sql_response_copy_window.show {
            Modal::new(Id::new("sql_response_copy_modal")).show(ctx, |ui| {
                let screen_rect = ctx.input(|i| i.screen_rect);

                widgets::modal_label(ui, self.trans.text_viewer());

                ui.set_width(if screen_rect.height() / 1.5 > 380.0 { screen_rect.height() / 1.5 } else { 380.0 });

                ScrollArea::both().max_height(screen_rect.height() / 1.5).show(ui, |ui| {
                    ui.label(self.sql_response_copy_window.response.clone().unwrap());
                });

                ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button(self.trans.copy()).clicked() {
                            ui.ctx().copy_text(self.sql_response_copy_window.response.clone().unwrap());
                            self.sql_response_copy_window = structs::SQLResponseCopyWindow::default();
                        }
                        if ui.button(self.trans.close()).clicked() {
                            self.sql_response_copy_window = structs::SQLResponseCopyWindow::default();
                        }
                    });
                });
            });
        }

        if self.settings_window.show {
            if self.settings_window.scale_factor < 1.0 || self.settings_window.scale_factor > 1.5 {
                self.settings_window.scale_factor = self.config.settings.scale_factor;
            }
            if !self.settings_window.theme.is_inited() {
                self.settings_window.theme = self.config.settings.theme.clone();
            }
            if self.settings_window.language.is_none() {
                self.settings_window.language = Some(self.config.settings.language.clone());
            }

            Modal::new(Id::new("settings_modal")).show(ctx, |ui| {
                widgets::modal_label(ui, self.trans.settings());

                Grid::new("settings_form")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(self.trans.scale_factor());
                            ui.add(Slider::new(&mut self.settings_window.scale_factor, 1.0..=1.5));
                            ui.end_row();

                            ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                                ui.label(self.trans.theme());
                            });
                            let theme_name = match self.settings_window.theme {
                                structs::Theme::Light => self.trans.light(),
                                structs::Theme::Dark => self.trans.dark(),
                                structs::Theme::NotInited => "".to_string(),
                            };
                            CollapsingHeader::new(theme_name).show(ui, |ui| {
                                if ui.button(self.trans.dark()).clicked() {
                                    self.settings_window.theme = structs::Theme::Dark;
                                }
                                if ui.button(self.trans.light()).clicked() {
                                    self.settings_window.theme = structs::Theme::Light;
                                }
                            });
                            ui.end_row();

                            ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                                ui.label(self.trans.language());
                            });
                            let language_name = match self.settings_window.language {
                                Some(translates::Language::English) => "English".to_string(),
                                Some(translates::Language::Russian) => "Russian".to_string(),
                                None => "".to_string(),
                            };
                            CollapsingHeader::new(language_name).show(ui, |ui| {
                                if ui.button("English").clicked() {
                                    self.settings_window.language = Some(translates::Language::English);
                                }
                                if ui.button("Russian").clicked() {
                                    self.settings_window.language = Some(translates::Language::Russian);
                                }
                            });
                            ui.end_row();
                        });

                ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button(self.trans.save()).clicked() {
                            self.config.settings.scale_factor = self.settings_window.scale_factor;
                            self.config.settings.theme = self.settings_window.theme.clone();
                            self.config.settings.language = self.settings_window.language.clone().unwrap();

                            self.settings_window = structs::SettingsWindow::default();

                            self.save_config();
                        }
                        if ui.button(self.trans.close()).clicked() {
                            self.settings_window = structs::SettingsWindow::default();
                        }
                    });
                });
            });
        }
    }

    fn update_pages(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        widgets::top_panel(ctx, |ui| {
            for (idx, page) in self.pages.pages.iter_mut().enumerate() {
                let mut button_title = page.title.clone();
                if button_title.chars().count() > 16 {
                    button_title = format!("{}...", &button_title.chars().take(16).collect::<String>());
                }

                let btn = ui.button(&button_title);
                let btn_id = Id::new(idx);

                if btn.clicked() {
                    self.pages.current_page_index = idx as u16;
                }
                if btn.secondary_clicked() {
                    ui.memory_mut(|mem| mem.open_popup(btn_id));
                }
                if btn.hovered() {
                    egui::show_tooltip_at_pointer(ui.ctx(), ui.layer_id(), btn_id, |ui| {
                        ui.label(page.title.clone());
                    });
                }

                if !page.scrolled {
                    btn.scroll_to_me(Some(Align::Center));
                    page.scrolled = true;
                }

                if ui.memory(|mem| mem.is_popup_open(btn_id)) {
                    btn.context_menu(|ui| {
                        if ui.button(self.trans.close()).clicked() {
                            ui.memory_mut(|mem| mem.close_popup());
                            self.actions.push(structs::Action::ClosePage(idx));
                        }
                    });
                }

                if idx == self.pages.current_page_index as usize {
                    btn.highlight();
                }
            }
        });


        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if self.pages.current_page_index as usize >= self.pages.pages.len() {
                        return;
                    }

                    let page = &mut self.pages.pages[self.pages.current_page_index as usize];

                    match &mut page.page_type {
                        structs::PageType::Welcome => {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.add(self.icons.rs_postgres.clone());
                                    ui.heading(self.trans.welcome());
                                });

                                ui.add_space(16.0);
                                ui.label(RichText::new(self.trans.features()).strong());
                                ui.separator();
                                ui.label(self.trans.features_content());

                                ui.add_space(16.0);
                                ui.label(RichText::new(self.trans.get_started()).strong());
                                ui.separator();
                                ui.label(self.trans.get_started_content());

                                ui.add_space(16.0);
                                ui.label(RichText::new(self.trans.resources()).strong());
                                ui.separator();
                                ui.horizontal(|ui| {
                                    if ui.add(Button::new(self.trans.github()).fill(Color32::TRANSPARENT))
                                        .on_hover_text(self.trans.open_repo())
                                        .clicked() {

                                        open::that("https://github.com/pywebsol/rs-postgres").unwrap();
                                    }
                                });
                                ui.horizontal(|ui| {
                                    if ui.add(Button::new(self.trans.license()).fill(Color32::TRANSPARENT))
                                        .on_hover_text(self.trans.open_license())
                                        .clicked() {

                                        open::that("https://github.com/pywebsol/rs-postgres/blob/main/LICENSE").unwrap();
                                    }
                                });
                                ui.horizontal(|ui| {
                                    if ui.add(Button::new(self.trans.support()).fill(Color32::TRANSPARENT))
                                        .on_hover_text(self.trans.open_support())
                                        .clicked() {

                                        open::that("https://t.me/bot_token").unwrap();
                                    }
                                });

                                ui.add_space(24.0);
                                ui.label(RichText::new(self.trans.version(env!("CARGO_PKG_VERSION"))).small().color(Color32::GRAY));
                            });
                        },
                        structs::PageType::SQLQuery(sqlquery_page) => {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    let code_is_empty = sqlquery_page.code.is_empty();

                                    if ui.add_enabled(!code_is_empty, Button::new(self.trans.run_f5())).clicked() || (ui.input(|i| i.key_pressed(Key::F5) && !code_is_empty)) {
                                        let runtime = &self.runtime;

                                        sqlquery_page.sql_query_execution_status = Some(Arc::new(Mutex::new(structs::SQLQueryExecutionStatusType::Running)));

                                        let database_clone = sqlquery_page.database.clone();
                                        let code_clone = sqlquery_page.code.clone();
                                        let sql_query_execution_status = sqlquery_page.sql_query_execution_status.clone();

                                        runtime.spawn(async move {
                                            Self::fetch_sql_query(database_clone, &code_clone, sql_query_execution_status).await;
                                        });
                                    }

                                    if ui.add_enabled(!code_is_empty, Button::new(self.trans.save())).clicked() || (ui.input(|i| i.modifiers.ctrl && i.key_pressed(Key::S)) && !code_is_empty) {
                                        if sqlquery_page.code_file_path.is_some() {
                                            Self::save_code(sqlquery_page);
                                        } else {
                                            self.select_file_dialog_action = Some(structs::SelectFileDialogAction::SaveFile);
                                            self.select_file_dialog.save_file();
                                        }
                                    }
                                    if ui.button(self.trans.open()).clicked() || (ui.input(|i| i.modifiers.ctrl && i.key_pressed(Key::O))) {
                                        self.select_file_dialog_action = Some(structs::SelectFileDialogAction::OpenFile);
                                        self.select_file_dialog.pick_file();
                                    }

                                    self.select_file_dialog.update(ctx);

                                    if let Some(action) = &self.select_file_dialog_action {
                                        match action {
                                            structs::SelectFileDialogAction::SaveFile => {
                                                if let Some(code_file_path) = self.select_file_dialog.take_picked() {
                                                    self.select_file_dialog_action = None;

                                                    sqlquery_page.code_file_path = Some(code_file_path.to_string_lossy().to_string());

                                                    Self::save_code(sqlquery_page);
                                                }
                                            },
                                            structs::SelectFileDialogAction::OpenFile => {
                                                if let Some(code_file_path) = self.select_file_dialog.take_picked() {
                                                    self.select_file_dialog_action = None;

                                                    sqlquery_page.code_file_path = Some(code_file_path.to_string_lossy().to_string());

                                                    match File::open(code_file_path) {
                                                        Ok(mut file) => {
                                                            let mut file_content = String::new();
                                                            let _ = file.read_to_string(&mut file_content);

                                                            sqlquery_page.code = file_content;
                                                        },
                                                        Err(_) => {},
                                                    }
                                                }
                                            },
                                        }
                                    }
                                });

                                ui.add_space(16.0);

                                if let Some(code_file_path) = &sqlquery_page.code_file_path {
                                    ui.horizontal(|ui| {
                                        ui.label(self.trans.file());
                                        ui.label(RichText::new(code_file_path).code().background_color(
                                            self.config.settings.theme.text_input_color()
                                        ));
                                    });
                                }

                                let mut theme = egui_extras::syntax_highlighting::CodeTheme::light(12.0);

                                let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                                    let mut layout_job = egui_extras::syntax_highlighting::highlight(
                                        ui.ctx(),
                                        ui.style(),
                                        &mut theme,
                                        string,
                                        "sql",
                                    );
                                    layout_job.wrap.max_width = wrap_width;
                                    ui.fonts(|f| f.layout_job(layout_job))
                                };

                                let code_editor = ui.add(
                                    TextEdit::multiline(&mut sqlquery_page.code)
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(10)
                                        .background_color(self.config.settings.theme.text_input_color())
                                        .hint_text("SELECT * FROM ...")
                                        .layouter(&mut layouter),
                                );
                                if code_editor.secondary_clicked() {
                                    ui.memory_mut(|mem| mem.open_popup(Id::new("code_editor_popup")));
                                }

                                if ui.memory(|mem| mem.is_popup_open(Id::new("code_editor_popup"))) {
                                    code_editor.context_menu(|ui| {
                                        if ui.button(self.trans.clear()).clicked() {
                                            sqlquery_page.code = String::new();
                                        }
                                    });
                                }

                                ui.add_space(8.0);

                                if let Some(sql_query_execution_status) = &sqlquery_page.sql_query_execution_status {
                                    let sql_query_execution_status = sql_query_execution_status.lock().unwrap().clone();
                                    match &sql_query_execution_status {
                                        structs::SQLQueryExecutionStatusType::Running => {
                                            ui.horizontal(|ui| {
                                                ui.add(Spinner::new());
                                                ui.label(self.trans.running());
                                            });

                                            ui.separator();
                                        }
                                        structs::SQLQueryExecutionStatusType::Success(result) => {
                                            let data = &result.result;
                                            let rows_count = match data.is_empty() {
                                                true => 0,
                                                false => data.values().next().unwrap().len(),
                                            };
                                            let execution_time = result.execution_time;

                                            let pages_count = (rows_count as f32 / ROWS_PER_PAGE as f32).ceil() as u16;
                                            let start_index = (result.page_index * ROWS_PER_PAGE as u32) as usize;
                                            let end_index = if start_index + ROWS_PER_PAGE as usize > data[data.keys().next().unwrap()].len() {
                                                data[data.keys().next().unwrap()].len()
                                            } else {
                                                start_index + ROWS_PER_PAGE as usize
                                            };

                                            let available_height = ui.available_height() - if pages_count > 1 {
                                                64.0
                                            } else {
                                                0.0
                                            };
                                            let available_width = ui.available_width();

                                            ui.horizontal(|ui| {
                                                ui.label(self.trans.success());
                                                ui.separator();
                                                ui.label(self.trans.rows(rows_count));
                                                ui.separator();
                                                ui.label(self.trans.time(execution_time));
                                            });

                                            ui.separator();

                                            if !data.is_empty() {
                                                ScrollArea::horizontal().auto_shrink([false, false]).max_width(available_width).max_height(available_height).show(ui, |ui| {
                                                    TableBuilder::new(ui)
                                                        .striped(true)
                                                        .auto_shrink([false, false])
                                                        .columns(Column::remainder().resizable(true), data.keys().len())
                                                        .header(16.0, |mut header| {
                                                            for column_name in data.keys() {
                                                                header.col(|ui| {
                                                                    ui.add(
                                                                        Label::new(RichText::new(column_name).strong().monospace())
                                                                            .wrap_mode(egui::TextWrapMode::Extend)
                                                                    );
                                                                });
                                                            }
                                                        })
                                                        .body(|mut body| {
                                                            for i in 0..(end_index - start_index) {
                                                                body.row(16.0, |mut row| {
                                                                    let values = data.values()
                                                                        .map(|v| v[start_index..end_index]
                                                                            .iter()
                                                                            .map(|x| x.to_string())
                                                                            .collect::<Vec<String>>())
                                                                        .collect::<Vec<Vec<String>>>();

                                                                    for value in values {
                                                                        row.col(|ui| {
                                                                            let content = value[i].to_string();
                                                                            let label = content.clone().replace("\n", " ");

                                                                            let label = Label::new(label)
                                                                                .wrap_mode(egui::TextWrapMode::Truncate);
                                                                            let label_widget = ui.add(label);

                                                                            if label_widget.clicked() {
                                                                                self.sql_response_copy_window.show = true;
                                                                                self.sql_response_copy_window.response = Some(content);
                                                                            } else if label_widget.hovered() {
                                                                                egui::show_tooltip_at_pointer(ui.ctx(), ui.layer_id(), Id::new("copy_tooltip"), |ui| {
                                                                                    ui.label(self.trans.click_to_copy());
                                                                                });
                                                                            }
                                                                        });
                                                                    }
                                                                });
                                                            }
                                                        });
                                                    });

                                                    if pages_count > 1 {
                                                        let sql_query_status_clone = sqlquery_page.sql_query_execution_status.clone();

                                                        ui.separator();

                                                        ui.horizontal_centered(|ui| {
                                                            if ui.add_enabled(result.page_index != 0, Button::new("<<<")).clicked() {
                                                                if let Some(status_clone) = sql_query_status_clone.clone() {
                                                                    let mut status = status_clone.lock().unwrap();
                                                                    if let structs::SQLQueryExecutionStatusType::Success(success) = &mut *status {
                                                                        success.page_index = 0;
                                                                    }
                                                                }
                                                            }
                                                            if ui.add_enabled(result.page_index != 0, Button::new("<")).clicked() {
                                                                if let Some(status_clone) = sql_query_status_clone.clone() {
                                                                    let mut status = status_clone.lock().unwrap();
                                                                    if let structs::SQLQueryExecutionStatusType::Success(success) = &mut *status {
                                                                        success.page_index -= 1;
                                                                    }
                                                                }
                                                            }

                                                            ui.separator();

                                                            ui.label(format!("{}/{}; {}..{}", result.page_index + 1, pages_count, result.page_index * ROWS_PER_PAGE as u32, if result.page_index == pages_count as u32 - 1 { data[data.keys().next().unwrap()].len() } else { ((result.page_index + 1) * ROWS_PER_PAGE as u32) as usize }));

                                                            ui.separator();

                                                            if ui.add_enabled(result.page_index != pages_count as u32 - 1, Button::new(">")).clicked() {
                                                                if let Some(status_clone) = sql_query_status_clone.clone() {
                                                                    let mut status = status_clone.lock().unwrap();
                                                                    if let structs::SQLQueryExecutionStatusType::Success(success) = &mut *status {
                                                                        success.page_index += 1;
                                                                    }
                                                                }
                                                            }
                                                            if ui.add_enabled(result.page_index != pages_count as u32 - 1, Button::new(">>>")).clicked() {
                                                                if let Some(status_clone) = sql_query_status_clone.clone() {
                                                                    let mut status = status_clone.lock().unwrap();
                                                                    if let structs::SQLQueryExecutionStatusType::Success(success) = &mut *status {
                                                                        success.page_index = pages_count as u32 - 1;
                                                                    }
                                                                }
                                                            }
                                                        });
                                                    }
                                                } else {
                                                    ui.heading(self.trans.no_data_returned());
                                                }
                                        }
                                        structs::SQLQueryExecutionStatusType::Error(e) => {
                                            ui.separator();

                                            let warning_icon = match self.config.settings.theme {
                                                structs::Theme::Light => self.icons.warning_dark.clone(),
                                                _ => self.icons.warning_light.clone(),
                                            };

                                            ui.horizontal(|ui| {
                                                ui.add(warning_icon);
                                                ui.label(self.trans.error());
                                            });

                                            ui.heading(e);
                                        }
                                    }
                                }
                            });
                        },
                    }
                });
        });

        let actions = std::mem::take(&mut self.actions);
        for action in actions {
            match action {
                structs::Action::ClosePage(idx) => {
                    if idx < self.pages.pages.len() {
                        self.pages.pages.remove(idx);
                        if self.pages.pages.is_empty() {
                            self.pages = structs::Pages::default();
                        } else if self.pages.current_page_index as usize >= self.pages.pages.len() {
                            self.pages.current_page_index = (self.pages.pages.len() - 1) as u16;
                        }
                    }
                }
            }
        }
    }
}

impl App for Main<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_theme(self.config.settings.theme.to_egui());
        ctx.set_zoom_factor(self.config.settings.scale_factor);

        if self.login_window.show {
            CentralPanel::default().show(ctx, |_| {});

            Modal::new(Id::new("login_window")).show(ctx, |ui| {
                ui.set_width(360.0);

                if self.login_window.clear_storage {
                    widgets::modal_label(ui, self.trans.clear_storage());

                    ui.label(RichText::new(self.trans.clear_storage_confirmation()));

                    ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                        ui.horizontal(|ui| {
                            if ui.button(self.trans.yes()).clicked() {
                                self.login_window = structs::LoginWindow::default();

                                self.config.servers = Vec::new();
                                self.config.password_hash = None;

                                self.save_config();
                            }
                            if ui.button(self.trans.no()).clicked() {
                                self.login_window.clear_storage = false;
                            }
                        });
                    });

                    return;
                }

                widgets::modal_label(ui, self.trans.login());

                if let Some(error) = &self.login_window.error {
                    ui.label(RichText::new(error).color(Color32::RED));
                }

                ui.horizontal(|ui| {
                    if self.config.password_hash.is_some() {
                        ui.label(self.trans.enter_encryption_password());
                    } else {
                        ui.label(self.trans.create_encryption_password());
                    }

                    TextEdit::singleline(&mut self.login_window.password).background_color(
                        self.config.settings.theme.text_input_color()
                    ).password(true).show(ui);
                });

                ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                    ui.add_space(8.0);
                    ui.separator();

                    ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                        ui.horizontal(|ui| {
                            if !self.config.servers.is_empty() {
                                if ui.button(RichText::new(self.trans.clear_storage()).color(Color32::RED)).clicked() {
                                    self.login_window.clear_storage = true;
                                }
                            }
                            if ui.button(self.trans.login()).clicked() || (ui.input(|i| i.key_pressed(Key::Enter))) {
                                let password = self.login_window.password.clone();

                                self.login_window.error = None;
                                self.password = Some(password.clone());

                                ui.spinner();

                                if self.config.password_hash.is_some() {
                                    if utils::create_checksum(&password) != self.config.password_hash.clone().unwrap() {
                                        self.login_window.error = Some(self.trans.incorrect_password_hash_mismatch());
                                    }
                                }

                                if self.login_window.error.is_none() {
                                    self.decrypt_passwords();
                                }
                                if self.login_window.error.is_none() {
                                    self.login_window.show = false;

                                    if self.config.password_hash.is_none() {
                                        self.config.password_hash = Some(utils::create_checksum(&password));
                                        self.save_config();
                                    }
                                }
                            }
                        });
                    });
                });
            });

            return;
        }

        widgets::left_panel(ctx, |ui| {
            CollapsingHeader::new(self.trans.servers())
                .default_open(true)
                .show(ui, |ui| {
                    let server_indices: Vec<usize> = (0..self.config.servers.len()).collect();

                    for &idx in &server_indices {
                        let server = self.config.servers[idx].clone();
                        ui.horizontal(|ui| {
                            let server_id = format!("server:{}:{}:{}", server.ip, server.port, server.user);

                            let db_state = {
                                let dbs = self.db_manager.dbs.lock().expect("Failed to lock dbs mutex");
                                dbs.get(&server_id).cloned()
                            };

                            let id_string = format!("server:{}:{}:{}:warning", server.ip, server.port, server.user);
                            let id = Id::new(&id_string);

                            let server_button: Option<egui::Response> = match db_state {
                                Some(structs::DbState::Loading) => {
                                    ui.add(Spinner::new());

                                    Some(ui.label(format!(
                                        "{} ({}:{})",
                                        server.alias, server.ip, server.port
                                    )))
                                }
                                Some(structs::DbState::Loaded(_db)) => {
                                    Some(CollapsingHeader::new(format!(
                                        "{} ({}:{})",
                                        server.alias, server.ip, server.port
                                    ))
                                    .show(ui, |ui| {
                                        CollapsingHeader::new(self.trans.databases()).show(ui, |ui| {
                                            let db_state = {
                                                let dbs = self.db_manager.dbs.lock().expect("Failed to lock dbs mutex");
                                                dbs.get(&server_id).cloned()
                                            };
                                            if let Some(structs::DbState::Loaded(_db)) = db_state {
                                                for database in _db {
                                                    let pages = &mut self.pages;
                                                    let server = &self.config.servers[idx];

                                                    CollapsingHeader::new(&database.name).id_salt(format!("db_{}_{}_{}", server.ip, server.port, database.name)).show(ui, |ui| {
                                                        CollapsingHeader::new(self.trans.tables()).id_salt(format!("tables_{}", database.name)).show(ui, |ui| {
                                                            for table in &database.tables {
                                                                CollapsingHeader::new(table).id_salt(format!("table_{}_{}", database.name, table)).show(ui, |ui| {
                                                                    CollapsingHeader::new(self.trans.scripts()).id_salt(format!("scripts_{}_{}_{}", server.ip, database.name, table)).show(ui, |ui| {
                                                                        widgets::script_preset(ui, pages, &database, server, "Insert", scripts::INSERT.replace("{table_name}", table));
                                                                        widgets::script_preset(ui, pages, &database, server, "Update", scripts::UPDATE.replace("{table_name}", table));
                                                                        widgets::script_preset(ui, pages, &database, server, "Delete", scripts::DELETE.replace("{table_name}", table));
                                                                        widgets::script_preset(ui, pages, &database, server, "Select", scripts::SELECT.replace("{table_name}", table));
                                                                        widgets::script_preset(ui, pages, &database, server, "Select 100", scripts::SELECT_100.replace("{table_name}", table));
                                                                        widgets::script_preset(ui, pages, &database, server, self.trans.get_columns(), scripts::GET_TABLE_COLUMNS.replace("{table_name}", table));
                                                                    });
                                                                });
                                                            }
                                                        });

                                                        CollapsingHeader::new(self.trans.scripts()).id_salt(format!("db_scripts_{}", database.name)).show(ui, |ui| {
                                                            widgets::script_preset(ui, pages, &database, server, "Create table", scripts::CREATE_TABLE);
                                                            widgets::script_preset(ui, pages, &database, server, "Create index", scripts::CREATE_INDEX);
                                                            widgets::script_preset(ui, pages, &database, server, "Drop table", scripts::DROP_TABLE);
                                                        });

                                                        widgets::script_preset(ui, pages, &database, server, "SQL Query", String::new());
                                                    });
                                                }
                                            }
                                        });
                                    }).header_response)
                                }
                                Some(structs::DbState::Error(e)) => {
                                    let warning_icon = match self.config.settings.theme {
                                        structs::Theme::Light => self.icons.warning_dark.clone(),
                                        _ => self.icons.warning_light.clone(),
                                    };
                                    let warning = ui.add(warning_icon);
                                    if warning.hovered() {
                                        egui::show_tooltip_at_pointer(ui.ctx(), ui.layer_id(), id, |ui| {
                                            ui.label(e);
                                        });
                                    }

                                    Some(ui.label(format!(
                                        "{} ({}:{})",
                                        server.alias, server.ip, server.port
                                    )))
                                }
                                None => {
                                    let dbs = self.db_manager.dbs.clone();
                                    let server_id_clone = server_id.clone();
                                    let server_clone = server.clone();
                                    {
                                        let mut dbs = dbs.lock().expect("Failed to lock dbs mutex");
                                        dbs.insert(server_id.clone(), structs::DbState::Loading);
                                    }
                                    self.runtime.spawn(async move {
                                        Self::load_db(server_id_clone, server_clone, dbs).await;
                                    });
                                    ui.add(Spinner::new());

                                    None
                                }
                            };

                            if let Some(server_button) = server_button {
                                if server_button.secondary_clicked() {
                                    ui.memory_mut(|mem| mem.open_popup(id));
                                }

                                if ui.memory(|mem| mem.is_popup_open(id)) {
                                    server_button.context_menu(|ui| {
                                        if ui.button(self.trans.delete()).clicked() {
                                            ui.memory_mut(|mem| mem.close_popup());
                                            self.delete_server_window.show = true;
                                            self.delete_server_window.server = Some(server.clone());
                                        } else if ui.button(self.trans.edit()).clicked() {
                                            self.edit_server_window.show = true;
                                            self.edit_server_window.server = Some(server.clone());
                                            self.edit_server_window.original_server = Some(server.clone());

                                            self.edit_server_window.name_field = server.alias.clone();
                                            self.edit_server_window.ip_field = server.ip.clone();
                                            self.edit_server_window.port_field = server.port.to_string();
                                            self.edit_server_window.user_field = server.user.clone();
                                            self.edit_server_window.password_field = server.password.clone();
                                            self.edit_server_window.service_database_field = server.service_database.clone();
                                        } else if ui.button(self.trans.reload()).clicked() {
                                            let dbs = self.db_manager.dbs.clone();
                                            let config = self.config.clone();
                                            let idx = idx.clone();

                                            ui.memory_mut(|mem| mem.close_popup());

                                            self.runtime.spawn(async move {
                                                Self::reload_server(idx, config, dbs).await;
                                            });
                                        }
                                    });
                                }
                            }
                        });
                    }

                    if ui.button(self.trans.add_server()).clicked() {
                        self.add_server_window.show = true;
                    }
                });

                ui.add_space(32.0);

                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    ui.add_space(4.0);

                    if ui.button(self.trans.settings()).clicked() {
                        self.settings_window.show = true;
                    }

                    ui.separator();
                });
            });

        self.update_windows(ctx, _frame);
        self.update_pages(ctx, _frame);
    }
}
