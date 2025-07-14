use std::collections::HashSet;

use anyhow::bail;
use yazi_shared::{event::CmdCow, url::Url};

#[derive(Default)]
pub struct UpdateYankedOpt {
	pub cut:  bool,
	pub urls: HashSet<Url>,
}

impl TryFrom<CmdCow> for UpdateYankedOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		// TODO: remove `BodyYankIter`
		if let Some(iter) = c.take_any::<yazi_dds::body::BodyYankIter>("urls") {
			Ok(Self { urls: iter.urls.into_iter().collect(), cut: iter.cut })
		} else {
			bail!("Invalid 'urls' argument in UpdateYankedOpt");
		}
	}
}
