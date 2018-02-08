use app_dirs::*;
use cli::Cli;
use error::{DictCliError, DictCliResult};
use std::fs::{canonicalize, File};
use std::io::BufReader;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use toml;

const APP_INFO: AppInfo = AppInfo { name: "dictcc-rust", author: "DeggelmannAndLengler" };
const CONFIG_NAME: &str = "config.toml";

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    last_database_path: PathBuf,
}

impl Config {
    fn new<P: AsRef<Path>>(last_database_path: P) -> DictCliResult<Self> {
        Ok(Config {
            last_database_path: Config::canonicalize_path(last_database_path)?,
        })
    }

    pub fn get_database_path(&self) -> &Path {
        self.last_database_path.as_path()
    }

    fn get_config_path() -> DictCliResult<PathBuf> {
        let mut user_config_path = app_root(AppDataType::UserConfig, &APP_INFO)?;

        user_config_path.push(CONFIG_NAME);

        Ok(user_config_path)
    }

    fn read() -> DictCliResult<Option<Config>> {
        let config_path = Config::get_config_path()?;

        if config_path.exists() {
            let file = File::open(config_path)?;
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();

            buf_reader.read_to_string(&mut contents)?;

            let config: Config = toml::from_str(&contents)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    fn canonicalize_path<P: AsRef<Path>>(path: P) -> DictCliResult<PathBuf> {
        Ok(canonicalize(path)?)
    }

    fn set_last_database_path<P: AsRef<Path>>(&mut self, path: P) -> DictCliResult<()> {
        self.last_database_path = Config::canonicalize_path(path)?;

        Ok(())
    }

    fn write(&self) -> DictCliResult<()> {
        let mut file = File::create(Config::get_config_path()?)?;

        let config_string = toml::to_string(self)?;

        file.write_all(config_string.as_bytes())?;

        Ok(())
    }

    pub fn update_with_cli(cli: &Cli) -> DictCliResult<Config> {
        let opt_config = Config::read()?;

        if let Some(mut config) = opt_config {
            if let Some(ref database_path) = cli.database_path {
                // Update Config
                config.set_last_database_path(database_path)?;

                config.write()?;

                Ok(config)
            } else {
                // Read Config
                Ok(config)
            }
        } else if let Some(ref database_path) = cli.database_path {
            // Create Config
            let config = Config::new(database_path)?;

            config.write()?;

            Ok(config)
        } else {
            Err(DictCliError::NoDatabasePath)
        }
    }
}