use std::{env, path::Path};

use fs_extra::dir::{CopyOptions, copy};

fn main() {
    //println!("cargo:warning=----------------------");

    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_config = Path::new(&root).join("config");

    let home = env::var("HOME").unwrap();
    let dst_config = Path::new(&home).join(".ado");

    /*
    println!(
        "cargo:warning=manifest: src={} dst={}",
        src_config.display(),
        dst_config.display()
    );
    */

    let options = CopyOptions::new()
        .overwrite(true)
        .skip_exist(false)
        .copy_inside(true);

    copy(&src_config, dst_config, &options).expect("Unable to copy files");

    //println!("cargo:warning=----------------------");
    //println!("cargo:warning=manifest: src={}", src_config.display(),);

    println!("cargo:rerun-if-changed={}", src_config.display());
}
