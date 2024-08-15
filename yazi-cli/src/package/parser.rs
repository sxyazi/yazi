use anyhow::{bail, Context, Result};
use tokio::fs;
use toml_edit::{Array, DocumentMut, InlineTable, Item, Value};
use yazi_shared::Xdg;

use super::Package;

impl Package {
	// TODO: remove this in the future
	pub(crate) async fn migrate() -> Result<()> {
		let path = Xdg::config_dir().join("package.toml");
		let mut doc = Self::ensure_config(&fs::read_to_string(&path).await.unwrap_or_default())?;

		fn impl_(deps: &mut Array) -> Result<()> {
			for dep in deps.iter_mut() {
				let dep = dep.as_inline_table_mut().context("Dependency must be an inline table")?;
				let use_ = dep.get("use").and_then(|d| d.as_str()).context("Missing `use` field")?;
				if use_.contains("#") {
					dep["use"] = use_.replace("#", ":").into();
				}
				if let Some(commit) = dep.get("commit").map(ToOwned::to_owned) {
					dep.remove("commit");
					dep.insert("rev", commit);
				}
			}
			Ok(())
		}

		impl_(doc["plugin"]["deps"].as_array_mut().unwrap())?;
		impl_(doc["flavor"]["deps"].as_array_mut().unwrap())?;

		fs::write(path, doc.to_string()).await?;
		Ok(())
	}

	pub(crate) async fn add_to_config(use_: &str) -> Result<()> {
		let mut package = Self::new(use_, None);
		let Some(name) = package.name() else { bail!("Invalid package `use`") };

		let path = Xdg::config_dir().join("package.toml");
		let mut doc = Self::ensure_config(&fs::read_to_string(&path).await.unwrap_or_default())?;

		Self::ensure_unique(&doc, name)?;
		package.add().await?;

		let mut table = InlineTable::new();
		table.insert("use", package.use_().as_ref().into());
		if !package.rev.is_empty() {
			table.insert("rev", package.rev.into());
		}

		if package.is_flavor {
			doc["flavor"]["deps"].as_array_mut().unwrap().push(table);
		} else {
			doc["plugin"]["deps"].as_array_mut().unwrap().push(table);
		}

		fs::write(path, doc.to_string()).await?;
		Ok(())
	}

	pub(crate) async fn install_from_config(section: &str, upgrade: bool) -> Result<()> {
		let path = Xdg::config_dir().join("package.toml");
		let Ok(s) = fs::read_to_string(&path).await else {
			return Ok(());
		};

		let mut doc = s.parse::<DocumentMut>().context("Failed to parse package.toml")?;
		let Some(deps) = doc.get_mut(section).and_then(|d| d.get_mut("deps")) else {
			return Ok(());
		};

		let deps = deps.as_array_mut().context("`deps` must be an array")?;
		for dep in deps.iter_mut() {
			let dep = dep.as_inline_table_mut().context("Dependency must be an inline table")?;
			let use_ = dep.get("use").and_then(|d| d.as_str()).context("Missing `use` field")?;
			let rev = dep.get("rev").and_then(|d| d.as_str());

			let mut package = Package::new(use_, rev);
			if upgrade {
				package.upgrade().await?;
			} else {
				package.install().await?;
			}

			if package.rev.is_empty() {
				dep.remove("rev");
			} else {
				dep.insert("rev", package.rev.into());
			}
		}

		fs::write(path, doc.to_string()).await.context("Failed to write package.toml")
	}

	pub(crate) async fn list_from_config(section: &str) -> Result<()> {
		let path = Xdg::config_dir().join("package.toml");
		let Ok(s) = fs::read_to_string(&path).await else {
			return Ok(());
		};

		let doc = s.parse::<DocumentMut>().context("Failed to parse package.toml")?;
		let Some(deps) = doc.get(section).and_then(|d| d.get("deps")) else {
			return Ok(());
		};

		let deps = deps.as_array().context("`deps` must be an array")?;
		println!("{section}s:");

		for dep in deps {
			if let Some(Value::String(use_)) = dep.as_inline_table().and_then(|t| t.get("use")) {
				println!("\t{}", use_.value());
			}
		}
		Ok(())
	}

	fn ensure_config(s: &str) -> Result<DocumentMut> {
		let mut doc = s.parse::<DocumentMut>().context("Failed to parse package.toml")?;

		doc
			.entry("plugin")
			.or_insert(toml_edit::table())
			.as_table_mut()
			.context("Failed to get `plugin` table")?
			.entry("deps")
			.or_insert(Item::Value(Array::new().into()))
			.as_array()
			.context("Failed to get `deps` array")?;

		doc
			.entry("flavor")
			.or_insert(toml_edit::table())
			.as_table_mut()
			.context("Failed to get `flavor` table")?
			.entry("deps")
			.or_insert(Item::Value(Array::new().into()))
			.as_array()
			.context("Failed to get `deps` array")?;

		Ok(doc)
	}

	fn ensure_unique(doc: &DocumentMut, name: &str) -> Result<()> {
		#[inline]
		fn same(v: &Value, name: &str) -> bool {
			v.as_inline_table()
				.and_then(|t| t.get("use"))
				.and_then(|v| v.as_str())
				.is_some_and(|s| Package::new(s, None).name() == Some(name))
		}

		if doc["plugin"]["deps"].as_array().unwrap().into_iter().any(|v| same(v, name)) {
			bail!("Plugin `{name}` already exists in package.toml");
		}
		if doc["flavor"]["deps"].as_array().unwrap().into_iter().any(|v| same(v, name)) {
			bail!("Flavor `{name}` already exists in package.toml");
		}

		Ok(())
	}
}
