use anyhow::{Result, bail};
use yazi_macro::{act, succ};
use yazi_parser::mgr::CopyOpt;
use yazi_shared::{data::Data, strand::ToStrand, url::UrlLike};
use yazi_widgets::CLIPBOARD;

use crate::{Actor, Ctx};

pub struct Copy;

impl Actor for Copy {
	type Options = CopyOpt;

	const NAME: &str = "copy";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		let mut s = Vec::<u8>::new();
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
					s.extend_from_slice(&opt.separator.transform(&u.to_strand()));
				}
				"dirname" => {
					if let Some(p) = u.parent() {
						s.extend_from_slice(&opt.separator.transform(&p.to_strand()));
					}
				}
				"filename" => {
					s.extend_from_slice(&opt.separator.transform(&u.name().unwrap_or_default()));
				}
				"name_without_ext" => {
					s.extend_from_slice(&opt.separator.transform(&u.stem().unwrap_or_default()));
				}
				_ => bail!("Unknown copy type: {}", opt.r#type),
			};
			if it.peek().is_some() {
				s.push(b'\n');
			}
		}

		// Copy the CWD path regardless even if the directory is empty
		if s.is_empty() && opt.r#type == "dirname" {
			s.extend_from_slice(&opt.separator.transform(&cx.cwd().to_strand()));
		}

		futures::executor::block_on(CLIPBOARD.set(s));
		succ!();
	}
}
