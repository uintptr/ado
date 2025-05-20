use std::{
    env,
    path::{Path, PathBuf},
};

use crate::error::{Error, Result};

fn find_file_cwd<P>(file_name: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let cwd = env::current_dir()?;

    let file_path = cwd.join(file_name);

    match file_path.exists() {
        true => Ok(file_path),
        false => Err(Error::FileNotFoundError { file_path }),
    }
}

fn find_file_sxs<P>(file_name: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let prog_dir = env::current_exe()?;

    let dirname = prog_dir.parent().ok_or(Error::DirnameError)?;

    let file_path = dirname.join(file_name);

    match file_path.exists() {
        true => Ok(file_path),
        false => Err(Error::FileNotFoundError { file_path }),
    }
}

pub fn find_file<P>(file_name: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    match find_file_sxs(&file_name) {
        Ok(v) => Ok(v),
        Err(_) => find_file_cwd(file_name),
    }
}
