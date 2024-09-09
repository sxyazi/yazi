use std::{ffi::OsStr, path::Path};

use anyhow::{bail, Result};

use super::Url;

#[derive(Clone, Debug)]
pub(super) struct Location {
	url:  Url,
	urn:  *const OsStr,
	name: *const OsStr,
}

unsafe impl Send for Location {}

impl Default for Location {
	fn default() -> Self {
		let url = Url::default();
		let urn = url.as_os_str() as *const OsStr;
		let name = url.as_os_str() as *const OsStr;
		Self { url, urn, name }
	}
}

impl Location {
	pub(super) fn from(url: Url) -> Result<Self> {
		if url.is_search() {
			bail!("url is from search results: {url:?}");
		}
		let Some(name) = url.file_name() else {
			bail!("url has no filename: {url:?}");
		};

		let urn = name as *const OsStr;
		let name = name as *const OsStr;
		Ok(Self { url, urn, name })
	}

	pub(super) fn from_search(cwd: &Url, url: Url) -> Result<Self> {
		if !url.is_search() {
			bail!("url is not from search results: {url:?}");
		}
		let Some(name) = url.file_name() else {
			bail!("url has no filename: {url:?}");
		};

		let urn = url.strip_prefix(cwd).unwrap_or(&url).as_os_str() as *const OsStr;
		let name = name as *const OsStr;
		Ok(Self { url, urn, name })
	}
}

impl Location {
	#[inline]
	pub(super) fn url(&self) -> &Url { &self.url }

	#[inline]
	pub(super) fn urn(&self) -> &Path { Path::new(unsafe { &*self.urn }) }

	#[inline]
	pub(super) fn name(&self) -> &OsStr { unsafe { &*self.name } }
}
