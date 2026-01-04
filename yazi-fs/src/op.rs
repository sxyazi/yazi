use std::path::Path;

use hashbrown::{HashMap, HashSet};
use yazi_macro::relay;
use yazi_shared::{Id, Ids, path::PathBufDyn, url::{UrlBuf, UrlLike}};

use super::File;
use crate::{cha::Cha, error::Error};

pub static FILES_TICKET: Ids = Ids::new();

#[derive(Clone, Debug)]
pub enum FilesOp {
	Full(UrlBuf, Vec<File>, Cha),
	Part(UrlBuf, Vec<File>, Id),
	Done(UrlBuf, Cha, Id),
	Size(UrlBuf, HashMap<PathBufDyn, u64>),
	IOErr(UrlBuf, Error),

	Creating(UrlBuf, Vec<File>),
	Deleting(UrlBuf, HashSet<PathBufDyn>),
	Updating(UrlBuf, HashMap<PathBufDyn, File>),
	Upserting(UrlBuf, HashMap<PathBufDyn, File>),
}

impl FilesOp {
	#[inline]
	pub fn cwd(&self) -> &UrlBuf {
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
		yazi_shared::event::Event::Call(relay!(mgr:update_files).with_any("op", self).into()).emit();
	}

	pub fn prepare(cwd: &UrlBuf) -> Id {
		let ticket = FILES_TICKET.next();
		Self::Part(cwd.clone(), vec![], ticket).emit();
		ticket
	}

	pub fn rename(map: HashMap<UrlBuf, File>) {
		let mut parents: HashMap<_, (HashSet<_>, HashMap<_, _>)> = Default::default();
		for (o, n) in map {
			let Some(o_p) = o.parent() else { continue };
			let Some(n_p) = n.url.parent() else { continue };
			if o_p == n_p {
				parents.entry_ref(&o_p).or_default().1.insert(o.urn().into(), n);
			} else {
				parents.entry_ref(&o_p).or_default().0.insert(o.urn().into());
				parents.entry_ref(&n_p).or_default().1.insert(n.urn().into(), n);
			}
		}
		for (p, (o, n)) in parents {
			match (o.is_empty(), n.is_empty()) {
				(true, true) => unreachable!(),
				(true, false) => Self::Upserting(p, n).emit(),
				(false, true) => Self::Deleting(p, o).emit(),
				(false, false) => {
					Self::Deleting(p.clone(), o).emit();
					Self::Upserting(p, n).emit();
				}
			}
		}
	}

	pub fn mutate(ops: Vec<Self>) {
		let mut parents: HashMap<_, (HashMap<_, _>, HashSet<_>)> = Default::default();
		for op in ops {
			match op {
				Self::Upserting(p, map) => parents.entry(p).or_default().0.extend(map),
				Self::Deleting(p, urns) => parents.entry(p).or_default().1.extend(urns),
				_ => unreachable!(),
			}
		}
		for (p, (u, d)) in parents {
			match (u.is_empty(), d.is_empty()) {
				(true, true) => unreachable!(),
				(true, false) => Self::Deleting(p, d).emit(),
				(false, true) => Self::Upserting(p, u).emit(),
				(false, false) => {
					Self::Deleting(p.clone(), d).emit();
					Self::Upserting(p, u).emit();
				}
			}
		}
	}

	pub fn chdir(&self, wd: &Path) -> Self {
		macro_rules! files {
			($files:expr) => {{ $files.iter().map(|file| file.chdir(wd)).collect() }};
		}
		macro_rules! map {
			($map:expr) => {{ $map.iter().map(|(urn, file)| (urn.clone(), file.chdir(wd))).collect() }};
		}

		let w = UrlBuf::from(wd);
		match self {
			Self::Full(_, files, cha) => Self::Full(w, files!(files), *cha),
			Self::Part(_, files, ticket) => Self::Part(w, files!(files), *ticket),
			Self::Done(_, cha, ticket) => Self::Done(w, *cha, *ticket),
			Self::Size(_, map) => Self::Size(w, map.iter().map(|(urn, &s)| (urn.clone(), s)).collect()),
			Self::IOErr(_, err) => Self::IOErr(w, err.clone()),

			Self::Creating(_, files) => Self::Creating(w, files!(files)),
			Self::Deleting(_, urns) => Self::Deleting(w, urns.clone()),
			Self::Updating(_, map) => Self::Updating(w, map!(map)),
			Self::Upserting(_, map) => Self::Upserting(w, map!(map)),
		}
	}

	pub fn diff_recoverable(&self, contains: impl Fn(&UrlBuf) -> bool) -> (Vec<UrlBuf>, Vec<UrlBuf>) {
		match self {
			Self::Deleting(cwd, urns) => {
				(urns.iter().filter_map(|u| cwd.try_join(u).ok()).collect(), vec![])
			}
			Self::Updating(cwd, urns) | Self::Upserting(cwd, urns) => urns
				.iter()
				.filter(|&(u, f)| u != f.urn())
				.filter_map(|(u, f)| cwd.try_join(u).ok().map(|u| (u, f)))
				.filter(|(u, _)| contains(u))
				.map(|(u, f)| (u, f.url_owned()))
				.unzip(),
			_ => (vec![], vec![]),
		}
	}
}
