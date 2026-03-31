use std::mem;

use yazi_fs::Splatter;
use yazi_scheduler::process::ProcessOpt;

use super::Tasks;

impl Tasks {
	// TODO: remove
	pub fn open_shell_compat(&self, mut opt: ProcessOpt) {
		if opt.spread {
			opt.cmd = Splatter::new(&opt.args).splat(opt.cmd);
			self.scheduler.process_open(opt);
			return;
		}
		if opt.args.is_empty() {
			return;
		}
		if opt.args.len() == 2 {
			opt.cmd = Splatter::new(&opt.args).splat(opt.cmd);
			self.scheduler.process_open(opt);
			return;
		}
		let hovered = mem::take(&mut opt.args[0]);
		for target in opt.args.into_iter().skip(1) {
			let args = vec![hovered.clone(), target];
			self.scheduler.process_open(ProcessOpt {
				cwd: opt.cwd.clone(),
				cmd: Splatter::new(&args).splat(&opt.cmd),
				args,
				block: opt.block,
				orphan: opt.orphan,
				spread: opt.spread,
			});
		}
	}
}
