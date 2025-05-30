use std::collections::{HashMap, HashSet};

use yazi_shared::{Id, Ids, event::Cmd, url::{Url, UrnBuf}};

use super::File;
use crate::{cha::Cha, maybe_exists};

pub static FILES_TICKET: Ids = Ids::new();

#[derive(Clone, Debug)]
pub enum FilesOp {
	Full(Url, Vec<File>, Cha),
	Part(Url, Vec<File>, Id),
	Done(Url, Cha, Id),
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
		yazi_shared::event::Event::Call(Cmd::new("mgr:update_files").with_any("op", self).into())
			.emit();
	}

	pub fn prepare(cwd: &Url) -> Id {
		let ticket = FILES_TICKET.next();
		Self::Part(cwd.clone(), vec![], ticket).emit();
		ticket
	}

	pub fn rename(map: HashMap<Url, File>) {
		let mut parents: HashMap<_, (HashSet<_>, HashMap<_, _>)> = Default::default();
		for (o, n) in map {
			let Some(o_p) = o.parent_url() else { continue };
			let Some(n_p) = n.url.parent_url() else { continue };
			if o_p != n_p {
				parents.entry(o_p).or_default().0.insert(o.urn_owned());
			}
			parents.entry(n_p).or_default().1.insert(n.urn_owned(), n);
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

	pub async fn issue_error(cwd: &Url, kind: std::io::ErrorKind) {
		use std::io::ErrorKind;
		if kind != ErrorKind::NotFound {
			Self::IOErr(cwd.clone(), kind).emit();
		} else if maybe_exists(cwd).await {
			Self::IOErr(cwd.clone(), kind).emit();
		} else if let Some((p, n)) = cwd.pair() {
			Self::Deleting(p, [n].into()).emit();
		}
	}

	pub fn diff_recoverable(&self, contains: impl Fn(&Url) -> bool) -> (Vec<Url>, Vec<Url>) {
		match self {
			Self::Deleting(cwd, urns) => (urns.iter().map(|u| cwd.join(u)).collect(), vec![]),
			Self::Updating(cwd, urns) | Self::Upserting(cwd, urns) => urns
				.iter()
				.filter(|&(u, f)| u != f.urn())
				.map(|(u, f)| (cwd.join(u), f))
				.filter(|(u, _)| contains(u))
				.map(|(u, f)| (u, f.url_owned()))
				.unzip(),
			_ => (vec![], vec![]),
		}
	}
}
