use std::path::PathBuf;

use futures::executor::block_on;
use hashbrown::HashSet;
use yazi_fs::{CWD, Xdg, path::clean_url};
use yazi_shared::{strand::StrandBuf, url::{UrlBuf, UrlLike}};
use yazi_vfs::provider;

#[derive(Debug, Default)]
pub struct Boot {
	pub cwds:  Vec<UrlBuf>,
	pub files: Vec<StrandBuf>,

	pub local_events:  HashSet<String>,
	pub remote_events: HashSet<String>,

	pub config_dir: PathBuf,
	pub flavor_dir: PathBuf,
	pub plugin_dir: PathBuf,
	pub state_dir:  PathBuf,
}

impl Boot {
	async fn parse_entries(entries: &[UrlBuf]) -> (Vec<UrlBuf>, Vec<StrandBuf>) {
		if entries.is_empty() {
			return (vec![CWD.load().as_ref().clone()], vec![Default::default()]);
		}

		async fn go(entry: &UrlBuf) -> (UrlBuf, StrandBuf) {
			let mut entry = clean_url(entry);

			if let Ok(u) = provider::absolute(&entry).await
				&& u.is_owned()
			{
				entry = u.into_owned();
			}

			let Some((parent, child)) = entry.pair() else {
				return (entry, Default::default());
			};

			if provider::metadata(&entry).await.is_ok_and(|m| m.is_file()) {
				(parent.into(), child.into())
			} else {
				(entry, Default::default())
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
