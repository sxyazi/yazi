use std::borrow::Cow;

use anyhow::Result;
use yazi_config::YAZI;
use yazi_core::mgr::MgrSnap;
use yazi_fs::Splatter;
use yazi_macro::{act, input, succ};
use yazi_parser::mgr::ShellForm;
use yazi_proxy::TasksProxy;
use yazi_scheduler::process::ShellOpt;
use yazi_shared::data::Data;
use yazi_widgets::input::InputEvent;

use crate::{Actor, Ctx};

pub struct Shell;

impl Actor for Shell {
	type Form = ShellForm;

	const NAME: &str = "shell";

	fn act(cx: &mut Ctx, mut form: Self::Form) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		let cwd = form.cwd.take().unwrap_or_else(|| cx.cwd().clone());
		let snap = MgrSnap::from(&cx.mgr);

		let input = if form.interactive {
			Some(input!(
				cx,
				YAZI.input.shell(form.block).with_value(&*form.run).with_cursor(form.cursor)
			)?)
		} else {
			None
		};

		tokio::spawn(async move {
			if let Some(mut rx) = input {
				match rx.recv().await {
					Some(InputEvent::Submit(e)) => form.run = Cow::Owned(e),
					_ => return,
				}
			}
			if form.run.is_empty() {
				return;
			}

			TasksProxy::process_open(ShellOpt {
				cwd,
				cmd: Splatter::new(snap).splat(&*form.run),
				block: form.block,
				orphan: form.orphan,
			});
		});

		succ!();
	}
}
