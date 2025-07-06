use std::{ffi::OsString, time::Duration};

use tokio::{select, time};
use yazi_boot::ARGS;
use yazi_config::popup::ConfirmCfg;
use yazi_macro::emit;
use yazi_parser::mgr::{OpenOpt, QuitOpt};
use yazi_proxy::ConfirmProxy;
use yazi_shared::{event::EventQuit, url::Url};

use crate::{mgr::Mgr, tasks::Tasks};

impl Mgr {
	#[yazi_codegen::command]
	pub fn quit(&self, opt: QuitOpt, tasks: &Tasks) {
		let event = opt.into();

		let ongoing = tasks.ongoing().clone();
		let (left, left_names) = {
			let ongoing = ongoing.lock();
			(ongoing.len(), ongoing.values().take(11).map(|t| t.name.clone()).collect())
		};

		if left == 0 {
			emit!(Quit(event));
			return;
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
	}

	pub(super) fn quit_with_selected<'a, I>(opt: OpenOpt, selected: I) -> bool
	where
		I: Iterator<Item = &'a Url>,
	{
		if opt.interactive || ARGS.chooser_file.is_none() {
			return false;
		}

		let paths = selected.fold(OsString::new(), |mut s, u| {
			s.push(u.as_os_str());
			s.push("\n");
			s
		});

		emit!(Quit(EventQuit { selected: Some(paths), ..Default::default() }));
		true
	}
}
