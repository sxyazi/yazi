use std::{collections::{HashMap, HashSet}, sync::atomic::{AtomicU64, Ordering}};

use super::{Cha, File, UrnBuf};
use crate::{Layer, emit, event::Cmd, fs::Url};

pub static FILES_TICKET: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub enum FilesOp {
	Full(Url, Vec<File>, Cha),
	Part(Url, Vec<File>, u64),
	Done(Url, Cha, u64),
	Size(Url, HashMap<UrnBuf, u64>),
	IOErr(Url, std::io::ErrorKind),

	Creating(Url, Vec<File>),
	Deleting(Url, HashSet<UrnBuf>),
	Updating(Url, HashMap<UrnBuf, File>),
	Upserting(Url, HashMap<UrnBuf, File>),
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

	pub fn rebase(&self, new: &Url) -> Self {
		macro_rules! files {
			($files:expr) => {{ $files.iter().map(|f| f.rebase(new)).collect() }};
		}
		macro_rules! map {
			($map:expr) => {{ $map.iter().map(|(u, f)| (u.clone(), f.rebase(new))).collect() }};
		}

		let n = new.clone();
		match self {
			Self::Full(_, files, cha) => Self::Full(n, files!(files), *cha),
			Self::Part(_, files, ticket) => Self::Part(n, files!(files), *ticket),
			Self::Done(_, cha, ticket) => Self::Done(n, *cha, *ticket),
			Self::Size(_, map) => Self::Size(n, map.iter().map(|(u, &s)| (u.clone(), s)).collect()),
			Self::IOErr(_, err) => Self::IOErr(n, *err),

			Self::Creating(_, files) => Self::Creating(n, files!(files)),
			Self::Deleting(_, urns) => Self::Deleting(n, urns.clone()),
			Self::Updating(_, map) => Self::Updating(n, map!(map)),
			Self::Upserting(_, map) => Self::Upserting(n, map!(map)),
		}
	}

	pub fn diff_recoverable(&self, contains: impl Fn(&Url) -> bool) -> (Vec<Url>, Vec<Url>) {
		match self {
			Self::Deleting(cwd, urns) => {
				(urns.iter().map(|u| cwd.join(u._deref()._as_path())).collect(), vec![])
			}
			Self::Updating(cwd, urns) | Self::Upserting(cwd, urns) => urns
				.iter()
				.filter(|&(u, f)| u != f.urn())
				.map(|(u, f)| (cwd.join(u._deref()._as_path()), f))
				.filter(|(u, _)| contains(u))
				.map(|(u, f)| (u, f.url_owned()))
				.unzip(),
			_ => (vec![], vec![]),
		}
	}
}
