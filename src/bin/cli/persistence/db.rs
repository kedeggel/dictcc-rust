use app_dirs::*;
use std::path::Path;
use cli::Cli;
use dictcc::sqlite::SqliteDict;
use error::DictCliResult;
use std::path::PathBuf;
use persistence::APP_INFO;
use persistence::DB_NAME;

pub enum ManageDB<'a, 'b> {
    List,
    Add(&'a Path),
    Delete(&'b str),
}

impl<'a, 'b> ManageDB<'a, 'b> {
    pub fn execute(self) -> DictCliResult<()> {
        match self {
            ManageDB::List => {
                unimplemented!()
            }
            ManageDB::Add(dictcc_db_path) => {
                SqliteDict::new(ManageDB::sqlite_db_path()?, dictcc_db_path)?;
            }
            ManageDB::Delete(dictcc_db_name) => {
                unimplemented!()
            }
        }

        Ok(())
    }

    pub fn sqlite_db_path() -> DictCliResult<PathBuf> {
        let mut user_config_path = app_root(AppDataType::UserData, &APP_INFO)?;

        user_config_path.push(DB_NAME);

        Ok(user_config_path)
    }
}

impl<'a> From<&'a Cli> for Option<ManageDB<'a, 'a>> {
    fn from(cli: &'a Cli) -> Self {
        let &Cli { ref list, ref add, ref delete, .. } = cli;

        if *list {
            Some(ManageDB::List)
        } else if let &Some(ref path) = add {
            Some(ManageDB::Add(&path))
        } else if let &Some(ref name) = delete {
            Some(ManageDB::Delete(&name))
        } else {
            None
        }
    }
}