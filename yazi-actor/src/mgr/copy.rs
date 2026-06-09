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

		let mut s = Vec::<u8>::new();
		let mut it = if form.hovered {
			Box::new(cx.hovered().map(|h| &h.url).into_iter())
		} else {
			cx.tab().selected_or_hovered()
		}
		.peekable();

		while let Some(u) = it.next() {
			match form.r#type.as_ref() {
				// TODO: rename to "url"
				"path" => {
					s.extend_from_slice(&form.separator.transform(&u.to_strand()));
				}
				"dirname" => {
					if let Some(p) = u.parent() {
						s.extend_from_slice(&form.separator.transform(&p.to_strand()));
					}
				}
				"filename" => {
					s.extend_from_slice(&form.separator.transform(&u.name().unwrap_or_default()));
				}
				"name_without_ext" => {
					s.extend_from_slice(&form.separator.transform(&u.stem().unwrap_or_default()));
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
		if s.is_empty() && form.r#type == "dirname" {
			s.extend_from_slice(&form.separator.transform(&cx.cwd().to_strand()));
		}

		if yazi_emulator::EMULATOR.osc_5522 {
			let mut data = Vec::<ClipboardData>::new();
			match form.r#type.as_ref() {
				"uri_list" => {
					data.push(ClipboardData {
						mime:    b"text/uri-list".to_vec(),
						payload: s.clone(),
						alias:   b"text/plain".to_vec(),
					});
					#[cfg(target_os = "linux")]
					// Because Thunar (and likely others) won't reconize `text/uri-list`
					data.push(ClipboardData {
						mime:    b"x-special/gnome-copied-files".to_vec(),
						payload: [b"copy\n".to_vec(), s].concat(),
						alias:   vec![],
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
