use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use toml::Value;
use yazi_codegen::DeserializeOver2;
use yazi_fs::Xdg;

#[derive(Default, Deserialize, DeserializeOver2, Serialize)]
pub struct Flavor {
	pub dark:  String,
	pub light: String,
}

impl From<&Value> for Flavor {
	fn from(value: &Value) -> Self {
		let mut me = Self::default();
		if let Value::Table(t) = value {
			if let Some(s) = t.get("dark").and_then(|v| v.as_str()) {
				me.dark = s.to_owned();
			}
			if let Some(s) = t.get("light").and_then(|v| v.as_str()) {
				me.light = s.to_owned();
			}
		}
		me
	}
}

impl Flavor {
	pub(crate) fn read(&self, light: bool) -> Result<String> {
		Ok(match if light { self.light.as_str() } else { self.dark.as_str() } {
			"" => String::new(),
			name => {
				let p = Xdg::config_dir().join(format!("flavors/{name}.yazi/flavor.toml"));
				std::fs::read_to_string(&p).with_context(|| format!("Failed to load flavor {p:?}"))?
			}
		})
	}

	pub(crate) fn syntect_path(&self, light: bool) -> Option<PathBuf> {
		match if light { self.light.as_str() } else { self.dark.as_str() } {
			"" => None,
			name => Some(Xdg::config_dir().join(format!("flavors/{name}.yazi/tmtheme.xml"))),
		}
	}
}
