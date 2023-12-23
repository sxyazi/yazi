use std::{collections::BTreeMap, sync::atomic::{AtomicU64, Ordering}};

use super::File;
use crate::{emit, event::Exec, fs::Url, Layer};

pub static FILES_TICKET: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum FilesOp {
	Full(Url, Vec<File>),
	Part(Url, Vec<File>, u64),
	Size(Url, BTreeMap<Url, u64>),
	IOErr(Url),

	Creating(Url, Vec<File>),
	Deleting(Url, Vec<Url>),
	Updating(Url, BTreeMap<Url, File>),
	Upserting(Url, BTreeMap<Url, File>),
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
			Self::Updating(url, _) => url,
			Self::Upserting(url, _) => url,
		}
	}

	#[inline]
	pub fn emit(self) {
		emit!(Call(Exec::call("update_files", vec![]).with_data(self).vec(), Layer::Manager));
	}

	pub fn prepare(url: &Url) -> u64 {
		let ticket = FILES_TICKET.fetch_add(1, Ordering::Relaxed);
		Self::Part(url.clone(), Vec::new(), ticket).emit();
		ticket
	}

	pub fn chroot(&self, new: Url) -> Self {
		todo!()
		// Full(Url, Vec<File>),
		// Part(Url, u64, Vec<File>),
		// Size(Url, BTreeMap<Url, u64>),
		// IOErr(Url),

		// Creating(Url, BTreeMap<Url, File>),
		// Deleting(Url, BTreeSet<Url>),
		// Replacing(Url, BTreeMap<Url, File>),

		// match self {
		// 	FilesOp::Full(_, vec) => {}
		// 	FilesOp::Part(_, _, vec) => todo!(),
		// 	FilesOp::Size(_, map) => todo!(),
		// 	FilesOp::IOErr(_) => todo!(),
		// 	FilesOp::Creating(_, map) => todo!(),
		// 	FilesOp::Deleting(_, set) => todo!(),
		// 	FilesOp::Replacing(_, map) => todo!(),
		// }
	}
}
