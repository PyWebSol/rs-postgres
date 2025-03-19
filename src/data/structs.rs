use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub servers: Vec<Server>,
}

#[derive(Serialize, Deserialize)]
pub struct Server {
    pub alias: String,
    pub ip: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            servers: Vec::new()
        }
    }
}
