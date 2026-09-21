use std::{mem, path::Path};

use hashbrown::{HashMap, HashSet};
use mlua::{UserData, UserDataFields};
use strum::IntoStaticStr;
use yazi_codegen::FromLuaOwned;
use yazi_macro::{impl_data_any, relay};
use yazi_shared::{id::{Id, Ids}, path::{PathBufDyn, PathLike}, url::{UrlBuf, UrlLike, UrlMapExt}};
use yazi_shim::{mlua::UserDataFieldsExt, strum::IntoStr};

use crate::file::File;

pub static FILES_TICKET: Ids = Ids::new();

#[derive(Clone, Debug, FromLuaOwned, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum FilesOp {
	Full(File, Vec<File>),
	Part(UrlBuf, Vec<File>, Id),
	Done(File, Id),
	Size(UrlBuf, HashMap<PathBufDyn, u64>),
	Rank(UrlBuf, HashMap<PathBufDyn, i64>),
	Fail(UrlBuf, yazi_shim::fs::Error),

	Create(UrlBuf, Vec<File>),
	Delete(UrlBuf, HashSet<PathBufDyn>),
	Update(UrlBuf, HashMap<PathBufDyn, File>),
	Upsert(UrlBuf, HashMap<PathBufDyn, File>),
}

impl_data_any!(FilesOp, from_into_lua = inherit);

impl FilesOp {
	pub fn cwd(&self) -> &UrlBuf {
		match self {
			Self::Full(f, ..) => &f.url,
			Self::Part(u, ..) => u,
			Self::Done(f, ..) => &f.url,
			Self::Size(u, _) => u,
			Self::Rank(u, _) => u,
			Self::Fail(u, _) => u,

			Self::Create(u, _) => u,
			Self::Delete(u, _) => u,
			Self::Update(u, _) => u,
			Self::Upsert(u, _) => u,
		}
	}

	pub fn emit(self) {
		yazi_shared::event::Event::Call(relay!(mgr:update_files).with_any("op", self).into()).emit();
	}

	pub fn create(files: Vec<File>) {
		let mut trails: HashMap<UrlBuf, Vec<_>> = Default::default();
		for file in files {
			let Some((t, _)) = file.pair() else { continue };
			trails.get_or_insert_default(t).push(file);
		}
		for (t, files) in trails {
			Self::Create(t, files).emit();
		}
	}

	pub fn rename(map: HashMap<UrlBuf, File>) {
		let mut trails: HashMap<UrlBuf, (HashSet<_>, HashMap<_, _>)> = Default::default();
		for (o, n) in map {
			let Some((o_t, o_k)) = o.pair() else { continue };
			let Some((n_t, n_k)) = n.pair() else { continue };
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
				(true, false) => Self::Upsert(t, n).emit(),
				(false, true) => Self::Delete(t, o).emit(),
				(false, false) => {
					Self::Delete(t.clone(), o).emit();
					Self::Upsert(t, n).emit();
				}
			}
		}
	}

	pub fn mutate(ops: Vec<Self>) {
		let mut trails: HashMap<_, (HashMap<_, _>, HashSet<_>)> = Default::default();
		for op in ops {
			match op {
				Self::Upsert(t, map) => trails
					.entry(t)
					.or_default()
					.0
					.extend(map.into_iter().filter(|(k, f)| !k.is_empty() && !f.key().is_empty())),
				Self::Delete(t, keys) => {
					trails.entry(t).or_default().1.extend(keys.into_iter().filter(|k| !k.is_empty()))
				}
				_ => unreachable!(),
			}
		}
		for (t, (u, d)) in trails {
			match (u.is_empty(), d.is_empty()) {
				(true, true) => {}
				(true, false) => Self::Delete(t, d).emit(),
				(false, true) => Self::Upsert(t, u).emit(),
				(false, false) => {
					Self::Delete(t.clone(), d).emit();
					Self::Upsert(t, u).emit();
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
			Self::Size(_, map) => Self::Size(w, map.clone()),
			Self::Rank(_, map) => Self::Rank(w, map.clone()),
			Self::Fail(_, err) => Self::Fail(w, err.clone()),

			Self::Create(_, files) => Self::Create(w, files!(files)),
			Self::Delete(_, urns) => Self::Delete(w, urns.clone()),
			Self::Update(_, map) => Self::Update(w, map!(map)),
			Self::Upsert(_, map) => Self::Upsert(w, map!(map)),
		}
	}
}

impl UserData for FilesOp {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_function_get("tab", |_, ud| ud.named_user_value::<Option<Id>>("tab"));
		fields.add_field_method_get("kind", |_, me| Ok(me.into_str()));
		fields.add_cached_field_mut("url", |_, me| {
			Ok(match me {
				Self::Part(url, ..)
				| Self::Size(url, ..)
				| Self::Rank(url, ..)
				| Self::Fail(url, ..)
				| Self::Create(url, ..)
				| Self::Delete(url, ..)
				| Self::Update(url, ..)
				| Self::Upsert(url, ..) => Some(mem::take(url)),
				_ => None,
			})
		});
		fields.add_cached_field_mut("file", |_, me| {
			Ok(match me {
				Self::Full(file, ..) | Self::Done(file, ..) => Some(mem::take(file)),
				_ => None,
			})
		});
		fields.add_cached_field_mut("entries", |lua, me| {
			Ok(match me {
				Self::Full(_, entries) | Self::Part(_, entries, _) | Self::Create(_, entries) => {
					Some(lua.create_sequence_from(mem::take(entries))?)
				}
				Self::Update(_, entries) | Self::Upsert(_, entries) => {
					Some(lua.create_table_from(mem::take(entries))?)
				}
				Self::Delete(_, keys) => Some(lua.create_sequence_from(mem::take(keys))?),
				Self::Size(_, sizes) => Some(lua.create_table_from(mem::take(sizes))?),
				Self::Rank(_, ranks) => Some(lua.create_table_from(mem::take(ranks))?),
				_ => None,
			})
		});
	}
}
