use std::{iter, path::Path};

use hashbrown::{HashMap, HashSet};
use yazi_macro::{impl_data_any, relay};
use yazi_shared::{id::{Id, Ids}, path::{PathBufDyn, PathLike}, url::{UrlBuf, UrlLike, UrlMapExt}};

use crate::{cha::Cha, file::File};

pub static FILES_TICKET: Ids = Ids::new();

#[derive(Clone, Debug)]
pub enum FilesOp {
	Full(UrlBuf, Vec<File>, Cha),
	Part(UrlBuf, Vec<File>, Id),
	Done(UrlBuf, Cha, Id),
	Size(UrlBuf, HashMap<PathBufDyn, u64>),
	IOErr(UrlBuf, yazi_shim::fs::Error),

	Creating(UrlBuf, Vec<File>),
	Deleting(UrlBuf, HashSet<PathBufDyn>),
	Updating(UrlBuf, HashMap<PathBufDyn, File>),
	Upserting(UrlBuf, HashMap<PathBufDyn, File>),
}

impl_data_any!(FilesOp);

impl FilesOp {
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

	pub fn files(&self) -> Box<dyn Iterator<Item = &File> + '_> {
		match self {
			Self::Full(_, files, _) | Self::Part(_, files, _) | Self::Creating(_, files) => {
				Box::new(files.iter().filter(|f| !f.entry_key().is_empty()))
			}
			Self::Done(..) => Box::new(iter::empty()),
			Self::Size(..) => Box::new(iter::empty()),
			Self::IOErr(..) => Box::new(iter::empty()),

			Self::Deleting(..) => Box::new(iter::empty()),
			Self::Updating(_, map) | Self::Upserting(_, map) => {
				Box::new(map.values().filter(|f| !f.entry_key().is_empty()))
			}
		}
	}

	pub fn emit(self) {
		yazi_shared::event::Event::Call(relay!(mgr:update_files).with_any("op", self).into()).emit();
	}

	pub fn prepare(cwd: &UrlBuf) -> Id {
		let ticket = FILES_TICKET.next();
		Self::Part(cwd.clone(), vec![], ticket).emit();
		ticket
	}

	pub fn create(files: Vec<File>) {
		let mut parents: HashMap<UrlBuf, Vec<_>> = Default::default();
		for file in files {
			if let Some((p, _)) = file.url.pair2() {
				parents.get_or_insert_default(p).push(file);
			}
		}
		for (p, files) in parents {
			Self::Creating(p, files).emit();
		}
	}

	pub fn rename(map: HashMap<UrlBuf, File>) {
		let mut parents: HashMap<UrlBuf, (HashSet<_>, HashMap<_, _>)> = Default::default();
		for (o, n) in map {
			let Some((o_p, o_k)) = o.pair2() else { continue };
			let Some((n_p, n_k)) = n.url.pair2() else { continue };
			if o_p == n_p {
				parents.get_or_insert_default(o_p).1.insert(o_k.into(), n);
			} else {
				parents.get_or_insert_default(o_p).0.insert(o_k.into());
				parents.get_or_insert_default(n_p).1.insert(n_k.into(), n);
			}
		}
		for (p, (o, n)) in parents {
			match (o.is_empty(), n.is_empty()) {
				(true, true) => {}
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
				Self::Upserting(p, map) => parents
					.entry(p)
					.or_default()
					.0
					.extend(map.into_iter().filter(|(k, f)| !k.is_empty() && !f.entry_key().is_empty())),
				Self::Deleting(p, keys) => {
					parents.entry(p).or_default().1.extend(keys.into_iter().filter(|k| !k.is_empty()))
				}
				_ => unreachable!(),
			}
		}
		for (p, (u, d)) in parents {
			match (u.is_empty(), d.is_empty()) {
				(true, true) => {}
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

	pub fn diff_recoverable<'a, I>(&self, urls: I) -> (Vec<UrlBuf>, Vec<File>)
	where
		I: IntoIterator<Item = &'a UrlBuf>,
	{
		let cwd = self.cwd();
		let it = urls
			.into_iter()
			.filter(|u| u.parent().is_some_and(|p| p == *cwd))
			.filter(|u| !u.entry_key().is_empty());

		match self {
			Self::Deleting(_, keys) => {
				(it.filter(|u| keys.contains(&u.entry_key())).cloned().collect(), vec![])
			}
			Self::Updating(_, files) | Self::Upserting(_, files) => it
				.filter_map(|u| files.get(&u.entry_key()).map(|f| (u, f)))
				.filter(|(_, f)| !f.entry_key().is_empty())
				.filter(|&(u, f)| u != f.url)
				.map(|(u, f)| (u.clone(), f.clone()))
				.unzip(),
			_ => (vec![], vec![]),
		}
	}
}
