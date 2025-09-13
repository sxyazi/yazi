use std::mem;

use crate::{ByteStr, Error, Session, fs::DirEntry, requests, responses};

pub struct ReadDir<'a> {
	dir:     ByteStr<'a>,
	handle:  String,
	session: &'a Session,

	name:   responses::Name<'a>,
	cursor: usize,
	done:   bool,
}

impl<'a> ReadDir<'a> {
	pub(crate) fn new(session: &'a Session, dir: ByteStr<'a>, handle: String) -> Self {
		Self { dir, handle, session, name: Default::default(), cursor: 0, done: false }
	}

	pub async fn next(&mut self) -> Result<Option<DirEntry<'_>>, Error> {
		loop {
			self.fetch().await?;
			let Some(item) = self.name.items.get_mut(self.cursor).map(mem::take) else {
				return Ok(None);
			};

			self.cursor += 1;
			if item.name != "." && item.name != ".." {
				return Ok(Some(DirEntry {
					dir:       ByteStr::from(&self.dir),
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
