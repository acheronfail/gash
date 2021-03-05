use std::env;
use std::fs::{self, File};
use std::path::Path;

use clap::{crate_name, IntoApp};
use clap_generate::{generate, generators};

#[allow(dead_code)]
#[path = "src/cli.rs"]
mod cli;

fn main() {
    let mut app = cli::Args::into_app();
    let name = crate_name!();
    let bin_name = "rgr";

    // https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script
    let outdir = env::var_os("OUT_DIR").expect("failed to find OUT_DIR");
    fs::create_dir_all(&outdir).expect("failed to create dirs for OUT_DIR");

    // Create a stamp file. (This is used by CI to help find the OUT_DIR.)
    fs::write(Path::new(&outdir).join("gash-stamp"), "").unwrap();

    // Generate completions.
    let f = |name: &str| File::create(Path::new(&outdir).join(name)).unwrap();
    generate::<generators::Zsh, _>(&mut app, name, &mut f(&format!("_{}", bin_name)));
    generate::<generators::Bash, _>(&mut app, name, &mut f(&format!("{}.bash", bin_name)));
    generate::<generators::Fish, _>(&mut app, name, &mut f(&format!("{}.fish", bin_name)));
    generate::<generators::Elvish, _>(&mut app, name, &mut f(&format!("{}.elvish", bin_name)));
    generate::<generators::PowerShell, _>(&mut app, name, &mut f(&format!("_{}.ps1", bin_name)));
}
