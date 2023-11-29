use std::{
    fs::File,
    path::{Path, PathBuf},
};

use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Debug)]
pub enum SharedError {
    ConfigParse(serde_ini::de::Error),
    IO(std::io::Error),
}

pub type SharedResult<T> = Result<T, SharedError>;

#[derive(Debug, Deserialize)]
pub struct Config {
    ui_path: PathBuf,
    my_epoch: String,
    cleanup_interval: u64,
    max_age: i64,
    auth_db: String,
    secret_path: PathBuf,
}

impl Config {
    pub fn get_epoch(&self) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(&self.my_epoch, "%Y-%m-%d %T")
            .expect("Could not parse thing")
            .into()
    }

    pub fn ui_path(&self) -> &Path {
        &self.ui_path
    }

    pub fn cleanup_interval(&self) -> u64 {
        self.cleanup_interval
    }

    pub fn max_age(&self) -> i64 {
        self.max_age
    }

    pub fn auth_db(&self) -> &str {
        &self.auth_db
    }

    pub fn secret_path(&self) -> &Path {
        &self.secret_path
    }
}

pub fn read_config<P>(path: P) -> SharedResult<Config>
where
    P: AsRef<Path>,
{
    let f = File::open(path).map_err(SharedError::IO)?;
    serde_ini::from_read(f).map_err(SharedError::ConfigParse)
}

pub fn custom_timestamp(custom_epoch: NaiveDateTime) -> i64 {
    chrono::offset::Local::now()
        .naive_local()
        .signed_duration_since(custom_epoch)
        .num_seconds()
}
