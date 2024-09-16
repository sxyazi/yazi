use std::{collections::HashMap, sync::atomic::{AtomicU64, Ordering}};

use super::{Cha, File};
use crate::{emit, event::Cmd, fs::Url, Layer};

pub static FILES_TICKET: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub enum FilesOp {
	Full(Url, Vec<File>, Cha),
	Part(Url, Vec<File>, u64),
	Done(Url, Cha, u64),
	Size(Url, HashMap<Url, u64>),
	IOErr(Url, std::io::ErrorKind),

	Creating(Url, Vec<File>),
	Deleting(Url, Vec<Url>),
	Updating(Url, HashMap<Url, File>),
	Upserting(Url, HashMap<Url, File>),
}

impl FilesOp {
	#[inline]
	pub fn cwd(&self) -> &Url {
		match self {
			Self::Full(u, ..) => u,
			Self::Part(u, ..) => u,
			Self::Done(u, ..) => u,
			Self::Size(u, _) => u,
			Self::IOErr(u, _) => u,

			Self::Creating(u, _) => u,
			Self::Deleting(u, _) => u,
			Self::Updating(u, _) => u,
			Self::Upserting(u, _) => u,
		}
	}

	#[inline]
	pub fn emit(self) {
		emit!(Call(Cmd::new("update_files").with_any("op", self), Layer::Manager));
	}

	pub fn prepare(cwd: &Url) -> u64 {
		let ticket = FILES_TICKET.fetch_add(1, Ordering::Relaxed);
		Self::Part(cwd.clone(), vec![], ticket).emit();
		ticket
	}

	pub fn chroot(&self, new: &Url) -> Self {
		macro_rules! new {
			($url:expr) => {{ new.join($url.file_name().unwrap()) }};
		}
		macro_rules! files {
			($files:expr) => {{ $files.iter().map(|f| f.rebase(new)).collect() }};
		}
		macro_rules! map {
			($map:expr) => {{ $map.iter().map(|(u, f)| (new!(u), f.rebase(new))).collect() }};
		}

		let n = new.clone();
		match self {
			Self::Full(_, files, mtime) => Self::Full(n, files!(files), *mtime),
			Self::Part(_, files, ticket) => Self::Part(n, files!(files), *ticket),
			Self::Done(_, mtime, ticket) => Self::Done(n, *mtime, *ticket),
			Self::Size(_, map) => Self::Size(n, map.iter().map(|(u, &s)| (new!(u), s)).collect()),
			Self::IOErr(_, err) => Self::IOErr(n, *err),

			Self::Creating(_, files) => Self::Creating(n, files!(files)),
			Self::Deleting(_, urls) => Self::Deleting(n, urls.iter().map(|u| new!(u)).collect()),
			Self::Updating(_, map) => Self::Updating(n, map!(map)),
			Self::Upserting(_, map) => Self::Upserting(n, map!(map)),
		}
	}
}
