use yazi_shared::event::{CmdCow, Data, EventQuit};

#[derive(Default)]
pub struct QuitOpt {
	pub code:        i32,
	pub no_cwd_file: bool,
}

impl From<CmdCow> for QuitOpt {
	fn from(c: CmdCow) -> Self {
		Self {
			code:        c.get("code").and_then(Data::as_i32).unwrap_or_default(),
			no_cwd_file: c.bool("no-cwd-file"),
		}
	}
}

impl From<QuitOpt> for EventQuit {
	fn from(value: QuitOpt) -> Self {
		EventQuit { code: value.code, no_cwd_file: value.no_cwd_file, ..Default::default() }
	}
}
