use std::process;

use anyhow::Result;
use tokio::{join, task};
use yazi_boot::ARGS;
use yazi_emulator::EMULATOR;
use yazi_fs::engine::{Engine, local::Local};
use yazi_macro::succ;
use yazi_parser::app::QuitForm;
use yazi_shared::{data::Data, strand::{StrandBuf, StrandLike, ToStrand}, url::UrlBuf};
use yazi_tui::Raterm;

use crate::{Actor, Ctx};

pub struct Quit;

impl Actor for Quit {
	type Form = QuitForm;

	const NAME: &str = "quit";

	fn act(cx: &mut Ctx, Self::Form { opt }: Self::Form) -> Result<Data> {
		cx.tasks.shutdown();
		cx.mgr.shutdown();
		yazi_plugin::shutdown();

		let cwd = cx.mgr.cwd().clone();
		task::spawn_local(async move {
			_ = join!(
				yazi_dds::shutdown(),
				yazi_dds::STATE.drain(),
				Self::cwd_to_file(&cwd, opt.no_cwd_file),
				Self::selected_to_file(opt.selected),
				Self::wait_probe(),
			);

			Raterm::stop();
			process::exit(opt.code);
		});

		succ!();
	}
}

impl Quit {
	async fn cwd_to_file(cwd: &UrlBuf, no: bool) {
		if let Some(p) = ARGS.cwd_file.as_ref().filter(|_| !no) {
			Local::regular(p).write(cwd.to_strand().encoded_bytes()).await.ok();
		}
	}

	async fn selected_to_file(selected: Option<StrandBuf>) {
		if let (Some(s), Some(p)) = (selected, &ARGS.chooser_file) {
			Local::regular(p).write(s.encoded_bytes()).await.ok();
		}
	}

	async fn wait_probe() {
		if let Some(id) = EMULATOR.probe.pending() {
			EMULATOR.probe.wait(id).await;
		}
	}
}
