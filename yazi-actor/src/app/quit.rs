use anyhow::Result;
use yazi_boot::ARGS;
use yazi_fs::provider::{Provider, local::Local};
use yazi_parser::app::QuitOpt;
use yazi_shared::{data::Data, strand::{StrandBuf, StrandLike, ToStrand}};
use yazi_term::Term;

use crate::{Actor, Ctx};

pub struct Quit;

impl Actor for Quit {
	type Options = QuitOpt;

	const NAME: &str = "quit";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		cx.core.tasks.shutdown();
		cx.core.mgr.shutdown();

		futures::executor::block_on(async {
			_ = futures::join!(
				yazi_dds::shutdown(),
				yazi_dds::STATE.drain(),
				Self::cwd_to_file(cx, opt.no_cwd_file),
				Self::selected_to_file(opt.selected)
			);
		});

		Term::goodbye(|| opt.code);
	}
}

impl Quit {
	async fn cwd_to_file(cx: &Ctx<'_>, no: bool) {
		if let Some(p) = ARGS.cwd_file.as_ref().filter(|_| !no) {
			let cwd = cx.core.mgr.cwd().to_strand();
			Local::regular(p).write(cwd.encoded_bytes()).await.ok();
		}
	}

	async fn selected_to_file(selected: Option<StrandBuf>) {
		if let (Some(s), Some(p)) = (selected, &ARGS.chooser_file) {
			Local::regular(p).write(s.encoded_bytes()).await.ok();
		}
	}
}
