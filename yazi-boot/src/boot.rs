use std::path::PathBuf;

use futures::executor::block_on;
use hashbrown::HashSet;
use serde::Serialize;
use yazi_fs::{CWD, path::expand_url, provider};
use yazi_shared::url::{UrlBuf, UrlCow, UrnBuf};
use yazi_vfs::local::Xdg;

#[derive(Debug, Default, Serialize)]
pub struct Boot {
	pub cwds:  Vec<UrlBuf>,
	pub files: Vec<UrnBuf>,

	pub local_events:  HashSet<String>,
	pub remote_events: HashSet<String>,

	pub config_dir: PathBuf,
	pub flavor_dir: PathBuf,
	pub plugin_dir: PathBuf,
	pub state_dir:  PathBuf,
}

impl Boot {
	async fn parse_entries(entries: &[UrlBuf]) -> (Vec<UrlBuf>, Vec<UrnBuf>) {
		if entries.is_empty() {
			return (vec![CWD.load().as_ref().clone()], vec![UrnBuf::default()]);
		}

		async fn go(entry: &UrlBuf) -> (UrlBuf, UrnBuf) {
			let mut entry = expand_url(entry);
			if let Ok(u @ UrlCow::Owned { .. }) = provider::absolute(&entry).await {
				entry = u.into_owned();
			}

			let Some((parent, child)) = entry.pair() else {
				return (entry, UrnBuf::default());
			};

			if provider::metadata(&entry).await.is_ok_and(|m| m.is_file()) {
				(parent.into(), child.into())
			} else {
				(entry, UrnBuf::default())
			}
		}

		futures::future::join_all(entries.iter().map(go)).await.into_iter().unzip()
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
