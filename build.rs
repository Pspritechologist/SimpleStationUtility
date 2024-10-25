use clap::{CommandFactory, ValueEnum};
use clap_complete::generate_to;
use std::io::Error;

include!("src/cmd/mod.rs");

fn main() -> Result<(), Error> {
	let out_dir = match std::env::var_os("OUT_DIR") {
		Some(out_dir) => out_dir,
		None => return Ok(()),
	};

    let mut cmd = CmdArgs::command();
	for &shell in clap_complete::Shell::value_variants() {
		generate_to(shell, &mut cmd, "ssu", &out_dir)?;
	}

    println!("cargo:warning=completion files generated to: {out_dir:?}");
	println!("cargo:rerun-if-changed=src/cmd/");

    Ok(())
}
