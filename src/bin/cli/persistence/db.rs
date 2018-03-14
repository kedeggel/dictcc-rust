use app_dirs::*;
use std::path::Path;
use cli::Cli;
use dictcc::sqlite::SqliteController;
use error::DictCliResult;
use std::path::PathBuf;
use persistence::APP_INFO;
use persistence::DB_NAME;
use error::DictCliError;

pub fn sqlite_db_path() -> DictCliResult<PathBuf> {
    let mut user_config_path = app_root(AppDataType::UserData, &APP_INFO)?;

    user_config_path.push(DB_NAME);

    Ok(user_config_path)
}

pub enum DBAction<'a, 'b> {
    List,
    Add(&'a Path),
    Delete(&'b str),
}

impl<'a, 'b> DBAction<'a, 'b> {
    pub fn execute(self) -> DictCliResult<()> {
        let mut controller = SqliteController::new(sqlite_db_path()?)?;
        match self {
            DBAction::List => {
                let dicts = controller.list_dicts()?;

                // TODO: format
                eprintln!("dicts = {:#?}", dicts);
            }
            DBAction::Add(dictcc_db_path) => {
                controller.add_dict(dictcc_db_path)?;
            }
            DBAction::Delete(dictcc_db_name) => {
                let dicts = controller.list_dicts()?;

                let opt_id = dicts.into_iter().filter_map(|metadata| {
                    if metadata.languages.to_string() == dictcc_db_name {
                        Some(metadata.dict_id)
                    } else {
                        None
                    }
                }).next();

                if let Some(id) = opt_id {
                    controller.delete(&id)?;
                } else {
                    return Err(DictCliError::InvalidDictId(dictcc_db_name.to_string()))
                }
            }
        }

        Ok(())
    }
}

impl<'a> From<&'a Cli> for Option<DBAction<'a, 'a>> {
    fn from(cli: &'a Cli) -> Self {
        let &Cli { ref list, ref add, ref delete, .. } = cli;

        if *list {
            Some(DBAction::List)
        } else if let &Some(ref path) = add {
            Some(DBAction::Add(&path))
        } else if let &Some(ref name) = delete {
            Some(DBAction::Delete(&name))
        } else {
            None
        }
    }
}