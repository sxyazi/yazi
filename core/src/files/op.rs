use std::collections::BTreeMap;

use shared::Url;

use super::File;

#[derive(Debug)]
pub enum FilesOp {
	Read(Url, Vec<File>),
	Size(Url, BTreeMap<Url, u64>),
	IOErr(Url),
}

impl FilesOp {
	#[inline]
	pub fn url(&self) -> Url {
		match self {
			Self::Read(url, _) => url,
			Self::Size(url, _) => url,
			Self::IOErr(url) => url,
		}
		.clone()
	}

	#[inline]
	pub fn clear(url: &Url) -> Self { Self::Read(url.clone(), Vec::new()) }
}
