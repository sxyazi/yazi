use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tokio::fs;
use yazi_fs::Xdg;
use yazi_macro::outln;

use super::Dependency;

#[derive(Default)]
pub(crate) struct Package {
	plugins: Vec<Dependency>,
	flavors: Vec<Dependency>,
}

impl Package {
	pub(crate) async fn load() -> Result<Self> {
		Ok(match fs::read_to_string(Self::path()).await {
			Ok(s) => toml::from_str(&s)?,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => Self::default(),
			Err(e) => Err(e)?,
		})
	}

	pub(crate) async fn add(&mut self, use_: &str) -> Result<()> {
		let mut dep = Dependency::new(use_, None);
		let Some(name) = dep.name() else { bail!("Invalid package `use`") };

		if self.plugins.iter().any(|d| d.repo == dep.repo && d.child == dep.child) {
			bail!("Plugin `{name}` already exists in package.toml");
		}
		if self.flavors.iter().any(|d| d.repo == dep.repo && d.child == dep.child) {
			bail!("Flavor `{name}` already exists in package.toml");
		}

		dep.add().await?;
		if dep.is_flavor {
			self.flavors.push(dep);
		} else {
			self.plugins.push(dep);
		}

		Ok(fs::write(Self::path(), toml::to_string_pretty(self)?).await?)
	}

	pub(crate) async fn install(&mut self, upgrade: bool) -> Result<()> {
		for d in &mut self.plugins {
			if upgrade {
				d.upgrade().await?;
			} else {
				d.install().await?;
			}
		}
		for d in &mut self.flavors {
			if upgrade {
				d.upgrade().await?;
			} else {
				d.install().await?;
			}
		}

		let s = toml::to_string_pretty(self)?;
		fs::write(Self::path(), s).await.context("Failed to write package.toml")
	}

	pub(crate) fn print(&self) -> Result<()> {
		outln!("Plugins:")?;
		for d in &self.plugins {
			if d.rev.is_empty() {
				outln!("\t{}", d.use_())?;
			} else {
				outln!("\t{} ({})", d.use_(), d.rev)?;
			}
		}

		outln!("Flavors:")?;
		for d in &self.flavors {
			if d.rev.is_empty() {
				outln!("\t{}", d.use_())?;
			} else {
				outln!("\t{} ({})", d.use_(), d.rev)?;
			}
		}

		Ok(())
	}

	#[inline]
	fn path() -> PathBuf { Xdg::config_dir().join("package.toml") }
}

impl<'de> Deserialize<'de> for Package {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Outer {
			#[serde(default)]
			plugin: Shadow,
			#[serde(default)]
			flavor: Shadow,
		}
		#[derive(Default, Deserialize)]
		struct Shadow {
			deps: Vec<Dependency>,
		}

		let outer = Outer::deserialize(deserializer)?;
		Ok(Self { plugins: outer.plugin.deps, flavors: outer.flavor.deps })
	}
}

impl Serialize for Package {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		#[derive(Serialize)]
		struct Outer<'a> {
			plugin: Shadow<'a>,
			flavor: Shadow<'a>,
		}
		#[derive(Serialize)]
		struct Shadow<'a> {
			deps: &'a [Dependency],
		}

		Outer { plugin: Shadow { deps: &self.plugins }, flavor: Shadow { deps: &self.flavors } }
			.serialize(serializer)
	}
}
