use yazi_config::plugin::Preloader;
use yazi_shared::{CompletionToken, Id};

#[derive(Clone, Debug)]
pub(crate) struct PreloadIn {
	pub(crate) id:     Id,
	pub(crate) plugin: &'static Preloader,
	pub(crate) target: yazi_fs::File,
	pub(crate) done:   CompletionToken,
}
