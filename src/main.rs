mod utils;
mod cmd;

use std::{borrow::Cow, io::Write, process::exit};

use clap::Parser;
use hcloud::apis::{self, servers_api};
use utils::{NodeTable, ShapeTable};

static CONFIG: utils::ConfigCell = utils::ConfigCell::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	human_panic::setup_panic!();

	let args = cmd::CmdArgs::parse();

	let token = if args.token_file {
		std::fs::read_to_string(args.token)?.trim().to_string()
	} else {
		args.token
	};

	CONFIG.set(utils::Configuration::default().with_access_token(token));
	
	match args.action {
		cmd::SubCommand::List(args) => list(args).await,
		cmd::SubCommand::Shutdown(args) => shutdown(args).await?,
		cmd::SubCommand::Startup(args) => startup(args).await?,
		cmd::SubCommand::Rescale(args) => rescale(args).await?,
	}

	Ok(())
}

async fn list(args: cmd::CmdList) {
	use cmd::NodeSortOpt::*;
	let mut nodes = servers_api::list_servers(&CONFIG, Default::default()).await.unwrap().servers;
	match args.sort {
		Alphabetical => nodes.sort_by(|a, b| a.name.cmp(&b.name)),
		Created => nodes.sort_by(|a, b| a.created.cmp(&b.created)),
		Cores => nodes.sort_by(|a, b| a.server_type.cores.cmp(&b.server_type.cores)),
		Memory => nodes.sort_by(|a, b| a.server_type.memory.partial_cmp(&b.server_type.memory).expect("Failed to compare memory")),
		Status => nodes.sort_by(|a, b| a.status.cmp(&b.status)),
	}
	if args.reverse {
		nodes.reverse();
	}

	if !args.full {
		if args.json {
			let data: Vec<_> = nodes.into_iter().map(|n| {
				let mut map = serde_json::Map::new();
				map.insert("id".to_string(), serde_json::Value::Number(serde_json::Number::from(n.id)));
				map.insert("name".to_string(), serde_json::Value::String(n.name));
				map
			}).collect();

			println!("{}", serde_json::to_string(&data).expect("Failed to serialize nodes"));
		} else {
			nodes.iter().for_each(|s| println!("{}: {}", s.id, s.name));
		}

		return;
	}
	
	if args.json {
		let data: Vec<_> = nodes.into_iter().map(|n| {
			let mut map = serde_json::Map::new();
			map.insert("id".to_string(), serde_json::Value::Number(serde_json::Number::from(n.id)));
			map.insert("name".to_string(), serde_json::Value::String(n.name));
			map.insert("status".to_string(), serde_json::Value::String(format!("{:?}", n.status)));
			map.insert("shape".to_string(), serde_json::Value::String(n.server_type.name));
			map
		}).collect();

		println!("{}", serde_json::to_string(&data).expect("Failed to serialize nodes"));
		return;
	} else {
		println!("{}", tabled::Table::new(nodes.into_iter().map(NodeTable::from)).with(tabled::settings::Style::markdown()));
	}
}

async fn shutdown(args: cmd::CmdShutdown) -> Result<(), Box<dyn std::error::Error>> {
	let Some(id) = parse_node_id(Cow::Borrowed(&args.node)).await? else {
		eprintln!("No node found with the name or ID '{}'", args.node);
		exit(1);
	};

	let action_id = match (args.force, args.restart) {
		(false, false) => servers_api::power_off_server(&CONFIG, servers_api::PowerOffServerParams { id }).await?.action.id,
		(true, false) => servers_api::shutdown_server(&CONFIG, servers_api::ShutdownServerParams { id }).await?.action.id,
		(false, true) => servers_api::soft_reboot_server(&CONFIG, servers_api::SoftRebootServerParams { id }).await?.action.id,
		(true, true) => servers_api::reset_server(&CONFIG, servers_api::ResetServerParams { id }).await?.action.id,
	};

	await_action(action_id).await?;

	Ok(())
}

