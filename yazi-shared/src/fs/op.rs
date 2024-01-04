use std::{collections::BTreeMap, sync::atomic::{AtomicU64, Ordering}, time::SystemTime};

use super::File;
use crate::{emit, event::Exec, fs::Url, Layer};

pub static FILES_TICKET: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum FilesOp {
	Full(Url, Vec<File>, Option<SystemTime>),
	Part(Url, Vec<File>, u64),
	Done(Url, Option<SystemTime>, u64),
	Size(Url, BTreeMap<Url, u64>),

	Creating(Url, Vec<File>),
	Deleting(Url, Vec<Url>),
	Updating(Url, BTreeMap<Url, File>),
	Upserting(Url, BTreeMap<Url, File>),
}

impl FilesOp {
	#[inline]
	pub fn url(&self) -> &Url {
		match self {
			Self::Full(url, ..) => url,
			Self::Part(url, ..) => url,
			Self::Done(url, ..) => url,
			Self::Size(url, _) => url,

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
		Self::Part(url.clone(), vec![], ticket).emit();
		ticket
	}

	pub fn chroot(&self, new: &Url) -> Self {
		let old = self.url();
		macro_rules! new {
			($url:expr) => {{ new.join($url.strip_prefix(old).unwrap()) }};
		}
		macro_rules! files {
			($files:expr) => {{
				$files
					.iter()
					.map(|file| {
						let mut f = file.clone();
						f.url = new!(f.url);
						f
					})
					.collect()
			}};
		}
		macro_rules! map {
			($map:expr) => {{
				$map
					.iter()
					.map(|(k, v)| {
						let mut f = v.clone();
						f.url = new!(f.url);
						(new!(k), f)
					})
					.collect()
			}};
		}

		let u = new.clone();
		match self {
			Self::Full(_, files, mtime) => Self::Full(u, files!(files), *mtime),
			Self::Part(_, files, ticket) => Self::Part(u, files!(files), *ticket),
			Self::Done(_, mtime, ticket) => Self::Done(u, *mtime, *ticket),
			Self::Size(_, map) => Self::Size(u, map.iter().map(|(k, v)| (new!(k), *v)).collect()),

			Self::Creating(_, files) => Self::Creating(u, files!(files)),
			Self::Deleting(_, urls) => Self::Deleting(u, urls.iter().map(|u| new!(u)).collect()),
			Self::Updating(_, map) => Self::Updating(u, map!(map)),
			Self::Upserting(_, map) => Self::Upserting(u, map!(map)),
		}
	}
}
