use anyhow::Result;

use super::Package;

impl Package {
	pub(super) async fn upgrade(&mut self) -> Result<()> {
		if self.commit.starts_with('=') { Ok(()) } else { self.add().await }
	}
}
