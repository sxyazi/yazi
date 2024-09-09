use std::{ffi::OsStr, fmt::{self, Debug, Formatter}, ops::Deref, path::Path};

use super::Url;

pub struct Loc {
	url:  Url,
	urn:  *const OsStr,
	name: *const OsStr,
}

unsafe impl Send for Loc {}

impl Default for Loc {
	fn default() -> Self { Self { url: Url::default(), urn: OsStr::new(""), name: OsStr::new("") } }
}

impl Deref for Loc {
	type Target = Url;

	fn deref(&self) -> &Self::Target { &self.url }
}

impl AsRef<Path> for Loc {
	fn as_ref(&self) -> &Path { self.url() }
}

impl Eq for Loc {}

impl PartialEq for Loc {
	fn eq(&self, other: &Self) -> bool {
		self.url == other.url && self.urn() == other.urn() && self.name() == other.name()
	}
}

impl Clone for Loc {
	fn clone(&self) -> Self {
		let url = self.url.clone();
		let name = url.file_name().unwrap_or(OsStr::new("")) as *const OsStr;
		let urn = if url.is_search() { self.twin_urn(&url) } else { name };
		Self { url, urn, name }
	}
}

impl Debug for Loc {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_struct("Loc")
			.field("url", &self.url)
			.field("urn", &self.urn())
			.field("name", &self.name())
			.finish()
	}
}

impl Loc {
	pub fn from(url: Url) -> Self {
		let urn = url.file_name().unwrap_or(OsStr::new("")) as *const OsStr;
		Self { url, urn, name: urn }
	}

	pub fn from_search(cwd: &Url, url: Url) -> Self {
		let urn = url.strip_prefix(cwd).unwrap_or(&url).as_os_str() as *const OsStr;
		let name = url.file_name().unwrap_or(OsStr::new("")) as *const OsStr;
		Self { url, urn, name }
	}

	pub fn rebase(&self, parent: &Url) -> Self {
		let url = parent.join(self.name());
		let name = url.file_name().unwrap_or(OsStr::new("")) as *const OsStr;
		let urn = if url.is_search() { self.twin_urn(&url) } else { name };
		Self { url, urn, name }
	}

	#[inline]
	fn twin_urn<'a>(&self, new: &'a Url) -> &'a OsStr {
		let total = new.components().count();
		let take = self.urn().components().count();

		let mut it = new.components();
		for _ in 0..total - take {
			it.next().unwrap();
		}

		it.as_path().as_os_str()
	}
}

impl Loc {
	#[inline]
	pub fn url(&self) -> &Url { &self.url }

	#[inline]
	pub fn urn(&self) -> &Path { Path::new(unsafe { &*self.urn }) }

	#[inline]
	pub fn name(&self) -> &OsStr { unsafe { &*self.name } }
}
