use serde::{Deserialize, Serialize};

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
            servers: Vec::new()
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
