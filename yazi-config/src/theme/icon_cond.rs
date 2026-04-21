use serde::Deserialize;
use yazi_fs::File;
use yazi_shared::Condition;

use crate::{Icon, Mixable};

#[derive(Debug, Deserialize)]
pub struct IconCond {
	pub r#if: Condition,
	#[serde(flatten)]
	pub icon: Icon,
}

impl IconCond {
	pub fn matches(&self, file: &File, hovered: bool) -> bool {
		let f = |s: &str| match s {
			"dir" => file.is_dir(),
			"hidden" => file.is_hidden(),
			"link" => file.is_link(),
			"orphan" => file.is_orphan(),
			"dummy" => file.is_dummy(),
			"block" => file.is_block(),
			"char" => file.is_char(),
			"fifo" => file.is_fifo(),
			"sock" => file.is_sock(),
			"exec" => file.is_exec(),
			"sticky" => file.is_sticky(),
			"hovered" => hovered,
			_ => false,
		};

		self.r#if.eval(f) == Some(true)
	}
}

impl Mixable for IconCond {}
