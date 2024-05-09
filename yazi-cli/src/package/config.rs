use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageConfig {
	pub plugin: PluginConfig,
	pub flavor: FlavorConfig,
}

impl Default for PackageConfig {
	fn default() -> Self {
		Self { plugin: PluginConfig { deps: vec![] }, flavor: FlavorConfig { deps: vec![] } }
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PluginConfig {
	pub deps: Vec<GitDependency>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlavorConfig {
	pub deps: Vec<GitDependency>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitDependency {
	#[serde(rename = "use")]
	pub use_:   String,
	pub commit: Option<String>,
}
