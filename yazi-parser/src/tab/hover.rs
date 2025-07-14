use yazi_shared::{event::CmdCow, url::Url};

#[derive(Default)]
pub struct HoverOpt {
	pub url: Option<Url>,
}

impl From<CmdCow> for HoverOpt {
	fn from(mut c: CmdCow) -> Self { Self { url: c.take_first_url() } }
}

impl From<Option<Url>> for HoverOpt {
	fn from(url: Option<Url>) -> Self { Self { url } }
}

// --- Do
pub struct HoverDoOpt {
	pub url: Url,
}

impl From<Url> for HoverDoOpt {
	fn from(url: Url) -> Self { Self { url } }
}
