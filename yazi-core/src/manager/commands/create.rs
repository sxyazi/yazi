use std::path::PathBuf;

use tokio::fs::{self};
use yazi_shared::Url;

use crate::{emit, files::{File, FilesOp}, input::InputOpt, manager::Manager};

impl Manager {
	pub fn create(&self, force: bool) -> bool {
		let cwd = self.cwd().to_owned();
		tokio::spawn(async move {
			let mut result = emit!(Input(InputOpt::top("Create:")));
			let Some(Ok(name)) = result.recv().await else {
				return Ok(());
			};

			let path = cwd.join(&name);
			if !force && fs::symlink_metadata(&path).await.is_ok() {
				match emit!(Input(InputOpt::top("Overwrite an existing file? (y/N)"))).recv().await {
					Some(Ok(c)) if c == "y" || c == "Y" => (),
					_ => return Ok(()),
				}
			}

			if name.ends_with('/') {
				fs::create_dir_all(&path).await?;
			} else {
				fs::create_dir_all(&path.parent().unwrap()).await.ok();
				fs::File::create(&path).await?;
			}

			let child =
				Url::from(path.components().take(cwd.components().count() + 1).collect::<PathBuf>());
			if let Ok(f) = File::from(child.clone()).await {
				emit!(Files(FilesOp::Creating(cwd, f.into_map())));
				emit!(Hover(child));
				emit!(Refresh);
			}
			Ok::<(), anyhow::Error>(())
		});
		false
	}
}
