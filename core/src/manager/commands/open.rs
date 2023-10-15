use config::OPEN;
use shared::MIME_DIR;

use crate::{emit, external, manager::Manager, select::SelectOpt};

impl Manager {
	pub fn open(&mut self, interactive: bool) -> bool {
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

			if !interactive {
				emit!(Open(files, None));
				return;
			}

			let openers = OPEN.common_openers(&files);
			if openers.is_empty() {
				return;
			}

			let result = emit!(Select(SelectOpt::hovered(
				"Open with:",
				openers.iter().map(|o| o.display_name.clone()).collect()
			)));
			if let Ok(choice) = result.await {
				emit!(Open(files, Some(openers[choice].clone())));
			}
		});
		false
	}
}
