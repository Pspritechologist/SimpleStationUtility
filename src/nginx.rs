use std::collections::HashMap;

use crate::cmd::CmdNginxConfig;

macro_rules! config_block {
	($name:expr, $port:expr, $node_name:expr) => {
		format!(
			r#"
server {{
	listen 80;
	listen 443 ssl;
	server_name {NAME}.node-{NODE}.simplestation.org;

	location / {{
		proxy_pass http://5.161.216.140:{PORT};
		include /etc/nginx/proxy_params;
		add_header 'Access-Control-Allow-Origin' '*';
}}

	ssl_certificate /etc/letsencrypt/live/{NAME}.node-{NODE}.simplestation.org/fullchain.pem; # managed by Certbot
	ssl_certificate_key /etc/letsencrypt/live/{NAME}.node-{NODE}.simplestation.org/privkey.pem; # managed by Certbot
}}
			"#,
			NAME = $name, PORT = $port, NODE = $node_name
		)
	};
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Config {
	node: String,
	servers: HashMap<String, u16>,
}

pub fn generate_servers(args: CmdNginxConfig) -> Result<(), Box<dyn std::error::Error>> {
	let config: Config = toml::from_str(&std::fs::read_to_string(args.config)?)?;

	let node = config.node;

	let mut output = String::new();
	for (name, port) in config.servers {
		output.push_str(&config_block!(name, port, node));
	}

	if let Some(output_file) = args.output {
		std::fs::write(&output_file, output)?;
	} else {
		println!("{}", output);
	}

	Ok(())
}
