use std::{
    env::{self},
    fs,
    path::Path,
};

const CONFIG_FILE_NAME: &str = "config.toml";

fn main() {
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_file = Path::new(&root).join("config").join(CONFIG_FILE_NAME);

    if !src_file.exists() {
        //println!("cargo:warning={} does not exist", src_file.display());
        return;
    }

    let dot_dir_name = env::var("CARGO_PKG_NAME").unwrap();
    let dot_dir_name = format!(".{}", dot_dir_name);

    //println!("cargo:warning=----------------------");

    let home = home::home_dir().unwrap();
    let dst_dir = Path::new(&home).join(dot_dir_name);

    if !dst_dir.exists() {
        fs::create_dir_all(&dst_dir).unwrap();
    }

    let dst_file = Path::new(&dst_dir).join(CONFIG_FILE_NAME);

    /*
    println!(
        "cargo:warning=copy({},{})",
        src_file.display(),
        dst_file.display()
    );
    */

    fs::copy(&src_file, dst_file).unwrap();

    //println!("cargo:warning=----------------------");

    println!("cargo:rerun-if-changed={}", src_file.display());
}
