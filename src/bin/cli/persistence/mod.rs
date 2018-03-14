use app_dirs::*;

const APP_INFO: AppInfo = AppInfo { name: "dictcc-rust", author: "DeggelmannAndLengler" };
const CONFIG_NAME: &str = "config.toml";
const DB_NAME: &str = "dictcc.db";

pub mod config;
pub mod db;