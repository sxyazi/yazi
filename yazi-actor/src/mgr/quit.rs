use std::{ffi::OsString, time::Duration};

use anyhow::Result;
use tokio::{select, time};
use yazi_config::popup::ConfirmCfg;
use yazi_dds::spark::SparkKind;
use yazi_macro::{emit, succ};
use yazi_parser::mgr::QuitOpt;
use yazi_proxy::ConfirmProxy;
use yazi_shared::{data::Data, event::EventQuit, url::UrlCow};

use crate::{Actor, Ctx};

pub struct Quit;

impl Actor for Quit {
	type Options = QuitOpt;

	const NAME: &str = "quit";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let event = opt.into();

		let ongoing = cx.tasks().ongoing().clone();
		let (left, left_names) = {
			let ongoing = ongoing.lock();
			(ongoing.len(), ongoing.values().take(11).map(|t| t.name.clone()).collect())
		};

		if left == 0 {
			succ!(emit!(Quit(event)));
		}

		tokio::spawn(async move {
			let mut i = 0;
			let mut rx = ConfirmProxy::show_rx(ConfirmCfg::quit(left, left_names));
			loop {
				select! {
					_ = time::sleep(Duration::from_millis(50)) => {
						i += 1;
						if i > 40 { break }
						else if ongoing.lock().is_empty() {
							emit!(Quit(event));
							return;
						}
					}
					b = &mut rx => {
						if b.unwrap_or(false) {
							emit!(Quit(event));
						}
						return;
					}
				}
			}

			if rx.await.unwrap_or(false) {
				emit!(Quit(event));
			}
		});
		succ!();
	}

	fn hook(cx: &Ctx, _opt: &Self::Options) -> Option<SparkKind> {
		Some(SparkKind::KeyQuit).filter(|_| cx.source().is_key())
	}
}

impl Quit {
	pub(super) fn with_selected<'a, I, T>(selected: I)
	where
		I: IntoIterator<Item = T>,
		T: Into<UrlCow<'a>>,
	{
		let paths = selected.into_iter().fold(OsString::new(), |mut s, u| {
			s.push(u.into().as_url().os_str());
			s.push("\n");
			s
		});

		emit!(Quit(EventQuit { selected: Some(paths), ..Default::default() }));
	}
}
