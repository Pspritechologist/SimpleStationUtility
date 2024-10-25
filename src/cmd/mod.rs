#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
/// A CLI application for managing SimpleStation servers
/// helping to bridge the gap between Hetzner and Pterodactyl.
pub struct CmdArgs {
	#[clap(subcommand)]
	pub action: SubCommand,
	#[arg(short, long, env="SSU_API_TOKEN")]
	/// The API token to use for authentication.
	pub token: String,
	#[arg(short='f', long, env="SSU_API_TOKEN_FILE", required=false)]
	/// If active, the API token will be interpreted as a file path to read the token from.
	pub token_file: bool,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum SubCommand {
	List(CmdList),
	Shutdown(CmdShutdown),
	Startup(CmdStartup),
	Rescale(CmdRescale),
	NginxConfig(CmdNginxConfig),
	#[cfg(debug_assertions)]
	Debug(CmdDebug),
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, clap::Parser)]
#[command()]
pub struct CmdDebug;

#[derive(Debug, Clone, clap::Parser)]
#[command(visible_alias="ls")]
/// Lists all nodes.
pub struct CmdList {
	#[arg(short='l', long)]
	/// List full details regarding the nodes.
	pub full: bool,
	#[arg(short, long)]
	/// Output in JSON format.
	pub json: bool,
	#[arg(short, long, value_enum, default_value_t=NodeSortOpt::Created)]
	/// Sort the nodes by a specific quality.
	pub sort: NodeSortOpt,
	#[arg(short, long)]
	/// Reverse the ordering of the Nodes.
	pub reverse: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum NodeSortOpt {
	#[clap(alias="a")]
	/// Sort by name alphabetically.
	Alphabetical,
	#[clap(alias="c")]
	/// Sort by creation date of the Node.
	Created,
	#[clap(alias="s")]
	/// Sort by Node's CPU core count.
	Cores,
	#[clap(alias="m")]
	/// Sort by Node's memory.
	Memory,
	#[clap(alias="s")]
	/// Sort by the Node's status.
	Status,
}

#[derive(Debug, Clone, clap::Parser)]
#[command()]
/// Shutdown a node.
pub struct CmdShutdown {
	#[arg(required=true)]
	/// The ID or name of the node to shutdown.
	/// 
	/// Using a name requires an additional request to the API. Use an ID when available.
	pub node: String,
	#[arg(short, long)]
	/// Restart the node after shutting it down.
	pub restart: bool,
	#[arg(long)]
	/// Forces a power down.
	/// 
	/// This is the equivalent of pulling the power cord on a physical server.
	pub force: bool,
}

#[derive(Debug, Clone, clap::Parser)]
#[command()]
/// Startup a node
pub struct CmdStartup {
	#[arg(required=true)]
	/// The ID or name of the node to startup
	/// 
	/// Using a name requires an additional request to the API. Use an ID when available.
	pub node: String,
}

#[derive(Debug, Clone, clap::Parser)]
#[command()]
/// Rescales a Node from its current shape to a compatible shape. Can also list all compatible shapes.
pub struct CmdRescale {
	#[arg(required=true)]
	/// The ID or name of the node to rescale.
	/// 
	/// Using a name requires an additional request to the API. Use an ID when available.
	pub node: String,
	#[arg()]
	/// The numeric ID of the new shape to rescale the Node to.
	/// 
	/// Lists all compatible shapes if omitted.
	pub shape: Option<u16>,
}

#[derive(Debug, Clone, clap::Parser)]
#[command(alias="nginx")]
/// Generates Nginx configuration files for SimpleStation servers.
pub struct CmdNginxConfig {
	#[arg(env="SSU_NGINX_CONFIG")]
	/// The path to the TOML file containing the server configurations.
	pub config: std::path::PathBuf,
	/// The Nginx configuration file to write to.
	/// 
	/// NOTE: This is a destructive action. If the file already exists, it will be overwritten.
	/// If omitted, the configuration will be printed to stdout.
	#[arg(env="SSU_NGINX_OUTPUT")]
	pub output: Option<std::path::PathBuf>,
}

#[test]
pub fn verify_cmd() {
	use clap::CommandFactory;
	CmdArgs::command().debug_assert();
}
