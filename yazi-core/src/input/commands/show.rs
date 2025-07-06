use yazi_config::YAZI;
use yazi_macro::render;
use yazi_parser::input::ShowOpt;
use yazi_shared::errors::InputError;
use yazi_widgets::input::InputCallback;

use crate::input::Input;

impl Input {
	pub fn show(&mut self, opt: impl TryInto<ShowOpt>) {
		let Ok(opt): Result<ShowOpt, _> = opt.try_into() else { return };

		self.close(false);
		self.visible = true;
		self.title = opt.cfg.title;
		self.position = opt.cfg.position;

		// Typing
		self.tx = Some(opt.tx.clone());
		let ticket = self.ticket.clone();

		// Reset input
		let cb: InputCallback = Box::new(move |before, after| {
			if opt.cfg.realtime {
				opt.tx.send(Err(InputError::Typed(format!("{before}{after}")))).ok();
			} else if opt.cfg.completion {
				opt.tx.send(Err(InputError::Completed(before.to_owned(), ticket.current()))).ok();
			}
		});
		self.inner = yazi_widgets::input::Input::new(
			opt.cfg.value,
			opt.cfg.position.offset.width.saturating_sub(YAZI.input.border()) as usize,
			opt.cfg.obscure,
			cb,
		);

		// Set cursor after reset
		// TODO: remove this
		if let Some(cursor) = opt.cfg.cursor {
			self.snap_mut().cursor = cursor;
			self.r#move(0);
		}

		render!();
	}
}
