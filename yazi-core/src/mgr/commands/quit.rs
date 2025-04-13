use std::{ffi::OsString, time::Duration};

use tokio::{select, time};
use yazi_boot::ARGS;
use yazi_config::popup::ConfirmCfg;
use yazi_macro::emit;
use yazi_proxy::ConfirmProxy;
use yazi_shared::{event::{CmdCow, Data, EventQuit}, url::Url};

use crate::{mgr::Mgr, tasks::Tasks};

#[derive(Default)]
pub(super) struct Opt {
	pub(super) code:        i32,
	pub(super) no_cwd_file: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self {
			code:        c.get("code").and_then(Data::as_i32).unwrap_or_default(),
			no_cwd_file: c.bool("no-cwd-file"),
		}
	}
}

impl From<Opt> for EventQuit {
	fn from(value: Opt) -> Self {
		EventQuit { code: value.code, no_cwd_file: value.no_cwd_file, ..Default::default() }
	}
}

impl Mgr {
	#[yazi_codegen::command]
	pub fn quit(&self, opt: Opt, tasks: &Tasks) {
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

	pub(super) fn quit_with_selected(opt: super::open::Opt, selected: &[&Url]) -> bool {
		if opt.interactive || ARGS.chooser_file.is_none() {
			return false;
		}

		let paths = selected.iter().fold(OsString::new(), |mut s, &u| {
			s.push(u.as_os_str());
			s.push("\n");
			s
		});

		emit!(Quit(EventQuit { selected: Some(paths), ..Default::default() }));
		true
	}
}
