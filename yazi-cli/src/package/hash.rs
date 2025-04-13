use anyhow::{Context, Result, bail};
use tokio::fs;
use twox_hash::XxHash3_128;
use yazi_fs::ok_or_not_found;

use super::Dependency;

impl Dependency {
	pub(crate) async fn hash(&self) -> Result<String> {
		let dir = self.target();
		let files = if self.is_flavor {
			&[
				"LICENSE",
				"LICENSE-tmtheme",
				"README.md",
				"filestyle.toml",
				"flavor.toml",
				"preview.png",
				"tmtheme.xml",
			][..]
		} else {
			&["LICENSE", "README.md", "main.lua"][..]
		};

		let mut h = XxHash3_128::new();
		for &file in files {
			h.write(file.as_bytes());
			h.write(b"VpvFw9Atb7cWGOdqhZCra634CcJJRlsRl72RbZeV0vpG1\0");
			h.write(&ok_or_not_found(fs::read(dir.join(file)).await)?);
		}

		let mut assets = vec![];
		match fs::read_dir(dir.join("assets")).await {
			Ok(mut it) => {
				while let Some(entry) = it.next_entry().await? {
					let Ok(name) = entry.file_name().into_string() else {
						bail!("asset path is not valid UTF-8: {}", entry.path().display());
					};
					assets.push((name, fs::read(entry.path()).await?));
				}
			}
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
			Err(e) => Err(e).context(format!("failed to read `{}`", dir.join("assets").display()))?,
		}

		assets.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
		for (name, data) in assets {
			h.write(name.as_bytes());
			h.write(b"pQU2in0xcsu97Y77Nuq2LnT8mczMlFj22idcYRmMrglqU\0");
			h.write(&data);
		}

		Ok(format!("{:x}", h.finish_128()))
	}

	pub(super) async fn hash_check(&self) -> Result<()> {
		if self.hash != self.hash().await? {
			bail!(
				"You have modified the contents of the `{}` {}. For safety, the operation has been aborted.
Please manually delete it from `{}` and re-run the command.",
				self.name,
				if self.is_flavor { "flavor" } else { "plugin" },
				self.target().display()
			);
		}
		Ok(())
	}
}
