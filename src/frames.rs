use crate::data::*;
use crate::database;

use eframe::{egui, App};
use egui::{
    Window, Modal, SidePanel, Spinner, Layout, Align, TextEdit, Color32,
    Button, CollapsingHeader, Id, TopBottomPanel, Grid, ScrollArea, Label,
    RichText,
};
use egui_extras::{TableBuilder, Column};
use log::{info, error};
use std::fs as std_fs;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct DbManager {
    dbs: Arc<Mutex<HashMap<String, structs::DbState>>>,
}

pub struct Main<'a> {
    db_manager: DbManager,
    config: structs::Config,
    add_server_window: structs::AddServerWindow,
    delete_server_window: structs::DeleteServerWindow,
    sql_response_copy_window: structs::SQLResponseCopyWindow,
    icons: structs::Icons<'a>,
    runtime: tokio::runtime::Runtime,
    pages: structs::Pages,
    actions: Vec<structs::Action>,
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
            icons: structs::Icons {
                warning: egui::Image::new(icons::WARNING).max_size(egui::vec2(32.0, 32.0)),
                rs_postgres: egui::Image::new(icons::RS_POSTGRES),
            },
            runtime,
            pages: structs::Pages::default(),
            actions: Vec::new(),
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

        std_fs::write(
            &config_path,
            serde_json::to_string(&self.config).unwrap(),
        ).unwrap();
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

    async fn fetch_sql_query(database: database::Database, code: &str, sql_query_execution_status: Option<Arc<Mutex<structs::SQLQueryExecutionStatus>>>) {
        let result = database.execute_query(&code).await;
        let execution_status = match result {
            Ok(result) => structs::SQLQueryExecutionStatus::Success(result),
            Err(e) => structs::SQLQueryExecutionStatus::Error(e.to_string()),
        };

        if let Some(sql_query_execution_status) = sql_query_execution_status {
            let mut sql_query_execution_status = sql_query_execution_status.lock().unwrap();
            *sql_query_execution_status = execution_status;
        }
    }

    fn update_windows(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.add_server_window.show {
            Window::new("New server")
                .scroll([false, false])
                .resizable(false)
                .show(ctx, |ui| {
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
                        } else if self.add_server_window.name_field.chars().any(|c| !c.is_alphanumeric()) {
                            ui.label("• Name must be alphanumeric");
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
                        ui.label("Are you sure you want to delete this server?");
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
                }
            }
        }

        if self.sql_response_copy_window.show {
            Modal::new(Id::new("sql_response_copy_modal")).show(ctx, |ui| {
                ui.set_width(320.0);

                ui.label(self.sql_response_copy_window.response.clone().unwrap());
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
        }
    }

    fn update_pages(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("pages_panel").show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                ui.horizontal_top(|ui| {
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
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
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
                            });
                        },
                        structs::PageType::SQLQuery(sqlquery_page) => {
                            ui.vertical(|ui| {
                                ui.label(format!("Database {}", sqlquery_page.name));

                                if ui.button("Run").clicked() {
                                    let runtime = &self.runtime;
    
                                    sqlquery_page.sql_query_execution_status = Some(Arc::new(Mutex::new(structs::SQLQueryExecutionStatus::Running)));

                                    let database_clone = sqlquery_page.database.clone();
                                    let code_clone = sqlquery_page.code.clone();
                                    let sql_query_execution_status = sqlquery_page.sql_query_execution_status.clone();
    
                                    runtime.spawn(async move {
                                        Self::fetch_sql_query(database_clone, &code_clone, sql_query_execution_status).await;
                                    });
                                }
                                ui.add(
                                    TextEdit::multiline(&mut sqlquery_page.code)
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(10)
                                        .background_color(Color32::from_hex("#242424").unwrap())
                                        .hint_text("SELECT * FROM ..."),
                                );
                                if let Some(sql_query_execution_status) = &sqlquery_page.sql_query_execution_status {
                                    let sql_query_execution_status = sql_query_execution_status.lock().unwrap().clone();
                                    match &sql_query_execution_status {
                                        structs::SQLQueryExecutionStatus::Running => {
                                            ui.horizontal(|ui| {
                                                ui.add(Spinner::new());
                                                ui.label("Running...");
                                            });
                                        }
                                        structs::SQLQueryExecutionStatus::Success(data) => {
                                            let available_height = ui.available_height();

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
                                                            for (key, value) in data.iter() {
                                                                row.col(|ui| {
                                                                    let content = value[i].to_string();
                                                                    let mut label = content.clone().replace("\n", " ");
                                                                    
                                                                    let label = Label::new(label)
                                                                        .wrap_mode(egui::TextWrapMode::Truncate);

                                                                    if ui.add(label).clicked() {
                                                                        self.sql_response_copy_window.show = true;
                                                                        self.sql_response_copy_window.response = Some(content);
                                                                    }
                                                                });
                                                            }
                                                        });
                                                    }
                                                });
                                        }
                                        structs::SQLQueryExecutionStatus::Error(e) => {
                                            ui.label(e);
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
        SidePanel::left("left_panel").show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
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
                                                                self.pages.pages.push(
                                                                    structs::Page {
                                                                        title: String::from(format!("{} ({}:{})", database.name, server.ip, server.port)),
                                                                        page_type: structs::PageType::SQLQuery(
                                                                            structs::SQLQueryPage {
                                                                                name: database.name,
                                                                                database: database.database,
                                                                                code: String::new(),
                                                                                output: None,
                                                                                sql_query_execution_status: None,
                                                                            }
                                                                        ),
                                                                    }
                                                                );
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
                    });

                    ui.separator();

                    if ui.button("Add server").clicked() {
                        self.add_server_window.show = true;
                    }
                });
        });

        self.update_windows(ctx, _frame);
        self.update_pages(ctx, _frame);
    }
}
