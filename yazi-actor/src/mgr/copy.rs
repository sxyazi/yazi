use std::ffi::OsString;

use anyhow::{Result, bail};
use yazi_macro::{act, succ};
use yazi_parser::mgr::CopyOpt;
use yazi_shared::event::Data;
use yazi_widgets::CLIPBOARD;

use crate::{Actor, Ctx};

pub struct Copy;

impl Actor for Copy {
	type Options = CopyOpt;

	const NAME: &str = "copy";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		let mut s = OsString::new();
		let mut it = if opt.hovered {
			Box::new(cx.hovered().map(|h| &h.url).into_iter())
		} else {
			cx.tab().selected_or_hovered()
		}
		.peekable();

		while let Some(u) = it.next() {
			match opt.r#type.as_ref() {
				// TODO: rename to "url"
				"path" => {
					s.push(opt.separator.transform(&u.os_str()));
				}
				"dirname" => {
					if let Some(p) = u.parent() {
						s.push(opt.separator.transform(&p.os_str()));
					}
				}
				"filename" => {
					s.push(opt.separator.transform(u.name().unwrap_or_default()));
				}
				"name_without_ext" => {
					s.push(opt.separator.transform(u.stem().unwrap_or_default()));
				}
				_ => bail!("Unknown copy type: {}", opt.r#type),
			};
			if it.peek().is_some() {
				s.push("\n");
			}
		}

		// Copy the CWD path regardless even if the directory is empty
		if s.is_empty() && opt.r#type == "dirname" {
			s.push(opt.separator.transform(&cx.cwd().os_str()));
		}

		futures::executor::block_on(CLIPBOARD.set(s));
		succ!();
	}
}
