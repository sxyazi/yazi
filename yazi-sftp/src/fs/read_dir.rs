use std::{mem, sync::Arc, time::Duration};

use crate::{Error, Operator, Session, SftpPath, fs::DirEntry, requests, responses};

pub struct ReadDir {
	session: Arc<Session>,
	dir:     Arc<typed_path::UnixPathBuf>,
	handle:  String,

	name:   responses::Name<'static>,
	cursor: usize,
	done:   bool,
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

			name: Default::default(),
			cursor: 0,
			done: false,
		}
	}

	pub async fn next(&mut self) -> Result<Option<DirEntry>, Error> {
		loop {
			self.fetch().await?;
			let Some(item) = self.name.items.get_mut(self.cursor).map(mem::take) else {
				return Ok(None);
			};

			self.cursor += 1;
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
		if self.cursor < self.name.items.len() || self.done {
			return Ok(());
		}

		let result = self
			.session
			.send_with_timeout(requests::ReadDir::new(&self.handle), Duration::from_mins(45))
			.await;

		self.name = match result {
			Ok(resp) => resp,
			Err(Error::Status(status)) if status.is_eof() => {
				self.done = true;
				return Ok(());
			}
			Err(e) => return Err(e),
		};

		self.cursor = 0;
		Ok(())
	}
}
