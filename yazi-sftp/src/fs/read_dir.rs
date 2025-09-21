use std::{mem, sync::Arc};

use crate::{ByteStr, Error, Session, fs::DirEntry, requests, responses};

pub struct ReadDir {
	session: Arc<Session>,
	dir:     Arc<ByteStr<'static>>,
	handle:  String,

	name:   responses::Name<'static>,
	cursor: usize,
	done:   bool,
}

impl ReadDir {
	pub(crate) fn new(session: &Arc<Session>, dir: ByteStr, handle: String) -> Self {
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
			if item.name != "." && item.name != ".." {
				return Ok(Some(DirEntry {
					dir:       self.dir.clone(),
					name:      item.name,
					long_name: item.long_name,
					attrs:     item.attrs,
				}));
			}
		}
	}

	async fn fetch(&mut self) -> Result<(), Error> {
		if self.cursor < self.name.items.len() || self.done {
			return Ok(());
		}

		self.name = match self.session.send(requests::ReadDir::new(&self.handle)).await {
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
