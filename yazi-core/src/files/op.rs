use std::{collections::{BTreeMap, BTreeSet}, sync::atomic::{AtomicU64, Ordering}};

use yazi_shared::Url;

use super::File;
use crate::emit;

pub(super) static FILES_TICKET: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum FilesOp {
	Full(Url, Vec<File>),
	Part(Url, u64, Vec<File>),
	Size(Url, BTreeMap<Url, u64>),
	IOErr(Url),

	Creating(Url, BTreeMap<Url, File>),
	Deleting(Url, BTreeSet<Url>),
	Replacing(Url, BTreeMap<Url, File>),
}

impl FilesOp {
	#[inline]
	pub fn url(&self) -> &Url {
		match self {
			Self::Full(url, _) => url,
			Self::Part(url, ..) => url,
			Self::Size(url, _) => url,
			Self::IOErr(url) => url,

			Self::Creating(url, _) => url,
			Self::Deleting(url, _) => url,
			Self::Replacing(url, _) => url,
		}
	}

	#[inline]
	pub fn prepare(url: &Url) -> u64 {
		let ticket = FILES_TICKET.fetch_add(1, Ordering::Relaxed);
		emit!(Files(Self::Part(url.clone(), ticket, Vec::new())));
		ticket
	}
}
