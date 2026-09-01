use futures::executor::block_on;
use hashbrown::HashSet;
use yazi_fs::{CWD, path::clean_url};
use yazi_shared::{strand::StrandBuf, url::{UrlBuf, UrlLike}};
use yazi_vfs::engine;

#[derive(Debug, Default)]
pub struct Boot {
	pub cwds:  Vec<UrlBuf>,
	pub files: Vec<StrandBuf>,

	pub local_events:  HashSet<String>,
	pub remote_events: HashSet<String>,
}

impl Boot {
	async fn parse_entries(entries: &[UrlBuf]) -> (Vec<UrlBuf>, Vec<StrandBuf>) {
		if entries.is_empty() {
			return (vec![CWD.load().as_ref().clone()], vec![Default::default()]);
		}

		async fn go(ent: &UrlBuf) -> (UrlBuf, StrandBuf) {
			let ent = clean_url(engine::absolute(ent).await.unwrap_or(ent.into()));

			let Some((trail, child)) = ent.pair() else {
				return (ent, Default::default());
			};

			if engine::metadata(&ent).await.is_ok_and(|m| m.is_file()) {
				(trail.into(), child.into())
			} else {
				(ent, Default::default())
			}
		}

		futures::future::join_all(entries.iter().map(go)).await.into_iter().unzip()
	}
}

impl From<&crate::Args> for Boot {
	fn from(args: &crate::Args) -> Self {
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

		Self { cwds, files, local_events, remote_events }
	}
}
