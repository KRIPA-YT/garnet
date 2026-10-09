use std::{env, net::SocketAddr};

pub struct Config {
    pub migration_database_url: String,
    pub app_database_url: String,
    pub listen_addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            migration_database_url: env::var("MIGRATOR_DATABASE_URL")?,
            app_database_url: env::var("APP_DATABASE_URL")?,
            listen_addr: "0.0.0.0:3000".parse()?,
        })
    }
}
