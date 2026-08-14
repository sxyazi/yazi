use std::{iter, path::Path};

use hashbrown::{HashMap, HashSet};
use mlua::{UserData, UserDataFields};
use yazi_codegen::FromLuaOwned;
use yazi_macro::{impl_data_any, relay};
use yazi_shared::{id::{Id, Ids}, path::{PathBufDyn, PathLike}, url::{UrlBuf, UrlLike, UrlMapExt}};

use crate::file::File;

pub static FILES_TICKET: Ids = Ids::new();

#[derive(Clone, Debug, FromLuaOwned)]
pub enum FilesOp {
	Full(File, Vec<File>),
	Part(UrlBuf, Vec<File>, Id),
	Done(File, Id),
	Size(UrlBuf, HashMap<PathBufDyn, u64>),
	IOErr(UrlBuf, yazi_shim::fs::Error),

	Creating(UrlBuf, Vec<File>),
	Deleting(UrlBuf, HashSet<PathBufDyn>),
	Updating(UrlBuf, HashMap<PathBufDyn, File>),
	Upserting(UrlBuf, HashMap<PathBufDyn, File>),
}

impl_data_any!(FilesOp, from_into_lua = inherit);

impl FilesOp {
	pub fn cwd(&self) -> &UrlBuf {
		match self {
			Self::Full(f, ..) => &f.url,
			Self::Part(u, ..) => u,
			Self::Done(f, ..) => &f.url,
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
			Self::Full(_, files) | Self::Part(_, files, _) | Self::Creating(_, files) => {
				Box::new(files.iter().filter(|f| !f.key().is_empty()))
			}
			Self::Done(..) => Box::new(iter::empty()),
			Self::Size(..) => Box::new(iter::empty()),
			Self::IOErr(..) => Box::new(iter::empty()),

			Self::Deleting(..) => Box::new(iter::empty()),
			Self::Updating(_, map) | Self::Upserting(_, map) => {
				Box::new(map.values().filter(|f| !f.key().is_empty()))
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
		let mut trails: HashMap<UrlBuf, Vec<_>> = Default::default();
		for file in files {
			if let Some((t, _)) = file.url.pair() {
				trails.get_or_insert_default(t).push(file);
			}
		}
		for (t, files) in trails {
			Self::Creating(t, files).emit();
		}
	}

	pub fn rename(map: HashMap<UrlBuf, File>) {
		let mut trails: HashMap<UrlBuf, (HashSet<_>, HashMap<_, _>)> = Default::default();
		for (o, n) in map {
			let Some((o_t, o_k)) = o.pair() else { continue };
			let Some((n_t, n_k)) = n.url.pair() else { continue };
			if o_t == n_t {
				trails.get_or_insert_default(o_t).1.insert(o_k.into(), n);
			} else {
				trails.get_or_insert_default(o_t).0.insert(o_k.into());
				trails.get_or_insert_default(n_t).1.insert(n_k.into(), n);
			}
		}
		for (t, (o, n)) in trails {
			match (o.is_empty(), n.is_empty()) {
				(true, true) => {}
				(true, false) => Self::Upserting(t, n).emit(),
				(false, true) => Self::Deleting(t, o).emit(),
				(false, false) => {
					Self::Deleting(t.clone(), o).emit();
					Self::Upserting(t, n).emit();
				}
			}
		}
	}

	pub fn mutate(ops: Vec<Self>) {
		let mut trails: HashMap<_, (HashMap<_, _>, HashSet<_>)> = Default::default();
		for op in ops {
			match op {
				Self::Upserting(t, map) => trails
					.entry(t)
					.or_default()
					.0
					.extend(map.into_iter().filter(|(k, f)| !k.is_empty() && !f.key().is_empty())),
				Self::Deleting(t, keys) => {
					trails.entry(t).or_default().1.extend(keys.into_iter().filter(|k| !k.is_empty()))
				}
				_ => unreachable!(),
			}
		}
		for (t, (u, d)) in trails {
			match (u.is_empty(), d.is_empty()) {
				(true, true) => {}
				(true, false) => Self::Deleting(t, d).emit(),
				(false, true) => Self::Upserting(t, u).emit(),
				(false, false) => {
					Self::Deleting(t.clone(), d).emit();
					Self::Upserting(t, u).emit();
				}
			}
		}
	}

	pub fn chdir(&self, wd: &Path) -> Self {
		macro_rules! files {
			($files:expr) => {{ $files.iter().map(|file| file.chdir(wd)).collect() }};
		}
		macro_rules! map {
			($map:expr) => {{ $map.iter().map(|(key, file)| (key.clone(), file.chdir(wd))).collect() }};
		}

		let w = UrlBuf::from(wd);
		match self {
			Self::Full(file, files) => Self::Full(file.chdir(wd), files!(files)),
			Self::Part(_, files, ticket) => Self::Part(w, files!(files), *ticket),
			Self::Done(file, ticket) => Self::Done(file.chdir(wd), *ticket),
			Self::Size(_, map) => Self::Size(w, map.iter().map(|(key, &s)| (key.clone(), s)).collect()),
			Self::IOErr(_, err) => Self::IOErr(w, err.clone()),

			Self::Creating(_, files) => Self::Creating(w, files!(files)),
			Self::Deleting(_, urns) => Self::Deleting(w, urns.clone()),
			Self::Updating(_, map) => Self::Updating(w, map!(map)),
			Self::Upserting(_, map) => Self::Upserting(w, map!(map)),
		}
	}
}

impl UserData for FilesOp {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("cwd", |_, me| Ok(me.cwd().clone()));
	}
}
