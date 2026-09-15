use std::ffi::OsString;

use anyhow::Result;
use yazi_actor::Ctx;
use yazi_boot::{ARGS, BOOT};
use yazi_core::mgr::CdSource;
use yazi_fs::path::clean_url;
use yazi_macro::{act, succ};
use yazi_parser::VoidForm;
use yazi_shared::{data::Data, strand::StrandLike, url::{UrlBuf, UrlCow, UrlLike}};
use yazi_vfs::engine;

use crate::Actor;

pub struct Bootstrap;

impl Actor for Bootstrap {
	type Form = VoidForm;

	const NAME: &str = "bootstrap";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		if ARGS.entries.is_empty() {
			act!(mgr:cd, cx, (BOOT.cwd.clone(), CdSource::Tab))?;
			succ!();
		}

		// Resize the tabs to match the number of entries.
		cx.mgr.tabs.resize_with(ARGS.entries.len(), Default::default);

		// Navigate to each entry in concurrent tabs.
		for (i, ent) in ARGS.entries.iter().enumerate().rev() {
			let url = match Self::parse_entry(ent.encoded_bytes()) {
				Ok(url) => url,
				Err(e) => UrlBuf::try_from(format!("error://boot//{e}"))?,
			};

			cx.with(i, |cx| act!(mgr:cd, cx, (url, CdSource::Tab)))?;
		}

		succ!();
	}
}

impl Bootstrap {
	fn parse_entry(b: &[u8]) -> Result<UrlBuf> {
		let mut url = UrlCow::try_from(b)?;
		if let Some(u) = engine::try_absolute(&url)
			&& u.is_owned()
		{
			url = u.into_owned().into();
		}

		let mut s = OsString::from("go://boot/");
		s.push(clean_url(url).os_str());
		Ok(UrlCow::try_from(s.into_encoded_bytes())?.into_owned())
	}
}
