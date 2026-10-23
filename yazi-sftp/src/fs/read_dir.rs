use std::{sync::Arc, time::Duration, vec};

use crate::{Error, Operator, Session, SftpPath, fs::DirEntry, requests, responses};

pub struct ReadDir {
	session: Arc<Session>,
	dir:     Arc<typed_path::UnixPathBuf>,
	handle:  String,

	items: vec::IntoIter<responses::NameItem<'static>>,
	done:  bool,
}

impl Drop for ReadDir {
	fn drop(&mut self) { Operator::from(&self.session).close(&self.handle).ok(); }
}

impl ReadDir {
	pub(crate) fn new(session: &Arc<Session>, dir: SftpPath, handle: String) -> Self {
		Self {
			session: session.clone(),
			dir: Arc::new(dir.into_owned()),
			handle,

			items: Default::default(),
			done: false,
		}
	}

	pub async fn next(&mut self) -> Result<Option<DirEntry>, Error> {
		loop {
			self.fetch().await?;
			let Some(item) = self.items.next() else {
				return Ok(None);
			};

			if &*item.name != b"." && &*item.name != b".." {
				return Ok(Some(DirEntry {
					dir:       self.dir.clone(),
					name:      item.name.into_owned(),
					long_name: item.long_name.into_owned(),
					attrs:     item.attrs,
				}));
			}
		}
	}

	async fn fetch(&mut self) -> Result<(), Error> {
		if !self.items.as_slice().is_empty() || self.done {
			return Ok(());
		}

		let result = self
			.session
			.send_with_timeout(requests::ReadDir::new(&self.handle), Duration::from_mins(45))
			.await;

		let name: responses::Name = match result {
			Ok(resp) => resp,
			Err(Error::Status(status)) if status.is_eof() => {
				self.done = true;
				return Ok(());
			}
			Err(e) => return Err(e),
		};

		self.items = name.items.into_iter();
		Ok(())
	}
}
