use std::{collections::HashSet, ffi::OsString, path::{Path, PathBuf}};

use serde::Serialize;
use yazi_shared::{fs::{current_cwd, expand_path}, Xdg};

#[derive(Debug, Default, Serialize)]
pub struct Boot {
	pub cwd:  PathBuf,
	pub file: Option<OsString>,

	pub local_events:  HashSet<String>,
	pub remote_events: HashSet<String>,

	pub config_dir: PathBuf,
	pub flavor_dir: PathBuf,
	pub plugin_dir: PathBuf,
	pub state_dir:  PathBuf,
}

impl Boot {
	fn parse_entry(entry: Option<&Path>) -> (PathBuf, Option<OsString>) {
		let entry = match entry {
			Some(p) => expand_path(p),
			None => return (current_cwd().unwrap(), None),
		};

		let parent = entry.parent();
		if parent.is_none() || entry.is_dir() {
			return (entry, None);
		}

		(parent.unwrap().to_owned(), Some(entry.file_name().unwrap().to_owned()))
	}
}

impl From<&crate::Args> for Boot {
	fn from(args: &crate::Args) -> Self {
		let config_dir = Xdg::config_dir();
		let (cwd, file) = Self::parse_entry(args.entry.as_deref());

		let local_events = args
			.local_events
			.as_ref()
			.map(|s| s.split(',').map(|s| s.to_owned()).collect())
			.unwrap_or_default();
		let remote_events = args
			.remote_events
			.as_ref()
			.map(|s| s.split(',').map(|s| s.to_owned()).collect())
			.unwrap_or_default();

		Self {
			cwd,
			file,

			local_events,
			remote_events,

			flavor_dir: config_dir.join("flavors"),
			plugin_dir: config_dir.join("plugins"),
			config_dir,
			state_dir: Xdg::state_dir(),
		}
	}
}
