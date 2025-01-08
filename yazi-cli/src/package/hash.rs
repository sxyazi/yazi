use anyhow::{Context, Result};
use tokio::fs;
use twox_hash::XxHash3_128;
use yazi_fs::{Xdg, ok_or_not_found};

use super::Dependency;

impl Dependency {
	pub(crate) async fn hash(&self) -> Result<String> {
		let dir = if self.is_flavor {
			Xdg::config_dir().join(format!("flavors/{}", self.name))
		} else {
			Xdg::config_dir().join(format!("plugins/{}", self.name))
		};

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

		let mut hasher = XxHash3_128::new();
		for file in files {
			hasher.write(file.as_bytes());
			hasher.write(b"VpvFw9Atb7cWGOdqhZCra634CcJJRlsRl72RbZeV0vpG1\0");
			hasher.write(&ok_or_not_found(fs::read(dir.join(file)).await)?);
		}

		let mut assets = vec![];
		match fs::read_dir(dir.join("assets")).await {
			Ok(mut it) => {
				while let Some(entry) = it.next_entry().await? {
					assets.push((entry.file_name(), fs::read(entry.path()).await?));
				}
			}
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
			Err(e) => Err(e).context(format!("failed to read `{}`", dir.join("assets").display()))?,
		}

		assets.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
		for (name, data) in assets {
			hasher.write(name.as_encoded_bytes());
			hasher.write(b"pQU2in0xcsu97Y77Nuq2LnT8mczMlFj22idcYRmMrglqU\0");
			hasher.write(&data);
		}

		Ok(format!("{:x}", hasher.finish_128()))
	}
}
