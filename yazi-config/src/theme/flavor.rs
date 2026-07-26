use std::path::PathBuf;

use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use toml::{Spanned, de::DeTable};
use yazi_codegen::{DeserializeOver, DeserializeOver2, Overlay};
use yazi_fs::Xdg;
use yazi_shim::toml::deserialize_spanned;

use crate::error_with_input;

#[derive(Default, Deserialize, DeserializeOver, DeserializeOver2, Overlay, Serialize)]
pub struct Flavor {
	#[serde(default)]
	pub dark:  ArcSwap<String>,
	#[serde(default)]
	pub light: ArcSwap<String>,
}

impl Flavor {
	pub(crate) fn from_theme(theme: &Spanned<DeTable>, input: &str) -> Result<Self, toml::de::Error> {
		if let Some(value) = theme.get_ref().get("flavor").cloned() {
			error_with_input(deserialize_spanned(value), input)
		} else {
			Ok(Self::default())
		}
	}

	pub(crate) fn read(&self, light: bool) -> Result<String> {
		Ok(match if light { self.light.load() } else { self.dark.load() }.as_str() {
			"" => String::new(),
			name => {
				let p = Xdg::config_dir().join(format!("flavors/{name}.yazi/flavor.toml"));
				std::fs::read_to_string(&p).with_context(|| format!("Failed to read flavor {p:?}"))?
			}
		})
	}

	pub(crate) fn syntect_path(&self, light: bool) -> Option<PathBuf> {
		match if light { self.light.load() } else { self.dark.load() }.as_str() {
			"" => None,
			name => Some(Xdg::config_dir().join(format!("flavors/{name}.yazi/tmtheme.xml"))),
		}
	}
}
