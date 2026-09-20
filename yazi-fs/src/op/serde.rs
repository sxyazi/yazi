use hashbrown::{HashMap, HashSet};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use yazi_shared::{id::Id, path::PathBufDyn, url::UrlBuf};

use super::FilesOp;
use crate::file::File;

impl Serialize for FilesOp {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		#[derive(Serialize)]
		#[serde(tag = "kind", rename_all = "lowercase")]
		enum Repr<'a> {
			Full { file: &'a File, entries: &'a [File] },
			Part { url: &'a UrlBuf, entries: &'a [File], id: Id },
			Done { file: &'a File, id: Id },
			Size { url: &'a UrlBuf, entries: &'a HashMap<PathBufDyn, u64> },
			Rank { url: &'a UrlBuf, entries: &'a HashMap<PathBufDyn, i64> },
			Fail { url: &'a UrlBuf, error: &'a yazi_shim::fs::Error },

			Create { url: &'a UrlBuf, entries: &'a [File] },
			Delete { url: &'a UrlBuf, entries: &'a HashSet<PathBufDyn> },
			Update { url: &'a UrlBuf, entries: &'a HashMap<PathBufDyn, File> },
			Upsert { url: &'a UrlBuf, entries: &'a HashMap<PathBufDyn, File> },
		}

		match self {
			Self::Full(file, entries) => Repr::Full { file, entries },
			Self::Part(url, entries, id) => Repr::Part { url, entries, id: *id },
			Self::Done(file, id) => Repr::Done { file, id: *id },
			Self::Size(url, entries) => Repr::Size { url, entries },
			Self::Rank(url, entries) => Repr::Rank { url, entries },
			Self::Fail(url, error) => Repr::Fail { url, error },

			Self::Create(url, entries) => Repr::Create { url, entries },
			Self::Delete(url, entries) => Repr::Delete { url, entries },
			Self::Update(url, entries) => Repr::Update { url, entries },
			Self::Upsert(url, entries) => Repr::Upsert { url, entries },
		}
		.serialize(serializer)
	}
}

impl<'de> Deserialize<'de> for FilesOp {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		#[derive(Deserialize)]
		#[serde(tag = "kind", rename_all = "lowercase")]
		enum Repr {
			Full { file: File, entries: Vec<File> },
			Part { url: UrlBuf, entries: Vec<File>, id: Id },
			Done { file: File, id: Id },
			Size { url: UrlBuf, entries: HashMap<PathBufDyn, u64> },
			Rank { url: UrlBuf, entries: HashMap<PathBufDyn, i64> },
			Fail { url: UrlBuf, error: yazi_shim::fs::Error },

			Create { url: UrlBuf, entries: Vec<File> },
			Delete { url: UrlBuf, entries: HashSet<PathBufDyn> },
			Update { url: UrlBuf, entries: HashMap<PathBufDyn, File> },
			Upsert { url: UrlBuf, entries: HashMap<PathBufDyn, File> },
		}

		Ok(match Repr::deserialize(deserializer)? {
			Repr::Full { file, entries } => Self::Full(file, entries),
			Repr::Part { url, entries, id } => Self::Part(url, entries, id),
			Repr::Done { file, id } => Self::Done(file, id),
			Repr::Size { url, entries } => Self::Size(url, entries),
			Repr::Rank { url, entries } => Self::Rank(url, entries),
			Repr::Fail { url, error } => Self::Fail(url, error),

			Repr::Create { url, entries } => Self::Create(url, entries),
			Repr::Delete { url, entries } => Self::Delete(url, entries),
			Repr::Update { url, entries } => Self::Update(url, entries),
			Repr::Upsert { url, entries } => Self::Upsert(url, entries),
		})
	}
}
