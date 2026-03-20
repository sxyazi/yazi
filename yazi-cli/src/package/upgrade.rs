use anyhow::Result;

use super::Dependency;

impl Dependency {
	pub(super) async fn upgrade(&mut self, overwrite: bool) -> Result<()> {
		if self.rev.starts_with('=') { Ok(()) } else { self.add(overwrite).await }
	}
}
