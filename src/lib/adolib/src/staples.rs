use std::{
    env,
    path::{Path, PathBuf},
};

use crate::error::{Error, Result};

fn find_file_cwd<P>(rel_file_path: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let cwd = env::current_dir()?;

    let file_path = cwd.join(rel_file_path);

    match file_path.exists() {
        true => Ok(file_path),
        false => Err(Error::FileNotFoundError { file_path }),
    }
}

fn find_file_sxs<P>(rel_file_path: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let prog_dir = env::current_exe()?;

    let dirname = prog_dir.parent().ok_or(Error::DirnameError)?;

    let file_path = dirname.join(rel_file_path);

    match file_path.exists() {
        true => Ok(file_path),
        false => Err(Error::FileNotFoundError { file_path }),
    }
}

pub fn find_file<P>(rel_file_path: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    match find_file_sxs(&rel_file_path) {
        Ok(v) => Ok(v),
        Err(_) => find_file_cwd(&rel_file_path),
    }
}
