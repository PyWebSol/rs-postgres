mod widgets;

use crate::data::*;
use crate::database;
use crate::utils;

use eframe::{egui, App};
use egui::{
    RichText, Modal, CentralPanel, Spinner, Layout, Align, TextEdit, Color32,
    Button, CollapsingHeader, Id, Grid, ScrollArea, Label,
    Key,
};
use egui_extras::{TableBuilder, Column};
use log::{info, error};
use std::fs as std_fs;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::utils::{encrypt_string, decrypt_string};

struct DbManager {
    dbs: Arc<Mutex<HashMap<String, structs::DbState>>>,
}

pub struct Main<'a> {
    db_manager: DbManager,
    config: structs::Config,
    add_server_window: structs::AddServerWindow,
    delete_server_window: structs::DeleteServerWindow,
    sql_response_copy_window: structs::SQLResponseCopyWindow,
    login_window: structs::LoginWindow,
    icons: structs::Icons<'a>,
    runtime: tokio::runtime::Runtime,
    pages: structs::Pages,
    actions: Vec<structs::Action>,
    password: Option<String>,
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
            sql_response_copy_window: structs::SQLResponseCopyWindow::default(),
            login_window: structs::LoginWindow::default(),
            icons: structs::Icons {
                warning: egui::Image::new(icons::WARNING).bg_fill(Color32::TRANSPARENT).max_size(egui::vec2(32.0, 32.0)),
                rs_postgres: egui::Image::new(icons::RS_POSTGRES).bg_fill(Color32::TRANSPARENT).max_size(egui::vec2(32.0, 32.0)),
            },
            runtime,
            pages: structs::Pages::default(),
            actions: Vec::new(),
            password: None,
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

        let config_file = std_fs::read_to_string(&config_path).unwrap();
        let config: structs::Config = serde_json::from_str(&config_file).unwrap();

        self.config = config;
    }

    fn save_config(&mut self) {
        let config_dir = dirs::config_dir().unwrap().join("rs-postgres");
        let config_path = config_dir.join("config.json");

        let mut config = self.config.clone();
        config.servers.iter_mut().for_each(|server| {
            server.password = encrypt_string(&server.password, self.password.as_ref().unwrap()).unwrap();
        });

        std_fs::write(
            &config_path,
            serde_json::to_string(&config).unwrap(),
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
                            databases.push(
                                structs::LoadedDatabase {
                                    name: name.clone(),
                                    database,
                                }
                            );
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
            }),
            Err(e) => structs::SQLQueryExecutionStatusType::Error(e.to_string()),
        };

        if let Some(sql_query_execution_status) = sql_query_execution_status {
            let mut sql_query_execution_status = sql_query_execution_status.lock().unwrap();
            *sql_query_execution_status = execution_status;
        }
    }

    fn update_windows(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.add_server_window.show {
            Modal::new(Id::new("add_server_modal"))
                .show(ctx, |ui| {
                    widgets::modal_label(ui, "Add server");
                    
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
                            let is_error = self.add_server_window.port_field.parse::<u16>().is_err();
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

                    let is_name_error = {
                        if self.add_server_window.name_field.is_empty() {
                            ui.label("• Name is required");
                            true
                        } else if self.add_server_window.name_field.chars().count() > 32 {
                            ui.label("• Name must be less than 32 characters");
                            true
                        } else if self.config.servers.iter().any(|server| server.alias == self.add_server_window.name_field) {
                            ui.label("• Name must be unique");
                            true
                        } else {
                            false
                        }
                    };
                    let is_ip_error = {
                        if self.add_server_window.ip_field.is_empty() {
                            ui.label("• IP is required");
                            true
                        } else {
                            false
                        }
                    };
                    let is_port_error = {
                        if self.add_server_window.port_field.parse::<u16>().is_err() {
                            ui.label("• Incorrect port value");
                            true
                        } else {
                            false
                        }
                    };
                    let is_user_error = {
                        if self.add_server_window.user_field.is_empty() {
                            ui.label("• User is required");
                            true
                        } else {
                            false
                        }
                    };

                    let enable_save_button = !is_name_error && !is_ip_error && !is_port_error && !is_user_error;

                    ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                        ui.separator();

                        ui.horizontal(|ui| {
                            if ui.add_enabled(enable_save_button, Button::new("Save")).clicked() {
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
                            if ui.button("Back").clicked() {
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
                        widgets::modal_label(ui, "Delete server");

                        ui.label("Are you sure you want to delete this server?");

                        ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                            ui.separator();

                            ui.horizontal(|ui| {
                                if ui.button("Yes").clicked() {
                                    self.config.servers.remove(idx_to_delete);
                                    self.save_config();
                                    self.delete_server_window = structs::DeleteServerWindow::default();
                                }
                                if ui.button("No").clicked() {
                                    self.delete_server_window = structs::DeleteServerWindow::default();
                                }
                            });
                        });
                    });
                }
            }
        }

        if self.sql_response_copy_window.show {
            Modal::new(Id::new("sql_response_copy_modal")).show(ctx, |ui| {
                let screen_rect = ctx.input(|i| i.screen_rect);

                widgets::modal_label(ui, "Text viewer");

                ui.set_width(if screen_rect.height() / 1.5 > 380.0 { screen_rect.height() / 1.5 } else { 380.0 });

                ScrollArea::both().max_height(screen_rect.height() / 1.5).show(ui, |ui| {
                    ui.label(self.sql_response_copy_window.response.clone().unwrap());
                });
                
                ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Copy").clicked() {
                        ui.ctx().copy_text(self.sql_response_copy_window.response.clone().unwrap());
                        self.sql_response_copy_window = structs::SQLResponseCopyWindow::default();
                    }
                    if ui.button("Close").clicked() {
                            self.sql_response_copy_window = structs::SQLResponseCopyWindow::default();
                        }
                    });
                });
            });
        }
    }

    fn update_pages(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        widgets::top_panel(ctx, |ui| {
            for (idx, page) in self.pages.pages.iter().enumerate() {
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

                if ui.memory(|mem| mem.is_popup_open(btn_id)) {
                    btn.context_menu(|ui| {
                        if ui.button("Close").clicked() {
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
                                    ui.heading("Welcome to Rs-Postgres: Rust-based PostgreSQL client.");
                                });

                                ui.add_space(16.0);
                                ui.label(RichText::new("Features").strong());
                                ui.separator();
                                ui.vertical(|ui| {
                                    ui.label("• Lightweight and fast");
                                    ui.label("• Secure encryption of server credentials");
                                    ui.label("• Connect to multiple PostgreSQL servers");
                                    ui.label("• Manage databases through GUI");
                                    ui.label("• Execute SQL queries with results preview");
                                });

                                ui.add_space(16.0);
                                ui.label(RichText::new("Getting Started").strong());
                                ui.separator();
                                ui.vertical(|ui| {
                                    ui.label("1. Click 'Add server' in left panel");
                                    ui.label("2. Enter server connection parameters");
                                    ui.label("3. Select database in connection tree");
                                    ui.label("4. Start working with SQL queries");
                                });

                                ui.add_space(16.0);
                                ui.label(RichText::new("Resources").strong());
                                ui.separator();
                                ui.horizontal(|ui| {
                                    if ui.add(Button::new("🐙 GitHub").fill(Color32::TRANSPARENT))
                                        .on_hover_text("Open repository")
                                        .clicked() {

                                        open::that("https://github.com/pywebsol/rs-postgres").unwrap();
                                    }
                                });
                                ui.horizontal(|ui| {
                                    if ui.add(Button::new("📝 License").fill(Color32::TRANSPARENT))
                                        .on_hover_text("Open license")
                                        .clicked() {

                                        open::that("https://github.com/pywebsol/rs-postgres/blob/main/LICENSE").unwrap();
                                    }
                                });
                                ui.horizontal(|ui| {
                                    if ui.add(Button::new("📨 Support").fill(Color32::TRANSPARENT))
                                        .on_hover_text("Open telegram")
                                        .clicked() {

                                        open::that("https://t.me/bot_token").unwrap();
                                    }
                                });

                                ui.add_space(24.0);
                                ui.label(RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION"))).small().color(Color32::GRAY));
                            });
                        },
                        structs::PageType::SQLQuery(sqlquery_page) => {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.heading(format!("SQL query tool for database {}", sqlquery_page.name));

                                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                        let code_is_empty = sqlquery_page.code.is_empty();

                                        if ui.add_enabled(!code_is_empty, Button::new("Run (F5)")).clicked() || (ui.input(|i| i.key_pressed(Key::F5) && !code_is_empty)) {
                                            let runtime = &self.runtime;
            
                                            sqlquery_page.sql_query_execution_status = Some(Arc::new(Mutex::new(structs::SQLQueryExecutionStatusType::Running)));

                                            let database_clone = sqlquery_page.database.clone();
                                            let code_clone = sqlquery_page.code.clone();
                                            let sql_query_execution_status = sqlquery_page.sql_query_execution_status.clone();
            
                                            runtime.spawn(async move {
                                                Self::fetch_sql_query(database_clone, &code_clone, sql_query_execution_status).await;
                                            });
                                        }
                                    });
                                });

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
                                        .background_color(Color32::from_hex("#242424").unwrap())
                                        .hint_text("SELECT * FROM ...")
                                        .layouter(&mut layouter),
                                );
                                if code_editor.secondary_clicked() {
                                    ui.memory_mut(|mem| mem.open_popup(Id::new("code_editor_popup")));
                                }

                                if ui.memory(|mem| mem.is_popup_open(Id::new("code_editor_popup"))) {
                                    code_editor.context_menu(|ui| {
                                        if ui.button("Clear").clicked() {
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
                                                ui.label("Running...");
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

                                            let available_height = ui.available_height();

                                            ui.horizontal(|ui| {
                                                ui.label("Success");
                                                ui.separator();
                                                ui.label(format!("Rows: {}", rows_count));
                                                ui.separator();
                                                ui.label(format!("Time: {} ms", execution_time));
                                            });

                                            ui.separator();

                                            if !data.is_empty() {
                                                TableBuilder::new(ui)
                                                    .striped(true)
                                                    .auto_shrink([false, false])
                                                    .max_scroll_height(available_height)
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
                                                        for i in 0..data[data.keys().next().unwrap()].len() {
                                                            body.row(16.0, |mut row| {
                                                                for value in data.values() {
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
                                                                                ui.label("Click to copy");
                                                                            });
                                                                        }
                                                                    });
                                                                }
                                                            });
                                                        }
                                                    });
                                                } else {
                                                    ui.horizontal(|ui| {
                                                        ui.add(self.icons.warning.clone());
                                                        ui.label("Error");
                                                    });

                                                    ui.heading("No data returned");
                                                }
                                        }
                                        structs::SQLQueryExecutionStatusType::Error(e) => {
                                            ui.separator();

                                            ui.horizontal(|ui| {
                                                ui.add(self.icons.warning.clone());
                                                ui.label("Error");
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
        if self.login_window.show {
            CentralPanel::default().show(ctx, |_| {});

            Modal::new(Id::new("login_window")).show(ctx, |ui| {
                ui.set_width(360.0);

                if self.login_window.clear_storage {
                    widgets::modal_label(ui, "Clear storage");

                    ui.label(RichText::new("Do you want to clear storage? This action is irreversible."));

                    ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Yes").clicked() {
                                self.login_window = structs::LoginWindow::default();

                                self.config.servers = Vec::new();
                                self.config.password_hash = None;

                                self.save_config();
                            }
                            if ui.button("No").clicked() {
                                self.login_window.clear_storage = false;
                            }
                        });
                    });

                    return;
                }
                
                widgets::modal_label(ui, "Login");

                if let Some(error) = &self.login_window.error {
                    ui.label(RichText::new(error).color(Color32::RED));
                }

                ui.horizontal(|ui| {
                    if self.config.password_hash.is_some() {
                        ui.label("Enter encryption password:");
                    } else {
                        ui.label("Create encryption password:");
                    }
                    
                    TextEdit::singleline(&mut self.login_window.password).password(true).show(ui);
                });

                ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                    ui.add_space(8.0);
                    ui.separator();

                    ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                        ui.horizontal(|ui| {
                            if !self.config.servers.is_empty() {
                                if ui.button(RichText::new("Clear storage").color(Color32::RED)).clicked() {
                                    self.login_window.clear_storage = true;
                                }
                            }
                            if ui.button("Login").clicked() || (ui.input(|i| i.key_pressed(Key::Enter))) {
                                let password = self.login_window.password.clone();

                                self.login_window.error = None;
                                self.password = Some(password.clone());

                                ui.spinner();

                                if self.config.password_hash.is_some() {
                                    if utils::create_checksum(&password) != self.config.password_hash.clone().unwrap() {
                                        self.login_window.error = Some("Incorrect password: hash mismatch".to_string());
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
            CollapsingHeader::new("Servers")
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
                                        CollapsingHeader::new("Databases").show(ui, |ui| {
                                            let db_state = {
                                                let dbs = self.db_manager.dbs.lock().expect("Failed to lock dbs mutex");
                                                dbs.get(&server_id).cloned()
                                            };
                                            if let Some(structs::DbState::Loaded(_db)) = db_state {
                                                for database in _db {
                                                    CollapsingHeader::new(&database.name).show(ui, |ui| {
                                                        if ui.button("SQL Query").clicked() {
                                                            self.pages.pages.push(structs::Page {
                                                                title: String::from(format!("{} ({}:{})", database.name, server.ip, server.port)),
                                                                page_type: structs::PageType::SQLQuery(structs::SQLQueryPage {
                                                                    name: database.name.clone(),
                                                                    database: database.database.clone(),
                                                                    code: String::new(),
                                                                    sql_query_execution_status: None,
                                                                }),
                                                            });
                                                            self.pages.current_page_index = (self.pages.pages.len() - 1) as u16;
                                                        }
                                                    });
                                                }
                                            }
                                        });
                                    }).header_response)
                                }
                                Some(structs::DbState::Error(e)) => {
                                    let warning = ui.add(self.icons.warning.clone());
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
                                        if ui.button("Delete").clicked() {
                                            ui.memory_mut(|mem| mem.close_popup());
                                            self.delete_server_window.show = true;
                                            self.delete_server_window.server = Some(server.clone());
                                        }
                                    });
                                }
                            }
                        });
                    }

                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.separator();
    
                        if ui.button("Add server").clicked() {
                            self.add_server_window.show = true;
                        }
                    });
                });
            });

        self.update_windows(ctx, _frame);
        self.update_pages(ctx, _frame);
    }
}
