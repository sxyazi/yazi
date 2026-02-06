use yazi_config::plugin::Preloader;
use yazi_shared::Id;

#[derive(Clone, Debug)]
pub(crate) struct PreloadIn {
	pub(crate) id:     Id,
	pub(crate) plugin: &'static Preloader,
	pub(crate) target: yazi_fs::File,
}

impl PreloadIn {
	pub(crate) fn id(&self) -> Id { self.id }
}
