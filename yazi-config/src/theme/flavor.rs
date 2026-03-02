use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize, de::IntoDeserializer};
use toml::{Spanned, de::DeTable};
use yazi_codegen::DeserializeOver2;
use yazi_fs::Xdg;

use crate::error_with_input;

#[derive(Default, Deserialize, DeserializeOver2, Serialize)]
pub struct Flavor {
	#[serde(default)]
	pub dark:  String,
	#[serde(default)]
	pub light: String,
}

impl Flavor {
	pub(crate) fn from_theme(theme: &Spanned<DeTable>, input: &str) -> Result<Self, toml::de::Error> {
		if let Some(value) = theme.get_ref().get("flavor").cloned() {
			error_with_input(Self::deserialize(value.into_deserializer()), input)
		} else {
			Ok(Self::default())
		}
	}

	pub(crate) fn read(&self, light: bool) -> Result<String> {
		Ok(match if light { self.light.as_str() } else { self.dark.as_str() } {
			"" => String::new(),
			name => {
				let p = Xdg::config_dir().join(format!("flavors/{name}.yazi/flavor.toml"));
				std::fs::read_to_string(&p).with_context(|| format!("Failed to read flavor {p:?}"))?
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
