use anyhow::{Result, bail};
use yazi_macro::{act, succ};
use yazi_parser::mgr::CopyForm;
use yazi_shared::{data::Data, strand::ToStrand, url::UrlLike};
use yazi_widgets::CLIPBOARD;

use crate::{Actor, Ctx};

pub struct Copy;

impl Actor for Copy {
	type Form = CopyForm;

	const NAME: &str = "copy";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;
		if form.r#type == "dirname" {
			yazi_proxy::deprecate!("`copy dirname` is deprecated, use `copy dirpath` instead");
		}

		let mut s = Vec::<u8>::new();
		let mut it = if form.hovered {
			Box::new(cx.hovered().into_iter())
		} else {
			cx.tab().selected_or_hovered_files()
		}
		.peekable();

		while let Some(f) = it.next() {
			match form.r#type.as_ref() {
				"path" => {
					s.extend_from_slice(&form.separator.transform(&f.content_path()));
				}
				"url" => {
					s.extend_from_slice(&form.separator.transform(&f.url.to_strand()));
				}
				"dirpath" | "dirname" => {
					if let Some(p) = f.content_path().parent() {
						s.extend_from_slice(&form.separator.transform(&p));
					}
				}
				"dirurl" => {
					if let Some(p) = f.url.parent() {
						s.extend_from_slice(&form.separator.transform(&p.to_strand()));
					}
				}
				"filename" => {
					s.extend_from_slice(&form.separator.transform(&f.name().unwrap_or_default()));
				}
				"name_without_ext" => {
					s.extend_from_slice(&form.separator.transform(&f.stem().unwrap_or_default()));
				}
				_ => bail!("Unknown copy type: {}", form.r#type),
			};
			if it.peek().is_some() {
				s.push(b'\n');
			}
		}

		// Copy the CWD path regardless even if the directory is empty
		if s.is_empty() && matches!(&*form.r#type, "dirpath" | "dirname") {
			s.extend_from_slice(&form.separator.transform(&cx.current().content_path()));
		} else if s.is_empty() && form.r#type == "dirurl" {
			s.extend_from_slice(&form.separator.transform(&cx.cwd().to_strand()));
		}

		futures::executor::block_on(CLIPBOARD.set(s));
		succ!();
	}
}
