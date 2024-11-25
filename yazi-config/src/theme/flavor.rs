use std::{borrow::Cow, path::PathBuf, str::FromStr};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use yazi_shared::Xdg;

#[derive(Default, Deserialize, Serialize)]
pub struct Flavor {
	#[serde(default)]
	pub dark:  String,
	#[serde(default)]
	pub light: String,
}

impl FromStr for Flavor {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		#[derive(Deserialize)]
		struct Outer {
			#[serde(default)]
			flavor: Flavor,
		}

		Ok(
			toml::from_str::<Outer>(s)
				.context("Failed to parse the [flavor] section in your theme.toml")?
				.flavor,
		)
	}
}

impl Flavor {
	pub(crate) fn read(&self, light: bool) -> Result<Cow<'static, str>> {
		Ok(match if light { self.light.as_str() } else { self.dark.as_str() } {
			"" => Cow::Borrowed(""),
			name => {
				let p = Xdg::config_dir().join(format!("flavors/{name}.yazi/flavor.toml"));
				std::fs::read_to_string(&p).with_context(|| format!("Failed to load flavor {p:?}"))?.into()
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
