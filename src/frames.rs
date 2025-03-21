use crate::data::*;
use crate::database;
use crate::utils::SyntaxHighlighter;

use eframe::{egui, App};
use egui::Widget;
use egui::{
    Window, SidePanel, Spinner, Layout, Align, TextEdit, Color32, Button,
    CollapsingHeader, Id, TopBottomPanel, Grid, ScrollArea,
};
use log::{info, error};
use std::fs as std_fs;
use std::collections::HashMap;
use std::ops::Index;
use std::sync::{Arc, Mutex};

struct DbManager {
    dbs: Arc<Mutex<HashMap<String, structs::DbState>>>,
}

pub struct Main<'a> {
    db_manager: DbManager,
    config: structs::Config,
    add_server_window: structs::AddServerWindow,
    delete_server_window: structs::DeleteServerWindow,
    icons: structs::Icons<'a>,
    runtime: tokio::runtime::Runtime,
    pages: structs::Pages,
    actions: Vec<structs::Action>,
    highlighter: SyntaxHighlighter,
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
            icons: structs::Icons {
                warning: egui::Image::new(icons::WARNING).max_size(egui::vec2(32.0, 32.0)),
                rs_postgres: egui::Image::new(icons::RS_POSTGRES),
            },
            runtime,
            pages: structs::Pages::default(),
            actions: Vec::new(),
            highlighter: SyntaxHighlighter::new(),
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
                let mut dbs = dbs.lock().unwrap();
                dbs.insert(id, structs::DbState::Loaded(db));
            }
            Err(e) => {
                error!("Error loading database for server {}: {}", server.ip, e);
                let mut dbs = dbs.lock().unwrap();
                dbs.insert(id, structs::DbState::Error(e.to_string()));
            }
        }
    }

    fn custom_sql_editor(ui: &mut egui::Ui, highlighter: &SyntaxHighlighter, code: &mut String) {
        let response = ui.add(
            TextEdit::multiline(code)
                .code_editor()
                .desired_width(f32::INFINITY)
                .desired_rows(10),
        );
    
        if response.changed() {}
    
        let highlighted = highlighter.highlight(code);
    
        ui.group(|ui| {
            for (style, text) in highlighted {
                let color = Color32::from_rgb(
                    style.foreground.r,
                    style.foreground.g,
                    style.foreground.b,
                );
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(text).color(color).monospace()
                    ),
                );
            }
        });
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

                    ui.with_layout(Layout::bottom_up(Align::RIGHT), |ui| {
                        ui.horizontal(|ui| {
                            let is_port_error = self.add_server_window.port_field.parse::<u16>().is_err();

                            if ui.add_enabled(!is_port_error, Button::new("Save")).clicked() {
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
                        });
                    });
                }
            }
        }
    }

    fn update_pages(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("pages_panel").show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    for (idx, page) in self.pages.pages.iter().enumerate() {
                        let mut button_title = page.title.clone();
                        if button_title.len() > 16 {
                            button_title = format!("{}...", &button_title[0..16]);
                        }

                        let btn = ui.button(&button_title);
                        let btn_id = Id::new(idx);
                        
                        if btn.clicked() {
                            self.pages.current_page_index = idx as u16;
                        }
                        if btn.secondary_clicked() {
                            ui.memory_mut(|mem| mem.open_popup(btn_id));
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
                    if let Some(page) = self.pages.pages.get(self.pages.current_page_index as usize) {
                        match &page.page_type {
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
                                    let mut page_code_content = match &mut self.pages.pages[self.pages.current_page_index as usize].page_type {
                                        structs::PageType::Welcome => {
                                            error!("Incorrect page data");
                                            None
                                        },
                                        structs::PageType::SQLQuery(sqlquery_page) => Some(&mut sqlquery_page.code),
                                    }.unwrap();

                                    if ui.button("Run").clicked() {
                                        page_code_content.clear();
                                    }
                                    ui.add(
                                        TextEdit::multiline(page_code_content)
                                            .code_editor()
                                            .desired_width(f32::INFINITY)
                                            .desired_rows(10)
                                            .background_color(Color32::from_hex("#242424").unwrap())
                                            .hint_text("SELECT * FROM ..."),
                                    );
                                });
                            },
                        }
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
                                        let db_state = {
                                            let dbs = self.db_manager.dbs.lock().expect("Failed to lock dbs mutex");
                                            dbs.get(&server_id).cloned()
                                        };
                                        if let Some(structs::DbState::Loaded(_db)) = db_state {
                                            if ui.button("SQL Query").clicked() {
                                                self.pages.pages.push(
                                                    structs::Page {
                                                        title: String::from("SQL Query"),
                                                        page_type: structs::PageType::SQLQuery(
                                                            structs::SQLQueryPage {
                                                                database: _db,
                                                                code: String::new(),
                                                                output: None,
                                                            }
                                                        ),
                                                    }
                                                );
                                                self.pages.current_page_index = (self.pages.pages.len() - 1) as u16;
                                            }
                                        }
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

        self.update_windows(ctx, _frame);
        self.update_pages(ctx, _frame);
    }
}
