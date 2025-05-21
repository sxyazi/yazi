use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tokio::fs;
use yazi_fs::Xdg;
use yazi_macro::outln;

use super::Dependency;

#[derive(Default)]
pub(crate) struct Package {
	pub(crate) plugins: Vec<Dependency>,
	pub(crate) flavors: Vec<Dependency>,
}

impl Package {
	pub(crate) async fn load() -> Result<Self> {
		Ok(match fs::read_to_string(Self::toml()).await {
			Ok(s) => toml::from_str(&s)?,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => Self::default(),
			Err(e) => Err(e)?,
		})
	}

	pub(crate) async fn add_many(&mut self, uses: &[String]) -> Result<()> {
		for u in uses {
			let r = self.add(u).await;
			self.save().await?;
			r?;
		}
		Ok(())
	}

	pub(crate) async fn delete_many(&mut self, uses: &[String]) -> Result<()> {
		for u in uses {
			let r = self.delete(u).await;
			self.save().await?;
			r?;
		}
		Ok(())
	}

	pub(crate) async fn install(&mut self, upgrade: bool) -> Result<()> {
		for i in 0..self.plugins.len() {
			let r =
				if upgrade { self.plugins[i].upgrade().await } else { self.plugins[i].install().await };
			self.save().await?;
			r?;
		}
		for i in 0..self.flavors.len() {
			let r =
				if upgrade { self.flavors[i].upgrade().await } else { self.flavors[i].install().await };
			self.save().await?;
			r?;
		}
		Ok(())
	}

	pub(crate) fn print(&self) -> Result<()> {
		outln!("Plugins:")?;
		for d in &self.plugins {
			if d.rev.is_empty() {
				outln!("\t{}", d.r#use)?;
			} else {
				outln!("\t{} ({})", d.r#use, d.rev)?;
			}
		}

		outln!("Flavors:")?;
		for d in &self.flavors {
			if d.rev.is_empty() {
				outln!("\t{}", d.r#use)?;
			} else {
				outln!("\t{} ({})", d.r#use, d.rev)?;
			}
		}

		Ok(())
	}

	async fn add(&mut self, r#use: &str) -> Result<()> {
		let mut dep = Dependency::from_str(r#use)?;
		if let Some(d) = self.identical(&dep) {
			bail!(
				"{} `{}` already exists in package.toml",
				if d.is_flavor { "Flavor" } else { "Plugin" },
				dep.name
			)
		}

		dep.add().await?;
		if dep.is_flavor {
			self.flavors.push(dep);
		} else {
			self.plugins.push(dep);
		}
		Ok(())
	}

	async fn delete(&mut self, r#use: &str) -> Result<()> {
		let Some(dep) = self.identical(&Dependency::from_str(r#use)?).cloned() else {
			bail!("`{}` was not found in package.toml", r#use)
		};

		dep.delete().await?;
		if dep.is_flavor {
			self.flavors.retain(|d| !d.identical(&dep));
		} else {
			self.plugins.retain(|d| !d.identical(&dep));
		}
		Ok(())
	}

	async fn save(&self) -> Result<()> {
		let s = toml::to_string_pretty(self)?;
		fs::write(Self::toml(), s).await.context("Failed to write package.toml")
	}

	#[inline]
	fn toml() -> PathBuf { Xdg::config_dir().join("package.toml") }

	#[inline]
	fn identical(&self, other: &Dependency) -> Option<&Dependency> {
		self.plugins.iter().chain(&self.flavors).find(|d| d.identical(other))
	}
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

		let mut outer = Outer::deserialize(deserializer)?;
		outer.flavor.deps.iter_mut().for_each(|d| d.is_flavor = true);

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
