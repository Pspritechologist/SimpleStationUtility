use std::{ops::{Deref, DerefMut}, sync::OnceLock};

#[derive(Debug, Clone)]
pub struct Configuration(hcloud::apis::configuration::Configuration);

impl Deref for Configuration {
	type Target = hcloud::apis::configuration::Configuration;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for Configuration {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
impl Default for Configuration {
	fn default() -> Self {
		Self(Default::default())
	}
}
impl Configuration {
	pub fn with_access_token(mut self, token: String) -> Self {
		self.bearer_access_token = Some(token);
		self
	}
}

pub struct ConfigCell(OnceLock<Configuration>);
impl Deref for ConfigCell {
	type Target = Configuration;
	fn deref(&self) -> &Self::Target {
		self.0.get().unwrap()
	}
}
impl DerefMut for ConfigCell {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0.get_mut().unwrap()
	}
}
impl Default for ConfigCell {
	fn default() -> Self {
		Self(OnceLock::new())
	}
}
impl ConfigCell {
	pub const fn new() -> Self {
		Self(OnceLock::new())
	}

	pub fn set(&self, config: Configuration) {
		self.0.set(config).unwrap();
	}
}

#[derive(Debug, Clone, tabled::Tabled)]
pub struct NodeTable {
	name: String,
	id: i64,
	status: String,
	shape: String,
}

impl From<hcloud::models::Server> for NodeTable {
	fn from(value: hcloud::models::Server) -> Self {
		Self {
			name: value.name,
			id: value.id,
			status: format!("{:?}", value.status),
			shape: value.server_type.name,
		}
	}
}

#[derive(Debug, Clone, tabled::Tabled)]
pub struct ShapeTable {
	name: String,
	id: i64,
	cores: i32,
	memory: f64,
	disk: f64,
	price: String,
}

impl ShapeTable {
	pub fn new(value: hcloud::models::ServerType, loc: &str) -> Self {
		let price = value.prices
			.into_iter()
			.find(|p| p.location == loc)
			.map(|p| format!("€{:.2}", p.price_monthly.gross.parse::<f64>().expect("Failed to parse price as float")))
			.unwrap_or("N/A".into());
		
		
		Self {
			name: value.name,
			id: value.id,
			cores: value.cores,
			memory: value.memory,
			disk: value.disk,
			price,
		}
	}
}
