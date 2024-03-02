use yazi_shared::{emit, event::Cmd, fs::Url, Layer};

#[derive(Default)]
pub struct OpenDoOpt {
	pub hovered:     Url,
	pub targets:     Vec<(Url, String)>,
	pub interactive: bool,
}

impl From<Cmd> for OpenDoOpt {
	fn from(mut c: Cmd) -> Self { c.take_data().unwrap_or_default() }
}

pub struct ManagerProxy;

impl ManagerProxy {
	#[inline]
	pub fn peek(force: bool) {
		emit!(Call(Cmd::new("peek").with_bool("force", force), Layer::Manager));
	}

	#[inline]
	pub fn hover(url: Option<Url>) {
		emit!(Call(
			Cmd::args("hover", url.map_or_else(Vec::new, |u| vec![u.to_string()])),
			Layer::Manager
		));
	}

	#[inline]
	pub fn refresh() {
		emit!(Call(Cmd::new("refresh"), Layer::Manager));
	}

	#[inline]
	pub fn open_do(opt: OpenDoOpt) {
		emit!(Call(Cmd::new("open_do").with_data(opt), Layer::Manager));
	}

	#[inline]
	pub fn remove_do(targets: Vec<Url>, permanently: bool) {
		emit!(Call(
			Cmd::new("remove_do").with_bool("permanently", permanently).with_data(targets),
			Layer::Manager
		));
	}

	#[inline]
	pub fn update_paged() {
		emit!(Call(Cmd::new("update_paged"), Layer::Manager));
	}

	#[inline]
	pub fn update_paged_by(page: usize, only_if: &Url) {
		emit!(Call(
			Cmd::args("update_paged", vec![page.to_string()]).with("only-if", only_if.to_string()),
			Layer::Manager
		));
	}
}
