use anyhow::{bail, Context, Result};
use tokio::fs;
use yazi_shared::Xdg;

use super::{config::{GitDependency, PackageConfig}, Package};

pub enum InstallFromConfig {
	Plugin,
	Flavor,
}

impl Package {
	pub(crate) async fn add_to_config(use_: &str) -> Result<()> {
		let mut package = Self::new(use_, None);
		if package.name().is_none() {
			bail!("Invalid package `use`: '{}'", use_);
		};

		let path = Xdg::config_dir().join("package.toml");
		let mut config = parse_config(&fs::read_to_string(&path).await.unwrap_or_default())?;

		ensure_unique(&config, &package)?;
		package.add().await?;

		let dep =
			GitDependency { use_: package.use_().as_ref().into(), commit: package.commit.into() };
		if package.is_flavor {
			config.flavor.deps.push(dep);
		} else {
			config.plugin.deps.push(dep);
		}

		fs::write(path, toml::to_string_pretty(&config)?).await.context("Failed to write package.toml")
	}

	pub(crate) async fn install_from_config(
		section: &InstallFromConfig,
		upgrade: bool,
	) -> Result<()> {
		let path = Xdg::config_dir().join("package.toml");
		let Ok(s) = fs::read_to_string(&path).await else {
			return Ok(());
		};

		let mut config = parse_config(&s)?;
		let deps = match section {
			InstallFromConfig::Plugin => &mut config.plugin.deps,
			InstallFromConfig::Flavor => &mut config.flavor.deps,
		};

		for dep in deps.iter_mut() {
			let mut package = Package::new(&dep.use_, dep.commit.as_deref());
			if upgrade {
				package.upgrade().await?;
			} else {
				package.install().await?;
			}

			if package.commit.is_empty() {
				dep.commit.take();
			} else {
				dep.commit = Some(package.commit);
			}
		}

		fs::write(path, toml::to_string_pretty(&config)?).await.context("Failed to write package.toml")
	}
}

fn parse_config(s: &str) -> Result<PackageConfig, anyhow::Error> {
	toml::from_str::<PackageConfig>(s).context("Failed to parse package.toml")
}

fn ensure_unique(doc: &PackageConfig, dep: &Package) -> Result<()> {
	let use_ = dep.use_().as_ref().to_owned();

	if doc.plugin.deps.iter().any(|v| v.use_ == use_) {
		bail!("Plugin `{use_}` already exists in package.toml");
	}
	if doc.flavor.deps.iter().any(|v| v.use_ == use_) {
		bail!("Flavor `{use_}` already exists in package.toml");
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::package::config::{FlavorConfig, GitDependency, PluginConfig};

	#[test]
	fn test_disallow_duplicate() {
		let config = PackageConfig::default();
		let package = Package::new("test", None);

		assert!(ensure_unique(&config, &package).is_ok());

		let config = PackageConfig {
			plugin: PluginConfig { deps: vec![GitDependency { use_: "test".into(), commit: None }] },
			flavor: FlavorConfig { deps: vec![] },
		};

		assert!(ensure_unique(&config, &package).is_err());

		let config = PackageConfig {
			plugin: PluginConfig { deps: vec![] },
			flavor: FlavorConfig { deps: vec![GitDependency { use_: "test".into(), commit: None }] },
		};

		assert!(ensure_unique(&config, &package).is_err());
	}
}
