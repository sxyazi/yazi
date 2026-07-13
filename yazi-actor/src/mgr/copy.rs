use anyhow::{Result, bail};
use yazi_macro::{act, succ};
use yazi_parser::mgr::CopyForm;
use yazi_shared::{data::Data, strand::ToStrand, url::UrlLike};
use yazi_shim::RFC_3986;
use yazi_widgets::{CLIPBOARD, ClipboardData};

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
				"uri_list" => {
					// Per the spec this should be CRLF line endings but everything i've tested on
					// linux works with just LF
					s.extend_from_slice(b"file://");
					s.extend_from_slice(
						percent_encoding::percent_encode(&form.separator.transform(&u.to_strand()), RFC_3986)
							.to_string()
							.as_bytes(),
					);
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

		if yazi_emulator::EMULATOR.osc_5522 {
			let mut data = Vec::<ClipboardData>::new();
			match form.r#type.as_ref() {
				"uri_list" => {
					data.push(ClipboardData {
						mime:    b"text/uri-list".to_vec(),
						payload: s,
						alias:   b"text/plain".to_vec(),
					});
				}
				_ => {
					data.push(ClipboardData { mime: b"text/plain".to_vec(), payload: s, alias: vec![] });
				}
			}

			futures::executor::block_on(CLIPBOARD.write(data));
		} else {
			futures::executor::block_on(CLIPBOARD.set(s));
		}
		succ!();
	}
}
