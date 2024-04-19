use std::path::PathBuf;

use tokio::fs;
use yazi_config::popup::InputCfg;
use yazi_proxy::{InputProxy, ManagerProxy};
use yazi_shared::{event::Cmd, fs::{accessible, File, FilesOp, Url}};

use crate::manager::Manager;

pub struct Opt {
	force: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { force: c.bool("force") } }
}

impl Manager {
	pub fn create(&self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		let cwd = self.cwd().to_owned();
		tokio::spawn(async move {
			let mut result = InputProxy::show(InputCfg::create());
			let Some(Ok(name)) = result.recv().await else {
				return Ok(());
			};

			let path = cwd.join(&name);
			if !opt.force && accessible(&path).await {
				match InputProxy::show(InputCfg::overwrite()).recv().await {
					Some(Ok(c)) if c == "y" || c == "Y" => (),
					_ => return Ok(()),
				}
			}

			if name.ends_with('/') || name.ends_with('\\') {
				fs::create_dir_all(&path).await?;
			} else {
				fs::create_dir_all(&path.parent().unwrap()).await.ok();
				fs::File::create(&path).await?;
			}

			let child =
				Url::from(path.components().take(cwd.components().count() + 1).collect::<PathBuf>());
			if let Ok(f) = File::from(child.clone()).await {
				FilesOp::Creating(cwd, vec![f]).emit();
				ManagerProxy::hover(Some(child));
			}
			Ok::<(), anyhow::Error>(())
		});
	}
}
