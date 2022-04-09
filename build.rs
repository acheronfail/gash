use clap::IntoApp;
use clap_complete::{generate_to, shells};
use std::env;
use std::io::Error;

#[allow(dead_code)]
#[path = "src/args.rs"]
mod args;

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut app = args::Args::command();
    macro_rules! gen {
        ($shell:expr) => {{
            let path = generate_to(
                $shell,
                &mut app,            // We need to specify what generator to use
                clap::crate_name!(), // We need to specify the bin name manually
                &outdir,             // We need to specify where to write to
            )?;

            println!("cargo:warning=completion file generated: {:?}", path);
        }};
    }

    gen!(shells::Bash);
    gen!(shells::Elvish);
    gen!(shells::Fish);
    gen!(shells::PowerShell);
    gen!(shells::Zsh);

    Ok(())
}
