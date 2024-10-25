use simple_server_utility::{cmd, nginx};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	human_panic::setup_panic!();

	let args = cmd::CmdArgs::parse();

	let token = if args.token_file {
		std::fs::read_to_string(args.token)?.trim().to_string()
	} else {
		args.token
	};

	simple_server_utility::set_config(token);
	
	match args.action {
		cmd::SubCommand::List(args) => simple_server_utility::list(args).await,
		cmd::SubCommand::Shutdown(args) => simple_server_utility::shutdown(args).await?,
		cmd::SubCommand::Startup(args) => simple_server_utility::startup(args).await?,
		cmd::SubCommand::Rescale(args) => simple_server_utility::rescale(args).await?,
		cmd::SubCommand::NginxConfig(args) => nginx::generate_servers(args)?,
		#[cfg(debug_assertions)]
		cmd::SubCommand::Debug(_) => (),
	}

	Ok(())
}
