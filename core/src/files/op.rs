use std::{collections::BTreeMap, sync::atomic::{AtomicU64, Ordering}};

use shared::Url;

use super::File;
use crate::emit;

pub(super) static FILES_VERSION: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum FilesOp {
	Full(Url, Vec<File>),
	Part(Url, u64, Vec<File>),
	Size(Url, BTreeMap<Url, u64>),
	IOErr(Url),
}

impl FilesOp {
	#[inline]
	pub fn url(&self) -> Url {
		match self {
			Self::Full(url, _) => url,
			Self::Part(url, ..) => url,
			Self::Size(url, _) => url,
			Self::IOErr(url) => url,
		}
		.clone()
	}

	#[inline]
	pub fn prepare(url: &Url) -> u64 {
		let version = FILES_VERSION.fetch_add(1, Ordering::Relaxed);
		emit!(Files(Self::Part(url.clone(), version, Vec::new())));
		version
	}
}
