use serde::{Deserialize, Serialize};

use indexmap::IndexMap;

use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub servers: Vec<Server>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Server {
    pub alias: String,
    pub ip: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub service_database: String,
}

impl Config {
    pub fn default() -> Self {
        Self {
            servers: Vec::new(),
        }
    }
}

pub struct AddServerWindow {
    pub show: bool,
    pub name_field: String,
    pub ip_field: String,
    pub port_field: String,
    pub user_field: String,
    pub password_field: String,
    pub service_database_field: String,
}

impl AddServerWindow {
    pub fn default() -> Self {
        Self {
            show: false,
            name_field: String::new(),
            ip_field: String::new(),
            port_field: String::from("5432"),
            user_field: String::new(),
            password_field: String::new(),
            service_database_field: String::from("postgres"),
        }
    }
}

pub struct DeleteServerWindow {
    pub show: bool,
    pub server: Option<Server>,
}

impl DeleteServerWindow {
    pub fn default() -> Self {
        Self {
            show: false,
            server: None,
        }
    }
}

pub struct SQLResponseCopyWindow {
    pub show: bool,
    pub response: Option<String>,
}

impl SQLResponseCopyWindow {
    pub fn default() -> Self {
        Self {
            show: false,
            response: None,
        }
    }
}

pub struct Icons<'a> {
    pub warning: egui::Image<'a>,
    pub rs_postgres: egui::Image<'a>,
}

#[derive(Clone)]
pub struct LoadedDatabase {
    pub name: String,
    pub database: crate::database::Database,
}

#[derive(Clone)]
pub enum DbState {
    Loading,
    Loaded(Vec<LoadedDatabase>),
    Error(String),
}

#[derive(Clone, Debug)]
pub enum SQLQueryExecutionStatus {
    Running,
    Success(IndexMap<String, Vec<ValueType>>),
    Error(String),
}

#[derive(Clone, Debug)]
pub enum ValueType {
    Null,
    Text(String),
    Int(i32),
    BigInt(i64),
    Float(f64),
    Bool(bool),
    Bytea(Vec<u8>),
    Array(Vec<ValueType>),
    Unknown(String),
}

impl ValueType {
    pub fn to_string(&self) -> String {
        match self {
            ValueType::Null => "None".to_string(),
            ValueType::Text(text) => text.clone(),
            ValueType::Int(int) => int.to_string(),
            ValueType::BigInt(big_int) => big_int.to_string(),
            ValueType::Float(float) => float.to_string(),
            ValueType::Bool(bool) => bool.to_string(),
            ValueType::Bytea(items) => items
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            ValueType::Array(value_types) => value_types
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            ValueType::Unknown(unknown) => unknown.clone(),
        }
    }
}

#[derive(Clone)]
pub struct SQLQueryPage {
    pub name: String,
    pub database: crate::database::Database,
    pub code: String,
    pub sql_query_execution_status: Option<Arc<Mutex<SQLQueryExecutionStatus>>>,
}

#[derive(Clone)]
pub enum PageType {
    Welcome,
    SQLQuery(SQLQueryPage),
}

#[derive(Clone)]
pub struct Page {
    pub title: String,
    pub page_type: PageType,
}

impl Page {
    pub fn default() -> Self {
        Self {
            title: String::from("Welcome"),
            page_type: PageType::Welcome,
        }
    }
}

#[derive(Clone)]
pub struct Pages {
    pub current_page_index: u16,
    pub pages: Vec<Page>,
}

impl Pages {
    pub fn default() -> Self {
        Self {
            current_page_index: 0,
            pages: vec![Page::default()],
        }
    }
}

#[derive(Clone)]
pub enum Action {
    ClosePage(usize),
}
