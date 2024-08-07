use std::{collections::HashSet, ffi::OsString, path::{Path, PathBuf}};

use serde::Serialize;
use yazi_shared::{fs::{current_cwd, expand_path}, Xdg};

#[derive(Debug, Default, Serialize)]
pub struct Boot {
	pub cwds:  Vec<PathBuf>,
	pub files: Vec<Option<OsString>>,

	pub local_events:  HashSet<String>,
	pub remote_events: HashSet<String>,

	pub config_dir: PathBuf,
	pub flavor_dir: PathBuf,
	pub plugin_dir: PathBuf,
	pub state_dir:  PathBuf,
}

impl Boot {
	fn parse_entries(entries: Vec<&Path>) -> (Vec<PathBuf>, Vec<Option<OsString>>) {
		if entries.len() == 0 {
			return (vec![current_cwd().unwrap()], vec![None]);
		}

		let mut cwds = vec![];
		let mut files = vec![];
		for entry in entries {
			let _entry = expand_path(entry);
			let parent = _entry.parent();
			if parent.is_none() || _entry.is_dir() {
				cwds.push(_entry);
				files.push(None);
			} else {
				cwds.push(parent.unwrap().to_owned());
				files.push(Some(_entry.file_name().unwrap().to_owned()));
			}
		}

		(cwds, files)
	}
}

impl From<&crate::Args> for Boot {
	fn from(args: &crate::Args) -> Self {
		let config_dir = Xdg::config_dir();
		let entries = args.entries.iter().map(PathBuf::as_path).collect();
		let (cwds, files) = Self::parse_entries(entries);

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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_entries() {
		use std::{env::temp_dir, fs};

		let foo_dir = temp_dir().join(&Path::new("foo"));
		let bar_dir = temp_dir().join(&Path::new("bar"));
		let poem_path = &foo_dir.join(&Path::new("poem.txt"));

		let _ = fs::create_dir_all(&foo_dir);
		let _ = fs::create_dir_all(&bar_dir);
		let _ =
			fs::OpenOptions::new().create(true).write(true).open(foo_dir.join(&Path::new("poem.txt")));

		assert_eq!(Boot::parse_entries(vec![]), (vec![current_cwd().unwrap()], vec![None]));
		assert_eq!(Boot::parse_entries(vec![&foo_dir]), (vec![foo_dir.clone()], vec![None]));
		assert_eq!(
			Boot::parse_entries(vec![&poem_path]),
			(vec![foo_dir.clone()], vec![Some(OsString::from("poem.txt"))])
		);
		assert_eq!(
			Boot::parse_entries(vec![&foo_dir, &bar_dir]),
			(vec![foo_dir.clone(), bar_dir.clone()], vec![None, None])
		);

		let _ = fs::remove_dir_all(&foo_dir);
		let _ = fs::remove_dir_all(&bar_dir);
	}
}
