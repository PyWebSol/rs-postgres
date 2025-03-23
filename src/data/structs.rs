use serde::{Deserialize, Serialize};

use indexmap::IndexMap;

use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    pub servers: Vec<Server>,
    pub password_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Server {
    pub alias: String,
    pub ip: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub service_database: String,
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

impl Default for AddServerWindow {
    fn default() -> Self {
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

#[derive(Default)]
pub struct DeleteServerWindow {
    pub show: bool,
    pub server: Option<Server>,
}

#[derive(Default)]
pub struct SQLResponseCopyWindow {
    pub show: bool,
    pub response: Option<String>,
}

pub struct LoginWindow {
    pub show: bool,
    pub clear_storage: bool,
    pub password: String,
    pub error: Option<String>,
}

impl Default for LoginWindow {
    fn default() -> Self {
        Self {
            show: true,
            clear_storage: false,
            password: String::new(),
            error: None,
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
pub struct SQLQueryExecutionSuccess {
    pub result: IndexMap<String, Vec<ValueType>>,
    pub execution_time: u64,
}

#[derive(Clone, Debug)]
pub enum SQLQueryExecutionStatusType {
    Running,
    Success(SQLQueryExecutionSuccess),
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
    pub sql_query_execution_status: Option<Arc<Mutex<SQLQueryExecutionStatusType>>>,
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

impl Default for Page {
    fn default() -> Self {
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

impl Default for Pages {
    fn default() -> Self {
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
