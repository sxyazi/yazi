use std::{borrow::Cow, collections::HashSet, path::PathBuf};

use futures::executor::block_on;
use serde::Serialize;
use yazi_fs::{CWD, Xdg, path::expand_url, provider};
use yazi_shared::url::{Url, UrnBuf};

#[derive(Debug, Default, Serialize)]
pub struct Boot {
	pub cwds:  Vec<Url>,
	pub files: Vec<UrnBuf>,

	pub local_events:  HashSet<String>,
	pub remote_events: HashSet<String>,

	pub config_dir: PathBuf,
	pub flavor_dir: PathBuf,
	pub plugin_dir: PathBuf,
	pub state_dir:  PathBuf,
}

impl Boot {
	async fn parse_entries(entries: &[Url]) -> (Vec<Url>, Vec<UrnBuf>) {
		if entries.is_empty() {
			return (vec![CWD.load().as_ref().clone()], vec![UrnBuf::default()]);
		}

		async fn go<'a>(entry: Cow<'a, Url>) -> (Url, UrnBuf) {
			let Some((parent, child)) = entry.pair() else {
				return (entry.into_owned(), UrnBuf::default());
			};

			if provider::metadata(&entry).await.is_ok_and(|m| m.is_file()) {
				(parent, child)
			} else {
				(entry.into_owned(), UrnBuf::default())
			}
		}

		futures::future::join_all(entries.iter().map(expand_url).map(go)).await.into_iter().unzip()
	}
}

impl From<&crate::Args> for Boot {
	fn from(args: &crate::Args) -> Self {
		let config_dir = Xdg::config_dir();
		let (cwds, files) = block_on(Self::parse_entries(&args.entries));

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