async fn startup(args: cmd::CmdStartup) -> Result<(), Box<dyn std::error::Error>> {
	let Some(id) = parse_node_id(Cow::Borrowed(&args.node)).await? else {
		eprintln!("No node found with the name or ID '{}'", args.node);
		exit(1);
	};

	let action_id = servers_api::power_on_server(&CONFIG, servers_api::PowerOnServerParams { id }).await?.action.id;

	await_action(action_id).await?;

	Ok(())
}

async fn rescale(args: cmd::CmdRescale) -> Result<(), Box<dyn std::error::Error>> {
	let Some(node_id) = parse_node_id(Cow::Borrowed(&args.node)).await? else {
		eprintln!("No node found with the name or ID '{}'", args.node);
		exit(1);
	};
	let node = *servers_api::get_server(&CONFIG, servers_api::GetServerParams { id: node_id }).await?.server.unwrap();

	if let Some(target_shape) = args.shape {
		if node.status != hcloud::models::server::Status::Off {
			println!("Shutting down the server...");
			shutdown(cmd::CmdShutdown { node: node.id.to_string(), force: false, restart: false }).await?;

			// Check every 300ms for the server to be offline, timeout after 5 seconds.
			let mut timeout = 50;
			print!("Waiting for server to shut down");
			std::io::stdout().flush().unwrap();
			loop {
				let node = servers_api::get_server(&CONFIG, servers_api::GetServerParams { id: node_id }).await?.server.unwrap();
				if node.status == hcloud::models::server::Status::Off {
					break;
				}

				if timeout == 0 {
					eprintln!("Server did not shut down in time, aborting...");
					exit(1);
				}

				tokio::time::sleep(std::time::Duration::from_millis(100)).await;
				timeout -= 1;

				// Add a dot every 300ms
				if timeout % 3 == 0 {
					print!(".");
					std::io::stdout().flush().unwrap();
				}
			}

			println!("");
		}

		let action_id = servers_api::change_type_of_server(
			&CONFIG,
			servers_api::ChangeTypeOfServerParams {
				id: node_id,
				change_type_of_server_request: Some(hcloud::models::ChangeTypeOfServerRequest {
					server_type: target_shape.to_string(),
					upgrade_disk: false
				})
			}
		).await?.action.id;

		await_action(action_id).await?;
	} else {
		let shapes = apis::server_types_api::list_server_types(&CONFIG, Default::default()).await?.server_types
			.into_iter()
			.filter(|s| s.disk >= node.primary_disk_size as f64)
			.filter(|s| s.architecture == node.server_type.architecture)
			.map(|s| ShapeTable::new(s, &node.datacenter.location.name));

		println!("Shapes compatible with current shape: {}", node.server_type.name);
		println!("{}", tabled::Table::new(shapes).with(tabled::settings::Style::markdown()));
	}

	Ok(())
}

async fn parse_node_id<'a>(node: Cow<'a, String>) -> Result<Option<i64>, Box<dyn std::error::Error>> {
	match node.parse() {
		Ok(id) => Ok(Some(id)),
		Err(_) => {
			let nodes = servers_api::list_servers(&CONFIG, servers_api::ListServersParams { name: Some(node.into_owned()), ..Default::default() }).await?.servers;
			Ok(nodes.first().map(|n| n.id))
		}
	}
}

async fn await_action(id: i64) -> Result<(), Box<dyn std::error::Error>> {
	println!("");
	loop {
		#[allow(deprecated)] // It says it'll be removed in 2023...
		let action = apis::actions_api::get_action(&CONFIG, apis::actions_api::GetActionParams { id }).await?.action;

		match action.status {
			hcloud::models::action::Status::Running => {
				if action.progress <= 0 {
					print!("Waiting...\r");
				} else {
					print!("Progress... {:.2}%\r", action.progress);
				}
				std::io::stdout().flush().unwrap();
				tokio::time::sleep(std::time::Duration::from_millis(300)).await;
			},
			hcloud::models::action::Status::Success => {
				println!("Finished       ");
				break;
			},
			hcloud::models::action::Status::Error => {
				let err = action.error.unwrap();
				let (code, msg) = (err.code, err.message);
				println!("Encountered an error: {code}: {msg}");
				break;
			},
		}
	}

	Ok(())
}
