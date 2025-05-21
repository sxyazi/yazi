use std::path::MAIN_SEPARATOR_STR;

use yazi_macro::render;
use yazi_proxy::options::CmpItem;
use yazi_shared::{Id, event::{CmdCow, Data}};

use crate::input::Input;

#[cfg(windows)]
const SEPARATOR: [char; 2] = ['/', '\\'];

#[cfg(not(windows))]
const SEPARATOR: char = std::path::MAIN_SEPARATOR;

pub struct Opt {
	item:    CmpItem,
	_ticket: Id, // FIXME: not used
}

impl TryFrom<CmdCow> for Opt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self {
			item:    c.take_any("item").ok_or(())?,
			_ticket: c.get("ticket").and_then(Data::as_id).unwrap_or_default(),
		})
	}
}

impl Input {
	pub fn complete(&mut self, opt: impl TryInto<Opt>) {
		let Ok(opt): Result<Opt, _> = opt.try_into() else { return };

		let (before, after) = self.partition();
		let new = if let Some((prefix, _)) = before.rsplit_once(SEPARATOR) {
			format!("{prefix}/{}{after}", opt.item.completable()).replace(SEPARATOR, MAIN_SEPARATOR_STR)
		} else {
			format!("{}{after}", opt.item.completable()).replace(SEPARATOR, MAIN_SEPARATOR_STR)
		};

		let snap = self.snap_mut();
		if new == snap.value {
			return;
		}

		let delta = new.chars().count() as isize - snap.count() as isize;
		snap.value = new;

		self.r#move(delta);
		self.flush_value();
		render!();
	}
}
