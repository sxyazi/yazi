use yazi_config::keymap::{Exec, KeymapLayer};
use yazi_shared::{ends_with_slash, Defer};

use crate::{emit, external::{self, FzfOpt, ZoxideOpt}, tab::Tab, Event, BLOCKER};

impl Tab {
	pub fn jump(&self, global: bool) -> bool {
		let cwd = self.current.cwd.clone();

		tokio::spawn(async move {
			let _guard = BLOCKER.acquire().await.unwrap();
			let _defer = Defer::new(|| Event::Stop(false, None).emit());
			emit!(Stop(true)).await;

			let url = if global {
				external::fzf(FzfOpt { cwd }).await
			} else {
				external::zoxide(ZoxideOpt { cwd }).await
			}?;

			let op = if global && !ends_with_slash(&url) { "reveal" } else { "cd" };
			emit!(Call(Exec::call(op, vec![url.to_string()]).vec(), KeymapLayer::Manager));
			Ok::<(), anyhow::Error>(())
		});
		false
	}
}
