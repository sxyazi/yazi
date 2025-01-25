use std::{path::{Path, PathBuf}, str::FromStr};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tokio::fs;
use yazi_fs::{Xdg, ok_or_not_found, unique_name};
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

	pub(crate) async fn add(&mut self, use_: &str) -> Result<()> {
		let mut dep = Dependency::from_str(use_)?;
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

		let s = toml::to_string_pretty(self)?;
		fs::write(Self::toml(), s).await.context("Failed to write package.toml")
	}

	pub(crate) async fn delete(&mut self, use_: &str) -> Result<()> {
		let Some(dep) = self.identical(&Dependency::from_str(use_)?).cloned() else {
			bail!("`{}` was not found in package.toml", use_)
		};

		dep.delete().await?;
		if dep.is_flavor {
			self.flavors.retain(|d| !d.identical(&dep));
		} else {
			self.plugins.retain(|d| !d.identical(&dep));
		}

		let s = toml::to_string_pretty(self)?;
		fs::write(Self::toml(), s).await.context("Failed to write package.toml")
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
		fs::write(Self::toml(), s).await.context("Failed to write package.toml")
	}

	pub(crate) fn print(&self) -> Result<()> {
		outln!("Plugins:")?;
		for d in &self.plugins {
			if d.rev.is_empty() {
				outln!("\t{}", d.use_)?;
			} else {
				outln!("\t{} ({})", d.use_, d.rev)?;
			}
		}

		outln!("Flavors:")?;
		for d in &self.flavors {
			if d.rev.is_empty() {
				outln!("\t{}", d.use_)?;
			} else {
				outln!("\t{} ({})", d.use_, d.rev)?;
			}
		}

		Ok(())
	}

	// TODO: remove this
	pub(crate) async fn sync(&mut self) -> Result<()> {
		async fn make_readonly(p: &Path) -> Result<()> {
			let mut perms = fs::metadata(p).await?.permissions();
			perms.set_readonly(true);
			fs::set_permissions(p, perms).await?;
			Ok(())
		}

		match fs::read_dir(Xdg::config_dir().join("plugins")).await {
			Ok(mut it) => {
				while let Some(entry) = it.next_entry().await? {
					let dir = entry.path();
					if !dir.is_dir() || dir.extension().is_none_or(|s| s != "yazi") {
						continue;
					}

					match fs::symlink_metadata(dir.join("init.lua")).await {
						Ok(_) => {}
						Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
						Err(e) => Err(e)?,
					}

					ok_or_not_found(
						fs::rename(
							dir.join("main.lua"),
							unique_name(dir.join("main.lua.bak").into(), async { false }).await?,
						)
						.await,
					)?;

					ok_or_not_found(fs::rename(dir.join("init.lua"), dir.join("main.lua")).await)?;
				}
			}
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
			Err(e) => Err(e)
				.context(format!("failed to read `{}`", Xdg::config_dir().join("plugins").display()))?,
		}

		for d in &mut self.plugins {
			let dir = Xdg::config_dir().join(format!("plugins/{}", d.name));
			for f in ["LICENSE", "README.md", "main.lua"] {
				make_readonly(&dir.join(f)).await.ok();
			}

			let tracker = dir.join("DO_NOT_MODIFY_ANYTHING_IN_THIS_DIRECTORY");
			if fs::read(&tracker).await.is_ok_and(|v| v.is_empty()) {
				if d.hash.is_empty() {
					d.hash = d.hash().await?;
				}
				fs::remove_file(&tracker).await.ok();
			}
		}
		for d in &mut self.flavors {
			let dir = Xdg::config_dir().join(format!("flavors/{}", d.name));
			for f in [
				"LICENSE",
				"LICENSE-tmtheme",
				"README.md",
				"filestyle.toml",
				"flavor.toml",
				"preview.png",
				"tmtheme.xml",
			] {
				make_readonly(&dir.join(f)).await.ok();
			}

			let tracker = dir.join("DO_NOT_MODIFY_ANYTHING_IN_THIS_DIRECTORY");
			if fs::read(&tracker).await.is_ok_and(|v| v.is_empty()) {
				if d.hash.is_empty() {
					d.hash = d.hash().await?;
				}
				fs::remove_file(&tracker).await.ok();
			}
		}

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
