use clap::{CommandFactory, ValueEnum};
use clap_complete::generate_to;
use std::env::current_dir;
use std::io::Error;

include!("src/cmd/mod.rs");

fn main() -> Result<(), Error> {
	// Output into project_root/completions/
    let out_dir = current_dir().expect("Failed to get current directory").join("completions");

    let mut cmd = CmdArgs::command();
	for &shell in clap_complete::Shell::value_variants() {
		generate_to(shell, &mut cmd, "ssu", &out_dir)?;
	}

    println!("cargo:warning=completion files generated to: {out_dir:?}");
	println!("cargo:rerun-if-changed=src/cmd/");

    Ok(())
}
