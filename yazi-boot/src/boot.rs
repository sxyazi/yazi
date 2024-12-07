use std::{collections::HashSet, ffi::OsString, path::PathBuf};

use serde::Serialize;
use yazi_fs::{CWD, Xdg, expand_path};

#[derive(Debug, Default, Serialize)]
pub struct Boot {
	pub cwds:  Vec<PathBuf>,
	pub files: Vec<OsString>,

	pub local_events:  HashSet<String>,
	pub remote_events: HashSet<String>,

	pub config_dir: PathBuf,
	pub flavor_dir: PathBuf,
	pub plugin_dir: PathBuf,
	pub state_dir:  PathBuf,
}

impl Boot {
	fn parse_entries(entries: &[PathBuf]) -> (Vec<PathBuf>, Vec<OsString>) {
		if entries.is_empty() {
			return (vec![CWD.load().to_path_buf()], vec![OsString::new()]);
		}

		let mut cwds = Vec::with_capacity(entries.len());
		let mut files = Vec::with_capacity(entries.len());
		for entry in entries.iter().map(expand_path) {
			if let Some(p) = entry.parent().filter(|_| !entry.is_dir()) {
				cwds.push(p.to_owned());
				files.push(entry.file_name().unwrap().to_owned());
			} else {
				cwds.push(entry);
				files.push(OsString::new());
			}
		}

		(cwds, files)
	}
}

impl From<&crate::Args> for Boot {
	fn from(args: &crate::Args) -> Self {
		let config_dir = Xdg::config_dir();
		let (cwds, files) = Self::parse_entries(&args.entries);

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
			cwds,
			files,

			local_events,
			remote_events,

			flavor_dir: config_dir.join("flavors"),
			plugin_dir: config_dir.join("plugins"),
			config_dir,
			state_dir: Xdg::state_dir(),
		}
	}
}
