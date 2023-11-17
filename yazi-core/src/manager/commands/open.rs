use std::ffi::OsString;

use yazi_config::{keymap::Exec, popup::SelectOpt, OPEN};
use yazi_shared::MIME_DIR;

use crate::{emit, external, manager::Manager};

pub struct Opt {
	interactive: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { interactive: e.named.contains_key("interactive") } }
}

impl Manager {
	async fn open_interactive(files: Vec<(OsString, String)>) {
		let openers = OPEN.common_openers(&files);
		if openers.is_empty() {
			return;
		}

		let result = emit!(Select(SelectOpt::open(openers.iter().map(|o| o.desc.clone()).collect())));
		if let Ok(choice) = result.await {
			emit!(Open(files, Some(openers[choice].clone())));
		}
	}

	pub fn open(&mut self, opt: impl Into<Opt>) -> bool {
		let mut files: Vec<_> = self
			.selected()
			.into_iter()
			.map(|f| {
				(
					f.url(),
					f.is_dir().then(|| MIME_DIR.to_owned()).or_else(|| self.mimetype.get(&f.url).cloned()),
				)
			})
			.collect();

		if files.is_empty() {
			return false;
		}

		let opt = opt.into() as Opt;
		tokio::spawn(async move {
			let todo: Vec<_> = files.iter().filter(|(_, m)| m.is_none()).map(|(u, _)| u).collect();
			if let Ok(mut mimes) = external::file(&todo).await {
				files = files
					.into_iter()
					.map(|(u, m)| {
						let mime = m.or_else(|| mimes.remove(&u));
						(u, mime)
					})
					.collect();
			}

			let files: Vec<_> =
				files.into_iter().filter_map(|(u, m)| m.map(|m| (u.into_os_string(), m))).collect();

			if opt.interactive {
				Self::open_interactive(files).await;
				return;
			}

			emit!(Open(files, None));
		});
		false
	}
}
